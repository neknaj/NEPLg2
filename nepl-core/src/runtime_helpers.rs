#![allow(dead_code)]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;

pub const ALLOC_RUNTIME_ABI: &str = "__nepl_rt_alloc";
pub const DEALLOC_RUNTIME_ABI: &str = "__nepl_rt_dealloc";
pub const REALLOC_RUNTIME_ABI: &str = "__nepl_rt_realloc";

pub const ALLOC_CANDIDATES: &[&str] = &[ALLOC_RUNTIME_ABI, "alloc_raw", "alloc"];
pub const DEALLOC_CANDIDATES: &[&str] = &[DEALLOC_RUNTIME_ABI, "dealloc_raw", "dealloc"];
pub const REALLOC_CANDIDATES: &[&str] = &[REALLOC_RUNTIME_ABI, "realloc_raw", "realloc"];

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

pub fn runtime_abi_name(kind: RuntimeHelperKind) -> &'static str {
    match kind {
        RuntimeHelperKind::Alloc => ALLOC_RUNTIME_ABI,
        RuntimeHelperKind::Dealloc => DEALLOC_RUNTIME_ABI,
        RuntimeHelperKind::Realloc => REALLOC_RUNTIME_ABI,
    }
}

pub fn helper_base_name(name: &str) -> &str {
    let tail = crate::qualified_name::member_tail(name);
    let suffix_pos = [ALLOC_RUNTIME_ABI, DEALLOC_RUNTIME_ABI, REALLOC_RUNTIME_ABI]
        .iter()
        .find_map(|abi| {
            let rest = tail.strip_prefix(*abi)?;
            if rest.starts_with("__") {
                Some(abi.len())
            } else {
                None
            }
        })
        .or_else(|| tail.find("__").filter(|pos| *pos > 0));
    if let Some(pos) = suffix_pos {
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
        assert_eq!(helper_base_name("__nepl_rt_alloc"), "__nepl_rt_alloc");
        assert_eq!(
            helper_base_name("::core/mem::__nepl_rt_alloc__i32"),
            "__nepl_rt_alloc"
        );
        assert_eq!(
            helper_base_name("::core/mem::__nepl_rt_alloc__i32__i32__pure"),
            "__nepl_rt_alloc"
        );
    }

    #[test]
    fn find_runtime_helper_key_prefers_compiler_runtime_abi() {
        let mut map = BTreeMap::new();
        map.insert(String::from("alloc_raw"), 1u32);
        map.insert(String::from("alloc"), 2u32);
        map.insert(String::from("__nepl_rt_alloc"), 3u32);
        let found = find_runtime_helper_key(&map, RuntimeHelperKind::Alloc);
        assert_eq!(found, Some("__nepl_rt_alloc"));

        let mut namespaced = BTreeMap::new();
        namespaced.insert(String::from("::core/mem::alloc_raw"), 1u32);
        namespaced.insert(String::from("::core/mem::__nepl_rt_alloc"), 2u32);
        let found_namespaced = find_runtime_helper_key(&namespaced, RuntimeHelperKind::Alloc);
        assert_eq!(found_namespaced, Some("::core/mem::__nepl_rt_alloc"));

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
        map.insert(String::from("__nepl_rt_alloc"), 6u32);
        map.insert(String::from("current"), 4u32);
        let idx = find_runtime_helper_index(&map, RuntimeHelperKind::Alloc, Some("current"));
        assert_eq!(idx, Some(6u32));
    }

    #[test]
    fn find_runtime_helper_index_falls_back_when_current_is_abi_helper() {
        let mut map = BTreeMap::new();
        map.insert(String::from("__nepl_rt_alloc"), 4u32);
        map.insert(String::from("::core/mem::alloc_raw__i32"), 5u32);
        let idx =
            find_runtime_helper_index(&map, RuntimeHelperKind::Alloc, Some("__nepl_rt_alloc"));
        assert_eq!(idx, Some(5u32));
    }

    #[test]
    fn runtime_abi_name_is_stable_for_each_helper_kind() {
        assert_eq!(
            runtime_abi_name(RuntimeHelperKind::Alloc),
            "__nepl_rt_alloc"
        );
        assert_eq!(
            runtime_abi_name(RuntimeHelperKind::Dealloc),
            "__nepl_rt_dealloc"
        );
        assert_eq!(
            runtime_abi_name(RuntimeHelperKind::Realloc),
            "__nepl_rt_realloc"
        );
    }
}
