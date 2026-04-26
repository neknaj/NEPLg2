//! Minimal nm parser for editor/LSP use.
//!
//! This keeps document comments as structured data inside the Rust compiler
//! pipeline so hover and documentation tooling can consume the same parsed
//! representation instead of re-parsing in JavaScript.

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub children: Vec<BlockNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockNode {
    Section(SectionNode),
    Paragraph(ParagraphNode),
    List(ListNode),
    Code(CodeBlock),
    Hr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionNode {
    pub level: usize,
    pub heading: Vec<InlineNode>,
    pub children: Vec<BlockNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphNode {
    pub inlines: Vec<InlineNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListNode {
    pub items: Vec<Vec<InlineNode>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    pub lang: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineNode {
    Text(String),
    CodeInline(String),
    Math {
        display: bool,
        text: String,
    },
    Ruby {
        base: Vec<InlineNode>,
        ruby: Vec<InlineNode>,
    },
    Gloss {
        base: Vec<InlineNode>,
        notes: Vec<Vec<InlineNode>>,
    },
    Link {
        text: Vec<InlineNode>,
        href: String,
    },
}

pub fn parse_document(source: &str) -> Document {
    let lines = normalize_lines(source);
    let mut root = Container::Document(Document {
        children: Vec::new(),
    });
    let mut path: Vec<usize> = Vec::new();
    let mut levels: Vec<usize> = Vec::new();
    let mut i = 0usize;

    while i < lines.len() {
        let line = &lines[i];

        if let Some(fence) = fence_start(line) {
            let mut j = i + 1;
            let mut code_lines = Vec::new();
            while j < lines.len() && !is_fence_end(&lines[j]) {
                code_lines.push(lines[j].clone());
                j += 1;
            }
            if j < lines.len() && is_fence_end(&lines[j]) {
                j += 1;
            }
            current_children_mut(&mut root, &path).push(BlockNode::Code(CodeBlock {
                lang: fence,
                text: join_lines(&code_lines),
            }));
            i = j;
            continue;
        }

        if let Some((level, text)) = heading_line(line) {
            while levels.last().is_some_and(|last| *last >= level) {
                levels.pop();
                path.pop();
            }
            let children = current_children_mut(&mut root, &path);
            let index = children.len();
            children.push(BlockNode::Section(SectionNode {
                level,
                heading: parse_inlines(&text),
                children: Vec::new(),
            }));
            levels.push(level);
            path.push(index);
            i += 1;
            continue;
        }

        if line == "---" {
            current_children_mut(&mut root, &path).push(BlockNode::Hr);
            if !path.is_empty() {
                path.pop();
                levels.pop();
            }
            i += 1;
            continue;
        }

        if line == ";;;" {
            if !path.is_empty() {
                path.pop();
                levels.pop();
            }
            i += 1;
            continue;
        }

        if is_hr_only(line) {
            current_children_mut(&mut root, &path).push(BlockNode::Hr);
            i += 1;
            continue;
        }

        if line.trim().is_empty() {
            i += 1;
            continue;
        }

        if line.strip_prefix("- ").is_some() {
            let mut items = Vec::new();
            let mut j = i;
            while j < lines.len() {
                if let Some(item) = lines[j].strip_prefix("- ") {
                    items.push(parse_inlines(item));
                    j += 1;
                } else {
                    break;
                }
            }
            current_children_mut(&mut root, &path).push(BlockNode::List(ListNode { items }));
            i = j;
            continue;
        }

        let mut para = Vec::new();
        let mut j = i;
        while j < lines.len() {
            let next = &lines[j];
            if next.trim().is_empty()
                || fence_start(next).is_some()
                || heading_line(next).is_some()
                || next == "---"
                || next == ";;;"
                || is_hr_only(next)
                || next.starts_with("- ")
            {
                break;
            }
            para.push(next.clone());
            j += 1;
        }
        current_children_mut(&mut root, &path).push(BlockNode::Paragraph(ParagraphNode {
            inlines: parse_inlines(&join_lines(&para)),
        }));
        i = j;
    }

    let Container::Document(doc) = root;
    doc
}

pub fn parse_inlines(text: &str) -> Vec<InlineNode> {
    let normalized = text.replace("\\n", "\n");
    let chars: Vec<char> = normalized.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;

    while i < chars.len() {
        if starts_with_chars(&chars, i, &['$', '$']) {
            if let Some(end) = find_subsequence(&chars, i + 2, &['$', '$']) {
                push_inline(
                    &mut out,
                    InlineNode::Math {
                        display: true,
                        text: chars[i + 2..end].iter().collect(),
                    },
                );
                i = end + 2;
                continue;
            }
        }

        if chars[i] == '$' {
            if let Some(end) = find_char(&chars, i + 1, '$') {
                push_inline(
                    &mut out,
                    InlineNode::Math {
                        display: false,
                        text: chars[i + 1..end].iter().collect(),
                    },
                );
                i = end + 1;
                continue;
            }
        }

        if chars[i] == '`' {
            if let Some(end) = find_char(&chars, i + 1, '`') {
                push_inline(
                    &mut out,
                    InlineNode::CodeInline(chars[i + 1..end].iter().collect()),
                );
                i = end + 1;
                continue;
            }
        }

        if chars[i] == '{' {
            if let Some(end) = find_char(&chars, i + 1, '}') {
                let inner: String = chars[i + 1..end].iter().collect();
                let parts = split_gloss_parts(&inner);
                if parts.len() >= 2 {
                    push_inline(
                        &mut out,
                        InlineNode::Gloss {
                            base: parse_inlines(parts[0].trim()),
                            notes: parts[1..]
                                .iter()
                                .map(|part| parse_inlines(part.trim()))
                                .collect(),
                        },
                    );
                    i = end + 1;
                    continue;
                }
            }
        }

        if chars[i] == '[' {
            if let Some(end) = find_char(&chars, i + 1, ']') {
                let inner: String = chars[i + 1..end].iter().collect();
                if end + 1 < chars.len() && chars[end + 1] == '(' {
                    if let Some(close) = find_char(&chars, end + 2, ')') {
                        let href: String = chars[end + 2..close].iter().collect();
                        push_inline(
                            &mut out,
                            InlineNode::Link {
                                text: parse_inlines(&inner),
                                href,
                            },
                        );
                        i = close + 1;
                        continue;
                    }
                }

                if let Some(slash) = inner.find('/') {
                    push_inline(
                        &mut out,
                        InlineNode::Ruby {
                            base: parse_inlines(&inner[..slash]),
                            ruby: parse_inlines(&inner[slash + 1..]),
                        },
                    );
                    i = end + 1;
                    continue;
                }
            }
        }

        push_text_char(&mut out, chars[i]);
        i += 1;
    }

    out
}

pub fn render_document_markdown(document: &Document) -> String {
    let mut out = String::new();
    render_blocks_markdown(&document.children, &mut out);
    out.trim().to_string()
}

enum Container {
    Document(Document),
}

fn current_children_mut<'a>(root: &'a mut Container, path: &[usize]) -> &'a mut Vec<BlockNode> {
    match root {
        Container::Document(doc) => descend_children_mut(&mut doc.children, path),
    }
}

fn descend_children_mut<'a>(
    children: &'a mut Vec<BlockNode>,
    path: &[usize],
) -> &'a mut Vec<BlockNode> {
    if let Some((idx, rest)) = path.split_first() {
        let (_, tail) = children.split_at_mut(*idx);
        match tail.first_mut() {
            Some(BlockNode::Section(section)) => descend_children_mut(&mut section.children, rest),
            _ => panic!("invalid nm section path"),
        }
    } else {
        children
    }
}

