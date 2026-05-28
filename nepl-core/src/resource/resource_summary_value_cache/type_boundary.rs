extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::stable_hash::ResourceSummaryStableHasher;
use super::stable_type_key::ResourceSummaryStableTypeKey;

// この module は type parameter boundary と generic type argument を stable key 入力へ
// 変換する。summary value 内の generic variable を別 compile session へ対応付けられない
// 場合は、store/hit 候補から外すために `None` を返す。

/// Resource summary value key に入れる function-local type parameter boundary hash。
///
/// type parameter boundary は summary value 内の generic variable を現在の compile の
/// どの parameter へ再投影するかを決める境界である。そのため、一般の stable type key
/// ではなく、unbound かつ label 付きの type variable だけを許可する。hash は ordinal、
/// label、copy/clone/drop capability を含め、同じ stable parameter key の重複も拒否する。
pub(super) fn resource_summary_type_parameter_boundary_hash(
    types: &TypeCtx,
    type_params: &[TypeId],
) -> Option<u64> {
    let mut hash =
        ResourceSummaryStableHasher::new("neplg2-resource-summary-type-parameter-boundary-v1");
    let mut seen = BTreeSet::new();
    hash.write_usize(type_params.len());
    for (index, ty) in type_params.iter().enumerate() {
        let key = type_parameter_boundary_key(types, *ty)?;
        if !seen.insert(key.clone()) {
            return None;
        }
        hash.write_usize(index);
        hash.write_str(&key);
    }
    Some(hash.finish())
}

fn type_parameter_boundary_key(types: &TypeCtx, ty: TypeId) -> Option<String> {
    let TypeKind::Var(var) = types.get_ref(ty) else {
        return None;
    };
    if var.binding.is_some() {
        return None;
    }
    let label = var.label.as_deref()?;
    Some(format!(
        "var({label}:copy={}:clone={}:drop={})",
        var.copy_cap, var.clone_cap, var.drop_cap
    ))
}

/// Resource summary value key に入れる generic type argument hash。
///
/// call site の type argument は順序付きの concrete substitution であるため、重複する型
/// argument 自体は許可する。ただし各 `TypeId` は stable type key へ変換できる必要がある。
/// nominal type は namespace / public-surface hash と組み合わせる stable definition-shape
/// key へ変換できる場合だけ許可する。未解決 `Named` placeholder のように現在 compile の
/// 定義へ一意に再投影できない型は、`stable_type_key` 側で拒否される。
pub(super) fn resource_summary_generic_type_argument_hash(
    types: &TypeCtx,
    type_args: &[TypeId],
) -> Option<u64> {
    let mut hash =
        ResourceSummaryStableHasher::new("neplg2-resource-summary-generic-type-arguments-v2");
    hash.write_usize(type_args.len());
    for (index, ty) in type_args.iter().enumerate() {
        let key = ResourceSummaryStableTypeKey::from_type(types, *ty)?;
        hash.write_usize(index);
        hash.write_str(key.as_str());
    }
    Some(hash.finish())
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::types::{NominalStableTypeIdentity, NominalStableTypeKind, TypeCtx, TypeKind};

    use super::*;

    #[test]
    fn type_parameter_boundary_hash_tracks_order_and_capabilities() {
        let mut types = TypeCtx::new();
        let first = types.fresh_var(Some("T".to_string()));
        let second = types.fresh_var(Some("U".to_string()));
        let cap = types.fresh_var(Some("T".to_string()));
        types.set_var_capabilities(cap, true, false, true);

        let base = resource_summary_type_parameter_boundary_hash(&types, &[first, second])
            .expect("labelled generic parameters should hash");

        assert_ne!(
            base,
            resource_summary_type_parameter_boundary_hash(&types, &[second, first])
                .expect("parameter order is part of the boundary")
        );
        assert_ne!(
            base,
            resource_summary_type_parameter_boundary_hash(&types, &[cap, second])
                .expect("capability changes are part of the boundary")
        );
    }

    #[test]
    fn type_parameter_boundary_hash_rejects_duplicate_stable_keys() {
        let mut types = TypeCtx::new();
        let first = types.fresh_var(Some("T".to_string()));
        let duplicate = types.fresh_var(Some("T".to_string()));

        assert!(
            resource_summary_type_parameter_boundary_hash(&types, &[first, duplicate]).is_none()
        );
    }

    #[test]
    fn generic_type_argument_hash_tracks_order_but_allows_duplicate_arguments() {
        let types = TypeCtx::new();
        let first = resource_summary_generic_type_argument_hash(
            &types,
            &[types.i32(), types.bool(), types.i32()],
        )
        .expect("primitive type arguments should hash");
        let second = resource_summary_generic_type_argument_hash(
            &types,
            &[types.bool(), types.i32(), types.i32()],
        )
        .expect("primitive type arguments should hash");

        assert_ne!(first, second);
    }

    #[test]
    fn type_boundary_hashes_reject_unstable_type_arguments() {
        let mut types = TypeCtx::new();
        let anonymous = types.fresh_var(None);
        let bound = types.fresh_var(Some("T".to_string()));
        types
            .unify(bound, types.i32())
            .expect("test setup should bind the type variable");
        let nominal = types.register_named(
            "Nominal".to_string(),
            TypeKind::Named("Nominal".to_string()),
        );

        assert!(resource_summary_type_parameter_boundary_hash(&types, &[anonymous]).is_none());
        assert!(resource_summary_type_parameter_boundary_hash(&types, &[bound]).is_none());
        assert!(resource_summary_type_parameter_boundary_hash(&types, &[types.i32()]).is_none());
        assert!(resource_summary_generic_type_argument_hash(&types, &[nominal]).is_none());
    }

    #[test]
    fn generic_type_argument_hash_accepts_resolved_nominal_definitions() {
        let mut types = TypeCtx::new();
        let field = types.i32();
        let nominal = types.register_named_with_stable_identity(
            "Nominal".to_string(),
            TypeKind::Struct {
                name: "Nominal".to_string(),
                type_params: Vec::new(),
                fields: vec![field],
                field_names: vec!["value".to_string()],
            },
            NominalStableTypeIdentity::new(
                NominalStableTypeKind::Struct,
                "/user/types.nepl".to_string(),
                "Nominal".to_string(),
                0,
                1,
            ),
        );

        assert!(resource_summary_generic_type_argument_hash(&types, &[nominal]).is_some());
    }

    #[test]
    fn empty_type_boundary_hashes_are_deterministic() {
        let types = TypeCtx::new();

        assert_eq!(
            resource_summary_type_parameter_boundary_hash(&types, &[]),
            resource_summary_type_parameter_boundary_hash(&types, &[])
        );
        assert_eq!(
            resource_summary_generic_type_argument_hash(&types, &[]),
            resource_summary_generic_type_argument_hash(&types, &[])
        );
    }
}
