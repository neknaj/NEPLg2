#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum RawAddressViewOwnership {
    AddressView,
    NonOwning,
    NonOwningProjection,
}

impl RawAddressViewOwnership {
    pub(super) fn is_non_owning(self) -> bool {
        matches!(
            self,
            RawAddressViewOwnership::NonOwning | RawAddressViewOwnership::NonOwningProjection
        )
    }

    pub(super) fn priority(self) -> u8 {
        match self {
            RawAddressViewOwnership::AddressView => 0,
            RawAddressViewOwnership::NonOwning => 1,
            RawAddressViewOwnership::NonOwningProjection => 2,
        }
    }
}
