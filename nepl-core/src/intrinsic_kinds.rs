use crate::layout::{storage_align_bytes, storage_size_bytes};
use crate::types::{TypeCtx, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldAccessorKind {
    Get,
    GetRef,
    Put,
}

impl FieldAccessorKind {
    pub(crate) fn from_intrinsic_name(name: &str) -> Option<Self> {
        match name {
            "get_field" => Some(Self::Get),
            "get_field_ref" => Some(Self::GetRef),
            "set_field" => Some(Self::Put),
            _ => None,
        }
    }

    pub(crate) fn from_core_field_member_name(name: &str) -> Option<Self> {
        match name {
            "get" => Some(Self::Get),
            "get_ref" => Some(Self::GetRef),
            "put" => Some(Self::Put),
            _ => None,
        }
    }

    pub(crate) fn from_call_base_name(name: &str) -> Option<Self> {
        Self::from_intrinsic_name(name).or_else(|| Self::from_core_field_member_name(name))
    }

    #[cfg(test)]
    pub(crate) const fn core_field_member_name(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::GetRef => "get_ref",
            Self::Put => "put",
        }
    }

    pub(crate) const fn intrinsic_name(self) -> &'static str {
        match self {
            Self::Get => "get_field",
            Self::GetRef => "get_field_ref",
            Self::Put => "set_field",
        }
    }

    pub(crate) const fn argument_count(self) -> usize {
        match self {
            Self::Get | Self::GetRef => 2,
            Self::Put => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoreIntrinsicKind {
    SizeOf,
    AlignOf,
    Load,
    Store,
    CallsiteSpan,
    Unreachable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CoreIntrinsicResultKind {
    I32,
    Unit,
    Never,
    FirstTypeArgOrUnit,
    FirstTypeArgOrDiagnostic,
}

impl CoreIntrinsicKind {
    pub(crate) fn from_intrinsic_name(name: &str) -> Option<Self> {
        match name {
            "size_of" => Some(Self::SizeOf),
            "align_of" => Some(Self::AlignOf),
            "load" => Some(Self::Load),
            "store" => Some(Self::Store),
            "callsite_span" => Some(Self::CallsiteSpan),
            "unreachable" => Some(Self::Unreachable),
            _ => None,
        }
    }

    pub(crate) const fn intrinsic_name(self) -> &'static str {
        match self {
            Self::SizeOf => "size_of",
            Self::AlignOf => "align_of",
            Self::Load => "load",
            Self::Store => "store",
            Self::CallsiteSpan => "callsite_span",
            Self::Unreachable => "unreachable",
        }
    }

    pub(crate) const fn result_kind(self) -> CoreIntrinsicResultKind {
        match self {
            Self::SizeOf | Self::AlignOf => CoreIntrinsicResultKind::I32,
            Self::Load => CoreIntrinsicResultKind::FirstTypeArgOrUnit,
            Self::Store => CoreIntrinsicResultKind::Unit,
            Self::CallsiteSpan => CoreIntrinsicResultKind::FirstTypeArgOrDiagnostic,
            Self::Unreachable => CoreIntrinsicResultKind::Never,
        }
    }

    pub(crate) fn layout_i32_value(self, types: &TypeCtx, type_args: &[TypeId]) -> Option<i32> {
        let [ty] = type_args else {
            return None;
        };
        let value = match self {
            Self::SizeOf => storage_size_bytes(types, *ty),
            Self::AlignOf => storage_align_bytes(types, *ty),
            Self::Load | Self::Store | Self::CallsiteSpan | Self::Unreachable => return None,
        };
        i32::try_from(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::{CoreIntrinsicKind, CoreIntrinsicResultKind, FieldAccessorKind};
    use crate::types::{TypeCtx, TypeKind};
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn field_accessor_intrinsic_names_round_trip_through_kind() {
        for kind in [
            FieldAccessorKind::Get,
            FieldAccessorKind::GetRef,
            FieldAccessorKind::Put,
        ] {
            assert_eq!(
                FieldAccessorKind::from_intrinsic_name(kind.intrinsic_name()),
                Some(kind)
            );
        }
        assert_eq!(FieldAccessorKind::from_intrinsic_name("get"), None);
    }

    #[test]
    fn field_accessor_core_field_members_round_trip_through_kind() {
        for kind in [
            FieldAccessorKind::Get,
            FieldAccessorKind::GetRef,
            FieldAccessorKind::Put,
        ] {
            assert_eq!(
                FieldAccessorKind::from_core_field_member_name(kind.core_field_member_name()),
                Some(kind)
            );
        }
        assert_eq!(
            FieldAccessorKind::from_core_field_member_name("get_field"),
            None
        );
    }

    #[test]
    fn field_accessor_intrinsic_argument_counts_are_kind_owned() {
        assert_eq!(FieldAccessorKind::Get.argument_count(), 2);
        assert_eq!(FieldAccessorKind::GetRef.argument_count(), 2);
        assert_eq!(FieldAccessorKind::Put.argument_count(), 3);
    }

    #[test]
    fn field_accessor_call_base_name_accepts_member_and_intrinsic_spelling() {
        assert_eq!(
            FieldAccessorKind::from_call_base_name(FieldAccessorKind::Get.core_field_member_name()),
            Some(FieldAccessorKind::Get)
        );
        assert_eq!(
            FieldAccessorKind::from_call_base_name(FieldAccessorKind::Get.intrinsic_name()),
            Some(FieldAccessorKind::Get)
        );
        assert_eq!(
            FieldAccessorKind::from_call_base_name(FieldAccessorKind::Put.core_field_member_name()),
            Some(FieldAccessorKind::Put)
        );
        assert_eq!(
            FieldAccessorKind::from_call_base_name("get_field_ref"),
            Some(FieldAccessorKind::GetRef)
        );
        assert_eq!(FieldAccessorKind::from_call_base_name("unknown"), None);
    }

    #[test]
    fn core_intrinsic_result_kinds_round_trip_through_kind() {
        for (kind, result_kind) in [
            (CoreIntrinsicKind::SizeOf, CoreIntrinsicResultKind::I32),
            (CoreIntrinsicKind::AlignOf, CoreIntrinsicResultKind::I32),
            (
                CoreIntrinsicKind::Load,
                CoreIntrinsicResultKind::FirstTypeArgOrUnit,
            ),
            (CoreIntrinsicKind::Store, CoreIntrinsicResultKind::Unit),
            (
                CoreIntrinsicKind::CallsiteSpan,
                CoreIntrinsicResultKind::FirstTypeArgOrDiagnostic,
            ),
            (
                CoreIntrinsicKind::Unreachable,
                CoreIntrinsicResultKind::Never,
            ),
        ] {
            assert_eq!(
                CoreIntrinsicKind::from_intrinsic_name(kind.intrinsic_name()),
                Some(kind)
            );
            assert_eq!(kind.result_kind(), result_kind);
        }
        assert_eq!(CoreIntrinsicKind::from_intrinsic_name("i32_to_f32"), None);
    }

    #[test]
    fn core_layout_intrinsic_value_is_kind_owned() {
        let mut types = TypeCtx::new();
        let pair = types.register_named(
            "Pair".to_string(),
            TypeKind::Struct {
                doc: None,
                name: "Pair".to_string(),
                type_params: vec![],
                fields: vec![types.i32(), types.i32()],
                field_names: vec!["left".to_string(), "right".to_string()],
            },
        );
        assert_eq!(
            CoreIntrinsicKind::SizeOf.layout_i32_value(&types, &[pair]),
            Some(8)
        );
        assert_eq!(
            CoreIntrinsicKind::AlignOf.layout_i32_value(&types, &[pair]),
            Some(4)
        );
        assert_eq!(
            CoreIntrinsicKind::Load.layout_i32_value(&types, &[pair]),
            None
        );
        assert_eq!(
            CoreIntrinsicKind::SizeOf.layout_i32_value(&types, &[]),
            None
        );
    }
}
