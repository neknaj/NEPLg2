use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::raw_place::RawPlaceInfo;

/// Tracks ownership state of variables.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub(super) enum VarState {
    Valid,
    BorrowedShared,
    BorrowedUnique,
    Moved,
    PossiblyMoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BorrowKind {
    Shared,
    Unique,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BorrowBinding {
    pub(super) source: String,
    pub(super) kind: BorrowKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExprBorrow {
    pub(super) binding: BorrowBinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct FieldMove {
    pub(super) offset: usize,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FieldMovePath {
    pub(super) owner: String,
    pub(super) offset: usize,
    pub(super) field_ty: TypeId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct BorrowCount {
    pub(super) shared: usize,
    pub(super) unique: usize,
}

#[derive(Clone)]
pub(super) struct ResourceStateSnapshot {
    pub(super) var_stacks: BTreeMap<String, Vec<VarState>>,
    pub(super) var_depth_stacks: BTreeMap<String, Vec<usize>>,
    pub(super) borrow_stacks: BTreeMap<String, Vec<Vec<BorrowBinding>>>,
    pub(super) field_move_stacks: BTreeMap<String, Vec<BTreeSet<FieldMove>>>,
    pub(super) raw_addr_alias_stacks: BTreeMap<String, Vec<Option<String>>>,
    pub(super) i32_const_stacks: BTreeMap<String, Vec<Option<i64>>>,
    pub(super) enum_payload_raw_alias_stacks: BTreeMap<String, Vec<BTreeMap<String, String>>>,
    pub(super) aggregate_field_raw_alias_stacks: BTreeMap<String, Vec<BTreeMap<usize, String>>>,
    pub(super) aggregate_field_function_alias_stacks:
        BTreeMap<String, Vec<BTreeMap<usize, BTreeSet<String>>>>,
    pub(super) enum_payload_aggregate_field_raw_alias_stacks:
        BTreeMap<String, Vec<BTreeMap<String, BTreeMap<usize, String>>>>,
    pub(super) enum_payload_aggregate_field_function_alias_stacks:
        BTreeMap<String, Vec<BTreeMap<String, BTreeMap<usize, BTreeSet<String>>>>>,
    pub(super) enum_payload_function_alias_stacks:
        BTreeMap<String, Vec<BTreeMap<String, BTreeSet<String>>>>,
    pub(super) function_value_alias_stacks: BTreeMap<String, Vec<BTreeSet<String>>>,
    pub(super) raw_place_states: BTreeMap<String, RawPlaceInfo>,
    pub(super) borrow_counts: BTreeMap<String, BorrowCount>,
}

impl ExprBorrow {
    pub(super) fn needs_retain(binding: BorrowBinding) -> Self {
        Self { binding }
    }
}
