use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    InsufficientFunds { client_id: u16 },
    InsufficientHeldFunds { client_id: u16 },
    AccountLocked { client_id: u16 },
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EngineError::InsufficientFunds { client_id } => {
                write!(f, "Insufficient funds for client {}", client_id)
            }
            EngineError::InsufficientHeldFunds { client_id } => {
                write!(f, "Insufficient funds held for client {}", client_id)
            }
            EngineError::AccountLocked { client_id } => {
                write!(f, "Account {} is locked", client_id)
            }
        }
    }
}

impl std::error::Error for EngineError {}
