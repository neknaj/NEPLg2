//! Source-level evidence for compiler-owned source capabilities.
//!
//! A loader may know that a file is compiler-owned, but privileged capabilities
//! are attached only when the parsed source contains structured evidence. The
//! later typecheck phase still proves the typed shape at each privileged use.

mod binding;
mod constructor_position;
mod memory_type_definition;
mod owner_aggregate;
mod prefix_call;
mod proof;
mod proof_builder;
mod raw_evidence_gate;
mod raw_memory;
mod raw_operation_proof;
mod scope;
mod top_level_raw_calls;
mod walk;

pub(crate) use crate::resource_primitives::compiler_memory_type_from_constructor_name;
pub(crate) use proof::module_source_capabilities;
