#![allow(dead_code)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;

pub const ALLOC_CANDIDATES: &[&str] = &["alloc_raw", "alloc"];
pub const DEALLOC_CANDIDATES: &[&str] = &["dealloc", "dealloc_raw"];
pub const REALLOC_CANDIDATES: &[&str] = &["realloc", "realloc_raw"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeHelperKind {
    Alloc,
    Dealloc,
    Realloc,
}

pub fn helper_candidates(kind: RuntimeHelperKind) -> &'static [&'static str] {
    match kind {
        RuntimeHelperKind::Alloc => ALLOC_CANDIDATES,
        RuntimeHelperKind::Dealloc => DEALLOC_CANDIDATES,
        RuntimeHelperKind::Realloc => REALLOC_CANDIDATES,
    }
}

pub fn helper_base_name(name: &str) -> &str {
    let tail = if let Some(pos) = name.rfind("::") {
        &name[pos + 2..]
    } else {
        name
    };
    if let Some(pos) = tail.find("__") {
        &tail[..pos]
    } else {
        tail
    }
}

pub fn find_runtime_helper_key<'a, T>(
    map: &'a BTreeMap<String, T>,
    kind: RuntimeHelperKind,
) -> Option<&'a str> {
    for base in helper_candidates(kind) {
        if let Some((name, _)) = map.get_key_value(*base) {
            return Some(name.as_str());
        }
        for (name, _) in map {
            if helper_base_name(name.as_str()) == *base {
                return Some(name.as_str());
            }
        }
    }
    None
}

pub fn find_runtime_helper_index(
    name_map: &BTreeMap<String, u32>,
    kind: RuntimeHelperKind,
    current_func: Option<&str>,
) -> Option<u32> {
    let skip_idx = current_func.and_then(|n| name_map.get(n)).copied();
    for base in helper_candidates(kind) {
        if let Some(idx) = name_map.get(*base) {
            if Some(*idx) != skip_idx {
                return Some(*idx);
            }
        }
        for (name, idx) in name_map {
            if Some(*idx) == skip_idx {
                continue;
            }
            if helper_base_name(name.as_str()) == *base {
                return Some(*idx);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helper_base_name_handles_namespaced_and_mangled_symbols() {
        assert_eq!(helper_base_name("alloc"), "alloc");
        assert_eq!(helper_base_name("alloc__i32"), "alloc");
        assert_eq!(helper_base_name("::core/mem::alloc"), "alloc");
        assert_eq!(helper_base_name("::core/mem::alloc__i32"), "alloc");
    }

    #[test]
    fn find_runtime_helper_key_prefers_raw_allocator_for_internal_codegen() {
        let mut map = BTreeMap::new();
        map.insert(String::from("alloc_raw"), 1u32);
        map.insert(String::from("alloc"), 2u32);
        let found = find_runtime_helper_key(&map, RuntimeHelperKind::Alloc);
        assert_eq!(found, Some("alloc_raw"));

        let mut raw_only = BTreeMap::new();
        raw_only.insert(String::from("::core/mem::alloc_raw__i32"), 10u32);
        let found_raw = find_runtime_helper_key(&raw_only, RuntimeHelperKind::Alloc);
        assert_eq!(found_raw, Some("::core/mem::alloc_raw__i32"));
    }

    #[test]
    fn find_runtime_helper_index_skips_current_function_index() {
        let mut map = BTreeMap::new();
        map.insert(String::from("alloc"), 4u32);
        map.insert(String::from("::core/mem::alloc_raw__i32"), 5u32);
        map.insert(String::from("current"), 4u32);
        let idx = find_runtime_helper_index(&map, RuntimeHelperKind::Alloc, Some("current"));
        assert_eq!(idx, Some(5u32));
    }
}
