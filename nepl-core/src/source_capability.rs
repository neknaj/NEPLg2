//! Source-level evidence for compiler-owned source capabilities.
//!
//! A loader may know that a file is compiler-owned, but privileged capabilities
//! are attached only when the parsed source contains structured evidence. The
//! later typecheck phase still proves the typed shape at each privileged use.

mod memory_type_definition;
mod owner_aggregate;
mod raw_memory;
mod scope;

pub(crate) use memory_type_definition::{
    compiler_memory_type_from_constructor_name, module_compiler_memory_type_definitions,
};
pub(crate) use owner_aggregate::{
    module_has_owner_aggregate_field_evidence, module_owner_aggregate_constructor_evidence,
};
pub(crate) use raw_memory::module_has_raw_memory_boundary_evidence;
