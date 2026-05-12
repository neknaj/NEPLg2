extern crate alloc;

use alloc::string::String;

use super::model::ResourceMatchPattern;

pub(super) fn normalize_variant_name(variant: &str) -> String {
    String::from(variant.rsplit("::").next().unwrap_or(variant))
}

pub(super) fn match_pattern_variant_name(pattern: &ResourceMatchPattern) -> Option<String> {
    let ResourceMatchPattern::Variant(variant) = pattern else {
        return None;
    };
    Some(normalize_variant_name(variant))
}
