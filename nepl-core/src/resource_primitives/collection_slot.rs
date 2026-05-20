#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectionSlotLifecyclePrimitive {
    InitializeEmpty,
    BorrowRead,
    MoveOut,
    ReplaceReturnOld,
    ReplaceDropOld,
    DropInitialized,
    StorageDealloc,
}

impl CollectionSlotLifecyclePrimitive {
    pub(crate) fn from_intrinsic_name(name: &str) -> Option<Self> {
        match name {
            "collection_slot_initialize_empty" => Some(Self::InitializeEmpty),
            "collection_slot_borrow_read" => Some(Self::BorrowRead),
            "collection_slot_move_out" => Some(Self::MoveOut),
            "collection_slot_replace_return_old" => Some(Self::ReplaceReturnOld),
            "collection_slot_replace_drop_old" => Some(Self::ReplaceDropOld),
            "collection_slot_drop_initialized" => Some(Self::DropInitialized),
            "collection_slot_storage_dealloc" => Some(Self::StorageDealloc),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn intrinsic_name(self) -> &'static str {
        match self {
            Self::InitializeEmpty => "collection_slot_initialize_empty",
            Self::BorrowRead => "collection_slot_borrow_read",
            Self::MoveOut => "collection_slot_move_out",
            Self::ReplaceReturnOld => "collection_slot_replace_return_old",
            Self::ReplaceDropOld => "collection_slot_replace_drop_old",
            Self::DropInitialized => "collection_slot_drop_initialized",
            Self::StorageDealloc => "collection_slot_storage_dealloc",
        }
    }

    pub(crate) const fn type_arg_count(self) -> usize {
        match self {
            Self::InitializeEmpty | Self::BorrowRead | Self::MoveOut | Self::DropInitialized => 1,
            Self::ReplaceReturnOld | Self::ReplaceDropOld => 2,
            Self::StorageDealloc => 0,
        }
    }

    pub(crate) const fn argument_count(self) -> usize {
        match self {
            Self::InitializeEmpty
            | Self::BorrowRead
            | Self::MoveOut
            | Self::ReplaceReturnOld
            | Self::ReplaceDropOld
            | Self::DropInitialized => 2,
            Self::StorageDealloc => 1,
        }
    }

    pub(crate) const fn has_slot_offset(self) -> bool {
        match self {
            Self::InitializeEmpty
            | Self::BorrowRead
            | Self::MoveOut
            | Self::ReplaceReturnOld
            | Self::ReplaceDropOld
            | Self::DropInitialized => true,
            Self::StorageDealloc => false,
        }
    }

    pub(crate) const fn slot_target_type_arg_index(self) -> Option<usize> {
        match self {
            Self::InitializeEmpty | Self::BorrowRead | Self::MoveOut | Self::DropInitialized => {
                Some(0)
            }
            Self::ReplaceReturnOld | Self::ReplaceDropOld => Some(0),
            Self::StorageDealloc => None,
        }
    }
}
