use crate::runtime_helpers::helper_base_name;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MemoryHelperPrimitive {
    MemPtrWrap,
    MemPtrAddr,
    MemPtrAdd,
    RegionNew,
    RegionPtr,
    RegionPtrAt,
    RegionTokenRawRef,
    StrAddr,
    StrFromAddrUnchecked,
}

impl MemoryHelperPrimitive {
    pub(crate) fn from_symbol(name: &str) -> Option<Self> {
        Self::from_base_name(helper_base_name(name))
    }

    pub(crate) fn from_base_name(base: &str) -> Option<Self> {
        match base {
            "mem_ptr_wrap" => Some(Self::MemPtrWrap),
            "mem_ptr_addr" => Some(Self::MemPtrAddr),
            "mem_ptr_add" => Some(Self::MemPtrAdd),
            "region_new" => Some(Self::RegionNew),
            "region_ptr" => Some(Self::RegionPtr),
            "region_ptr_at" => Some(Self::RegionPtrAt),
            "region_token_raw_ref" => Some(Self::RegionTokenRawRef),
            "str_addr" => Some(Self::StrAddr),
            "str_from_addr_unchecked" => Some(Self::StrFromAddrUnchecked),
            _ => None,
        }
    }

    pub(crate) const fn has_resource_call_lowering(self) -> bool {
        matches!(
            self,
            Self::MemPtrWrap
                | Self::MemPtrAddr
                | Self::MemPtrAdd
                | Self::RegionNew
                | Self::RegionPtr
                | Self::RegionPtrAt
                | Self::RegionTokenRawRef
        )
    }

    pub(crate) const fn has_dedicated_raw_address_lowering(self) -> bool {
        self.has_resource_call_lowering()
    }

    pub(crate) const fn returns_non_owning_address_view(self) -> bool {
        matches!(self, Self::MemPtrAddr | Self::RegionPtr | Self::StrAddr)
    }

    pub(crate) const fn is_raw_address_alias_boundary_evidence(self) -> bool {
        match self {
            Self::MemPtrWrap => true,
            Self::MemPtrAddr
            | Self::MemPtrAdd
            | Self::RegionNew
            | Self::RegionPtr
            | Self::RegionPtrAt
            | Self::RegionTokenRawRef
            | Self::StrAddr
            | Self::StrFromAddrUnchecked => false,
        }
    }

    pub(crate) const fn is_raw_address_view_boundary_evidence(self) -> bool {
        match self {
            Self::MemPtrAddr
            | Self::MemPtrAdd
            | Self::RegionTokenRawRef
            | Self::StrAddr
            | Self::StrFromAddrUnchecked => true,
            Self::MemPtrWrap | Self::RegionNew | Self::RegionPtr | Self::RegionPtrAt => false,
        }
    }
}