fn normalize_lines(source: &str) -> Vec<String> {
    source
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .split('\n')
        .map(|line| line.to_string())
        .collect()
}

fn heading_line(line: &str) -> Option<(usize, String)> {
    let bytes = line.as_bytes();
    let mut level = 0usize;
    while level < bytes.len() && bytes[level] == b'#' {
        level += 1;
    }
    if level == 0 || level > 6 || bytes.get(level) != Some(&b' ') {
        return None;
    }
    Some((level, line[level + 1..].to_string()))
}

fn fence_start(line: &str) -> Option<String> {
    let rest = line.strip_prefix("```")?;
    if rest.contains('`') || rest.contains(' ') {
        if rest.trim().is_empty() {
            return Some(String::new());
        }
        return None;
    }
    Some(rest.to_string())
}

fn is_fence_end(line: &str) -> bool {
    line.trim() == "```"
}

fn is_hr_only(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 4 && trimmed.bytes().all(|b| b == b'-')
}

fn join_lines(lines: &[String]) -> String {
    let mut out = String::new();
    for (idx, line) in lines.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(line);
    }
    out
}

fn split_gloss_parts(inner: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut buf = String::new();
    let mut bracket_depth = 0usize;
    for ch in inner.chars() {
        match ch {
            '[' => {
                bracket_depth += 1;
                buf.push(ch);
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                buf.push(ch);
            }
            '/' if bracket_depth == 0 => {
                parts.push(buf);
                buf = String::new();
            }
            _ => buf.push(ch),
        }
    }
    parts.push(buf);
    parts
}

