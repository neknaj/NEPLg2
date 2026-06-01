extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_counts::ResourceEffectCounts;
use super::effect_diagnostic::ResourceEffectBoundaryDiagnostic;
use super::effect_raw_provenance::{
    function_needs_effect_provenance_tracking, report_internal_alloc_escapes_from_summary,
};
use super::effect_summary::{RawIdentityReturnSummaryIndex, RawPointerReturnSummaryIndex};
use super::effect_summary_identity::compute_raw_identity_return_summaries;
use super::effect_summary_pointer::compute_raw_pointer_return_summaries;
use super::model::ResourceModule;
use super::private_cache_mask::PrivateCacheMaskProofIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEffectBoundaryReport {
    pub functions: Vec<ResourceEffectFunctionCheck>,
    pub diagnostics: Vec<ResourceEffectBoundaryDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceEffectFunctionCheck {
    pub name: String,
    pub counts: ResourceEffectCounts,
}

pub fn check_resource_effect_boundaries(module: &ResourceModule) -> ResourceEffectBoundaryReport {
    check_resource_effect_boundaries_with_types(module, None)
}

pub fn check_resource_effect_boundaries_typed(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceEffectBoundaryReport {
    check_resource_effect_boundaries_with_types(module, Some(types))
}

fn check_resource_effect_boundaries_with_types(
    module: &ResourceModule,
    types: Option<&TypeCtx>,
) -> ResourceEffectBoundaryReport {
    check_resource_effect_boundaries_with_types_and_private_cache_mask_proofs(
        module,
        types,
        &PrivateCacheMaskProofIndex::empty(),
    )
}

fn check_resource_effect_boundaries_with_types_and_private_cache_mask_proofs(
    module: &ResourceModule,
    types: Option<&TypeCtx>,
    private_cache_mask_proofs: &PrivateCacheMaskProofIndex,
) -> ResourceEffectBoundaryReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let pointer_summaries = compute_raw_pointer_return_summaries(module, types);
    let pointer_summary_index = RawPointerReturnSummaryIndex::new(&pointer_summaries);
    let summaries = compute_raw_identity_return_summaries(module, &pointer_summaries, types);
    let summary_index = RawIdentityReturnSummaryIndex::new(&summaries);

    for function in &module.functions {
        let track_effect_provenance = function_needs_effect_provenance_tracking(function, types);
        let mut engine = ResourceEffectBoundaryEngine {
            function: function.name.as_str(),
            effect: function.effect,
            summaries: &summary_index,
            pointer_summaries: &pointer_summary_index,
            types,
            track_alloc_identities: track_effect_provenance,
            propagate_return_provenance: track_effect_provenance,
            private_cache_mask_proofs,
            diagnostics: Vec::new(),
            counts: ResourceEffectCounts::default(),
        };
        engine.check_function(function);
        diagnostics.extend(engine.diagnostics);
        report_internal_alloc_escapes_from_summary(
            &mut diagnostics,
            function,
            summary_index.get(function.name.as_str()),
            types,
        );
        functions.push(ResourceEffectFunctionCheck {
            name: function.name.clone(),
            counts: engine.counts,
        });
    }

    ResourceEffectBoundaryReport {
        functions,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::effects::PrivateEffectRegionId;
    use crate::span::Span;
    use crate::types::TypeId;

    use super::super::effect_diagnostic::ResourceEffectBoundaryDiagnostic;
    use super::super::model::{
        EffectOp, Place, PrivateCacheOp, PrivateEffectRegion, PrivateStateOp, ResourceBlock,
        ResourceBlockId, ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp,
        ResourceTerminator,
    };
    use super::super::private_cache_mask::{PrivateCacheMaskProof, PrivateCacheMaskProofIndex};
    use super::{
        check_resource_effect_boundaries, check_resource_effect_boundaries_typed,
        check_resource_effect_boundaries_with_types_and_private_cache_mask_proofs,
    };

    fn module_with_effect(function_effect: Effect, effect: EffectOp) -> ResourceModule {
        ResourceModule {
            functions: vec![ResourceFunction {
                name: String::from("uses_private_effect"),
                origin_name: String::from("uses_private_effect"),
                type_params: Vec::new(),
                params: Vec::new(),
                result: TypeId(0),
                effect: function_effect,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::CallEffect {
                        effect,
                        span: Span::dummy(),
                    }],
                    terminator: ResourceTerminator::Return {
                        value: None,
                        span: Span::dummy(),
                    },
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            entry: None,
            string_literals: Vec::new(),
        }
    }

    #[test]
    fn private_cache_effect_boundary_rejects_all_ops_in_pure_function_until_masked() {
        for operation in PrivateCacheOp::ALL {
            let report = check_resource_effect_boundaries(&module_with_effect(
                Effect::Pure,
                EffectOp::PrivateCache {
                    operation,
                    region: PrivateEffectRegion::UnsealedIntrinsic,
                },
            ));

            assert_eq!(
                report.diagnostics,
                vec![
                    ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
                        function: String::from("uses_private_effect"),
                        operation,
                        region: PrivateEffectRegion::UnsealedIntrinsic,
                        span: Span::dummy(),
                    },
                    ResourceEffectBoundaryDiagnostic::PrivateCacheInPureFunction {
                        function: String::from("uses_private_effect"),
                        operation,
                        region: PrivateEffectRegion::UnsealedIntrinsic,
                        span: Span::dummy(),
                    }
                ],
                "{operation} must remain fail-closed until a sealed mask proof exists"
            );
            assert_eq!(report.functions[0].counts.private_cache_ops, 1);
        }
    }

    fn module_returning_private_cache_create_output(region: PrivateEffectRegion) -> ResourceModule {
        let cache = Place::temporary(super::super::model::ResourceId(0), TypeId(0));
        ResourceModule {
            functions: vec![ResourceFunction {
                name: String::from("returns_cache"),
                origin_name: String::from("returns_cache"),
                type_params: Vec::new(),
                params: Vec::new(),
                result: TypeId(0),
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::CallEffect {
                            effect: EffectOp::PrivateCache {
                                operation: PrivateCacheOp::Create,
                                region,
                            },
                            span: Span::dummy(),
                        },
                        ResourceOp::Call {
                            output: cache.clone(),
                            target: ResourceCallTarget::Builtin {
                                name: String::from("private_cache_create"),
                            },
                            args: Vec::new(),
                            effect: EffectOp::PrivateCache {
                                operation: PrivateCacheOp::Create,
                                region,
                            },
                            span: Span::dummy(),
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(cache),
                        span: Span::dummy(),
                    },
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }],
            entry: None,
            string_literals: Vec::new(),
        }
    }

    #[test]
    fn private_cache_mask_proof_suppresses_only_pure_context_diagnostic() {
        let region = PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(1));
        let proof_index = PrivateCacheMaskProofIndex::from_proofs(vec![PrivateCacheMaskProof {
            function: String::from("uses_private_effect"),
            region,
            operations: vec![PrivateCacheOp::Lookup],
        }]);
        let report = check_resource_effect_boundaries_with_types_and_private_cache_mask_proofs(
            &module_with_effect(
                Effect::Pure,
                EffectOp::PrivateCache {
                    operation: PrivateCacheOp::Lookup,
                    region,
                },
            ),
            None,
            &proof_index,
        );

        assert_eq!(
            report.diagnostics,
            vec![ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
                function: String::from("uses_private_effect"),
                operation: PrivateCacheOp::Lookup,
                region,
                span: Span::dummy(),
            }],
            "non-escape proof only answers the Pure mask question; SourceCapability still guards the operation boundary",
        );
    }

    #[test]
    fn private_cache_mask_proof_rejects_unproven_region() {
        let proven_region =
            PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(1));
        let unproven_region =
            PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(2));
        let proof_index = PrivateCacheMaskProofIndex::from_proofs(vec![PrivateCacheMaskProof {
            function: String::from("uses_private_effect"),
            region: proven_region,
            operations: vec![PrivateCacheOp::Lookup],
        }]);
        let report = check_resource_effect_boundaries_with_types_and_private_cache_mask_proofs(
            &module_with_effect(
                Effect::Pure,
                EffectOp::PrivateCache {
                    operation: PrivateCacheOp::Lookup,
                    region: unproven_region,
                },
            ),
            None,
            &proof_index,
        );

        assert_eq!(
            report.diagnostics,
            vec![
                ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
                    function: String::from("uses_private_effect"),
                    operation: PrivateCacheOp::Lookup,
                    region: unproven_region,
                    span: Span::dummy(),
                },
                ResourceEffectBoundaryDiagnostic::PrivateCacheInPureFunction {
                    function: String::from("uses_private_effect"),
                    operation: PrivateCacheOp::Lookup,
                    region: unproven_region,
                    span: Span::dummy(),
                }
            ],
            "a sealed proof must not mask another sealed private cache region",
        );
    }

    #[test]
    fn sealed_private_cache_create_output_cannot_escape_through_return() {
        let region = PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(11));
        let cache = Place::temporary(super::super::model::ResourceId(0), TypeId(0));
        let report =
            check_resource_effect_boundaries(&module_returning_private_cache_create_output(region));

        assert_eq!(
            report.diagnostics,
            vec![
                ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
                    function: String::from("returns_cache"),
                    operation: PrivateCacheOp::Create,
                    region,
                    span: Span::dummy(),
                },
                ResourceEffectBoundaryDiagnostic::PrivateCacheInPureFunction {
                    function: String::from("returns_cache"),
                    operation: PrivateCacheOp::Create,
                    region,
                    span: Span::dummy(),
                },
                ResourceEffectBoundaryDiagnostic::PrivateCacheRegionEscape {
                    function: String::from("returns_cache"),
                    region,
                    place: cache,
                    span: Span::dummy(),
                },
            ],
            "a sealed cache handle must not be allowed to escape just because its region is known",
        );
    }

    #[test]
    fn typed_effect_check_tracks_private_cache_taint_without_checked_mem_ptr_access() {
        let region = PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(12));
        let cache = Place::temporary(super::super::model::ResourceId(0), TypeId(0));
        let types = crate::types::TypeCtx::new();
        let report = check_resource_effect_boundaries_typed(
            &module_returning_private_cache_create_output(region),
            &types,
        );

        assert!(
            report.diagnostics.contains(
                &ResourceEffectBoundaryDiagnostic::PrivateCacheRegionEscape {
                    function: String::from("returns_cache"),
                    region,
                    place: cache,
                    span: Span::dummy(),
                }
            ),
            "typed effect checking must not skip private cache non-escape tracking just because raw MemPtr provenance is not needed",
        );
    }

    #[test]
    fn private_state_effect_is_rejected_in_pure_function_until_masked() {
        let report = check_resource_effect_boundaries(&module_with_effect(
            Effect::Pure,
            EffectOp::PrivateState {
                operation: PrivateStateOp::Write,
                region: PrivateEffectRegion::UnsealedIntrinsic,
            },
        ));

        assert_eq!(
            report.diagnostics,
            vec![
                ResourceEffectBoundaryDiagnostic::PrivateStateInPureFunction {
                    function: String::from("uses_private_effect"),
                    operation: PrivateStateOp::Write,
                    region: PrivateEffectRegion::UnsealedIntrinsic,
                    span: Span::dummy(),
                }
            ]
        );
        assert_eq!(report.functions[0].counts.private_state_ops, 1);
    }

    #[test]
    fn private_effect_is_not_silently_counted_as_unknown_in_impure_function() {
        for operation in PrivateCacheOp::ALL {
            let report = check_resource_effect_boundaries(&module_with_effect(
                Effect::Impure,
                EffectOp::PrivateCache {
                    operation,
                    region: PrivateEffectRegion::UnsealedIntrinsic,
                },
            ));

            assert_eq!(
                report.diagnostics,
                vec![
                    ResourceEffectBoundaryDiagnostic::PrivateCacheOutsideBoundary {
                        function: String::from("uses_private_effect"),
                        operation,
                        region: PrivateEffectRegion::UnsealedIntrinsic,
                        span: Span::dummy(),
                    }
                ]
            );
            assert_eq!(report.functions[0].counts.private_cache_ops, 1);
            assert_eq!(report.functions[0].counts.unknown_ops, 0);
        }
    }
}
