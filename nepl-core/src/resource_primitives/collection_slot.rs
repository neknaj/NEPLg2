#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectionSlotLifecyclePrimitive {
    InitializeEmpty,
    BorrowRead,
    MoveOut,
    ReplaceReturnOld,
    ReplaceDropOld,
    DropInitialized,
    DropTraversal,
    TransformRange,
    StorageDealloc,
    StorageRelocate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CollectionSlotBorrowPrimitive {
    BorrowRef,
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
            "collection_slot_drop_traversal" => Some(Self::DropTraversal),
            "collection_slot_transform_range" => Some(Self::TransformRange),
            "collection_slot_storage_dealloc" => Some(Self::StorageDealloc),
            "collection_slot_storage_relocate" => Some(Self::StorageRelocate),
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
            Self::DropTraversal => "collection_slot_drop_traversal",
            Self::TransformRange => "collection_slot_transform_range",
            Self::StorageDealloc => "collection_slot_storage_dealloc",
            Self::StorageRelocate => "collection_slot_storage_relocate",
        }
    }

    pub(crate) const fn type_arg_count(self) -> usize {
        match self {
            Self::InitializeEmpty
            | Self::BorrowRead
            | Self::MoveOut
            | Self::DropInitialized
            | Self::DropTraversal
            | Self::TransformRange => 1,
            Self::ReplaceReturnOld | Self::ReplaceDropOld => 2,
            Self::StorageDealloc => 1,
            Self::StorageRelocate => 0,
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
            Self::DropTraversal => 2,
            Self::TransformRange => 4,
            Self::StorageRelocate => 2,
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
            Self::DropTraversal
            | Self::TransformRange
            | Self::StorageDealloc
            | Self::StorageRelocate => false,
        }
    }

    pub(crate) const fn requires_storage_pair(self) -> bool {
        match self {
            Self::StorageRelocate => true,
            Self::InitializeEmpty
            | Self::BorrowRead
            | Self::MoveOut
            | Self::ReplaceReturnOld
            | Self::ReplaceDropOld
            | Self::DropInitialized
            | Self::DropTraversal
            | Self::TransformRange
            | Self::StorageDealloc => false,
        }
    }

    pub(crate) const fn requires_storage_drop_traversal(self) -> bool {
        match self {
            Self::DropTraversal => true,
            Self::InitializeEmpty
            | Self::BorrowRead
            | Self::MoveOut
            | Self::ReplaceReturnOld
            | Self::ReplaceDropOld
            | Self::DropInitialized
            | Self::StorageDealloc
            | Self::TransformRange
            | Self::StorageRelocate => false,
        }
    }

    pub(crate) const fn requires_storage_transform_range(self) -> bool {
        match self {
            Self::TransformRange => true,
            Self::InitializeEmpty
            | Self::BorrowRead
            | Self::MoveOut
            | Self::ReplaceReturnOld
            | Self::ReplaceDropOld
            | Self::DropInitialized
            | Self::DropTraversal
            | Self::StorageDealloc
            | Self::StorageRelocate => false,
        }
    }

    pub(crate) const fn slot_target_type_arg_index(self) -> Option<usize> {
        match self {
            Self::InitializeEmpty
            | Self::BorrowRead
            | Self::MoveOut
            | Self::DropInitialized
            | Self::DropTraversal
            | Self::TransformRange => Some(0),
            Self::ReplaceReturnOld | Self::ReplaceDropOld => Some(0),
            Self::StorageDealloc => Some(0),
            Self::StorageRelocate => None,
        }
    }
}

impl CollectionSlotBorrowPrimitive {
    pub(crate) fn from_intrinsic_name(name: &str) -> Option<Self> {
        match name {
            "collection_slot_borrow_ref" => Some(Self::BorrowRef),
            _ => None,
        }
    }

    pub(crate) const fn intrinsic_name(self) -> &'static str {
        match self {
            Self::BorrowRef => "collection_slot_borrow_ref",
        }
    }

    pub(crate) const fn type_arg_count(self) -> usize {
        match self {
            Self::BorrowRef => 1,
        }
    }

    pub(crate) const fn argument_count(self) -> usize {
        match self {
            Self::BorrowRef => 2,
        }
    }

    pub(crate) const fn lifecycle_event(self) -> CollectionSlotLifecyclePrimitive {
        match self {
            Self::BorrowRef => CollectionSlotLifecyclePrimitive::BorrowRead,
        }
    }
}
