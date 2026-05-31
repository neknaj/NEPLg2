use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_identity::copy_pointer_alias;
use super::effect_pointer_alias::RawPointerAliasTable;
use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::function_alias::FunctionAliasTable;
use super::model::{Place, ResourceCallTarget};
use super::place_utils::place_with_checked_suffix;

impl ResourceEffectBoundaryEngine<'_> {
    pub(super) fn copy_call_return_pointer_alias(
        &self,
        pointer_aliases: &mut RawPointerAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
    ) {
        if !self.propagate_return_provenance {
            return;
        }
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        let Some(summary) = self.pointer_summaries.get(name) else {
            return;
        };
        for parameter_return in &summary.parameter_returns {
            let Some(arg) = args.get(parameter_return.parameter_index) else {
                continue;
            };
            let Some(source) = place_with_checked_suffix(
                self.types,
                arg,
                &parameter_return.source_projections,
                parameter_return.source_ty,
            ) else {
                continue;
            };
            let Some(target) = place_with_checked_suffix(
                self.types,
                output,
                &parameter_return.return_projections,
                parameter_return.return_ty,
            ) else {
                continue;
            };
            copy_pointer_alias(pointer_aliases, raw_memory_identities, &source, &target);
        }
    }

    pub(super) fn copy_indirect_call_return_pointer_alias(
        &self,
        pointer_aliases: &mut RawPointerAliasTable,
        raw_memory_identities: &mut RawMemoryIdentityTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
        function_aliases: &FunctionAliasTable,
    ) {
        if !self.propagate_return_provenance {
            return;
        }
        let functions = function_aliases.function_symbols(callee);
        if functions.is_empty() {
            for arg in args {
                copy_pointer_alias(pointer_aliases, raw_memory_identities, arg, output);
            }
            return;
        }
        for function in functions {
            let Some(summary) = self.pointer_summaries.get(function) else {
                continue;
            };
            for parameter_return in &summary.parameter_returns {
                let Some(arg) = args.get(parameter_return.parameter_index) else {
                    continue;
                };
                let Some(source) = place_with_checked_suffix(
                    self.types,
                    arg,
                    &parameter_return.source_projections,
                    parameter_return.source_ty,
                ) else {
                    continue;
                };
                let Some(target) = place_with_checked_suffix(
                    self.types,
                    output,
                    &parameter_return.return_projections,
                    parameter_return.return_ty,
                ) else {
                    continue;
                };
                copy_pointer_alias(pointer_aliases, raw_memory_identities, &source, &target);
            }
        }
    }
}
