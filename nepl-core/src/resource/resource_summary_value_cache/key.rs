extern crate alloc;

use alloc::string::{String, ToString};

use super::super::model::ResourceFunction;
use super::stable_hash::ResourceSummaryStableHasher;

// この module は Resource summary value cache の key 形状を定義する。
// key は stale hit を避けるための invalidation 入力を field として保持し、保存 map の
// ordering と deterministic hash の両方に使う。

/// Resource summary value cache の value 単位 key。
///
/// module-level namespace key だけでは、同じ compile namespace 内のどの関数・どの
/// summary kind の証明かを区別できない。この key は store/hit 実装に進む前の
/// safety boundary として、stale hit に関係する入力を field として分けて保持する。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(super) struct ResourceSummaryValueCacheKey {
    namespace_hash: u64,
    function_identity: ResourceSummaryFunctionIdentity,
    function_body_hash: u64,
    dependency_closure_hash: u64,
    type_parameter_boundary_hash: u64,
    generic_type_argument_hash: u64,
    source_capability_policy_hash: u64,
    summary_kind: ResourceSummaryValueKind,
    stable_hash: u64,
}

/// Resource IR function を compile session 間で対応付けるための関数 identity。
///
/// `ResourceFunction.name` は monomorphize 後の symbol に近く、`origin_name` は元の
/// callable 境界を表す。将来 stdlib artifact へ進む段階では、この identity に
/// canonical module path / definition identity を含める。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(super) struct ResourceSummaryFunctionIdentity {
    canonical_symbol: String,
    origin_name: String,
}

/// Resource summary value cache が区別する summary kind と format version。
///
/// stable mirror の構造が変わる場合は、既存 key と衝突しないように kind tag を
/// 増やす。summary kind は「どの解析 stage のどの完結 leaf entry か」まで含め、
/// partial summary と complete summary が同じ key 空間で混ざらないようにする。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub(super) enum ResourceSummaryValueKind {
    CollectionSlotDropTraversalForallLeafEntryV1,
    I32ScalarReturnFactsEntryV1,
    InitializedFunctionCheckEntryV1,
    OwnerObligationCheckEntryV1,
    RawAliasReturnEntryV1,
    RawInitCompleteLeafEntryV1,
}

impl ResourceSummaryValueCacheKey {
    pub(super) fn namespace_hash(&self) -> u64 {
        self.namespace_hash
    }

    pub(super) fn new_drop_traversal_forall_leaf_entry(
        namespace_hash: u64,
        function_identity: ResourceSummaryFunctionIdentity,
        function_body_hash: u64,
        type_parameter_boundary_hash: u64,
        generic_type_argument_hash: u64,
        source_capability_policy_hash: u64,
    ) -> Self {
        let summary_kind = ResourceSummaryValueKind::CollectionSlotDropTraversalForallLeafEntryV1;
        let stable_hash = resource_summary_value_cache_key_hash(
            namespace_hash,
            &function_identity,
            function_body_hash,
            0,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
        );
        Self {
            namespace_hash,
            function_identity,
            function_body_hash,
            dependency_closure_hash: 0,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
            stable_hash,
        }
    }

    pub(super) fn new_raw_init_complete_leaf_entry(
        namespace_hash: u64,
        function_identity: ResourceSummaryFunctionIdentity,
        function_body_hash: u64,
        dependency_closure_hash: u64,
        type_parameter_boundary_hash: u64,
        generic_type_argument_hash: u64,
        source_capability_policy_hash: u64,
    ) -> Self {
        let summary_kind = ResourceSummaryValueKind::RawInitCompleteLeafEntryV1;
        let stable_hash = resource_summary_value_cache_key_hash(
            namespace_hash,
            &function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
        );
        Self {
            namespace_hash,
            function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
            stable_hash,
        }
    }

    pub(super) fn new_raw_alias_return_entry(
        namespace_hash: u64,
        function_identity: ResourceSummaryFunctionIdentity,
        function_body_hash: u64,
        dependency_closure_hash: u64,
        type_parameter_boundary_hash: u64,
        generic_type_argument_hash: u64,
        source_capability_policy_hash: u64,
    ) -> Self {
        let summary_kind = ResourceSummaryValueKind::RawAliasReturnEntryV1;
        let stable_hash = resource_summary_value_cache_key_hash(
            namespace_hash,
            &function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
        );
        Self {
            namespace_hash,
            function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
            stable_hash,
        }
    }

