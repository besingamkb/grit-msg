use grit_msg::commit::{
    WATERMARK_LINE, normalize_commit_message, shell_escape_single_quoted, with_optional_watermark,
};

#[test]
fn keeps_header_word_boundary() {
    let input = "refactor(forms): remove unused controller and registry wiring";
    let normalized = normalize_commit_message(input);
    assert!(!normalized.ends_with('c'));
    assert!(normalized.chars().count() <= 50);
}

#[test]
fn drops_long_scope_before_subject_cut() {
    let input = "refactor(FormsTemplateController): remove unused controller methods";
    let normalized = normalize_commit_message(input);
    assert!(normalized.starts_with("refactor: "));
    assert!(normalized.chars().count() <= 50);
}

#[test]
fn wraps_body_to_72_chars() {
    let input = "feat(cli): improve output formatting\nThis body line is intentionally long so that the normalizer must wrap it at seventy two characters without breaking the final output format.";
    let normalized = normalize_commit_message(input);
    let mut lines = normalized.lines();
    let _header = lines.next().expect("header line");
    for line in lines {
        if !line.trim().is_empty() {
            assert!(line.chars().count() <= 72);
        }
    }
}

#[test]
fn strips_code_fences_from_model_output() {
    let input = "```fix(cli): trim trailing whitespace```";
    let normalized = normalize_commit_message(input);
    assert_eq!(normalized, "fix(cli): trim trailing whitespace");
}

#[test]
fn escapes_single_quotes_for_shell_command() {
    let msg = "fix: don't break user's setup";
    let escaped = shell_escape_single_quoted(msg);
    assert_eq!(escaped, "'fix: don'\"'\"'t break user'\"'\"'s setup'");
}

#[test]
fn does_not_append_watermark_when_disabled() {
    let msg = "fix(cli): tighten validation";
    let out = with_optional_watermark(msg, false);
    assert_eq!(out, msg);
}

#[test]
fn appends_watermark_on_new_line_when_enabled() {
    let msg = "fix(cli): tighten validation\n\nKeep parser strict.";
    let out = with_optional_watermark(msg, true);
    assert!(out.ends_with(WATERMARK_LINE));
    assert!(out.contains("\n\nby grit-msg "));
}
