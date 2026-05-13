//! Source-level evidence for compiler-owned source capabilities.
//!
//! A loader may know that a file is compiler-owned, but privileged capabilities
//! are attached only when the parsed source contains typed evidence.

mod raw_memory;

pub(crate) use raw_memory::module_has_raw_memory_boundary_evidence;