    pub(super) fn new_i32_scalar_return_facts_entry(
        namespace_hash: u64,
        function_identity: ResourceSummaryFunctionIdentity,
        function_body_hash: u64,
        dependency_closure_hash: u64,
        type_parameter_boundary_hash: u64,
        generic_type_argument_hash: u64,
        source_capability_policy_hash: u64,
    ) -> Self {
        let summary_kind = ResourceSummaryValueKind::I32ScalarReturnFactsEntryV1;
        let stable_hash = resource_summary_value_cache_key_hash(
            namespace_hash,
            &function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
        );
        Self {
            namespace_hash,
            function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
            stable_hash,
        }
    }

    pub(super) fn new_initialized_function_check_entry(
        namespace_hash: u64,
        function_identity: ResourceSummaryFunctionIdentity,
        function_body_hash: u64,
        dependency_closure_hash: u64,
        type_parameter_boundary_hash: u64,
        generic_type_argument_hash: u64,
        source_capability_policy_hash: u64,
    ) -> Self {
        let summary_kind = ResourceSummaryValueKind::InitializedFunctionCheckEntryV1;
        let stable_hash = resource_summary_value_cache_key_hash(
            namespace_hash,
            &function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
        );
        Self {
            namespace_hash,
            function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
            stable_hash,
        }
    }

    pub(super) fn new_owner_obligation_check_entry(
        namespace_hash: u64,
        function_identity: ResourceSummaryFunctionIdentity,
        function_body_hash: u64,
        dependency_closure_hash: u64,
        type_parameter_boundary_hash: u64,
        generic_type_argument_hash: u64,
        source_capability_policy_hash: u64,
    ) -> Self {
        let summary_kind = ResourceSummaryValueKind::OwnerObligationCheckEntryV1;
        let stable_hash = resource_summary_value_cache_key_hash(
            namespace_hash,
            &function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
        );
        Self {
            namespace_hash,
            function_identity,
            function_body_hash,
            dependency_closure_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
            stable_hash,
        }
    }

    #[cfg(test)]
    fn stable_hash(&self) -> u64 {
        self.stable_hash
    }

    #[cfg(test)]
    fn function_identity(&self) -> &ResourceSummaryFunctionIdentity {
        &self.function_identity
    }
}

impl ResourceSummaryFunctionIdentity {
    pub(super) fn new(canonical_symbol: &str, origin_name: &str) -> Option<Self> {
        if canonical_symbol.is_empty() || origin_name.is_empty() {
            return None;
        }
        Some(Self {
            canonical_symbol: canonical_symbol.to_string(),
            origin_name: origin_name.to_string(),
        })
    }

    pub(super) fn from_resource_function(function: &ResourceFunction) -> Option<Self> {
        let canonical_symbol = normalize_definition_span_mangle(&function.name);
        Self::new(&canonical_symbol, &function.origin_name)
    }

    #[cfg(test)]
    fn canonical_symbol(&self) -> &str {
        &self.canonical_symbol
    }

    pub(super) fn write_stable(&self, hash: &mut ResourceSummaryStableHasher) {
        hash.write_str(&self.canonical_symbol);
        hash.write_str(&self.origin_name);
    }
}

pub(super) fn normalize_definition_span_mangle(symbol: &str) -> String {
    let mut search_start = 0usize;
    while let Some(relative) = symbol[search_start..].find("__def") {
        let marker_start = search_start + relative;
        let digits_start = marker_start + "__def".len();
        if let Some(component_end) = definition_span_mangle_end(symbol, digits_start) {
            let mut normalized = String::new();
            normalized.push_str(&symbol[..marker_start]);
            normalized.push_str(&symbol[component_end..]);
            return normalized;
        }
        search_start = marker_start + "__def".len();
    }
    symbol.to_string()
}

fn definition_span_mangle_end(symbol: &str, mut cursor: usize) -> Option<usize> {
    cursor = consume_ascii_digits(symbol, cursor)?;
    cursor = consume_byte(symbol, cursor, b'_')?;
    cursor = consume_ascii_digits(symbol, cursor)?;
    cursor = consume_byte(symbol, cursor, b'_')?;
    cursor = consume_ascii_digits(symbol, cursor)?;
    if symbol[cursor..].starts_with("__") {
        Some(cursor)
    } else {
        None
    }
}

fn consume_ascii_digits(symbol: &str, mut cursor: usize) -> Option<usize> {
    let start = cursor;
    let bytes = symbol.as_bytes();
    while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
        cursor += 1;
    }
    if cursor == start {
        None
    } else {
        Some(cursor)
    }
}

fn consume_byte(symbol: &str, cursor: usize, expected: u8) -> Option<usize> {
    match symbol.as_bytes().get(cursor) {
        Some(actual) if *actual == expected => Some(cursor + 1),
        _ => None,
    }
}

