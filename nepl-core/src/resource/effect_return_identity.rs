use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_identity::RawIdentityTable;
use super::effect_summary::{RawIdentityParameterReturn, RawIdentityReturnProjection};
use super::function_alias::FunctionAliasTable;
use super::model::{Place, ResourceCallTarget};
use super::place_utils::place_with_checked_suffix;

impl ResourceEffectBoundaryEngine<'_> {
    pub(super) fn copy_call_return_identity(
        &self,
        identities: &mut RawIdentityTable,
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
        let Some(summary) = self.summaries.get(name) else {
            return;
        };
        self.apply_internal_alloc_return_identities(
            identities,
            output,
            &summary.internal_alloc_returns,
        );
        for parameter_return in &summary.parameter_returns {
            self.copy_parameter_return_identity(identities, output, args, parameter_return);
        }
    }

    pub(super) fn copy_indirect_call_return_identity(
        &self,
        identities: &mut RawIdentityTable,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
    ) {
        if !self.propagate_return_provenance {
            return;
        }
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            for arg in args {
                identities.merge_identity(arg, output);
            }
            return;
        }
        for function in functions {
            if let Some(summary) = self.summaries.get(function) {
                self.apply_internal_alloc_return_identities(
                    identities,
                    output,
                    &summary.internal_alloc_returns,
                );
                for parameter_return in &summary.parameter_returns {
                    self.copy_parameter_return_identity(identities, output, args, parameter_return);
                }
            }
        }
    }

    fn apply_internal_alloc_return_identities(
        &self,
        identities: &mut RawIdentityTable,
        output: &Place,
        returns: &[RawIdentityReturnProjection],
    ) {
        for returned in returns {
            let Some(target) =
                place_with_checked_suffix(self.types, output, &returned.projections, returned.ty)
            else {
                continue;
            };
            identities.mark_many(&target, &returned.origins);
        }
    }

    fn copy_parameter_return_identity(
        &self,
        identities: &mut RawIdentityTable,
        output: &Place,
        args: &[Place],
        parameter_return: &RawIdentityParameterReturn,
    ) {
        let Some(arg) = args.get(parameter_return.parameter_index) else {
            return;
        };
        let Some(source) = place_with_checked_suffix(
            self.types,
            arg,
            &parameter_return.source_projections,
            parameter_return.source_ty,
        ) else {
            return;
        };
        let Some(target) = place_with_checked_suffix(
            self.types,
            output,
            &parameter_return.return_projections,
            parameter_return.return_ty,
        ) else {
            return;
        };
        identities.merge_identity(&source, &target);
    }
}
