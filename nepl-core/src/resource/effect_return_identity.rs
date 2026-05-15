use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_identity::RawIdentityTable;
use super::effect_summary::{RawIdentityParameterReturn, RawIdentityReturnProjection};
use super::function_alias::FunctionAliasTable;
use super::model::{Place, ResourceCallTarget};
use super::place_utils::place_with_suffix;

impl ResourceEffectBoundaryEngine<'_> {
    pub(super) fn copy_call_return_identity(
        &self,
        identities: &mut RawIdentityTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
    ) {
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
            let target = place_with_suffix(output, &returned.projections, returned.ty);
            identities.mark_many(&target, &returned.operations);
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
        let source = place_with_suffix(
            arg,
            &parameter_return.source_projections,
            parameter_return.source_ty,
        );
        let target = place_with_suffix(
            output,
            &parameter_return.return_projections,
            parameter_return.return_ty,
        );
        identities.merge_identity(&source, &target);
    }
}
