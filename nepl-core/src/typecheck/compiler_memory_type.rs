use alloc::string::String;

use crate::ast::{StructDef, Visibility};
use crate::source_capability::compiler_memory_type_from_constructor_name;
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
            source_map.compiler_memory_type_definition_allowed(def.name.span.file_id, memory_type)
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
    match memory_type {
        CompilerMemoryType::RawPointer => {
            matches!(field_names, [raw] if raw == "raw")
                && fields.len() == 1
                && type_id_is_i32(ctx, fields[0])
        }
        CompilerMemoryType::OwnerToken => {
            matches!(field_names, [raw, size] if raw == "raw" && size == "size")
                && fields.len() == 2
                && type_id_is_i32(ctx, fields[0])
                && type_id_is_i32(ctx, fields[1])
        }
    }
}

fn type_id_is_i32(ctx: &TypeCtx, ty: TypeId) -> bool {
    matches!(ctx.get(ctx.resolve_id(ty)), TypeKind::I32)
}
