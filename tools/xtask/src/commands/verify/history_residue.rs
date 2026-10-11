//! The `history-residue` gate: no program ids in code or code READMEs. Comments and READMEs say
//! what the code does now; bug-catalogue ids (`F-012`, `OF-020`), decision ids (`D25`) and
//! milestone ids (`M3`) belong in `documentation/` only.

#[cfg(test)]
#[path = "tests/history_residue.rs"]
mod tests;

use anyhow::Result;

use crate::support::repository::{self, Repository};

use super::gate::Violation;

/// The folders whose `.rs` files and `README.md` files are checked.
const CHECKED_FOLDERS: &[&str] = &["crates/", "tools/"];

/// The gate and its tests, which spell out sample ids.
const OWN_FILE_NAME: &str = "history_residue.rs";

/// Runs the gate.
pub(crate) fn check(repository: &Repository) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();
    for file in repository.files()? {
        let checked = CHECKED_FOLDERS
            .iter()
            .any(|folder| file.starts_with(folder))
            && (repository::has_extension(file, "rs")
                || repository::file_name(file) == "README.md");
        if !checked || repository::file_name(file) == OWN_FILE_NAME {
            continue;
        }
        let text = repository.read_text(file)?;
        for (index, line) in text.lines().enumerate() {
            if let Some(id) = line.split(is_separator).find(|word| is_program_id(word)) {
                violations.push(Violation::new(
                    file,
                    index + 1,
                    format!(
                        "`{id}` is a program id: say what the code does now; ids live in documentation/"
                    ),
                ));
            }
        }
    }
    Ok(violations)
}

/// Whether a character ends a word: anything but a letter, a digit, `_` or `-`.
fn is_separator(character: char) -> bool {
    !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
}

/// Whether a whole word is a program id: `F-` or `OF-` with three digits, `D` with one or two
/// digits, or `M` with one digit.
pub(crate) fn is_program_id(word: &str) -> bool {
    let all_digits = |digits: &str, lengths: &[usize]| {
        lengths.contains(&digits.len()) && digits.bytes().all(|byte| byte.is_ascii_digit())
    };
    if let Some(digits) = word.strip_prefix("OF-").or_else(|| word.strip_prefix("F-")) {
        return all_digits(digits, &[3]);
    }
    if let Some(digits) = word.strip_prefix('D') {
        return all_digits(digits, &[1, 2]);
    }
    word.strip_prefix('M')
        .is_some_and(|digits| all_digits(digits, &[1]))
}
