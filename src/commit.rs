use anyhow::{Context, Result, bail};
use std::process::Command;

pub const SYSTEM_PROMPT: &str = r#"You are an expert Git commit assistant.

Output rules:
- Return ONLY the commit message text. No markdown, no code fences, no quotes.
- Use Conventional Commits format: <type>(optional-scope): <subject>
- Subject/header length MUST be <= 50 characters.
- Body lines MUST wrap at <= 72 characters.
- Use imperative mood.
- Describe what changed and why, not low-value implementation detail.
- If one line is enough, output one line only.
- Ignore lockfiles and generated files when determining intent.
- Prefer the smallest accurate type from: feat, fix, refactor, perf, docs, test, build, ci, chore.

Examples:
feat(cli): add staged diff size guard

Prevent oversized prompts sent to the AI backend.
Summarize hunks when the diff exceeds token budget.

fix(parser): handle empty hunk header

Avoid panic on malformed unified diff input and
return a structured validation error."#;

pub fn run_git_commit(message: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .context("failed to run git commit")?;

    if !status.success() {
        bail!("git commit failed with status {}", status);
    }
    Ok(())
}

pub fn shell_escape_single_quoted(input: &str) -> String {
    format!("'{}'", input.replace('\'', r#"'"'"'"#))
}

pub fn normalize_commit_message(raw: &str) -> String {
    let stripped = raw
        .trim()
        .strip_prefix("```")
        .unwrap_or(raw.trim())
        .strip_suffix("```")
        .unwrap_or(raw.trim())
        .trim()
        .to_owned();

    if let Some((first, rest)) = stripped.split_once('\n') {
        let header = shorten_header(first.trim(), 50);
        let body = wrap_body(rest.trim(), 72);
        if body.is_empty() {
            header
        } else {
            format!("{header}\n{body}")
        }
    } else {
        shorten_header(stripped.trim(), 50)
    }
}

fn shorten_header(header: &str, max: usize) -> String {
    if header.chars().count() <= max {
        return header.to_owned();
    }

    if let Some((prefix, subject)) = header.split_once(": ") {
        let scoped_prefix = format!("{prefix}: ");
        let scoped_remaining = max.saturating_sub(scoped_prefix.chars().count());
        if scoped_prefix.chars().count() <= max && scoped_remaining >= 20 {
            let trimmed_subject = truncate_words(subject, scoped_remaining);
            return format!("{scoped_prefix}{trimmed_subject}");
        }
    }

    if let Some((prefix, subject)) = header.split_once(": ")
        && let Some(scope_start) = prefix.find('(')
        && prefix.ends_with(')')
    {
        let commit_type = &prefix[..scope_start];
        let compact_prefix = format!("{commit_type}: ");
        if compact_prefix.chars().count() < max {
            let remaining = max - compact_prefix.chars().count();
            let trimmed_subject = truncate_words(subject, remaining);
            return format!("{compact_prefix}{trimmed_subject}");
        }
    }

    if let Some((prefix, subject)) = header.split_once(": ") {
        let scoped_prefix = format!("{prefix}: ");
        if scoped_prefix.chars().count() <= max {
            let remaining = max - scoped_prefix.chars().count();
            let trimmed_subject = truncate_words(subject, remaining);
            return format!("{scoped_prefix}{trimmed_subject}");
        }
    }

    truncate_words(header, max)
}

fn truncate_words(input: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }

    let mut out = String::new();
    for word in input.split_whitespace() {
        let candidate = if out.is_empty() {
            word.to_owned()
        } else {
            format!("{out} {word}")
        };

        if candidate.chars().count() > max {
            break;
        }
        out = candidate;
    }

    if !out.is_empty() {
        return out;
    }

    input.chars().take(max).collect()
}

fn wrap_body(body: &str, width: usize) -> String {
    if body.is_empty() {
        return String::new();
    }

    let mut paragraphs = Vec::new();
    for para in body.split("\n\n") {
        let words: Vec<&str> = para.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }

        let mut lines: Vec<String> = Vec::new();
        let mut line = String::new();
        for word in words {
            let candidate = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if candidate.chars().count() > width {
                if !line.is_empty() {
                    lines.push(line);
                }
                line = word.to_owned();
            } else {
                line = candidate;
            }
        }
        if !line.is_empty() {
            lines.push(line);
        }
        paragraphs.push(lines.join("\n"));
    }

    paragraphs.join("\n\n")
}
