//! Small utilities and the famous ASCII tattler logo.

use console::style;

/// Print a delightful startup banner.
pub fn print_banner() {
    println!();
    println!(
        "{}",
        style("╔══════════════════════════════════════════════════════════════╗").cyan()
    );
    println!(
        "{} {} {}",
        style("║").cyan(),
        style("TXT TATTLER").bold().magenta(),
        style("— Your documents have never sounded so dramatic. ║").cyan()
    );
    println!(
        "{}",
        style("╚══════════════════════════════════════════════════════════════╝").cyan()
    );
    println!();
}

/// A cheeky little footer on success.
pub fn print_success_footer(result: &crate::domain::entities::TtsResult) {
    use console::style;

    println!();
    println!(
        "{} The tattler spilled {} chars across {} chunks using {} @ {}.",
        style("✨").bold(),
        style(result.total_chars).bold(),
        style(result.total_chunks).bold(),
        style(result.voice_used).yellow(),
        style(result.model_used).yellow()
    );

    if let Some(p) = &result.output_path {
        println!("   Final gossip saved to: {}", style(p.display()).green().bold());
    }
    println!("{}", style("   Thank you for choosing TxtTattler. Now go gossip responsibly.").dim());
    println!();
}
