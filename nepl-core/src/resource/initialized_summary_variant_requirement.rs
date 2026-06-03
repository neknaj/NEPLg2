extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::{raw_cell_suffix_after_address, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::initialized_summary_apply_param::apply_param_initialization_summary;
use super::initialized_summary_apply_return::apply_return_initialization_summary;
use super::initialized_summary_variant_model::RawCellInitializationVariantParamRequirement;
use super::model::{CellState, Place, RawMemoryOp, ResourceCallTarget, ResourceLocal, ResourceOp};
use super::place_utils::raw_memory_cell_place;
use super::raw_cell_lifecycle::{CopyRawElementType, RawCellLifecycleEvent};
use super::summary_projection::summary_suffix_for_params;
use super::variant_name::normalize_variant_name;

pub(super) fn collect_variant_param_required_raw_cells(
    out: &mut Vec<RawCellInitializationVariantParamRequirement>,
    types: &TypeCtx,
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    initial_cells: &CellTable,
    variant: &str,
    path_ops: &[ResourceOp],
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    let mut local_cells = initial_cells.clone();
    let mut local_aliases = raw_aliases.clone();
    for op in path_ops {
        match op {
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output,
                args,
                ..
            } => {
                collect_load_requirements(
                    out,
                    types,
                    variant,
                    &mut local_cells,
                    &mut local_aliases,
                    output.ty,
                    args,
                    params,
                );
            }
            ResourceOp::RawMemory {
                operation,
                output: _,
                args,
                ..
            } => {
                apply_local_raw_memory_initialization(
                    types,
                    &mut local_cells,
                    &mut local_aliases,
                    operation,
                    args,
                );
            }
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } => {
                apply_local_call_initialization_summary(
                    types,
                    raw_init_summaries,
                    &mut local_cells,
                    &mut local_aliases,
                    output,
                    target,
                    args,
                );
            }
            _ => {}
        }
    }
}

fn apply_local_call_initialization_summary(
    types: &TypeCtx,
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    local_cells: &mut CellTable,
    local_aliases: &mut RawCellAddressAliases,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
) {
    let ResourceCallTarget::User { name, type_args } = target else {
        return;
    };
    let Some(summary) = raw_init_summaries.get(name) else {
        return;
    };
    apply_return_initialization_summary(
        types,
        local_cells,
        local_aliases,
        output,
        type_args,
        summary,
    );
    apply_param_initialization_summary(types, local_cells, local_aliases, args, type_args, summary);
}

#[allow(clippy::too_many_arguments)]
fn collect_load_requirements(
    out: &mut Vec<RawCellInitializationVariantParamRequirement>,
    types: &TypeCtx,
    variant: &str,
    local_cells: &mut CellTable,
    local_aliases: &mut RawCellAddressAliases,
    cell_ty: TypeId,
    args: &[Place],
    params: &[ResourceLocal],
) {
    let Some(address) = args.first() else {
        return;
    };
    let address = local_aliases.canonicalize(address);
    let has_local_initialization =
        raw_load_has_local_initialization(local_cells, local_aliases, &address, cell_ty, types);
    if !has_local_initialization {
        for address_alias in local_aliases.aliases_for(&address) {
            let cell = raw_memory_cell_place(&address_alias, cell_ty);
            for (param_index, param) in params.iter().enumerate() {
                for param_alias in local_aliases.aliases_for(&param.place) {
                    let Some(suffix) = raw_cell_suffix_after_address(&cell, &param_alias) else {
                        continue;
                    };
                    let Some(suffix) = summary_suffix_for_params(params, &suffix) else {
                        continue;
                    };
                    push_unique_variant_param_requirement(
                        out,
                        RawCellInitializationVariantParamRequirement {
                            variant: normalize_variant_name(variant),
                            param_index,
                            suffix,
                            ty: cell_ty,
                        },
                    );
                }
            }
        }
    }
    if has_local_initialization {
        local_cells.apply_raw_cell_lifecycle_event(
            RawCellLifecycleEvent::MoveOutLoadedCell {
                address: &address,
                cell_ty,
            },
            local_aliases,
            types,
        );
    }
}

