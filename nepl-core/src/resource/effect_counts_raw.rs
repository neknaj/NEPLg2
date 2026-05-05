use super::model::RawMemoryOp;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RawMemoryEffectCounts {
    pub alloc: usize,
    pub dealloc: usize,
    pub realloc: usize,
    pub load: usize,
    pub store: usize,
    pub bulk_copy: usize,
    pub bulk_move: usize,
    pub memory_size: usize,
    pub memory_grow: usize,
    pub fill: usize,
}

impl RawMemoryEffectCounts {
    pub fn record(&mut self, operation: RawMemoryOp) {
        match operation {
            RawMemoryOp::Alloc => self.alloc += 1,
            RawMemoryOp::Dealloc => self.dealloc += 1,
            RawMemoryOp::Realloc => self.realloc += 1,
            RawMemoryOp::Load => self.load += 1,
            RawMemoryOp::Store => self.store += 1,
            RawMemoryOp::BulkCopy => self.bulk_copy += 1,
            RawMemoryOp::BulkMove => self.bulk_move += 1,
            RawMemoryOp::MemorySize => self.memory_size += 1,
            RawMemoryOp::MemoryGrow => self.memory_grow += 1,
            RawMemoryOp::Fill => self.fill += 1,
        }
    }

    pub fn total(self) -> usize {
        self.alloc
            + self.dealloc
            + self.realloc
            + self.load
            + self.store
            + self.bulk_copy
            + self.bulk_move
            + self.memory_size
            + self.memory_grow
            + self.fill
    }
}
