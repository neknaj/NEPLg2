use crate::ast::{StructDef, TypeExpr, Visibility};
use crate::resource_primitives::{
    compiler_memory_type_field_specs, compiler_memory_type_from_constructor_name,
    CompilerMemoryFieldSpec,
};
use crate::source_map::CompilerMemoryType;

pub(in crate::source_capability) fn compiler_memory_type_from_struct_def(
    def: &StructDef,
) -> Option<CompilerMemoryType> {
    let memory_type = compiler_memory_type_from_constructor_name(def.name.name.as_str())?;
    if compiler_memory_type_definition_shape_holds(memory_type, def) {
        Some(memory_type)
    } else {
        None
    }
}

fn compiler_memory_type_definition_shape_holds(
    memory_type: CompilerMemoryType,
    def: &StructDef,
) -> bool {
    if def.vis != Visibility::Pub || single_type_param_name(def).is_none() {
        return false;
    }
    let field_specs = compiler_memory_type_field_specs(memory_type);
    def.fields.len() == field_specs.len()
        && def
            .fields
            .iter()
            .zip(field_specs)
            .all(|((name, ty), spec)| source_field_shape_holds(*spec, name.name.as_str(), ty))
}

fn source_field_shape_holds(
    spec: CompilerMemoryFieldSpec,
    field_name: &str,
    field_ty: &TypeExpr,
) -> bool {
    field_name == spec.name() && (!spec.requires_i32() || type_expr_is_i32(field_ty))
}

fn single_type_param_name(def: &StructDef) -> Option<&str> {
    match def.type_params.as_slice() {
        [param] if param.bounds.is_empty() => Some(param.name.name.as_str()),
        _ => None,
    }
}

fn type_expr_is_i32(expr: &TypeExpr) -> bool {
    matches!(expr.as_unspanned(), TypeExpr::I32)
}
