extern crate alloc;

use alloc::string::String;

use crate::types::TypeCtx;

use super::model::{Place, PlaceProjection};
use super::place_utils::enum_payload_type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResultVariant {
    Ok,
}

impl ResultVariant {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Ok => "Ok",
        }
    }

    pub(super) fn payload_projection(self) -> PlaceProjection {
        PlaceProjection::EnumPayload {
            variant: String::from(self.name()),
        }
    }

    pub(super) fn payload_place(self, types: &TypeCtx, output: &Place) -> Option<Place> {
        let payload_ty = enum_payload_type(types, output.ty, self.name())?;
        Some(
            output
                .clone()
                .with_projection(self.payload_projection(), payload_ty),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeKind;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    #[test]
    fn result_variant_names_are_owned_by_enum() {
        assert_eq!(ResultVariant::Ok.name(), "Ok");
    }

    #[test]
    fn result_variant_payload_place_uses_named_payload_type() {
        let mut types = TypeCtx::new();
        let ok_ty = types.i32();
        let result_ty = types.register_named(
            "Result".to_string(),
            TypeKind::Enum {
                doc: None,
                name: "Result".to_string(),
                type_params: Vec::new(),
                variants: vec![crate::types::EnumVariantInfo {
                    name: ResultVariant::Ok.name().to_string(),
                    payload: Some(ok_ty),
                }],
            },
        );
        let output = Place::local(String::from("out"), result_ty);
        let payload = ResultVariant::Ok
            .payload_place(&types, &output)
            .expect("Ok payload must be projected");

        assert_eq!(payload.ty, ok_ty);
        assert_eq!(
            payload.projections,
            vec![PlaceProjection::EnumPayload {
                variant: ResultVariant::Ok.name().to_string()
            }]
        );
    }
}
