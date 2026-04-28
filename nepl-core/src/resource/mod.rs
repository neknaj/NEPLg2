//! Resource IR used as the typed input for ownership and lifetime checks.
//!
//! This module is intentionally separate from the current HIR move checker.
//! Stage 2 exposes the data model, a non-enforcing lowering skeleton, and a
//! deterministic dump so later stages can migrate checks one operation class
//! at a time.

extern crate alloc;

mod cell_state;
mod check;
mod coverage;
mod dump;
mod effect;
mod lower;
mod model;
mod owner_state;
mod place_utils;

pub use check::{
    check_hir_resource_safety_shadow, check_resource_borrow_lifetimes,
    check_resource_initialized_moves, check_resource_owner_obligations,
    ResourceBorrowCheckDeferred, ResourceBorrowCheckReport, ResourceBorrowDiagnostic,
    ResourceBorrowFunctionCheck, ResourceBorrowOperation, ResourceCheckDeferred,
    ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport, ResourceFunctionCheck,
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerDiagnostic,
    ResourceOwnerFunctionCheck, ResourceOwnerOperation, ResourceSafetyShadowReport,
};
pub use coverage::{
    compare_hir_resource_lowering, ResourceCoverageCounts, ResourceCoverageDiagnostic,
    ResourceCoverageKind, ResourceFunctionCoverage, ResourceLoweringCoverage,
};
pub use effect::{
    check_resource_effect_boundaries, ResourceEffectBoundaryDiagnostic,
    ResourceEffectBoundaryReport, ResourceEffectCounts, ResourceEffectFunctionCheck,
};
pub use lower::lower_hir_module_skeleton;
pub use model::{
    AggregateKind, BorrowKind, BorrowState, BorrowStateEntry, CellState, CellStateEntry, EffectOp,
    OwnerState, OwnerStateEntry, Place, PlaceProjection, PlaceRoot, PointerProvenance, RawBodyKind,
    RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceCallTarget, ResourceExprKind,
    ResourceFunction, ResourceId, ResourceLocal, ResourceMatchArm, ResourceMatchPattern,
    ResourceModule, ResourceOffset, ResourceOp, ResourceState, ResourceTerminator, StorageId,
};
