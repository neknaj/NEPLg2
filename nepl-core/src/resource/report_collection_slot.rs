use crate::diagnostic_codes::{
    DiagnosticCode, ResourceCollectionSlotDiagnosticCode, ResourceDiagnosticCode,
};

use super::collection_slot_lifecycle::CollectionSlotLifecycleRefutation;

pub(super) fn resource_collection_slot_refutation_diagnostic_code(
    reason: CollectionSlotLifecycleRefutation,
) -> DiagnosticCode {
    let code = match reason {
        CollectionSlotLifecycleRefutation::Unavailable { .. } => {
            ResourceCollectionSlotDiagnosticCode::Unavailable
        }
        CollectionSlotLifecycleRefutation::TypeMismatch { .. } => {
            ResourceCollectionSlotDiagnosticCode::TypeMismatch
        }
        CollectionSlotLifecycleRefutation::LiveSlotOverwrite { .. } => {
            ResourceCollectionSlotDiagnosticCode::LiveSlotOverwrite
        }
        CollectionSlotLifecycleRefutation::MaybeLiveSlotOverwrite { .. } => {
            ResourceCollectionSlotDiagnosticCode::MaybeLiveSlotOverwrite
        }
        CollectionSlotLifecycleRefutation::OwnerTransferRequiresValueProof { .. } => {
            ResourceCollectionSlotDiagnosticCode::OwnerTransferRequiresValueProof
        }
        CollectionSlotLifecycleRefutation::DropRequiresElaboration { .. } => {
            ResourceCollectionSlotDiagnosticCode::DropRequiresElaboration
        }
        CollectionSlotLifecycleRefutation::StorageRelocateRequiresRawMoveProof => {
            ResourceCollectionSlotDiagnosticCode::StorageRelocateRequiresRawMoveProof
        }
        CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { .. } => {
            ResourceCollectionSlotDiagnosticCode::LiveSlotDuringStorageDealloc
        }
        CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc { .. } => {
            ResourceCollectionSlotDiagnosticCode::MaybeLiveSlotDuringStorageDealloc
        }
    };
    DiagnosticCode::Resource(ResourceDiagnosticCode::CollectionSlot(code))
}
