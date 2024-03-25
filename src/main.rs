use paymentengine::process_transactions;
use std::env;
use std::error::Error;
use std::io::stdout;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- transactions.csv > accounts.csv");
        std::process::exit(1);
    }
    let filepath = &args[1];
    let file = std::fs::File::open(filepath)?;
    let stdout = stdout();
    let mut handle = stdout.lock();

    process_transactions(file, &mut handle)?;

    Ok(())
}
