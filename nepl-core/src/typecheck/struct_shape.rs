use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StructConstructorShape {
    UnitLikeTag,
    FieldList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitLikeStructField {
    Tag,
}

impl StructConstructorShape {
    pub(super) fn classify(ctx: &TypeCtx, fields: &[TypeId], field_names: &[String]) -> Self {
        match (fields, field_names) {
            ([field_ty], [field_name]) => {
                let unit_field = UnitLikeStructField::from_name(field_name.as_str());
                match unit_field {
                    Some(UnitLikeStructField::Tag) => {
                        if type_id_is_unit(ctx, *field_ty) {
                            Self::UnitLikeTag
                        } else {
                            Self::FieldList
                        }
                    }
                    None => Self::FieldList,
                }
            }
            _ => Self::FieldList,
        }
    }

    pub(super) fn constructor_params(self, fields: &[TypeId]) -> Vec<TypeId> {
        match self {
            Self::UnitLikeTag => Vec::new(),
            Self::FieldList => fields.to_vec(),
        }
    }

    pub(super) const fn constructor_arity(self, field_count: usize) -> usize {
        match self {
            Self::UnitLikeTag => 0,
            Self::FieldList => field_count,
        }
    }
}

impl UnitLikeStructField {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "tag" => Some(Self::Tag),
            _ => None,
        }
    }
}

fn type_id_is_unit(ctx: &TypeCtx, ty: TypeId) -> bool {
    matches!(ctx.get(ctx.resolve_id(ty)), TypeKind::Unit)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::types::TypeCtx;

    use super::StructConstructorShape;

    #[test]
    fn unit_like_tag_shape_has_zero_argument_constructor() {
        let ctx = TypeCtx::new();
        let shape = StructConstructorShape::classify(&ctx, &[ctx.unit()], &["tag".to_string()]);

        assert_eq!(shape, StructConstructorShape::UnitLikeTag);
        assert_eq!(shape.constructor_arity(1), 0);
        assert!(shape.constructor_params(&[ctx.unit()]).is_empty());
    }

    #[test]
    fn tag_name_with_non_unit_field_remains_field_list() {
        let ctx = TypeCtx::new();
        let shape = StructConstructorShape::classify(&ctx, &[ctx.i32()], &["tag".to_string()]);

        assert_eq!(shape, StructConstructorShape::FieldList);
        assert_eq!(shape.constructor_arity(1), 1);
        assert_eq!(
            shape.constructor_params(&[ctx.i32()]),
            alloc::vec![ctx.i32()]
        );
    }

    #[test]
    fn non_tag_single_unit_field_remains_field_list() {
        let ctx = TypeCtx::new();
        let shape = StructConstructorShape::classify(&ctx, &[ctx.unit()], &["value".to_string()]);

        assert_eq!(shape, StructConstructorShape::FieldList);
        assert_eq!(shape.constructor_arity(1), 1);
        assert_eq!(
            shape.constructor_params(&[ctx.unit()]),
            alloc::vec![ctx.unit()]
        );
    }
}
