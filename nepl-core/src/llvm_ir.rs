extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::LlvmIrBlock;

pub(crate) fn collect_defined_functions_from_llvmir_block(
    block: &LlvmIrBlock,
    out: &mut Vec<String>,
) {
    for line in &block.lines {
        if let Some(name) = parse_defined_function_name(line) {
            if !out.iter().any(|n| n == name) {
                out.push(name.to_string());
            }
        }
    }
}

pub(crate) fn parse_defined_function_name(line: &str) -> Option<&str> {
    parse_signature_function_name(line, true)
}

pub(crate) fn parse_declared_or_defined_function_name(line: &str) -> Option<&str> {
    parse_signature_function_name(line, false)
}

fn parse_signature_function_name(line: &str, define_only: bool) -> Option<&str> {
    let trimmed = line.trim_start();
    let is_define = trimmed.starts_with("define ");
    let is_declare = trimmed.starts_with("declare ");
    if define_only {
        if !is_define {
            return None;
        }
    } else if !is_define && !is_declare {
        return None;
    }
    let at = trimmed.find('@')?;
    let rest = &trimmed[(at + 1)..];
    let end = rest.find('(')?;
    let mut name = &rest[..end];
    if name.starts_with('"') && name.ends_with('"') && name.len() >= 2 {
        name = &name[1..name.len() - 1];
    }
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}
