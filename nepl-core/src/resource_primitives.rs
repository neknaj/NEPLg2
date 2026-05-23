#[path = "resource_primitives/collection_slot.rs"]
mod collection_slot;
#[path = "resource_primitives/compiler_memory.rs"]
mod compiler_memory;
#[path = "resource_primitives/compiler_memory_value.rs"]
mod compiler_memory_value;
#[path = "resource_primitives/memory_helper.rs"]
mod memory_helper;

pub use self::collection_slot::{CollectionSlotBorrowPrimitive, CollectionSlotLifecyclePrimitive};
pub(crate) use self::compiler_memory::*;
pub(crate) use self::compiler_memory_value::*;
pub(crate) use self::memory_helper::*;

#[cfg(test)]
#[path = "resource_primitives_tests.rs"]
mod tests;
