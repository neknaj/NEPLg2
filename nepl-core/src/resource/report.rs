use alloc::string::String;
use alloc::vec::Vec;

use crate::diagnostic_codes::{
    DiagnosticCode, ResourceBorrowDiagnosticCode, ResourceCellDiagnosticCode,
    ResourceDiagnosticCode, ResourceOwnerDiagnosticCode,
};
use crate::span::Span;

use super::collection_slot_lifecycle::CollectionSlotLifecycleRefutation;
use super::collection_slot_state_table::CollectionSlotStateEntry;
use super::coverage::ResourceLoweringCoverage;
use super::drop_model::ResourceDropPoint;
use super::effect::ResourceEffectBoundaryReport;
use super::model::{
    BorrowState, BorrowStateEntry, CellState, CellStateEntry, OwnerState, OwnerStateEntry, Place,
    StorageId,
};
use super::report_collection_slot::resource_collection_slot_refutation_diagnostic_code;

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
    pub final_collection_slots: Vec<CollectionSlotStateEntry>,
    pub auto_drop_points: Vec<ResourceDropPoint>,
    pub deferred: ResourceCheckDeferred,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    CollectionSlotRefuted {
        function: String,
        target: Place,
        reason: CollectionSlotLifecycleRefutation,
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
    RawMemoryBulkCount,
    IndirectCallee,
}

impl ResourceCheckDiagnostic {
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            ResourceCheckDiagnostic::CellUnavailable { state, .. } => {
                resource_cell_state_diagnostic_code(state)
            }
            ResourceCheckDiagnostic::CollectionSlotRefuted { reason, .. } => {
                resource_collection_slot_refutation_diagnostic_code(*reason)
            }
        }
    }
}

fn resource_cell_state_diagnostic_code(state: &CellState) -> DiagnosticCode {
    let code = match state {
        CellState::Uninit => ResourceCellDiagnosticCode::Uninit,
        CellState::Initialized(_) => ResourceCellDiagnosticCode::InitializedConflict,
        CellState::Moved => ResourceCellDiagnosticCode::Moved,
        CellState::Dropped => ResourceCellDiagnosticCode::Dropped,
        CellState::MaybeMoved => ResourceCellDiagnosticCode::PossiblyMoved,
    };
    DiagnosticCode::Resource(ResourceDiagnosticCode::Cell(code))
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceOwnerOperation {
    DeclareInitializer,
    Read,
    Move,
    AssignValue,
    ReturnValue,
    Drop,
    Dealloc,
    DeallocExtent,
    ReallocInput,
    ReallocExtent,
    ExternalIoPayloadExtent,
    RawMemoryPayloadExtent,
    RawMemoryLoadCell,
    RawMemoryStoreValue,
    CallArgument,
    BranchValue,
    MatchValue,
    ConstructInput,
}

impl ResourceOwnerDiagnostic {
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            ResourceOwnerDiagnostic::OwnerUnavailable { state, .. } => {
                resource_owner_state_diagnostic_code(state)
            }
            ResourceOwnerDiagnostic::OwnerLeaked { .. } => DiagnosticCode::Resource(
                ResourceDiagnosticCode::Owner(ResourceOwnerDiagnosticCode::Leak),
            ),
            ResourceOwnerDiagnostic::OwnerMaybeLeaked { .. } => DiagnosticCode::Resource(
                ResourceDiagnosticCode::Owner(ResourceOwnerDiagnosticCode::MaybeLeak),
            ),
        }
    }
}

fn resource_owner_state_diagnostic_code(state: &OwnerState) -> DiagnosticCode {
    let code = match state {
        OwnerState::NoFreeObligation => ResourceOwnerDiagnosticCode::NoFreeObligation,
        OwnerState::Live { .. } => ResourceOwnerDiagnosticCode::Unavailable,
        OwnerState::Reserved { .. } => ResourceOwnerDiagnosticCode::Reserved,
        OwnerState::Moved => ResourceOwnerDiagnosticCode::UseAfterMove,
        OwnerState::Freed => ResourceOwnerDiagnosticCode::DoubleFree,
        OwnerState::MaybeFreed { .. } => ResourceOwnerDiagnosticCode::MaybeFreed,
    };
    DiagnosticCode::Resource(ResourceDiagnosticCode::Owner(code))
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

impl ResourceBorrowDiagnostic {
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            ResourceBorrowDiagnostic::BorrowConflict {
                operation, active, ..
            } => resource_borrow_conflict_diagnostic_code(*operation, active),
        }
    }
}

fn resource_borrow_conflict_diagnostic_code(
    operation: ResourceBorrowOperation,
    active: &BorrowState,
) -> DiagnosticCode {
    match operation {
        ResourceBorrowOperation::ReturnValue => DiagnosticCode::Resource(
            ResourceDiagnosticCode::Borrow(ResourceBorrowDiagnosticCode::ReturnEscape),
        ),
        ResourceBorrowOperation::SharedBorrow => DiagnosticCode::Resource(
            ResourceDiagnosticCode::Borrow(ResourceBorrowDiagnosticCode::BorrowDuringUnique),
        ),
        ResourceBorrowOperation::UniqueBorrow => match active {
            BorrowState::Shared { .. } => DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
                ResourceBorrowDiagnosticCode::UniqueDuringShared,
            )),
            BorrowState::Unique { .. } | BorrowState::Unborrowed | BorrowState::Released => {
                DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
                    ResourceBorrowDiagnosticCode::BorrowDuringUnique,
                ))
            }
        },
        ResourceBorrowOperation::Read => DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
            ResourceBorrowDiagnosticCode::UseDuringUnique,
        )),
        ResourceBorrowOperation::Move => match active {
            BorrowState::Shared { .. } => DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
                ResourceBorrowDiagnosticCode::MoveFromShared,
            )),
            BorrowState::Unique { .. } | BorrowState::Unborrowed | BorrowState::Released => {
                DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
                    ResourceBorrowDiagnosticCode::UseDuringUnique,
                ))
            }
        },
        ResourceBorrowOperation::Assign => match active {
            BorrowState::Shared { .. } => DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
                ResourceBorrowDiagnosticCode::AssignDuringShared,
            )),
            BorrowState::Unique { .. } | BorrowState::Unborrowed | BorrowState::Released => {
                DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
                    ResourceBorrowDiagnosticCode::AssignDuringUnique,
                ))
            }
        },
        ResourceBorrowOperation::Drop => match active {
            BorrowState::Shared { .. } => DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
                ResourceBorrowDiagnosticCode::DropDuringShared,
            )),
            BorrowState::Unique { .. } | BorrowState::Unborrowed | BorrowState::Released => {
                DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(
                    ResourceBorrowDiagnosticCode::DropDuringUnique,
                ))
            }
        },
    }
}
