use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::coverage::ResourceLoweringCoverage;
use super::effect::ResourceEffectBoundaryReport;
use super::model::{
    BorrowState, BorrowStateEntry, CellState, CellStateEntry, OwnerState, OwnerStateEntry, Place,
    StorageId,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceSafetyShadowReport {
    pub lowering_coverage: ResourceLoweringCoverage,
    pub initialized_moves: ResourceCheckReport,
    pub owner_obligations: ResourceOwnerCheckReport,
    pub borrow_lifetimes: ResourceBorrowCheckReport,
    pub effect_boundaries: ResourceEffectBoundaryReport,
}

impl ResourceSafetyShadowReport {
    pub fn lowering_diagnostic_count(&self) -> usize {
        self.lowering_coverage.diagnostics.len()
    }

    pub fn resource_diagnostic_count(&self) -> usize {
        self.initialized_moves.diagnostics.len()
            + self.owner_obligations.diagnostics.len()
            + self.borrow_lifetimes.diagnostics.len()
            + self.effect_boundaries.diagnostics.len()
    }

    pub fn has_lowering_diagnostics(&self) -> bool {
        !self.lowering_coverage.diagnostics.is_empty()
    }

    pub fn has_resource_diagnostics(&self) -> bool {
        !self.initialized_moves.diagnostics.is_empty()
            || !self.owner_obligations.diagnostics.is_empty()
            || !self.borrow_lifetimes.diagnostics.is_empty()
            || !self.effect_boundaries.diagnostics.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceCheckReport {
    pub functions: Vec<ResourceFunctionCheck>,
    pub diagnostics: Vec<ResourceCheckDiagnostic>,
    pub deferred: ResourceCheckDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceFunctionCheck {
    pub name: String,
    pub final_cells: Vec<CellStateEntry>,
    pub deferred: ResourceCheckDeferred,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceCheckDeferred {
    pub branch_merges: usize,
    pub loop_merges: usize,
    pub match_merges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCheckDiagnostic {
    CellUnavailable {
        function: String,
        operation: ResourceCheckOperation,
        place: Place,
        state: CellState,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceCheckOperation {
    Read,
    Move,
    Drop,
    Borrow,
    DeclareInitializer,
    AssignValue,
    CallArgument,
    ConstructInput,
    ReturnValue,
    BranchCondition,
    BranchValue,
    LoopCondition,
    MatchScrutinee,
    MatchValue,
    RawMemoryArgument,
    RawMemoryLoadAddress,
    RawMemoryLoadCell,
    RawMemoryStoreAddress,
    RawMemoryStoreValue,
    RawMemoryStoreCell,
    RawMemoryDeallocAddress,
    RawMemoryDeallocCell,
    RawMemoryReallocAddress,
    RawMemoryReallocCell,
    RawMemoryFillAddress,
    RawMemoryFillCell,
    RawMemoryBulkDestinationAddress,
    RawMemoryBulkDestinationCell,
    RawMemoryBulkSourceAddress,
    RawMemoryBulkSourceCell,
    IndirectCallee,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOwnerCheckReport {
    pub functions: Vec<ResourceOwnerFunctionCheck>,
    pub diagnostics: Vec<ResourceOwnerDiagnostic>,
    pub deferred: ResourceOwnerCheckDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOwnerFunctionCheck {
    pub name: String,
    pub final_owners: Vec<OwnerStateEntry>,
    pub deferred: ResourceOwnerCheckDeferred,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceOwnerCheckDeferred {
    pub branch_merges: usize,
    pub loop_merges: usize,
    pub match_merges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOwnerDiagnostic {
    OwnerUnavailable {
        function: String,
        operation: ResourceOwnerOperation,
        place: Place,
        state: OwnerState,
        span: Span,
    },
    OwnerLeaked {
        function: String,
        place: Place,
        storage: StorageId,
        span: Span,
    },
    OwnerMaybeLeaked {
        function: String,
        place: Place,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceOwnerOperation {
    DeclareInitializer,
    Read,
    Move,
    AssignValue,
    ReturnValue,
    Dealloc,
    ReallocInput,
    CallArgument,
    BranchValue,
    MatchValue,
    ConstructInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBorrowCheckReport {
    pub functions: Vec<ResourceBorrowFunctionCheck>,
    pub diagnostics: Vec<ResourceBorrowDiagnostic>,
    pub deferred: ResourceBorrowCheckDeferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBorrowFunctionCheck {
    pub name: String,
    pub final_borrows: Vec<BorrowStateEntry>,
    pub deferred: ResourceBorrowCheckDeferred,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResourceBorrowCheckDeferred {
    pub branch_merges: usize,
    pub loop_merges: usize,
    pub match_merges: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceBorrowDiagnostic {
    BorrowConflict {
        function: String,
        operation: ResourceBorrowOperation,
        place: Place,
        active: BorrowState,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceBorrowOperation {
    SharedBorrow,
    UniqueBorrow,
    Read,
    Move,
    Assign,
    Drop,
    ReturnValue,
}