fn starts_with_chars(chars: &[char], start: usize, needle: &[char]) -> bool {
    chars.get(start..start + needle.len()) == Some(needle)
}

fn find_subsequence(chars: &[char], start: usize, needle: &[char]) -> Option<usize> {
    let mut i = start;
    while i + needle.len() <= chars.len() {
        if &chars[i..i + needle.len()] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn find_char(chars: &[char], start: usize, needle: char) -> Option<usize> {
    let mut i = start;
    while i < chars.len() {
        if chars[i] == needle {
            return Some(i);
        }
        i += 1;
    }
    None
}

fn push_inline(out: &mut Vec<InlineNode>, node: InlineNode) {
    out.push(node);
}

fn push_text_char(out: &mut Vec<InlineNode>, ch: char) {
    match out.last_mut() {
        Some(InlineNode::Text(text)) => text.push(ch),
        _ => out.push(InlineNode::Text(ch.to_string())),
    }
}

fn render_blocks_markdown(blocks: &[BlockNode], out: &mut String) {
    for (index, block) in blocks.iter().enumerate() {
        if index > 0 && !out.ends_with("\n\n") {
            if out.ends_with('\n') {
                out.push('\n');
            } else {
                out.push_str("\n\n");
            }
        }
        match block {
            BlockNode::Section(section) => render_section_markdown(section, out),
            BlockNode::Paragraph(paragraph) => {
                out.push_str(&render_inlines_markdown(&paragraph.inlines))
            }
            BlockNode::List(list) => {
                for item in &list.items {
                    out.push_str("- ");
                    out.push_str(&render_inlines_markdown(item));
                    out.push('\n');
                }
                if out.ends_with('\n') {
                    out.pop();
                }
            }
            BlockNode::Code(code) => {
                out.push_str("```");
                out.push_str(&code.lang);
                out.push('\n');
                out.push_str(&code.text);
                if !code.text.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```");
            }
            BlockNode::Hr => out.push_str("---"),
        }
    }
}

fn render_section_markdown(section: &SectionNode, out: &mut String) {
    for _ in 0..section.level.min(6) {
        out.push('#');
    }
    out.push(' ');
    out.push_str(&render_inlines_markdown(&section.heading));
    if !section.children.is_empty() {
        out.push_str("\n\n");
        render_blocks_markdown(&section.children, out);
    }
}

fn render_inlines_markdown(inlines: &[InlineNode]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            InlineNode::Text(text) => out.push_str(text),
            InlineNode::CodeInline(text) => {
                out.push('`');
                out.push_str(text);
                out.push('`');
            }
            InlineNode::Math { display, text } => {
                if *display {
                    out.push_str("$$");
                    out.push_str(text);
                    out.push_str("$$");
                } else {
                    out.push('$');
                    out.push_str(text);
                    out.push('$');
                }
            }
            InlineNode::Ruby { base, ruby } => {
                out.push('[');
                out.push_str(&render_inlines_markdown(base));
                out.push('/');
                out.push_str(&render_inlines_markdown(ruby));
                out.push(']');
            }
            InlineNode::Gloss { base, notes } => {
                out.push('{');
                out.push_str(&render_inlines_markdown(base));
                for note in notes {
                    out.push('/');
                    out.push_str(&render_inlines_markdown(note));
                }
                out.push('}');
            }
            InlineNode::Link { text, href } => {
                out.push('[');
                out.push_str(&render_inlines_markdown(text));
                out.push_str("](");
                out.push_str(href);
                out.push(')');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_heading_and_paragraph() {
        let doc = parse_document("# Title\n\nhello\n");
        assert!(matches!(doc.children.first(), Some(BlockNode::Section(_))));
    }

    #[test]
    fn parse_gloss_and_ruby() {
        let inlines = parse_inlines("{[日本語/にほんご]/Japanese} [漢字/かんじ]");
        assert!(inlines
            .iter()
            .any(|node| matches!(node, InlineNode::Gloss { .. })));
        assert!(inlines
            .iter()
            .any(|node| matches!(node, InlineNode::Ruby { .. })));
    }

    #[test]
    fn render_document_preserves_structure() {
        let doc = parse_document("# Title\n\n- a\n- b\n\n```neplg2\nfn main ():\n```\n");
        let rendered = render_document_markdown(&doc);
        assert!(rendered.contains("# Title"));
        assert!(rendered.contains("- a\n- b"));
        assert!(rendered.contains("```neplg2"));
    }
}
