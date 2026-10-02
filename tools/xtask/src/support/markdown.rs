//! The small Markdown reader the documentation gates share: fences, headings, links, inline code
//! spans and heading anchors.
//!
//! **Role:** splits a Markdown file into what the gates read (headings and links outside code
//! fences, the lines of fenced blocks, inline code spans) without a full Markdown parser.
//!
//! **Position:** used by `readme-coverage`, `readme-sections`, `link-check` and `new-crate`; pure.
//!
//! **Signals and state:** none; pure functions.
//!
//! **Invariants:** line numbers are 1-based; text inside a fence is never read as a heading or a
//! link; a fence closes only with the same character at least as long as its opening.

#[cfg(test)]
#[path = "tests/markdown.rs"]
mod tests;

/// One line of a Markdown file and whether it lies inside a fenced code block (fence lines
/// themselves count as inside).
pub(crate) struct MarkdownLine<'text> {
    /// The 1-based line number.
    pub(crate) number: usize,
    /// The line without its line end.
    pub(crate) text: &'text str,
    /// Whether the line is a fence line or inside a fence.
    pub(crate) in_fence: bool,
}

/// A fenced code block.
pub(crate) struct FencedBlock<'text> {
    /// The info string after the opening fence (`text`, `rust`, `bash`).
    pub(crate) info: &'text str,
    /// The 1-based line number of the opening fence.
    pub(crate) opening_line: usize,
    /// The lines between the fences, with their 1-based numbers.
    pub(crate) lines: Vec<(usize, &'text str)>,
}

/// A heading outside the fences.
pub(crate) struct Heading<'text> {
    /// The number of `#` characters.
    pub(crate) level: usize,
    /// The heading text after the `#` characters.
    pub(crate) text: &'text str,
    /// The 1-based line number.
    pub(crate) line: usize,
}

/// The fence opening of a line: its character, its length and its info string.
fn fence_opening(line: &str) -> Option<(char, usize, &str)> {
    let trimmed = line.trim_start();
    let fence_char = trimmed.chars().next()?;
    if fence_char != '`' && fence_char != '~' {
        return None;
    }
    let length = trimmed
        .chars()
        .take_while(|character| *character == fence_char)
        .count();
    if length < 3 {
        return None;
    }
    let info = trimmed.get(length..).unwrap_or("").trim();
    if fence_char == '`' && info.contains('`') {
        return None;
    }
    Some((fence_char, length, info))
}

/// Every line, marked inside or outside the fences.
pub(crate) fn lines(text: &str) -> Vec<MarkdownLine<'_>> {
    let mut result = Vec::new();
    let mut open_fence: Option<(char, usize)> = None;
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        match open_fence {
            Some((fence_char, length)) => {
                let trimmed = line.trim();
                let closing_length = trimmed
                    .chars()
                    .take_while(|character| *character == fence_char)
                    .count();
                if closing_length >= length && closing_length == trimmed.chars().count() {
                    open_fence = None;
                }
                result.push(MarkdownLine {
                    number,
                    text: line,
                    in_fence: true,
                });
            }
            None => {
                if let Some((fence_char, length, _)) = fence_opening(line) {
                    open_fence = Some((fence_char, length));
                    result.push(MarkdownLine {
                        number,
                        text: line,
                        in_fence: true,
                    });
                } else {
                    result.push(MarkdownLine {
                        number,
                        text: line,
                        in_fence: false,
                    });
                }
            }
        }
    }
    result
}

/// Every fenced block, in order.
pub(crate) fn fenced_blocks(text: &str) -> Vec<FencedBlock<'_>> {
    let mut blocks = Vec::new();
    let mut current: Option<(char, usize, FencedBlock<'_>)> = None;
    for (index, line) in text.lines().enumerate() {
        let number = index + 1;
        if let Some((fence_char, length, mut block)) = current.take() {
            let trimmed = line.trim();
            let closing_length = trimmed
                .chars()
                .take_while(|character| *character == fence_char)
                .count();
            if closing_length >= length && closing_length == trimmed.chars().count() {
                blocks.push(block);
            } else {
                block.lines.push((number, line));
                current = Some((fence_char, length, block));
            }
        } else if let Some((fence_char, length, info)) = fence_opening(line) {
            let block = FencedBlock {
                info,
                opening_line: number,
                lines: Vec::new(),
            };
            current = Some((fence_char, length, block));
        }
    }
    if let Some((_, _, block)) = current {
        blocks.push(block);
    }
    blocks
}

