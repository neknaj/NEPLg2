#[path = "resource_primitives/compiler_memory.rs"]
mod compiler_memory;
#[path = "resource_primitives/memory_helper.rs"]
mod memory_helper;

pub(crate) use self::compiler_memory::*;
pub(crate) use self::memory_helper::*;

#[cfg(test)]
#[path = "resource_primitives_tests.rs"]
mod tests;
