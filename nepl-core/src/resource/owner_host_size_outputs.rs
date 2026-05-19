use super::host_size_contract::host_size_outputs;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, Place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::place_utils::raw_memory_cell_place;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn record_host_size_outputs(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        effect: &EffectOp,
        args: &[Place],
    ) {
        for output in host_size_outputs(effect) {
            let Some(address) = args.get(output.address_arg) else {
                continue;
            };
            let address = raw_aliases.canonicalize(address);
            let cell = raw_memory_cell_place(&address, self.types.i32());
            raw_aliases.set_host_size_kind(&cell, output.kind);
        }
    }
}
