use anyhow::{Result, bail};
use clap::Parser;
use grit_msg::{ai, commit, git, secrets, ui};

const DEFAULT_MODEL: &str = "llama-3.3-70b-versatile";
const DEFAULT_DIFF_TOKEN_BUDGET: usize = 6_000;

#[derive(Parser, Debug)]
#[command(
    name = "grit-msg",
    version,
    about = "Generate commit messages with Groq"
)]
struct Args {
    #[arg(long, default_value = DEFAULT_MODEL)]
    model: String,
    #[arg(long, default_value_t = DEFAULT_DIFF_TOKEN_BUDGET)]
    diff_token_budget: usize,
    #[arg(long)]
    clear_aiapi_keys: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.clear_aiapi_keys {
        secrets::clear_stored_groq_keys()?;
        println!("Cleared stored Groq API keys from local storages.");
        println!(
            "Note: environment variable GROQ_API_KEY (if set) is not cleared by this command."
        );
        return Ok(());
    }

    let diff = git::staged_diff()?;
    if diff.trim().is_empty() {
        bail!("No staged changes found. Stage files before running this tool.");
    }

    let compact_diff = git::truncate_diff_for_tokens(&diff, args.diff_token_budget);
    let api_key = secrets::load_or_prompt_groq_key()?;
    let raw = ai::generate_commit_message(&api_key, &args.model, &compact_diff).await?;
    let message = commit::normalize_commit_message(&raw);

    ui::print_generated_message(&message);
    match ui::prompt_commit_action()? {
        ui::Action::Yes => commit::run_git_commit(&message)?,
        ui::Action::No => {}
        ui::Action::Copy => ui::copy_and_print_manual_command(&message)?,
    }

    Ok(())
}
