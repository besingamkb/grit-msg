use grit_msg::git::truncate_diff_for_tokens;

fn sample_diff() -> String {
    r#"diff --git a/src/main.rs b/src/main.rs
index 1111111..2222222 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,2 +1,4 @@
 fn main() {
-    println!("old");
+    println!("new");
+    println!("line2");
 }
diff --git a/src/lib.rs b/src/lib.rs
index 3333333..4444444 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,3 @@
-pub fn old() {}
+pub fn new() {}
+pub fn another() {}
"#
    .to_owned()
}

#[test]
fn keeps_full_diff_when_within_budget() {
    let diff = sample_diff();
    let out = truncate_diff_for_tokens(&diff, 10_000);
    assert_eq!(out, diff);
}

#[test]
fn truncation_includes_file_and_hunk_sections() {
    let diff = sample_diff();
    let out = truncate_diff_for_tokens(&diff, 80);

    assert!(out.contains("# Files changed:"));
    assert!(out.contains("# Hunk summaries:"));
    assert!(out.contains("diff --git a/src/main.rs b/src/main.rs"));
    assert!(out.contains("@@ -1,2 +1,4 @@"));
}

#[test]
fn truncation_caps_changed_lines_per_file() {
    let mut diff = String::from(
        "diff --git a/src/big.rs b/src/big.rs\n\
         @@ -1,1 +1,1 @@\n",
    );
    for i in 0..50 {
        diff.push_str(&format!("+added line {i}\n"));
    }

    let out = truncate_diff_for_tokens(&diff, 1);
    let kept_adds = out
        .lines()
        .filter(|line| line.starts_with("+added line"))
        .count();
    assert!(kept_adds <= 30);
}

#[test]
fn handles_non_diff_text_without_panicking() {
    let input = "some random text without diff markers";
    let out = truncate_diff_for_tokens(input, 1);
    assert!(out.contains("# Files changed:"));
    assert!(out.contains("# Hunk summaries:"));
}
