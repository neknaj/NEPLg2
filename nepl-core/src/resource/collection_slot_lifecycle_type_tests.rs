use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotState,
};
use crate::types::TypeCtx;

#[test]
fn generic_expected_type_matches_initialized_payload_type() {
    let mut types = TypeCtx::new();
    let owned = types.i32();
    let generic = types.fresh_var(Some("T".into()));

    assert_eq!(
        apply_collection_slot_lifecycle_event(
            &types,
            CollectionSlotState::Initialized(owned),
            CollectionSlotLifecycleEvent::BorrowRead {
                expected_ty: generic,
            },
        ),
        Ok(CollectionSlotState::Initialized(owned))
    );
}