/// Every ATX heading outside the fences.
pub(crate) fn headings(text: &str) -> Vec<Heading<'_>> {
    lines(text)
        .into_iter()
        .filter(|line| !line.in_fence)
        .filter_map(|line| {
            let level = line
                .text
                .chars()
                .take_while(|character| *character == '#')
                .count();
            let rest = line.text.get(level..)?;
            (level > 0 && level <= 6 && rest.starts_with(' ')).then(|| Heading {
                level,
                text: rest.trim(),
                line: line.number,
            })
        })
        .collect()
}

/// The inline code spans of one line, without their backticks.
pub(crate) fn inline_code_spans(line: &str) -> Vec<&str> {
    let mut spans = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('`') {
        let after_start = rest.get(start..).unwrap_or("");
        let ticks = after_start
            .chars()
            .take_while(|character| *character == '`')
            .count();
        let body_start = start + ticks;
        let delimiter = "`".repeat(ticks);
        let Some(body) = rest.get(body_start..) else {
            break;
        };
        let Some(end) = body.find(&delimiter) else {
            break;
        };
        if let Some(span) = body.get(..end) {
            spans.push(span.trim());
        }
        rest = body.get(end + ticks..).unwrap_or("");
    }
    spans
}

/// The line with every inline code span replaced by spaces, so links inside code are skipped.
pub(crate) fn without_inline_code(line: &str) -> String {
    let mut result = String::with_capacity(line.len());
    let mut inside: Option<usize> = None;
    let chars: Vec<char> = line.chars().collect();
    let mut index = 0;
    while let Some(&character) = chars.get(index) {
        if character == '`' {
            let ticks = chars.get(index..).map_or(0, |rest| {
                rest.iter()
                    .take_while(|candidate| **candidate == '`')
                    .count()
            });
            match inside {
                None if has_closing_ticks(&chars, index + ticks, ticks) => inside = Some(ticks),
                Some(open) if open == ticks => inside = None,
                _ => {}
            }
            result.push_str(&" ".repeat(ticks));
            index += ticks;
            continue;
        }
        result.push(if inside.is_some() { ' ' } else { character });
        index += 1;
    }
    result
}

fn has_closing_ticks(chars: &[char], from: usize, ticks: usize) -> bool {
    let mut index = from;
    while let Some(&character) = chars.get(index) {
        if character == '`' {
            let run = chars.get(index..).map_or(0, |rest| {
                rest.iter()
                    .take_while(|candidate| **candidate == '`')
                    .count()
            });
            if run == ticks {
                return true;
            }
            index += run;
        } else {
            index += 1;
        }
    }
    false
}

/// The targets of every inline link `[text](target)` on a line outside code spans, with the
/// title stripped.
pub(crate) fn link_targets(line: &str) -> Vec<String> {
    let plain = without_inline_code(line);
    let mut targets = Vec::new();
    let mut rest = plain.as_str();
    while let Some(position) = rest.find("](") {
        let after = rest.get(position + 2..).unwrap_or("");
        let Some(end) = after.find(')') else {
            break;
        };
        let inside = after.get(..end).unwrap_or("").trim();
        let target = inside.split_whitespace().next().unwrap_or("");
        let target = target.trim_start_matches('<').trim_end_matches('>');
        if !target.is_empty() {
            targets.push(target.to_owned());
        }
        rest = after.get(end..).unwrap_or("");
    }
    targets
}

/// The anchor GitHub gives a heading: lowercase, punctuation dropped, spaces as hyphens.
pub(crate) fn heading_anchor(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter_map(|character| match character {
            ' ' => Some('-'),
            '-' | '_' => Some(character),
            _ if character.is_alphanumeric() => Some(character),
            _ => None,
        })
        .collect()
}

/// Every anchor a file's headings provide, with GitHub's `-1`, `-2` suffixes for repeats.
pub(crate) fn heading_anchors(text: &str) -> Vec<String> {
    let mut anchors: Vec<String> = Vec::new();
    for heading in headings(text) {
        let base = heading_anchor(heading.text);
        let repeats = anchors
            .iter()
            .filter(|anchor| {
                **anchor == base
                    || anchor
                        .strip_prefix(&base)
                        .and_then(|suffix| suffix.strip_prefix('-'))
                        .is_some_and(|number| number.chars().all(|digit| digit.is_ascii_digit()))
            })
            .count();
        if repeats == 0 {
            anchors.push(base);
        } else {
            anchors.push(format!("{base}-{repeats}"));
        }
    }
    anchors
}
