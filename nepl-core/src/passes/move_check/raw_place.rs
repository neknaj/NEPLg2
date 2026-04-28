use alloc::string::{String, ToString};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawPlaceState {
    Initialized,
    Moved,
    PossiblyMoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RawPlaceInfo {
    pub(super) state: RawPlaceState,
    pub(super) size: usize,
}

pub(super) fn format_raw_memory_place_key_parts(base: &str, offset: Option<i64>) -> String {
    match offset {
        Some(offset) => format_raw_memory_place_key(base, offset),
        None => format_raw_memory_unknown_offset_key(base),
    }
}

pub(super) fn combine_raw_memory_offsets(
    base_offset: Option<i64>,
    offset: Option<i64>,
) -> Option<i64> {
    match (base_offset, offset) {
        (Some(base_offset), Some(offset)) => Some(base_offset.saturating_add(offset)),
        _ => None,
    }
}

pub(super) fn parse_raw_memory_place_key(key: &str) -> (String, Option<i64>) {
    let Some((base, offset)) = key.rsplit_once('+') else {
        return (key.to_string(), Some(0));
    };
    if offset == "?" {
        return (base.to_string(), None);
    }
    match offset.parse::<i64>() {
        Ok(offset) => (base.to_string(), Some(offset)),
        Err(_) => (key.to_string(), Some(0)),
    }
}

pub(super) fn raw_place_ranges_overlap(
    left_key: &str,
    left_size: usize,
    right_key: &str,
    right_size: usize,
) -> bool {
    if left_size == 0 || right_size == 0 {
        return false;
    }
    let (left_base, left_offset) = parse_raw_memory_place_key(left_key);
    let (right_base, right_offset) = parse_raw_memory_place_key(right_key);
    if left_base != right_base {
        return false;
    }
    let (Some(left_offset), Some(right_offset)) = (left_offset, right_offset) else {
        return true;
    };
    let left_end = left_offset.saturating_add(left_size as i64);
    let right_end = right_offset.saturating_add(right_size as i64);
    left_offset < right_end && right_offset < left_end
}

pub(super) fn raw_place_key_has_unknown_offset(key: &str) -> bool {
    parse_raw_memory_place_key(key).1.is_none()
}

fn format_raw_memory_place_key(base: &str, offset: i64) -> String {
    if offset == 0 {
        base.to_string()
    } else {
        alloc::format!("{}+{}", base, offset)
    }
}

fn format_raw_memory_unknown_offset_key(base: &str) -> String {
    alloc::format!("{}+?", base)
}
