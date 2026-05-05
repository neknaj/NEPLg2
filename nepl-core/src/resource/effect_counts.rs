use super::effect_counts_host::{ExternalIoEffectCounts, NondetEffectCounts};
use super::effect_counts_raw::RawMemoryEffectCounts;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceEffectCounts {
    pub internal_memory_ops: RawMemoryEffectCounts,
    pub unsafe_memory_ops: RawMemoryEffectCounts,
    pub external_io_ops: ExternalIoEffectCounts,
    pub nondet_ops: NondetEffectCounts,
    pub unknown_ops: usize,
}
