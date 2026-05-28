#![allow(dead_code)]

extern crate alloc;

use alloc::string::{String, ToString};

// この module は store/hit 実装の直前に key 形状を固定する staging module である。
// function body hash と source capability policy hash を compiler pipeline から渡すまで
// 実 cache path へ接続しないため、module 全体の未使用 warning はここで局所的に抑止する。

/// Resource summary value cache の value 単位 key。
///
/// module-level namespace key だけでは、同じ compile namespace 内のどの関数・どの
/// summary kind の証明かを区別できない。この key は store/hit 実装に進む前の
/// safety boundary として、stale hit に関係する入力を field として分けて保持する。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ResourceSummaryValueCacheKey {
    namespace_hash: u64,
    function_identity: ResourceSummaryFunctionIdentity,
    function_body_hash: u64,
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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ResourceSummaryFunctionIdentity {
    canonical_symbol: String,
    origin_name: String,
}

/// Resource summary value cache が区別する summary kind と format version。
///
/// stable mirror の構造が変わる場合は、既存 key と衝突しないように kind tag を
/// 増やす。初期 MVP は collection slot の `DropTraversal + ForallInitializedRange`
/// だけを対象にする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ResourceSummaryValueKind {
    CollectionSlotDropTraversalForallV1,
}

impl ResourceSummaryValueCacheKey {
    pub(super) fn new_drop_traversal_forall(
        namespace_hash: u64,
        function_identity: ResourceSummaryFunctionIdentity,
        function_body_hash: u64,
        type_parameter_boundary_hash: u64,
        generic_type_argument_hash: u64,
        source_capability_policy_hash: u64,
    ) -> Self {
        let summary_kind = ResourceSummaryValueKind::CollectionSlotDropTraversalForallV1;
        let stable_hash = resource_summary_value_cache_key_hash(
            namespace_hash,
            &function_identity,
            function_body_hash,
            type_parameter_boundary_hash,
            generic_type_argument_hash,
            source_capability_policy_hash,
            summary_kind,
        );
        Self {
            namespace_hash,
            function_identity,
            function_body_hash,
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

    #[cfg(test)]
    fn canonical_symbol(&self) -> &str {
        &self.canonical_symbol
    }
}

fn resource_summary_value_cache_key_hash(
    namespace_hash: u64,
    function_identity: &ResourceSummaryFunctionIdentity,
    function_body_hash: u64,
    type_parameter_boundary_hash: u64,
    generic_type_argument_hash: u64,
    source_capability_policy_hash: u64,
    summary_kind: ResourceSummaryValueKind,
) -> u64 {
    let mut hash = 0xcbf29ce484222325;
    resource_summary_value_hash_str(&mut hash, "neplg2-resource-summary-value-key-v1");
    resource_summary_value_hash_u64(&mut hash, namespace_hash);
    resource_summary_value_hash_str(&mut hash, &function_identity.canonical_symbol);
    resource_summary_value_hash_str(&mut hash, &function_identity.origin_name);
    resource_summary_value_hash_u64(&mut hash, function_body_hash);
    resource_summary_value_hash_u64(&mut hash, type_parameter_boundary_hash);
    resource_summary_value_hash_u64(&mut hash, generic_type_argument_hash);
    resource_summary_value_hash_u64(&mut hash, source_capability_policy_hash);
    resource_summary_value_hash_str(&mut hash, summary_kind.tag());
    hash
}

impl ResourceSummaryValueKind {
    fn tag(self) -> &'static str {
        match self {
            ResourceSummaryValueKind::CollectionSlotDropTraversalForallV1 => {
                "collection-slot-drop-traversal-forall-v1"
            }
        }
    }
}

fn resource_summary_value_hash_str(hash: &mut u64, value: &str) {
    resource_summary_value_hash_bytes(hash, value.as_bytes());
    resource_summary_value_hash_bytes(hash, &[0]);
}

fn resource_summary_value_hash_u64(hash: &mut u64, value: u64) {
    resource_summary_value_hash_bytes(hash, &value.to_le_bytes());
}

fn resource_summary_value_hash_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
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
        ResourceSummaryValueCacheKey::new_drop_traversal_forall(
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
}
