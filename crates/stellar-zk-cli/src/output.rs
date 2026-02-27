//! Terminal output formatting for the stellar-zk CLI.
//!
//! Provides consistent, colored output using the [`console`] crate.

use console::style;

/// Print a bold cyan header with an underline separator.
pub fn print_header(text: &str) {
    println!("\n{}", style(text).bold().cyan());
    println!("{}", style("=".repeat(text.len())).dim());
}

/// Print a success message prefixed with green `[OK]`.
pub fn print_success(text: &str) {
    println!("{} {}", style("[OK]").green().bold(), text);
}

/// Print a warning message prefixed with yellow `[WARN]`.
pub fn print_warning(text: &str) {
    println!("{} {}", style("[WARN]").yellow().bold(), text);
}

/// Print an error message prefixed with red `[ERROR]`.
pub fn print_error(text: &str) {
    println!("{} {}", style("[ERROR]").red().bold(), text);
}

/// Print a progress step indicator like `[1/3] Compiling circuit...`.
pub fn print_step(step: u32, total: u32, text: &str) {
    println!(
        "{} {}",
        style(format!("[{step}/{total}]")).dim(),
        text
    );
}

/// Print a key-value pair with dimmed key formatting.
pub fn print_key_value(key: &str, value: &str) {
    println!("  {}: {}", style(key).dim(), value);
}