fn apply_local_raw_memory_initialization(
    types: &TypeCtx,
    local_cells: &mut CellTable,
    local_aliases: &mut RawCellAddressAliases,
    operation: &RawMemoryOp,
    args: &[Place],
) {
    match operation {
        RawMemoryOp::Store | RawMemoryOp::StoreU8 => {
            let Some(address) = args.first() else {
                return;
            };
            let Some(stored_ty) = raw_store_cell_type(types, operation, args) else {
                return;
            };
            let address = local_aliases.canonicalize(address);
            let value = args
                .get(1)
                .cloned()
                .unwrap_or_else(|| raw_memory_cell_place(&address, stored_ty));
            local_cells.apply_raw_cell_lifecycle_event(
                RawCellLifecycleEvent::StoreValue {
                    address: &address,
                    value: &value,
                    stored_ty,
                },
                local_aliases,
                types,
            );
        }
        RawMemoryOp::FillBytes => {
            let (Some(address), Some(count), Some(_value)) =
                (args.first(), args.get(1), args.get(2))
            else {
                return;
            };
            let address = local_aliases.canonicalize(address);
            local_cells.apply_raw_cell_lifecycle_event(
                RawCellLifecycleEvent::FillBytes {
                    address: &address,
                    count,
                },
                local_aliases,
                types,
            );
        }
        RawMemoryOp::Fill => {
            let (Some(address), Some(count), Some(value)) =
                (args.first(), args.get(1), args.get(2))
            else {
                return;
            };
            let address = local_aliases.canonicalize(address);
            if let Some(element_ty) = CopyRawElementType::new(value.ty, types) {
                local_cells.apply_raw_cell_lifecycle_event(
                    RawCellLifecycleEvent::FillCopyElements {
                        address: &address,
                        count,
                        element_ty,
                    },
                    local_aliases,
                    types,
                );
            }
        }
        RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
            let (Some(destination), Some(source), Some(count)) =
                (args.first(), args.get(1), args.get(2))
            else {
                return;
            };
            let destination = local_aliases.canonicalize(destination);
            let source = local_aliases.canonicalize(source);
            local_cells.apply_raw_cell_lifecycle_event(
                RawCellLifecycleEvent::BulkCopyInitializedRawState {
                    source: &source,
                    destination: &destination,
                    count: Some(count),
                },
                local_aliases,
                types,
            );
        }
        RawMemoryOp::Dealloc | RawMemoryOp::Realloc => {
            if let Some(address) = args.first() {
                let address = local_aliases.canonicalize(address);
                local_cells.apply_raw_cell_lifecycle_event(
                    RawCellLifecycleEvent::DiscardCellsUnderAddress { address: &address },
                    local_aliases,
                    types,
                );
            }
        }
        RawMemoryOp::Alloc
        | RawMemoryOp::Load
        | RawMemoryOp::LoadU8
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow => {}
    }
}

fn raw_store_cell_type(
    types: &TypeCtx,
    operation: &RawMemoryOp,
    args: &[Place],
) -> Option<TypeId> {
    match operation {
        RawMemoryOp::Store => args.get(1).map(|value| value.ty),
        RawMemoryOp::StoreU8 => Some(types.u8()),
        _ => None,
    }
}

fn raw_load_has_local_initialization(
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    address: &Place,
    cell_ty: TypeId,
    types: &TypeCtx,
) -> bool {
    raw_aliases.aliases_for(address).iter().any(|alias| {
        let cell = raw_memory_cell_place(alias, cell_ty);
        matches!(
            cells.availability_state_with_types(types, &cell),
            CellState::Initialized(_)
        ) || cells.raw_cell_initialized_by_byte_range(alias, cell_ty, raw_aliases, types)
    })
}

