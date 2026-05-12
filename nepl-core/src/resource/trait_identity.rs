extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTraitId(String);

impl ResourceTraitId {
    pub fn from_name(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTraitMethodId(String);

impl ResourceTraitMethodId {
    pub fn from_name(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceTraitApplication {
    pub trait_id: ResourceTraitId,
    pub args: Vec<TypeId>,
}

impl ResourceTraitApplication {
    pub fn new(base_name: String, args: Vec<TypeId>) -> Self {
        Self {
            trait_id: ResourceTraitId::from_name(base_name),
            args,
        }
    }
}
