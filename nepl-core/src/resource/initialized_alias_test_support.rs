extern crate alloc;

use alloc::string::String;

use crate::types::TypeId;

use super::model::Place;

pub(super) fn local(name: &str) -> Place {
    Place::local(String::from(name), TypeId(1))
}
