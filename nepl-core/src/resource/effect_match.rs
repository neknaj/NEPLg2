use super::effect_identity::{copy_pointer_alias, RawIdentityTable, RawPointerAliasTable};
use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::function_alias::FunctionAliasTable;
use super::model::{Place, ResourceMatchArm};
use super::place_utils::match_bind_payload_place;

pub(super) fn copy_match_payload_bind_identity(
    identities: &mut RawIdentityTable,
    pointer_aliases: &mut RawPointerAliasTable,
    function_aliases: &mut FunctionAliasTable,
    raw_memory_identities: &mut RawMemoryIdentityTable,
    scrutinee: &Place,
    arm: &ResourceMatchArm,
) {
    let Some(bind_local) = &arm.bind_local else {
        return;
    };
    let Some(payload) = match_bind_payload_place(scrutinee, arm, bind_local) else {
        return;
    };
    identities.copy_identity(&payload, bind_local);
    copy_pointer_alias(pointer_aliases, raw_memory_identities, &payload, bind_local);
    function_aliases.copy_alias(&payload, bind_local);
}
