use paymentengine::process_transactions;
use std::io::Cursor;

fn run_test_scenario(input_data: &str, expected_output: &str) {
    // Simulate reading from and writing to in-memory strings
    let input_cursor = Cursor::new(input_data);
    let mut output_cursor = Cursor::new(Vec::new());

    process_transactions(input_cursor, &mut output_cursor).unwrap();

    let output_string = String::from_utf8(output_cursor.into_inner()).unwrap();

    // Parse CSV strings into vectors of rows, excluding headers
    let mut actual_rows: Vec<&str> = output_string.lines().skip(1).collect();
    let mut expected_rows: Vec<&str> = expected_output.lines().skip(1).collect();

    // Sort the rows since the order is not guaranteed
    actual_rows.sort_unstable();
    expected_rows.sort_unstable();

    assert_eq!(
        actual_rows, expected_rows,
        "The actual output rows do not match the expected rows."
    );
}

#[test]
fn test_deposit_withdrawal() {
    let input_data = "\
type, client, tx, amount
deposit, 1, 1, 2.0
deposit, 2, 2, 3.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 2.5
withdrawal, 2, 5, 3.5";

    let expected_output = "\
client,available,held,total,locked
1,1.5,0,1.5,false
2,3,0,3,false";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_deposit_withdrawal_dispute_chargeback_resolve() {
    let input_data = "\
type,      client,tx,amount
deposit,   1,     1, 1.0
deposit,   2,     2, 2.0
deposit,   1,     3, 2
withdrawal,1,     4, 1.5
withdrawal,2,     5, 3.0
deposit,   3,     6, 37.0
dispute,   3,     6,
chargeback,3,     6,
deposit,   4,     7, 20
dispute,   4,     7,
resolve,   4,     7,";

    let expected_output = "\
client,available,held,total,locked
1,1.5,0,1.5,false
2,2,0,2,false
3,0,0,0,true
4,20,0,20,false
";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_resolve_and_chargeback_with_funds_scenario() {
    let input_data = "\
type,       client, tx, amount
deposit,    1,      1, 10.0
deposit,    1,      2, 5.0
deposit,    2,      3, 8.0
withdrawal, 1,      4, 3.0
dispute,    1,      1,
chargeback, 1,      1,
dispute,    2,      3,
resolve,    2,      3,
";

    let expected_output = "\
client,available,held,total,locked
1,2,0,2,true
2,8,0,8,false
";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_withdrawal_exceeding_balance() {
    let input_data = "\
type, client, tx, amount
deposit, 1, 1, 10.0
withdrawal, 1, 2, 15.0";

    let expected_output = "\
client,available,held,total,locked
1,10,0,10,false";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_locked_account_transactions() {
    let input_data = "\
type,       client, tx, amount
deposit,    1,      1, 100.0
dispute,    1,      1,
chargeback, 1,      1,
deposit,    1,      2, 50.0
withdrawal, 1,      3, 20.0";

    let expected_output = "\
client,available,held,total,locked
1,0,0,0,true";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_multiple_disputes_same_transaction() {
    let input_data = "\
type,       client, tx, amount
deposit,    1,      1, 100.0
deposit,    1,      2, 100.0
deposit,    1,      3, 100.0
dispute,    1,      1,
dispute,    1,      1,
dispute,    1,      1,
dispute,    1,      1,
dispute,    1,      2,
resolve,    1,      1,";

    let expected_output = "\
client,available,held,total,locked
1,200,100,300,false";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_dispute_nonexistent_transaction() {
    let input_data = "\
type,       client, tx, amount
deposit,    1,      1, 50.0
dispute,    1,      2,";

    let expected_output = "\
client,available,held,total,locked
1,50,0,50,false";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_incorrect_input() {
    let input_data = "\
type,        client, tx, amount
deposit,     1,      1, 20.0
incorrectly, 1,      2, 10.0
deposit,     1,      1, 20.0,
deposit,     1,      1, not a number
dispute,     1,      1
withdrawal,  1,      3, 5.0";

    let expected_output = "\
client,available,held,total,locked
1,15,0,15,false";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_unauthorized_transaction_updates() {
    let input_data = "\
type,       client, tx, amount
deposit,    1,      1, 100.0
deposit,    2,      2, 200.0
dispute,    2,      1,
chargeback, 2,      1,
resolve,    2,      1,
withdrawal, 2,      3, 50.0";

    let expected_output = "\
client,available,held,total,locked
1,100,0,100,false
2,150,0,150,false";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_partial_dispute_after_withdrawal() {
    let input_data = "\
type,       client, tx, amount
deposit,    1,      1, 10.0
withdrawal, 1,      2, 5.0
dispute,    1,      1
resolve,    1,      1,";

    // we are assuming that disputing a tx when there is not enough funds to cover it is not allowed
    let expected_output = "\
client,available,held,total,locked
1,5,0,5,false";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_successfully_dispute_after_failed_dispute_request() {
    let input_data = "\
type,       client, tx, amount
deposit,    1,      1, 10.0
withdrawal, 1,      2, 5.0
dispute,    1,      1,
deposit,    1,      3, 5.0
dispute,    1,      1,";

    let expected_output = "\
client,available,held,total,locked
1,0,10,10,false";

    run_test_scenario(input_data, expected_output);
}

#[test]
fn test_high_precision_transactions() {
    let input_data = "\
type,       client, tx, amount
deposit,    1,      1, 10.12345
deposit,    1,      2, 20.5678
withdrawal, 1,      3, 5.1234";

    let expected_output = "\
client,available,held,total,locked
1,25.5678,0,25.5678,false";

    run_test_scenario(input_data, expected_output);
}
