extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;

use crate::ast::Effect;
use crate::backend_scalar_type::BackendScalarType;
use crate::types::{TypeCtx, TypeId, TypeKind};

/// Resource summary cache に保存できる型 key。
///
/// `TypeId` は typecheck arena の slot 番号であり、compile session をまたいで意味が
/// 安定しない。そのため cache に入る key や value では、型を決定的な文字列表現へ
/// 落とした key だけを保持する。無名の未解決 type variable は arena slot に依存するため
/// 拒否し、呼び出し側は cache bypass として扱う。
///
/// nominal type は cache key 側の namespace / public-surface hash と組み合わせて扱う。
/// この value 側 key では `TypeId` や `Span` ではなく、source path、定義名、type
/// parameter 境界、field / variant 形状から作った definition fingerprint を含める。
/// 未解決の `Named` placeholder は、どの定義を指すかを再投影時に検証できないため拒否する。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub(super) struct ResourceSummaryStableTypeKey(String);

impl ResourceSummaryStableTypeKey {
    pub(super) fn from_type(types: &TypeCtx, ty: TypeId) -> Option<Self> {
        let mut seen = BTreeSet::new();
        stable_type_key_string(types, ty, &mut seen).map(Self)
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn matches_type(&self, types: &TypeCtx, ty: TypeId) -> bool {
        Self::from_type(types, ty).is_some_and(|key| key == *self)
    }

    /// function boundary に出ていない labelled generic key かどうかを判定する。
    ///
    /// 同じ label の別 generic を stable key だけで区別することはできないため、
    /// boundary で対応付けられていない open generic は TypeCtx 全体検索の対象にしない。
    pub(super) fn is_open_generic(&self) -> bool {
        self.0.starts_with("var(")
    }
}

fn stable_type_key_string(
    types: &TypeCtx,
    ty: TypeId,
    seen: &mut BTreeSet<TypeId>,
) -> Option<String> {
    let resolved = types.resolve_named_type_id(ty);
    if !seen.insert(resolved) {
        return None;
    }
    let result = match types.get_ref(resolved) {
        TypeKind::Unit => Some(String::from("unit")),
        TypeKind::I32 => Some(String::from("i32")),
        TypeKind::U8 => Some(String::from("u8")),
        TypeKind::F32 => Some(String::from("f32")),
        TypeKind::Bool => Some(String::from("bool")),
        TypeKind::Char => Some(String::from("char")),
        TypeKind::Str => Some(String::from("str")),
        TypeKind::Never => Some(String::from("never")),
        TypeKind::Named(name) => BackendScalarType::from_name(name.as_str())
            .map(|scalar| format!("backend-scalar({})", scalar.source_name())),
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => stable_enum_key(
            types,
            types
                .nominal_stable_identity(resolved)?
                .stable_key_component(),
            type_params,
            variants,
            seen,
        ),
        TypeKind::Struct {
            type_params,
            fields,
            field_names,
            ..
        } => stable_struct_key(
            types,
            types
                .nominal_stable_identity(resolved)?
                .stable_key_component(),
            type_params,
            fields,
            field_names,
            seen,
        ),
        TypeKind::Tuple { items } => {
            stable_type_key_list(types, items, seen).map(|items| format!("tuple({items})"))
        }
        TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } => {
            let type_params = stable_type_key_list(types, type_params, seen)?;
            let params = stable_type_key_list(types, params, seen)?;
            let result = stable_type_key_string(types, *result, seen)?;
            Some(format!(
                "fn<{type_params}>({params})->{result}:{}",
                stable_effect_tag(*effect)
            ))
        }
        TypeKind::Var(var) => match var.binding {
            Some(binding) => stable_type_key_string(types, binding, seen),
            None => var.label.as_ref().map(|label| {
                // label 付き generic variable は、function-local type parameter 境界と
                // generic type-argument hash を store key 側へ含める場合にだけ再投影できる。
                format!(
                    "var({label}:copy={}:clone={}:drop={})",
                    var.copy_cap, var.clone_cap, var.drop_cap
                )
            }),
        },
        TypeKind::Apply { base, args } => {
            let base = stable_type_key_string(types, *base, seen)?;
            let args = stable_type_key_list(types, args, seen)?;
            Some(format!("apply({base})<{args}>"))
        }
        TypeKind::Box(inner) => {
            stable_type_key_string(types, *inner, seen).map(|inner| format!("box({inner})"))
        }
        TypeKind::Reference(inner, is_mut) => stable_type_key_string(types, *inner, seen)
            .map(|inner| format!("ref(mut={is_mut},{inner})")),
    };
    seen.remove(&resolved);
    result
}

