extern crate alloc;

use alloc::string::String;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MonoTraitId(String);

impl MonoTraitId {
    pub(super) fn from_name(name: &str) -> Self {
        Self(String::from(name))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MonoTraitMethodId(String);

impl MonoTraitMethodId {
    pub(super) fn from_name(name: &str) -> Self {
        Self(String::from(name))
    }
}