fn push_unique_variant_param_requirement(
    cells: &mut Vec<RawCellInitializationVariantParamRequirement>,
    cell: RawCellInitializationVariantParamRequirement,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::String, vec, vec::Vec};

    use crate::span::Span;
    use crate::types::TypeCtx;

    use super::*;
    use crate::resource::model::{ResourceId, ResourceLocal};

    #[test]
    fn variant_requirement_skips_raw_load_initialized_by_prefix_state() {
        let types = TypeCtx::new();
        let pointer = Place::local(String::from("p"), types.i32());
        let loaded = Place::temporary(ResourceId(1), types.i32());
        let param = param("p", pointer.clone());
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.mark(&pointer);
        let mut initial_cells = CellTable::default();
        initial_cells.mark_initialized(&raw_memory_cell_place(&pointer, types.i32()));
        let path_ops = vec![raw_load(loaded, pointer.clone())];

        let out = collect_requirements(&types, &initial_cells, &raw_aliases, &[param], &path_ops);

        assert!(
            out.is_empty(),
            "raw loads proven by branch-prefix stores must not become caller requirements: {out:#?}"
        );
    }

    #[test]
    fn variant_requirement_keeps_raw_load_before_local_store() {
        let types = TypeCtx::new();
        let pointer = Place::local(String::from("p"), types.i32());
        let loaded = Place::temporary(ResourceId(1), types.i32());
        let stored = Place::temporary(ResourceId(2), types.unit());
        let value = Place::temporary(ResourceId(3), types.i32());
        let param = param("p", pointer.clone());
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.mark(&pointer);
        let initial_cells = CellTable::default();
        let path_ops = vec![
            raw_load(loaded, pointer.clone()),
            raw_store(stored, pointer.clone(), value),
        ];

        let out = collect_requirements(&types, &initial_cells, &raw_aliases, &[param], &path_ops);

        assert_eq!(
            out.len(),
            1,
            "a load that happens before the local store still depends on caller-provided raw cell state"
        );
    }

    #[test]
    fn variant_requirement_skips_raw_load_after_local_store() {
        let types = TypeCtx::new();
        let pointer = Place::local(String::from("p"), types.i32());
        let loaded = Place::temporary(ResourceId(1), types.i32());
        let stored = Place::temporary(ResourceId(2), types.unit());
        let value = Place::temporary(ResourceId(3), types.i32());
        let param = param("p", pointer.clone());
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.mark(&pointer);
        let initial_cells = CellTable::default();
        let path_ops = vec![
            raw_store(stored, pointer.clone(), value),
            raw_load(loaded, pointer.clone()),
        ];

        let out = collect_requirements(&types, &initial_cells, &raw_aliases, &[param], &path_ops);

        assert!(
            out.is_empty(),
            "a local store must discharge the later raw-load requirement in the same variant path"
        );
    }

    fn collect_requirements(
        types: &TypeCtx,
        initial_cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        params: &[ResourceLocal],
        path_ops: &[ResourceOp],
    ) -> Vec<RawCellInitializationVariantParamRequirement> {
        let empty_summaries = [];
        let raw_init_summaries = RawCellInitializationFunctionSummaryIndex::new(&empty_summaries);
        let mut out = Vec::new();
        collect_variant_param_required_raw_cells(
            &mut out,
            types,
            &raw_init_summaries,
            initial_cells,
            "Result::Ok",
            path_ops,
            raw_aliases,
            params,
        );
        out
    }

    fn param(name: &str, place: Place) -> ResourceLocal {
        ResourceLocal {
            name: String::from(name),
            ty: place.ty,
            mutable: false,
            place,
        }
    }

    fn raw_load(output: Place, address: Place) -> ResourceOp {
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output,
            args: vec![address],
            span: Span::dummy(),
        }
    }

    fn raw_store(output: Place, address: Place, value: Place) -> ResourceOp {
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Store,
            output,
            args: vec![address, value],
            span: Span::dummy(),
        }
    }
}
