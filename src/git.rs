use anyhow::{Context, Result, bail};
use std::process::Command;

const CHANGED_LINES_PER_FILE: usize = 30;

pub fn staged_diff() -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--no-color"])
        .output()
        .context("failed to run git diff --cached")?;

    if !output.status.success() {
        bail!("git diff --cached failed with status {}", output.status);
    }

    String::from_utf8(output.stdout).context("git diff output was not valid UTF-8")
}

pub fn truncate_diff_for_tokens(diff: &str, token_budget: usize) -> String {
    if estimate_tokens(diff) <= token_budget {
        return diff.to_owned();
    }

    let mut files = Vec::new();
    let mut hunks = Vec::new();

    for section in split_diff_sections(diff) {
        let mut file_header = None;
        let mut captured_hunks = Vec::new();
        let mut captured_changed_lines = 0usize;

        for line in section.lines() {
            if line.starts_with("diff --git ") {
                file_header = Some(line.to_owned());
                continue;
            }

            if line.starts_with("@@") {
                captured_hunks.push(line.to_owned());
                continue;
            }

            if (line.starts_with('+') || line.starts_with('-'))
                && !line.starts_with("+++")
                && !line.starts_with("---")
                && captured_changed_lines < CHANGED_LINES_PER_FILE
            {
                captured_hunks.push(line.to_owned());
                captured_changed_lines += 1;
            }
        }

        if let Some(header) = file_header {
            files.push(header.clone());
            if !captured_hunks.is_empty() {
                let mut summarized = String::new();
                summarized.push_str(&header);
                summarized.push('\n');
                summarized.push_str(&captured_hunks.join("\n"));
                hunks.push(summarized);
            }
        }
    }

    let mut out = String::from(
        "# Diff truncated for model context window.\n# Prioritize commit intent from file list and hunk summaries.\n\n# Files changed:\n",
    );

    for file in files {
        out.push_str(&file);
        out.push('\n');
    }

    out.push_str("\n# Hunk summaries:\n");
    for hunk in hunks {
        if estimate_tokens(&out) >= token_budget {
            break;
        }
        out.push_str(&hunk);
        out.push_str("\n\n");
    }

    out
}

fn split_diff_sections(diff: &str) -> impl Iterator<Item = String> + '_ {
    diff.split("\ndiff --git ")
        .enumerate()
        .filter_map(|(idx, chunk)| {
            if idx == 0 && chunk.trim().is_empty() {
                return None;
            }
            if idx == 0 && chunk.starts_with("diff --git ") {
                return Some(chunk.to_owned());
            }
            if idx == 0 {
                return Some(chunk.to_owned());
            }
            Some(format!("diff --git {chunk}"))
        })
}

fn estimate_tokens(input: &str) -> usize {
    // Conservative heuristic for mixed code + prose.
    input.chars().count().div_ceil(4)
}
