use alloc::string::String;

use crate::ast::{StructDef, Visibility};
use crate::resource_primitives::{
    compiler_memory_type_field_specs, compiler_memory_type_from_constructor_name,
    CompilerMemoryFieldSpec,
};
use crate::source_map::{CompilerMemoryType, SourceMap};
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn compiler_memory_type_definition_allowed(
    def: &StructDef,
    fields: &[TypeId],
    field_names: &[String],
    type_params: &[TypeId],
    ctx: &TypeCtx,
    source_map: Option<&SourceMap>,
) -> Option<CompilerMemoryType> {
    let memory_type = compiler_memory_type_from_constructor_name(def.name.name.as_str())?;
    let source_allows_definition = source_map
        .map(|source_map| {
            source_map.compiler_memory_type_definition_allowed_at(def.name.span, memory_type)
        })
        .unwrap_or(false);
    if !source_allows_definition
        || !compiler_memory_type_definition_shape_holds(
            memory_type,
            def,
            fields,
            field_names,
            type_params,
            ctx,
        )
    {
        return None;
    }
    Some(memory_type)
}

fn compiler_memory_type_definition_shape_holds(
    memory_type: CompilerMemoryType,
    def: &StructDef,
    fields: &[TypeId],
    field_names: &[String],
    type_params: &[TypeId],
    ctx: &TypeCtx,
) -> bool {
    if def.vis != Visibility::Pub
        || type_params.len() != 1
        || def.type_params.len() != 1
        || !def.type_params[0].bounds.is_empty()
    {
        return false;
    }
    let field_specs = compiler_memory_type_field_specs(memory_type);
    field_names.len() == field_specs.len()
        && fields.len() == field_specs.len()
        && field_names
            .iter()
            .zip(fields)
            .zip(field_specs)
            .all(|((name, ty), spec)| typed_field_shape_holds(*spec, name, *ty, ctx))
}

fn typed_field_shape_holds(
    spec: CompilerMemoryFieldSpec,
    field_name: &str,
    field_ty: TypeId,
    ctx: &TypeCtx,
) -> bool {
    field_name == spec.name() && (!spec.requires_i32() || type_id_is_i32(ctx, field_ty))
}

fn type_id_is_i32(ctx: &TypeCtx, ty: TypeId) -> bool {
    matches!(ctx.get(ctx.resolve_id(ty)), TypeKind::I32)
}
