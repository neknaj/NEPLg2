use alloc::vec::Vec;

use crate::ast::{Module, Stmt, StructDef, TypeExpr, Visibility};
use crate::source_map::CompilerMemoryType;

pub(crate) fn compiler_memory_type_from_constructor_name(name: &str) -> Option<CompilerMemoryType> {
    match name {
        "MemPtr" => Some(CompilerMemoryType::RawPointer),
        "RegionToken" => Some(CompilerMemoryType::OwnerToken),
        _ => None,
    }
}

pub(crate) fn module_compiler_memory_type_definitions(module: &Module) -> Vec<CompilerMemoryType> {
    let mut definitions = Vec::new();
    for item in &module.root.items {
        let Stmt::StructDef(def) = item else {
            continue;
        };
        let Some(memory_type) = compiler_memory_type_from_struct_def(def) else {
            continue;
        };
        if !definitions.contains(&memory_type) {
            definitions.push(memory_type);
        }
    }
    definitions
}

fn compiler_memory_type_from_struct_def(def: &StructDef) -> Option<CompilerMemoryType> {
    match compiler_memory_type_from_constructor_name(def.name.name.as_str())? {
        CompilerMemoryType::RawPointer if is_mem_ptr_definition(def) => {
            Some(CompilerMemoryType::RawPointer)
        }
        CompilerMemoryType::OwnerToken if is_region_token_definition(def) => {
            Some(CompilerMemoryType::OwnerToken)
        }
        _ => None,
    }
}

fn is_mem_ptr_definition(def: &StructDef) -> bool {
    def.vis == Visibility::Pub
        && single_type_param_name(def).is_some()
        && def.fields.len() == 1
        && def.fields[0].0.name == "raw"
        && type_expr_is_i32(&def.fields[0].1)
}

fn is_region_token_definition(def: &StructDef) -> bool {
    if single_type_param_name(def).is_none() {
        return false;
    }
    def.vis == Visibility::Pub
        && def.fields.len() == 2
        && def.fields[0].0.name == "raw"
        && type_expr_is_i32(&def.fields[0].1)
        && def.fields[1].0.name == "size"
        && type_expr_is_i32(&def.fields[1].1)
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
