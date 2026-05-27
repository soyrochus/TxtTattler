//! Thin binary wrapper: parse args, hand off to the library, print errors prettily.

use clap::Parser;
use console::style;

use txttattler::cli::Cli;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Err(err) = txttattler::run(cli).await {
        // Print the whole error chain so the user sees the root cause.
        eprintln!("{} {}", style("✗ oops:").red().bold(), err);
        for cause in err.chain().skip(1) {
            eprintln!("  {} {}", style("↳").red(), cause);
        }
        std::process::exit(1);
    }
}
