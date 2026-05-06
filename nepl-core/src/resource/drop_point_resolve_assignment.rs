use crate::span::Span;

use super::drop_point_path::ResourceDropPointPath;
use super::drop_point_resolve::{
    op_kind, resolve_resource_drop_point_path, ResourceDropPointResolutionError,
};
use super::model::{Place, ResourceFunction, ResourceOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDropPointAssignment<'a> {
    pub target: &'a Place,
    pub value: &'a Place,
    pub span: Span,
}

pub fn resolve_resource_drop_point_assignment<'a>(
    function: &'a ResourceFunction,
    path: &ResourceDropPointPath,
) -> Result<ResourceDropPointAssignment<'a>, ResourceDropPointResolutionError> {
    match resolve_resource_drop_point_path(function, path)? {
        ResourceOp::Assign {
            target,
            value,
            span,
        } => Ok(ResourceDropPointAssignment {
            target,
            value,
            span: *span,
        }),
        op => Err(
            ResourceDropPointResolutionError::PathDoesNotSelectAssignment {
                actual: op_kind(op),
            },
        ),
    }
}
