# Simple Payment Engine

This payment engine is a toy I built during a mini hackathon a Sunday night.

It handles various types of transactions, including deposits, withrawals, disputes, resolutions, and chargebacks.

It's also got some basic error handling with it's custom `EngineError`.

## Getting Started

Rust and Cargo (latest stable version recommended). See [Rust's installation guide](https://www.rust-lang.org/tools/install) for instructions.

## Getting Started

1. Clone the repo `git clone https://example.com/toy-payments-engine.git && cd toy-payments-engine`
2. Run it with either `cargo run -- <your-file.csv>` or build with `cargo build --release`, then run it with the same argument.

## Input Format

The input CSV file should have the following cols, `type`, `client`, `tx`, and `amount`. See integration tests for inspiration.

## Output Format

The app outputs csv, which you can pipe to whichever app or redirect to file. E.g.

`cat many_transactions.csv | target/debug/paymentengine | vim -`
or
`cargo run -- transactions.csv > accounts.csv`

## Tests

You can run the test suite with `cargo test`.

## TODO

- [ ] Add unit tests
- [ ] Add benchmark testing (maybe with [criterion](https://docs.rs/criterion/latest/criterion/))
- [ ] Look into threading and/or async for web server. E.g. we could use one thread for reading, one for processing, one for writing, and use send the data between them through channels.
- [ ] Optimize memory usage.
- [ ] Add more logging
- [ ] Add dockerfile
- [ ] Set up cicd pipeline