fn stable_struct_key(
    types: &TypeCtx,
    identity: String,
    type_params: &[TypeId],
    fields: &[TypeId],
    field_names: &[String],
    seen: &mut BTreeSet<TypeId>,
) -> Option<String> {
    if fields.len() != field_names.len() {
        return None;
    }
    let type_params = stable_type_key_list(types, type_params, seen)?;
    let mut field_key = String::new();
    for (index, (field_name, field_ty)) in field_names.iter().zip(fields.iter()).enumerate() {
        if index > 0 {
            field_key.push(',');
        }
        field_key.push_str("field(");
        field_key.push_str(&stable_text_component(field_name));
        field_key.push(':');
        field_key.push_str(&stable_type_key_string(types, *field_ty, seen)?);
        field_key.push(')');
    }
    Some(format!("struct({identity})<{type_params}>{{{field_key}}}"))
}

fn stable_enum_key(
    types: &TypeCtx,
    identity: String,
    type_params: &[TypeId],
    variants: &[crate::types::EnumVariantInfo],
    seen: &mut BTreeSet<TypeId>,
) -> Option<String> {
    let type_params = stable_type_key_list(types, type_params, seen)?;
    let mut variant_key = String::new();
    for (index, variant) in variants.iter().enumerate() {
        if index > 0 {
            variant_key.push(',');
        }
        variant_key.push_str("variant(");
        variant_key.push_str(&stable_text_component(&variant.name));
        variant_key.push(':');
        match variant.payload {
            Some(payload) => {
                variant_key.push_str("some(");
                variant_key.push_str(&stable_type_key_string(types, payload, seen)?);
                variant_key.push(')');
            }
            None => variant_key.push_str("none"),
        }
        variant_key.push(')');
    }
    Some(format!("enum({identity})<{type_params}>{{{variant_key}}}"))
}

fn stable_type_key_list(
    types: &TypeCtx,
    items: &[TypeId],
    seen: &mut BTreeSet<TypeId>,
) -> Option<String> {
    let mut out = String::new();
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&stable_type_key_string(types, *item, seen)?);
    }
    Some(out)
}

fn stable_text_component(text: &str) -> String {
    format!("{}:{text}", text.len())
}

