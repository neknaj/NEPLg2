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

#[cfg(test)]
mod tests {
    use super::FieldAccessorKind;

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
    fn field_accessor_intrinsic_argument_counts_are_kind_owned() {
        assert_eq!(FieldAccessorKind::Get.argument_count(), 2);
        assert_eq!(FieldAccessorKind::GetRef.argument_count(), 2);
        assert_eq!(FieldAccessorKind::Put.argument_count(), 3);
    }
}
