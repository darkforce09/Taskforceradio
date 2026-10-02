//! Tests for the code-only view of Rust sources.

use super::*;

#[test]
fn comments_and_literal_contents_are_blanked() {
    let source = "let a = \"mod tests {\"; // unsafe {\n/* mod x { */ let b = 'x';\n";
    let lines = code_lines(source);
    assert_eq!(lines.len(), 2);
    assert!(!lines[0].contains("mod tests") && !lines[0].contains("unsafe"));
    assert!(lines[0].starts_with("let a = \""));
    assert!(!lines[1].contains("mod x"));
    assert!(lines[1].contains("let b ="));
}

#[test]
fn raw_strings_escapes_and_lifetimes_are_handled() {
    let source = "let r = r#\"unsafe { \"# ;\nlet e = \"\\\" unsafe {\";\nfn f<'a>(x: &'a str) { unsafe {} }\n";
    let lines = code_lines(source);
    assert!(!lines[0].contains("unsafe"));
    assert!(!lines[1].contains("unsafe"));
    assert!(lines[2].contains("unsafe {"));
}

#[test]
fn a_multi_line_string_stays_blank_on_every_line() {
    let lines = code_lines("const A: &str = \"first \\\n     mod tests {\";\nmod real;\n");
    assert!(!lines[1].contains("mod tests"));
    assert_eq!(lines[2], "mod real;");
}

#[test]
fn comment_lines_are_recognised() {
    assert!(is_comment_line("    // SAFETY: fine"));
    assert!(is_comment_line("//! doc"));
    assert!(!is_comment_line("let a = 1; // trailing"));
}
