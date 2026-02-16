use anyhow::{Context, Result, bail};
use arboard::Clipboard;
use colored::Colorize;
use dialoguer::Input;

use crate::commit::shell_escape_single_quoted;

pub enum Action {
    Yes,
    No,
    Copy,
}

pub fn print_generated_message(message: &str) {
    println!();
    println!("{}", "Generated commit message".green().bold());
    println!("{}", message.green().bold());
    println!();
}

pub fn prompt_commit_action() -> Result<Action> {
    let input = Input::<String>::new()
        .with_prompt("Commit these changes? [y/n/copy]")
        .interact_text()
        .context("failed to read commit confirmation input")?;

    match input.trim().to_ascii_lowercase().as_str() {
        "y" | "yes" => Ok(Action::Yes),
        "n" | "no" => Ok(Action::No),
        "copy" => Ok(Action::Copy),
        _ => bail!("Invalid input. Use y, n, or copy."),
    }
}

pub fn copy_and_print_manual_command(message: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("clipboard is unavailable")?;
    clipboard
        .set_text(message.to_owned())
        .context("failed to copy commit message to clipboard")?;

    let escaped = shell_escape_single_quoted(message);
    println!("{}", "Commit message copied to clipboard.".green().bold());
    println!(
        "{} {}",
        "Run this manually:".bold(),
        format!("git commit -m {escaped}").yellow()
    );
    Ok(())
}
