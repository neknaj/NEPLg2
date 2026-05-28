extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;

use crate::ast::Effect;
use crate::types::{TypeCtx, TypeId, TypeKind};

/// Resource summary cache に保存できる型 key。
///
/// `TypeId` は typecheck arena の slot 番号であり、compile session をまたいで意味が
/// 安定しない。そのため cache に入る key や value では、型を決定的な文字列表現へ
/// 落とした key だけを保持する。無名の未解決 type variable は arena slot に依存するため
/// 拒否し、呼び出し側は cache bypass として扱う。nominal type は現時点で module/path
/// level の definition identity を持たないため、同名別定義への stale hit を避けるために
/// qualified identity が導入されるまで拒否する。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
}

fn stable_type_key_string(
    types: &TypeCtx,
    ty: TypeId,
    seen: &mut BTreeSet<TypeId>,
) -> Option<String> {
    let resolved = types.resolve_id(ty);
    if !seen.insert(resolved) {
        return None;
    }
    let result = match types.get(resolved) {
        TypeKind::Unit => Some(String::from("unit")),
        TypeKind::I32 => Some(String::from("i32")),
        TypeKind::U8 => Some(String::from("u8")),
        TypeKind::F32 => Some(String::from("f32")),
        TypeKind::Bool => Some(String::from("bool")),
        TypeKind::Char => Some(String::from("char")),
        TypeKind::Str => Some(String::from("str")),
        TypeKind::Never => Some(String::from("never")),
        TypeKind::Named(_) | TypeKind::Enum { .. } | TypeKind::Struct { .. } => None,
        TypeKind::Tuple { items } => {
            stable_type_key_list(types, &items, seen).map(|items| format!("tuple({items})"))
        }
        TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } => {
            let type_params = stable_type_key_list(types, &type_params, seen)?;
            let params = stable_type_key_list(types, &params, seen)?;
            let result = stable_type_key_string(types, result, seen)?;
            Some(format!(
                "fn<{type_params}>({params})->{result}:{}",
                stable_effect_tag(effect)
            ))
        }
        TypeKind::Var(var) => match var.binding {
            Some(binding) => stable_type_key_string(types, binding, seen),
            None => var.label.map(|label| {
                // label 付き generic variable は、function-local type parameter 境界と
                // generic type-argument hash を store key 側へ含める場合にだけ再投影できる。
                format!(
                    "var({label}:copy={}:clone={}:drop={})",
                    var.copy_cap, var.clone_cap, var.drop_cap
                )
            }),
        },
        TypeKind::Apply { base, args } => {
            let base = stable_type_key_string(types, base, seen)?;
            let args = stable_type_key_list(types, &args, seen)?;
            Some(format!("apply({base})<{args}>"))
        }
        TypeKind::Box(inner) => {
            stable_type_key_string(types, inner, seen).map(|inner| format!("box({inner})"))
        }
        TypeKind::Reference(inner, is_mut) => stable_type_key_string(types, inner, seen)
            .map(|inner| format!("ref(mut={is_mut},{inner})")),
    };
    seen.remove(&resolved);
    result
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

fn stable_effect_tag(effect: Effect) -> &'static str {
    match effect {
        Effect::Pure => "pure",
        Effect::Impure => "impure",
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec::Vec;

    use crate::types::{TypeCtx, TypeKind};

    use super::*;

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
    fn stable_type_key_rejects_nominal_types_without_definition_identity() {
        let mut types = TypeCtx::new();
        let nominal = types.register_named("User".to_string(), TypeKind::Named("User".to_string()));
        let record = types.register_named(
            "Record".to_string(),
            TypeKind::Struct {
                name: "Record".to_string(),
                type_params: Vec::new(),
                fields: Vec::new(),
                field_names: Vec::new(),
            },
        );

        assert!(ResourceSummaryStableTypeKey::from_type(&types, nominal).is_none());
        assert!(ResourceSummaryStableTypeKey::from_type(&types, record).is_none());
    }
}
