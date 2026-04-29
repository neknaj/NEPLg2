use crate::runtime_helpers::helper_base_name;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawAddressReturnOwnership {
    NonOwningAddressView,
}

pub(super) fn raw_address_return_ownership(name: &str) -> Option<RawAddressReturnOwnership> {
    match helper_base_name(name) {
        "mem_ptr_addr" => Some(RawAddressReturnOwnership::NonOwningAddressView),
        _ => None,
    }
}
