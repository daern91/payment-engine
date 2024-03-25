mod error;

use csv::Writer;
use error::EngineError;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum TransactionState {
    Normal,
    Disputed,
    Resolved,
    Chargeback,
}

#[derive(Debug, Deserialize, Clone)]
struct TransactionRecord {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

#[derive(Debug, Clone)]
struct TransactionDetails {
    record: TransactionRecord,
    state: TransactionState,
}

#[derive(Debug, Serialize)]
struct Account {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl Account {
    fn new(client_id: u16) -> Account {
        Account {
            client: client_id,
            available: Decimal::new(0, 0),
            held: Decimal::new(0, 0),
            total: Decimal::new(0, 0),
            locked: false,
        }
    }

    fn is_locked(&self) -> Result<(), EngineError> {
        if self.locked {
            return Err(EngineError::AccountLocked {
                client_id: (self.client),
            });
        }
        Ok(())
    }

    fn has_sufficient_funds(&self, amount: Decimal) -> Result<(), EngineError> {
        if self.available < amount {
            return Err(EngineError::InsufficientFunds {
                client_id: self.client,
            });
        }
        Ok(())
    }

    fn has_sufficient_held_funds(&self, amount: Decimal) -> Result<(), EngineError> {
        if self.held < amount {
            return Err(EngineError::InsufficientHeldFunds {
                client_id: self.client,
            });
        }
        Ok(())
    }

    fn deposit(&mut self, amount: Decimal) -> Result<(), EngineError> {
        self.is_locked()?;
        self.available = (self.available + amount).round_dp(4);
        self.total = (self.total + amount).round_dp(4);
        Ok(())
    }

    fn withdraw(&mut self, amount: Decimal) -> Result<(), EngineError> {
        self.is_locked()?;
        self.has_sufficient_funds(amount)?;
        self.available = (self.available - amount).round_dp(4);
        self.total = (self.total - amount).round_dp(4);
        Ok(())
    }

    fn dispute(&mut self, amount: Decimal) -> Result<(), EngineError> {
        self.is_locked()?;
        self.has_sufficient_funds(amount)?;
        self.available = (self.available - amount).round_dp(4);
        self.held = (self.held + amount).round_dp(4);
        Ok(())
    }

    fn resolve(&mut self, amount: Decimal) -> Result<(), EngineError> {
        self.is_locked()?;
        self.has_sufficient_held_funds(amount)?;
        self.available = (self.available + amount).round_dp(4);
        self.held = (self.held - amount).round_dp(4);
        Ok(())
    }

    fn chargeback(&mut self, amount: Decimal) -> Result<(), EngineError> {
        self.is_locked()?;
        self.has_sufficient_held_funds(amount)?;
        self.total = (self.total - amount).round_dp(4);
        self.held = (self.held - amount).round_dp(4);
        self.locked = true;
        Ok(())
    }
}

pub fn process_transactions<R: Read, W: Write>(
    reader: R,
    writer: &mut W,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(reader);
    let mut wtr = Writer::from_writer(writer);

    let mut transactions: HashMap<u32, TransactionDetails> = HashMap::new();
    let mut accounts: HashMap<u16, Account> = HashMap::new();

    for result in rdr.deserialize::<TransactionRecord>() {
        let record = match result {
            Ok(record) => record,
            Err(e) => {
                eprintln!("Error deserializing a record: {:?}", e);
                continue;
            }
        };

        let client_id = record.client;
        let transaction_id = record.tx;
        let amount = record.amount.unwrap_or(Decimal::new(0, 0));

        let account = accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));

        if account.locked {
            continue;
        }

        match record.transaction_type {
            TransactionType::Deposit => {
                let details = TransactionDetails {
                    record: record.clone(),
                    state: TransactionState::Normal,
                };
                transactions.insert(transaction_id, details);
                if let Err(e) = account.deposit(amount) {
                    eprintln!("Error processing deposit: {:?}", e);
                }
            }
            TransactionType::Withdrawal => {
                if let Err(e) = account.withdraw(amount) {
                    eprintln!("Error processing withdrawal: {:?}", e);
                }
            }
            TransactionType::Dispute => {
                if let Some(details) = transactions.get_mut(&transaction_id) {
                    if details.record.client != client_id {
                        continue;
                    }
                    if details.state == TransactionState::Normal {
                        if let Some(amount) = details.record.amount {
                            if let Err(e) = account.dispute(amount) {
                                eprintln!(
                                    "Could not dispute transaction {}: {}",
                                    transaction_id, e
                                );
                                continue;
                            }
                            details.state = TransactionState::Disputed;
                        }
                    }
                }
            }
            TransactionType::Resolve => {
                if let Some(details) = transactions.get_mut(&transaction_id) {
                    if details.record.client != client_id {
                        continue;
                    }
                    if details.state == TransactionState::Disputed {
                        if let Some(amount) = details.record.amount {
                            if let Err(e) = account.resolve(amount) {
                                eprintln!(
                                    "Could not resolve transaction {}: {}",
                                    transaction_id, e
                                );
                                continue;
                            }
                            details.state = TransactionState::Resolved;
                        }
                    }
                }
            }
            TransactionType::Chargeback => {
                if let Some(details) = transactions.get_mut(&transaction_id) {
                    if details.record.client != client_id {
                        continue;
                    }
                    if details.state == TransactionState::Disputed {
                        if let Some(amount) = details.record.amount {
                            if let Err(e) = account.chargeback(amount) {
                                eprintln!(
                                    "Could not chargeback transaction {}: {}",
                                    transaction_id, e
                                );
                                continue;
                            }
                            details.state = TransactionState::Chargeback;
                        }
                    }
                }
            }
        }
    }

    for account in accounts.values() {
        wtr.serialize(account)?;
    }
    wtr.flush()?;

    Ok(())
}