fn stable_effect_tag(effect: Effect) -> &'static str {
    match effect {
        Effect::Pure => "pure",
        Effect::Impure => "impure",
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::backend_scalar_type::BackendScalarType;
    use crate::types::{
        EnumVariantInfo, NominalStableTypeIdentity, NominalStableTypeKind, TypeCtx, TypeKind,
    };

    use super::*;

    fn nominal_identity(
        kind: NominalStableTypeKind,
        name: &str,
        definition_hash: u64,
    ) -> NominalStableTypeIdentity {
        NominalStableTypeIdentity::new(
            kind,
            "/stdlib/core/types.nepl".to_string(),
            name.to_string(),
            0,
            definition_hash,
        )
    }

    #[test]
    fn stable_type_key_rejects_unlabeled_type_variables() {
        let mut types = TypeCtx::new();
        let anonymous = types.fresh_var(None);

        assert!(ResourceSummaryStableTypeKey::from_type(&types, anonymous).is_none());
    }

    #[test]
    fn stable_type_key_uses_labels_and_capabilities_for_generic_variables() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));

        let key = ResourceSummaryStableTypeKey::from_type(&types, generic)
            .expect("labelled generic parameter should have a stable key");

        assert_eq!(key.as_str(), "var(T:copy=false:clone=false:drop=false)");
    }

    #[test]
    fn stable_type_key_accepts_backend_scalar_named_types() {
        let mut types = TypeCtx::new();
        let scalar = BackendScalarType::U64.type_id(&mut types);

        let key = ResourceSummaryStableTypeKey::from_type(&types, scalar)
            .expect("compiler-owned backend scalar names are stable");

        assert_eq!(key.as_str(), "backend-scalar(u64)");
    }

    #[test]
    fn stable_type_key_accepts_nominal_definitions_with_identity() {
        let mut types = TypeCtx::new();
        let value_field = types.i32();
        let record = types.register_named_with_stable_identity(
            "Record".to_string(),
            TypeKind::Struct {
                name: "Record".to_string(),
                type_params: Vec::new(),
                fields: vec![value_field],
                field_names: vec!["value".to_string()],
            },
            nominal_identity(NominalStableTypeKind::Struct, "Record", 1),
        );

        let key = ResourceSummaryStableTypeKey::from_type(&types, record)
            .expect("nominal definition with stable identity should be stable");

        assert_eq!(
            key.as_str(),
            "struct(nominal(kind=struct,path=23:/stdlib/core/types.nepl,name=6:Record,arity=0,hash=0000000000000001))<>{field(5:value:i32)}"
        );
    }

    #[test]
    fn stable_type_key_tracks_nominal_definition_identity_edits() {
        let mut first_types = TypeCtx::new();
        let first_field = first_types.i32();
        let first = first_types.register_named_with_stable_identity(
            "Record".to_string(),
            TypeKind::Struct {
                name: "Record".to_string(),
                type_params: Vec::new(),
                fields: vec![first_field],
                field_names: vec!["value".to_string()],
            },
            nominal_identity(NominalStableTypeKind::Struct, "Record", 1),
        );
        let first_key = ResourceSummaryStableTypeKey::from_type(&first_types, first)
            .expect("first identity should be stable");

        let mut second_types = TypeCtx::new();
        let second_field = second_types.u8();
        let second = second_types.register_named_with_stable_identity(
            "Record".to_string(),
            TypeKind::Struct {
                name: "Record".to_string(),
                type_params: Vec::new(),
                fields: vec![second_field],
                field_names: vec!["value".to_string()],
            },
            nominal_identity(NominalStableTypeKind::Struct, "Record", 2),
        );
        let second_key = ResourceSummaryStableTypeKey::from_type(&second_types, second)
            .expect("second identity should be stable");

        assert_ne!(first_key, second_key);
    }

    #[test]
    fn stable_type_key_rejects_unresolved_nominal_placeholders() {
        let mut types = TypeCtx::new();
        let unresolved =
            types.register_named("User".to_string(), TypeKind::Named("User".to_string()));

        assert!(ResourceSummaryStableTypeKey::from_type(&types, unresolved).is_none());
    }

    #[test]
    fn stable_type_key_accepts_nominal_enum_definitions() {
        let mut types = TypeCtx::new();
        let ok_payload = types.i32();
        let result = types.register_named_with_stable_identity(
            "ResultI32".to_string(),
            TypeKind::Enum {
                name: "ResultI32".to_string(),
                type_params: Vec::new(),
                variants: vec![
                    EnumVariantInfo {
                        name: "Ok".to_string(),
                        payload: Some(ok_payload),
                    },
                    EnumVariantInfo {
                        name: "Err".to_string(),
                        payload: None,
                    },
                ],
            },
            nominal_identity(NominalStableTypeKind::Enum, "ResultI32", 3),
        );

        let key = ResourceSummaryStableTypeKey::from_type(&types, result)
            .expect("enum nominal definition should be stable");

        assert_eq!(
            key.as_str(),
            "enum(nominal(kind=enum,path=23:/stdlib/core/types.nepl,name=9:ResultI32,arity=0,hash=0000000000000003))<>{variant(2:Ok:some(i32)),variant(3:Err:none)}"
        );
    }
}
