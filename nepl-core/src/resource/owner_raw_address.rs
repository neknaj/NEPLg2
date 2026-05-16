use crate::resource_primitives::MemoryHelperPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawAddressReturnOwnership {
    NonOwningAddressView,
}

pub(super) fn raw_address_return_ownership(name: &str) -> Option<RawAddressReturnOwnership> {
    if MemoryHelperPrimitive::from_symbol(name)
        .is_some_and(MemoryHelperPrimitive::returns_non_owning_address_view)
    {
        Some(RawAddressReturnOwnership::NonOwningAddressView)
    } else {
        None
    }
}
