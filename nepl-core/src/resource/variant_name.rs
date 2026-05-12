extern crate alloc;

use alloc::string::String;

use super::model::ResourceMatchPattern;

pub(super) fn variant_name_tail(variant: &str) -> &str {
    variant.rsplit("::").next().unwrap_or(variant)
}

pub(super) fn normalize_variant_name(variant: &str) -> String {
    String::from(variant_name_tail(variant))
}

pub(super) fn variant_names_match(left: &str, right: &str) -> bool {
    variant_name_tail(left) == variant_name_tail(right)
}

pub(super) fn match_pattern_variant_name(pattern: &ResourceMatchPattern) -> Option<String> {
    let ResourceMatchPattern::Variant(variant) = pattern else {
        return None;
    };
    Some(normalize_variant_name(variant))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variant_name_tail_strips_qualified_path() {
        assert_eq!(variant_name_tail("Result::Ok"), "Ok");
        assert_eq!(variant_name_tail("Ok"), "Ok");
    }

    #[test]
    fn variant_names_match_compares_canonical_tail() {
        assert!(variant_names_match("Result::Ok", "Ok"));
        assert!(variant_names_match("Result<i32,str>::Ok", "Result::Ok"));
        assert!(!variant_names_match("Result::Ok", "Result::Err"));
    }
}