fn resource_summary_value_cache_key_hash(
    namespace_hash: u64,
    function_identity: &ResourceSummaryFunctionIdentity,
    function_body_hash: u64,
    dependency_closure_hash: u64,
    type_parameter_boundary_hash: u64,
    generic_type_argument_hash: u64,
    source_capability_policy_hash: u64,
    summary_kind: ResourceSummaryValueKind,
) -> u64 {
    let mut hash = ResourceSummaryStableHasher::new("neplg2-resource-summary-value-key-v2");
    hash.write_u64(namespace_hash);
    function_identity.write_stable(&mut hash);
    hash.write_u64(function_body_hash);
    hash.write_u64(dependency_closure_hash);
    hash.write_u64(type_parameter_boundary_hash);
    hash.write_u64(generic_type_argument_hash);
    hash.write_u64(source_capability_policy_hash);
    hash.write_str(summary_kind.tag());
    hash.finish()
}

impl ResourceSummaryValueKind {
    fn tag(self) -> &'static str {
        match self {
            ResourceSummaryValueKind::CollectionSlotDropTraversalForallLeafEntryV1 => {
                "collection-slot-drop-traversal-forall-leaf-entry-v1"
            }
            ResourceSummaryValueKind::I32ScalarReturnFactsEntryV1 => {
                "i32-scalar-return-facts-entry-v1"
            }
            ResourceSummaryValueKind::InitializedFunctionCheckEntryV1 => {
                "initialized-function-check-entry-v1"
            }
            ResourceSummaryValueKind::OwnerObligationCheckEntryV1 => {
                "owner-obligation-check-entry-v1"
            }
            ResourceSummaryValueKind::RawAliasReturnEntryV1 => "raw-alias-return-entry-v1",
            ResourceSummaryValueKind::RawInitCompleteLeafEntryV1 => {
                "raw-init-complete-leaf-entry-v1"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn function_identity() -> ResourceSummaryFunctionIdentity {
        ResourceSummaryFunctionIdentity::new("std::vec::clear#i32", "clear")
            .expect("test identity should be valid")
    }

    fn key_with_parts(
        namespace_hash: u64,
        function_body_hash: u64,
        type_parameter_boundary_hash: u64,
        generic_type_argument_hash: u64,
        source_capability_policy_hash: u64,
    ) -> ResourceSummaryValueCacheKey {
        ResourceSummaryValueCacheKey::new_drop_traversal_forall_leaf_entry(
            namespace_hash,
            function_identity(),
            function_body_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
        )
    }

    #[test]
    fn resource_summary_value_cache_key_keeps_function_identity_structured() {
        let key = key_with_parts(1, 2, 3, 4, 5);

        assert_eq!(
            key.function_identity().canonical_symbol(),
            "std::vec::clear#i32"
        );
    }

    #[test]
    fn resource_summary_function_identity_strips_definition_span_mangle() {
        let identity = ResourceSummaryFunctionIdentity::new(
            &normalize_definition_span_mangle("add__def7_123_150__i32_i32__i32__pure"),
            "add",
        )
        .expect("normalized identity should be valid");

        assert_eq!(identity.canonical_symbol(), "add__i32_i32__i32__pure");
    }

    #[test]
    fn resource_summary_function_identity_keeps_non_span_def_text() {
        assert_eq!(
            normalize_definition_span_mangle("user__def_name__i32__i32__pure"),
            "user__def_name__i32__i32__pure"
        );
    }

    #[test]
    fn resource_summary_value_cache_key_tracks_invalidation_inputs() {
        let base = key_with_parts(1, 2, 3, 4, 5);

        assert_ne!(base, key_with_parts(9, 2, 3, 4, 5));
        assert_ne!(base, key_with_parts(1, 9, 3, 4, 5));
        assert_ne!(base, key_with_parts(1, 2, 9, 4, 5));
        assert_ne!(base, key_with_parts(1, 2, 3, 9, 5));
        assert_ne!(base, key_with_parts(1, 2, 3, 4, 9));
    }

    #[test]
    fn resource_summary_value_cache_key_rejects_empty_function_identity() {
        assert!(ResourceSummaryFunctionIdentity::new("", "clear").is_none());
        assert!(ResourceSummaryFunctionIdentity::new("std::vec::clear", "").is_none());
    }

    #[test]
    fn resource_summary_value_cache_key_hash_is_deterministic() {
        let first = key_with_parts(1, 2, 3, 4, 5);
        let second = key_with_parts(1, 2, 3, 4, 5);

        assert_eq!(first.stable_hash(), second.stable_hash());
    }

    #[test]
    fn resource_summary_value_cache_key_hash_has_fixed_golden_value() {
        let key = key_with_parts(1, 2, 3, 4, 5);

        assert_eq!(key.stable_hash(), 0x1e1a04999fb1863e);
    }
}
