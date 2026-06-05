#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const selfhostRoot = path.join(repoRoot, "stdlib", "neplg2");
const DOC_GAP_TRACKING_ISSUE = "issues/items/ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41.md";

const BASELINE = {
    moduleNoDoc: 71,
    moduleNoDoctest: 65,
    declarationNoDoc: 64,
    declarationNoDoctest: 1668,
    publicNoDoc: 28,
    publicNoDoctest: 1257,
    privateNoDoc: 36,
    privateNoDoctest: 411,
};
const HARD_DOC_BASELINE_KEYS = [
    "moduleNoDoc",
    "declarationNoDoc",
    "publicNoDoc",
    "privateNoDoc",
];
const REPORT_ONLY_DOCTEST_BASELINE_KEYS = [
    "moduleNoDoctest",
    "declarationNoDoctest",
    "publicNoDoctest",
    "privateNoDoctest",
];

const PUBLIC_DOC_REQUIRED_PREFIXES = [
    "stdlib/neplg2/cli/args/emit.nepl",
    "stdlib/neplg2/core/check/expr/argument.nepl",
    "stdlib/neplg2/core/check/expr/ascription.nepl",
    "stdlib/neplg2/core/check/expr/call_reduce.nepl",
    "stdlib/neplg2/core/check/module/",
    "stdlib/neplg2/core/hir/hir/expr.nepl",
    "stdlib/neplg2/core/proof.nepl",
    "stdlib/neplg2/core/proof/solver/module.nepl",
    "stdlib/neplg2/core/proof/solver/resource.nepl",
    "stdlib/neplg2/core/proof/solver/type.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/project.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/build.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/model.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/plan.nepl",
    "stdlib/neplg2/core/syntax/lexer/",
];
const REQUIRED_SCANNER_SENTINELS = [
    "stdlib/neplg2/cli/args/emit.nepl",
    "stdlib/neplg2/core/check/module/summary.nepl",
    "stdlib/neplg2/core/check/module/declaration_adapter.nepl",
    "stdlib/neplg2/core/hir/hir/expr.nepl",
    "stdlib/neplg2/core/proof.nepl",
    "stdlib/neplg2/core/proof/solver/module.nepl",
    "stdlib/neplg2/core/proof/solver/resource.nepl",
    "stdlib/neplg2/core/proof/solver/type.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/project.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/build.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/model.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/plan.nepl",
    "stdlib/neplg2/core/syntax/lexer/byte.nepl",
];
const MODULE_DOC_SECTION_REQUIREMENTS = [
    moduleRequirement("stdlib/neplg2/core/proof.nepl", ["purpose", "contract", "current", "complexity", "doctest", "authorityBoundary"], {
        doctestUses: [
            "selfhost_proof_source_span_valid",
            "SelfhostProofResult",
            "Result::Ok",
            "Result::Err",
        ],
        requiredPatterns: [
            requiredPattern("typed fact model", /\bSelfhostProofFact\b/),
            requiredPattern("typed obligation model", /\bSelfhostProofObligation\b/),
            requiredPattern("typed proof evidence", /\bSelfhostProofEvidence\b/),
            requiredPattern("typed proof refutation", /\bSelfhostProofRefutation\b/),
            requiredPattern("typed proof result", /\bSelfhostProofResult\b/),
            requiredPattern("solver implementation boundary", /\bproof\/solver\b/),
            requiredPattern("public API wrapper boundary", /\bproof\/api\b/),
            requiredPattern("facade re-export boundary", /\bre-export\b/),
            requiredPattern("no source text reread in facade", /source text の再読/),
            requiredPattern("no checker-local allowlist proof substitute", /allowlist 判定/),
        ],
    }),
    moduleRequirement("stdlib/neplg2/core/resolve/type_resolver/project.nepl", ["purpose", "contract", "current", "complexity", "authorityBoundary", "ownerBoundary", "typeBoundary"], {
        requiredPatterns: [
            requiredPattern("resolved tree root projection", /\bSelfhostResolvedTypeTreeRoot\b/),
            requiredPattern("arena allocation return boundary", /\bSelfhostTypeArenaAlloc\b/),
            requiredPattern("constructor table authority", /\bSelfhostTypeConstructorTable\b/),
            requiredPattern("fail-closed project error", /\bSelfhostTypeProjectErrorKind\b/),
            requiredPattern("current binder depth limitation", /\bbinder_depth = 0\b/),
            requiredPattern("no import graph scan", /import graph/),
        ],
    }),
    moduleRequirement("stdlib/neplg2/core/resolve/type_resolver/reduce.nepl", ["purpose", "contract", "current", "complexity", "authorityBoundary", "ownerBoundary", "typeBoundary", "errorVariant"], {
        requiredPatterns: [
            requiredPattern("plain reduce plan boundary", /\bSelfhostTypeReducePlan\b/),
            requiredPattern("bound reduce plan boundary", /\bSelfhostTypeBoundPlan\b/),
            requiredPattern("resolved tree root output", /\bSelfhostResolvedTypeTreeRoot\b/),
            requiredPattern("zero-argument void marker", /fn void T|void.*0 引数 marker/),
            requiredPattern("unit remains type and value", /unit.*型.*値|unit 型.*unit 値/),
            requiredPattern("generic argument missing error", /\bSelfhostTypeReduceErrorKind::GenericTypeArgumentMissing\b/),
            requiredPattern("constructor type parameter conflict", /\bSelfhostTypeReduceErrorKind::TypeParameterConstructorNameConflict\b/),
            requiredPattern("trailing items error", /\bTrailingItems\b/),
            requiredPattern("no import graph scan", /import graph/),
        ],
    }),
    moduleRequirement("stdlib/neplg2/core/resolve/type_resolver/reduce/build.nepl", ["purpose", "contract", "current", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], {
        requiredPatterns: [
            requiredPattern("reduce build state owner", /\bSelfhostTypeReduceBuildState\b/),
            requiredPattern("reduce step owner result", /\bSelfhostTypeReduceStep\b/),
            requiredPattern("function argument range", /\bSelfhostResolvedFunctionArgRange\b/),
            requiredPattern("empty function argument range", /\bSelfhostResolvedFunctionArgRange::Empty\b/),
            requiredPattern("void marker is not node", /void.*型 node ではない/),
            requiredPattern("unit remains normal type", /unit.*通常の型/),
            requiredPattern("internal invariant or out of memory", /\bSelfhostTypeReduceErrorKind::(?:InternalInvariant|OutOfMemory)\b/),
            requiredPattern("source text is not reread", /source text を読まず/),
        ],
    }),
    moduleRequirement("stdlib/neplg2/core/resolve/type_resolver/reduce/model.nepl", ["purpose", "contract", "current", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], {
        requiredPatterns: [
            requiredPattern("typed reduce error enum", /\bSelfhostTypeReduceErrorKind\b/),
            requiredPattern("all reduce error variants", /\bEmptyInput\b[\s\S]*\bUnexpectedEnd\b[\s\S]*\bVoidAsType\b[\s\S]*\bFunctionMissingArgument\b[\s\S]*\bFunctionMissingResult\b[\s\S]*\bGenericTypeArgumentMissing\b[\s\S]*\bTypeParameterConstructorNameConflict\b[\s\S]*\bUnsupportedTypePrefixItem\b[\s\S]*\bTrailingItems\b[\s\S]*\bOutOfMemory\b[\s\S]*\bInternalInvariant\b/),
            requiredPattern("build state owner payload", /\bSelfhostTypeReduceBuildState\b/),
            requiredPattern("type args owner", /\btype_args\b/),
            requiredPattern("build state cleanup helper", /\bselfhost_type_reduce_build_state_free\b/),
            requiredPattern("reduce fail helper", /\bselfhost_type_reduce_fail\b/),
            requiredPattern("void is not type", /void.*型ではない|VoidAsType/),
            requiredPattern("unit remains ordinary type", /unit.*通常|unit.*型/),
        ],
    }),
    moduleRequirement("stdlib/neplg2/core/resolve/type_resolver/reduce/plan.nepl", ["purpose", "contract", "current", "complexity", "authorityBoundary", "ownerBoundary", "typeBoundary", "errorVariant"], {
        requiredPatterns: [
            requiredPattern("reduce plan owner", /\bSelfhostTypeReducePlan\b/),
            requiredPattern("bound plan owner", /\bSelfhostTypeBoundPlan\b/),
            requiredPattern("constructor table authority", /\bSelfhostTypeConstructorTable\b/),
            requiredPattern("type parameter environment authority", /\bSelfhostTypeParameterEnv\b/),
            requiredPattern("void marker classification", /\bSelfhostTypeReduceDispatchKind::VoidMarker\b/),
            requiredPattern("void as type error", /\bSelfhostTypeReduceErrorKind::VoidAsType\b/),
            requiredPattern("unit primitive kind", /\bSelfhostPrimitiveTypeKind::Unit\b/),
            requiredPattern("conflict bound name", /\bSelfhostTypeBoundName::Conflict\b/),
            requiredPattern("linear lookup current implementation", /線形検索/),
        ],
    }),
];
const DOC_SECTION_REQUIREMENTS = [
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_new", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_empty", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_all", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/cli/args/emit.nepl", "selfhost_cli_emit_set_add", ["purpose", "contract", "complexity"]),
    typeProjectRequirement("SelfhostTypeProjectErrorKind", ["purpose", "contract", "complexity", "errorVariant", "authorityBoundary"], [
        requiredPattern("missing node error", /\bSelfhostTypeProjectErrorKind::MissingResolvedNode\b|`MissingResolvedNode`/),
        requiredPattern("unsupported named type error", /\bSelfhostTypeProjectErrorKind::UnsupportedNamedType\b|`UnsupportedNamedType`/),
        requiredPattern("unknown named type error", /\bSelfhostTypeProjectErrorKind::UnknownNamedType\b|`UnknownNamedType`/),
        requiredPattern("generic constructor needs args error", /\bSelfhostTypeProjectErrorKind::GenericConstructorNeedsArguments\b|`GenericConstructorNeedsArguments`/),
        requiredPattern("generic constructor arity error", /\bSelfhostTypeProjectErrorKind::GenericConstructorArgumentArityMismatch\b|`GenericConstructorArgumentArityMismatch`/),
        requiredPattern("out of memory error", /\bSelfhostTypeProjectErrorKind::OutOfMemory\b|`OutOfMemory`/),
        requiredPattern("internal invariant error", /\bSelfhostTypeProjectErrorKind::InternalInvariant\b|`InternalInvariant`/),
        requiredPattern("display layer separation", /表示文言/),
    ]),
    typeProjectRequirement("SelfhostTypeProjectError", ["purpose", "contract", "complexity", "errorVariant", "authorityBoundary"], [
        requiredPattern("typed project error kind", /\bSelfhostTypeProjectErrorKind\b/),
        requiredPattern("diagnostic span payload", /\bSelfhostSourceSpan\b/),
    ]),
    typeProjectRequirement("SelfhostTypeProjectArgBuild", ["purpose", "contract", "complexity", "ownerBoundary"], [
        requiredPattern("arena owner payload", /\bSelfhostTypeArena\b/),
        requiredPattern("parameter vector owner payload", /\bVec SelfhostTypeId\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_error_kind_eq", ["purpose", "returns", "complexity", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_error_new", ["purpose", "contract", "returns", "complexity", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_empty_span", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    typeProjectRequirement("selfhost_type_project_arg_build_new", ["purpose", "contract", "returns", "complexity", "ownerBoundary"]),
    typeProjectRequirement("selfhost_type_project_arg_build_free", ["purpose", "contract", "returns", "complexity", "ownerBoundary"]),
    typeProjectRequirement("selfhost_type_project_fail", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_fail_with_params", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_std_error_kind", ["purpose", "returns", "complexity", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_push_arg_id", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_alloc_function_type", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant", "typeBoundary"], [
        requiredPattern("function arena add consumes params", /\bselfhost_type_arena_add_function\b/),
        requiredPattern("function record type", /\bSelfhostTypeRecord::Function\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_parameter_binding_for_current_binder", ["purpose", "current", "returns", "complexity", "typeBoundary"], [
        requiredPattern("binder depth current limitation", /\bbinder_depth = 0\b/),
        requiredPattern("type parameter binding", /\bSelfhostTypeParameterBinding\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_parameter", ["purpose", "contract", "returns", "complexity", "typeBoundary", "errorVariant"], [
        requiredPattern("parameter node projection", /\bSelfhostResolvedTypeNode::Parameter\b/),
        requiredPattern("type parameter record", /\bSelfhostTypeRecord::Parameter\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_function_args_loop", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_function_args", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_function", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary"]),
    typeProjectRequirement("selfhost_type_project_node", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "typeBoundary"], [
        requiredPattern("primitive node branch", /\bSelfhostResolvedTypeNode::Primitive\b/),
        requiredPattern("parameter node branch", /\bSelfhostResolvedTypeNode::Parameter\b/),
        requiredPattern("function node branch", /\bSelfhostResolvedTypeNode::Function\b/),
        requiredPattern("named node rejected without constructors", /\bSelfhostResolvedTypeNode::Named\b/),
        requiredPattern("applied node rejected without constructors", /\bSelfhostResolvedTypeNode::Applied\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_applied_args_loop_with_constructors", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "authorityBoundary", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_applied_args_with_constructors", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary"]),
    typeProjectRequirement("selfhost_type_project_applied_constructor_arg_count_is_valid", ["purpose", "contract", "returns", "complexity", "typeBoundary"], [
        requiredPattern("type constructor kind", /\bSelfhostTypeConstructorKind::TypeConstructor\b|TypeConstructor/),
        requiredPattern("bare type is not applied", /\bSelfhostTypeConstructorKind::Type\b|`Type` constructor/),
    ]),
    typeProjectRequirement("selfhost_type_project_applied_with_constructors", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("constructor table lookup", /\bselfhost_type_constructor_table_get\b|\bSelfhostTypeConstructorTable\b/),
        requiredPattern("applied record", /\bSelfhostTypeRecord::Applied\b/),
        requiredPattern("unknown named type error", /\bSelfhostTypeProjectErrorKind::UnknownNamedType\b|\bUnknownNamedType\b/),
        requiredPattern("arity mismatch error", /\bSelfhostTypeProjectErrorKind::GenericConstructorArgumentArityMismatch\b|\bGenericConstructorArgumentArityMismatch\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_named_with_constructors", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("span lookup authority", /\bselfhost_type_constructor_table_find_span\b/),
        requiredPattern("named record", /\bSelfhostTypeRecord::Named\b/),
        requiredPattern("generic constructor needs arguments", /\bSelfhostTypeProjectErrorKind::GenericConstructorNeedsArguments\b|\bGenericConstructorNeedsArguments\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_function_args_loop_with_constructors", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "authorityBoundary", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_function_args_with_constructors", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"]),
    typeProjectRequirement("selfhost_type_project_function_with_constructors", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary"]),
    typeProjectRequirement("selfhost_type_project_node_with_constructors", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("constructor-aware named branch", /\bSelfhostResolvedTypeNode::Named\b/),
        requiredPattern("constructor-aware applied branch", /\bSelfhostResolvedTypeNode::Applied\b/),
        requiredPattern("missing node error", /\bSelfhostTypeProjectErrorKind::MissingResolvedNode\b|\bMissingResolvedNode\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_root_into_arena", ["purpose", "contract", "current", "returns", "complexity", "ownerBoundary", "errorVariant", "typeBoundary"], [
        requiredPattern("root projection without constructors", /\bSelfhostResolvedTypeTreeRoot\b/),
        requiredPattern("unsupported named type fail closed", /\bUnsupportedNamedType\b/),
    ]),
    typeProjectRequirement("selfhost_type_project_root_with_constructors_into_arena", ["purpose", "contract", "current", "returns", "complexity", "authorityBoundary", "ownerBoundary", "errorVariant", "typeBoundary"], [
        requiredPattern("constructor table public boundary", /\bSelfhostTypeConstructorTable\b/),
        requiredPattern("applied type record", /\bSelfhostTypeRecord::Applied\b/),
        requiredPattern("no full module scan", /全module探索|import graph/),
    ]),
    typeReduceModelRequirement("SelfhostTypeReduceErrorKind", ["purpose", "contract", "complexity", "errorVariant", "typeBoundary"], [
        requiredPattern("empty input error", /\bEmptyInput\b/),
        requiredPattern("unexpected end error", /\bUnexpectedEnd\b/),
        requiredPattern("void as type error", /\bVoidAsType\b/),
        requiredPattern("function missing argument error", /\bFunctionMissingArgument\b/),
        requiredPattern("function missing result error", /\bFunctionMissingResult\b/),
        requiredPattern("generic type argument missing error", /\bGenericTypeArgumentMissing\b/),
        requiredPattern("type parameter constructor conflict error", /\bTypeParameterConstructorNameConflict\b/),
        requiredPattern("unsupported type prefix item error", /\bUnsupportedTypePrefixItem\b/),
        requiredPattern("trailing items error", /\bTrailingItems\b/),
        requiredPattern("out of memory error", /\bOutOfMemory\b/),
        requiredPattern("internal invariant error", /\bInternalInvariant\b/),
        requiredPattern("display layer separation", /表示文言|message string/),
    ]),
    typeReduceModelRequirement("SelfhostTypeReduceError", ["purpose", "contract", "complexity", "errorVariant"], [
        requiredPattern("typed error kind", /\bSelfhostTypeReduceErrorKind\b/),
        requiredPattern("source span payload", /\bSelfhostSourceSpan\b/),
    ]),
    typeReduceModelRequirement("SelfhostTypeReduceBuildState", ["purpose", "contract", "complexity", "ownerBoundary", "typeBoundary"], [
        requiredPattern("node owner table", /\bnodes\b/),
        requiredPattern("function args owner table", /\bfunction_args\b/),
        requiredPattern("type args owner table", /\btype_args\b/),
        requiredPattern("applied arg range", /\bSelfhostResolvedAppliedTypeArgRange\b/),
        requiredPattern("function arg range", /\bSelfhostResolvedFunctionArgRange\b/),
        requiredPattern("build state cleanup helper", /\bselfhost_type_reduce_build_state_free\b/),
        requiredPattern("reduce fail cleanup helper", /\bselfhost_type_reduce_fail\b/),
    ]),
    typeReduceModelRequirement("SelfhostTypeReduceStep", ["purpose", "contract", "complexity", "ownerBoundary", "typeBoundary"], [
        requiredPattern("node id boundary", /\bSelfhostResolvedTypeNodeId\b/),
        requiredPattern("next index boundary", /\bnext_index\b/),
        requiredPattern("into tree owner move", /\bselfhost_type_reduce_step_into_tree\b/),
        requiredPattern("into build state owner move", /\bselfhost_type_reduce_step_into_build_state\b/),
    ]),
    typeReduceModelRequirement("SelfhostTypeReducePlan", ["purpose", "contract", "complexity", "ownerBoundary", "authorityBoundary", "typeBoundary"], [
        requiredPattern("plan free owner", /\bselfhost_type_reduce_plan_free\b/),
        requiredPattern("source text authority", /source text|source string/),
        requiredPattern("plan item payload", /\bSelfhostTypeReducePlanItem\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_error_kind_eq", ["purpose", "contract", "returns", "complexity", "errorVariant"], [
        requiredPattern("all variants in match", /\bEmptyInput\b[\s\S]*\bUnexpectedEnd\b[\s\S]*\bVoidAsType\b[\s\S]*\bFunctionMissingArgument\b[\s\S]*\bFunctionMissingResult\b[\s\S]*\bGenericTypeArgumentMissing\b[\s\S]*\bTypeParameterConstructorNameConflict\b[\s\S]*\bUnsupportedTypePrefixItem\b[\s\S]*\bTrailingItems\b[\s\S]*\bOutOfMemory\b[\s\S]*\bInternalInvariant\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_error_new", ["purpose", "contract", "returns", "complexity", "errorVariant"]),
    typeReduceModelRequirement("selfhost_type_reduce_build_state_new", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary"], [
        requiredPattern("type args owner", /\btype_args\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_build_state_from_tree", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary"], [
        requiredPattern("resolved tree owner", /\bSelfhostResolvedTypeTree\b/),
        requiredPattern("type args owner", /\btype_args\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_build_state_free", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary"], [
        requiredPattern("nodes cleanup", /\bnodes\b/),
        requiredPattern("function args cleanup", /\bfunction_args\b/),
        requiredPattern("type args cleanup", /\btype_args\b/),
        requiredPattern("applied arg range", /\bSelfhostResolvedAppliedTypeArgRange\b/),
        requiredPattern("function arg range", /\bSelfhostResolvedFunctionArgRange\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_build_state_function_arg", ["purpose", "contract", "returns", "complexity", "typeBoundary"], [
        requiredPattern("empty function arg range branch", /\bSelfhostResolvedFunctionArgRange::Empty\b/),
        requiredPattern("range branch", /\bSelfhostResolvedFunctionArgRange::Range\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_step_into_tree", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary"], [
        requiredPattern("resolved tree owner", /\bSelfhostResolvedTypeTree\b/),
        requiredPattern("type args owner", /\btype_args\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_step_into_build_state", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary"], [
        requiredPattern("build state owner", /\bSelfhostTypeReduceBuildState\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_prefix_reduce_prefix_result_free", ["contract", "returns", "complexity", "ownerBoundary", "typeBoundary"], [
        requiredPattern("resolved tree root free", /\bselfhost_resolved_type_tree_root_free\b|\bSelfhostResolvedTypeTreeRoot\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_fail", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"], [
        requiredPattern("build state cleanup helper", /\bselfhost_type_reduce_build_state_free\b/),
        requiredPattern("typed reduce error kind", /\bSelfhostTypeReduceErrorKind\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_empty_span", ["purpose", "contract", "returns", "complexity", "authorityBoundary"], [
        requiredPattern("empty source span", /\bsource_span_empty_unchecked\b/),
    ]),
    typeReduceModelRequirement("selfhost_type_reduce_dispatch_kind_error", ["purpose", "contract", "returns", "complexity", "errorVariant", "typeBoundary"], [
        requiredPattern("void marker maps to void as type", /\bSelfhostTypeReduceDispatchKind::VoidMarker\b[\s\S]*\bSelfhostTypeReduceErrorKind::VoidAsType\b/),
        requiredPattern("unsupported item maps to unsupported error", /\bUnsupportedTypePrefixItem\b/),
    ]),
    typeReducePlanRequirement("selfhost_type_reduce_dispatch_kind", ["purpose", "contract", "returns", "complexity", "typeBoundary"], [
        requiredPattern("void marker dispatch", /\bSelfhostTypeReduceDispatchKind::VoidMarker\b/),
        requiredPattern("unsupported dispatch", /\bSelfhostTypeReduceDispatchKind::UnsupportedTypePrefixItem\b/),
    ]),
    typeReducePlanRequirement("selfhost_type_reduce_primitive_from_lexeme", ["purpose", "contract", "returns", "complexity", "typeBoundary"], [
        requiredPattern("unit primitive", /\bSelfhostPrimitiveTypeKind::Unit\b/),
        requiredPattern("void is not primitive", /void.*primitive type ではない/),
    ]),
    typeReducePlanRequirement("selfhost_type_reduce_primitive_from_span", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "typeBoundary"], [
        requiredPattern("source read boundary", /source text を読む/),
        requiredPattern("primitive type kind", /\bSelfhostPrimitiveTypeKind\b/),
    ]),
    typeReducePlanRequirement("selfhost_type_reduce_plan_item_from_prefix_item", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("void marker item", /\bVoidMarker\b/),
        requiredPattern("unsupported type prefix item", /\bSelfhostTypeReduceErrorKind::UnsupportedTypePrefixItem\b|\bUnsupportedTypePrefixItem\b/),
    ]),
    typeReducePlanRequirement("selfhost_type_reduce_plan_from_prefix_list_loop", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "ownerBoundary", "errorVariant", "typeBoundary"], [
        requiredPattern("source primitive authority", /source text から primitive 判定/),
        requiredPattern("unexpected end error", /\bSelfhostTypeReduceErrorKind::UnexpectedEnd\b/),
        requiredPattern("out of memory error", /\bSelfhostTypeReduceErrorKind::OutOfMemory\b/),
    ]),
    typeReducePlanRequirement("selfhost_type_reduce_plan_from_prefix_list", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "ownerBoundary", "errorVariant", "typeBoundary"], [
        requiredPattern("reduce plan owner", /\bSelfhostTypeReducePlan\b/),
        requiredPattern("reduce plan free", /\bselfhost_type_reduce_plan_free\b/),
    ]),
    typeReducePlanRequirement("selfhost_type_prefix_list_validate_at", ["purpose", "contract", "returns", "complexity", "typeBoundary", "errorVariant"], [
        requiredPattern("void as type error", /\bSelfhostTypeReduceErrorKind::VoidAsType\b/),
        requiredPattern("unexpected end error", /\bSelfhostTypeReduceErrorKind::UnexpectedEnd\b/),
    ]),
    typeReducePlanRequirement("selfhost_type_prefix_list_validate_function_nonvoid_arg", ["purpose", "contract", "returns", "complexity", "typeBoundary", "errorVariant"], [
        requiredPattern("function missing result", /\bSelfhostTypeReduceErrorKind::FunctionMissingResult\b/),
        requiredPattern("non void argument", /void marker ではない/),
    ]),
    typeReducePlanRequirement("selfhost_type_prefix_list_validate_function", ["purpose", "contract", "returns", "complexity", "typeBoundary", "errorVariant"], [
        requiredPattern("void function marker", /\bfn void T\b/),
        requiredPattern("unit argument function", /\bfn unit T\b/),
        requiredPattern("function missing argument", /\bSelfhostTypeReduceErrorKind::FunctionMissingArgument\b/),
        requiredPattern("function missing result", /\bSelfhostTypeReduceErrorKind::FunctionMissingResult\b/),
    ]),
    typeReducePlanRequirement("SelfhostTypeBoundPlanItem", ["purpose", "contract", "complexity", "typeBoundary"], [
        requiredPattern("constructor branch", /\bConstructor\b/),
        requiredPattern("type parameter branch", /\bTypeParameter\b/),
        requiredPattern("conflict branch", /\bConflict\b/),
        requiredPattern("unresolved branch", /\bUnresolved\b/),
    ]),
    typeReducePlanRequirement("SelfhostTypeBoundPlan", ["purpose", "contract", "complexity", "ownerBoundary", "authorityBoundary", "typeBoundary"], [
        requiredPattern("bound plan free owner", /\bselfhost_type_bound_plan_free\b/),
        requiredPattern("bound item payload", /\bSelfhostTypeBoundPlanItem\b/),
        requiredPattern("constructor authority", /constructor arity/),
        requiredPattern("type parameter authority", /type parameter/),
    ]),
    typeReduceRequirement("SelfhostTypeReduceGenericArgBuild", ["purpose", "contract", "complexity", "ownerBoundary"], [
        requiredPattern("reduce build state owner payload", /\bSelfhostTypeReduceBuildState\b/),
        requiredPattern("generic arg vector owner payload", /\bVec SelfhostResolvedTypeNodeId\b/),
    ]),
    typeReduceRequirement("selfhost_type_reduce_generic_arg_build_new", ["purpose", "contract", "returns", "complexity", "ownerBoundary"]),
    typeReduceRequirement("selfhost_type_reduce_generic_arg_build_fail", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"], [
        requiredPattern("state cleanup helper", /\bselfhost_type_reduce_build_state_free\b/),
        requiredPattern("generic argument vector cleanup", /\bv::free\b/),
        requiredPattern("typed reduce error kind", /\bSelfhostTypeReduceErrorKind\b/),
    ]),
    typeReduceRequirement("selfhost_type_reduce_generic_arg_push", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"], [
        requiredPattern("out of memory branch", /\bSelfhostTypeReduceErrorKind::OutOfMemory\b/),
        requiredPattern("push error vector cleanup", /\bvec_push_error_vec\b/),
    ]),
    typeReduceRequirement("selfhost_type_reduce_push_node_with_state", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("resolved node variants", /\bSelfhostResolvedTypeNode::\{Primitive, Named, Parameter, Applied, Function\}\b|\bSelfhostResolvedTypeNode::(?:Applied|Function)\b/),
        requiredPattern("node id from table length", /\bSelfhostResolvedTypeNodeId\b/),
        requiredPattern("out of memory branch", /\bSelfhostTypeReduceErrorKind::OutOfMemory\b/),
    ]),
    typeReduceRequirement("selfhost_type_reduce_copy_applied_args", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"], [
        requiredPattern("index out of bounds branch", /\bStdErrorKind::IndexOutOfBounds\b/),
        requiredPattern("applied arg vector owner", /\bVec SelfhostResolvedTypeNodeId\b/),
    ]),
    typeReduceRequirement("selfhost_type_reduce_add_applied_named_from_params", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("applied type arg range", /\bSelfhostResolvedAppliedTypeArgRange\b/),
        requiredPattern("applied node variant", /\bSelfhostResolvedTypeNode::Applied\b/),
        requiredPattern("internal invariant branch", /\bSelfhostTypeReduceErrorKind::InternalInvariant\b/),
        requiredPattern("out of memory branch", /\bSelfhostTypeReduceErrorKind::OutOfMemory\b/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_reduce_prefix_from_plan", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("plain plan borrow boundary", /\bSelfhostTypeReducePlan\b/),
        requiredPattern("prefix reduce result", /\bSelfhostTypePrefixReducePrefixResult\b/),
        requiredPattern("trailing items not checked", /\bTrailingItems\b/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_reduce_prefix_from_bound_plan", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("bound plan borrow boundary", /\bSelfhostTypeBoundPlan\b/),
        requiredPattern("void marker no resolved node", /void.*SelfhostResolvedTypeNode/),
        requiredPattern("generic and conflict errors", /\bSelfhostTypeReduceErrorKind::(?:GenericTypeArgumentMissing|TypeParameterConstructorNameConflict|VoidAsType)\b/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_validate_at_bound", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("bound plan authority", /\bSelfhostTypeBoundPlan\b/),
        requiredPattern("void marker error", /\bSelfhostTypeReduceErrorKind::VoidAsType\b/),
        requiredPattern("generic missing error", /\bSelfhostTypeReduceErrorKind::GenericTypeArgumentMissing\b/),
        requiredPattern("conflict error", /\bSelfhostTypeReduceErrorKind::TypeParameterConstructorNameConflict\b/),
        requiredPattern("void function marker", /\bfn void T\b/),
        requiredPattern("unit argument function", /\bfn unit T\b/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_build_at_bound", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("bound plan authority", /\bSelfhostTypeBoundPlan\b/),
        requiredPattern("reduce step owner", /\bSelfhostTypeReduceStep\b/),
        requiredPattern("applied node", /\bSelfhostResolvedTypeNode::Applied\b/),
        requiredPattern("parameter node", /\bSelfhostResolvedTypeNode::Parameter\b/),
        requiredPattern("empty function arg range", /\bSelfhostResolvedFunctionArgRange::Empty\b/),
        requiredPattern("out of memory error", /\bSelfhostTypeReduceErrorKind::OutOfMemory\b/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_reduce", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("resolved root owner", /\bSelfhostResolvedTypeTreeRoot\b/),
        requiredPattern("trailing items error", /\bSelfhostTypeReduceErrorKind::TrailingItems\b/),
        requiredPattern("empty input error", /\bSelfhostTypeReduceErrorKind::EmptyInput\b/),
        requiredPattern("void as type error", /\bSelfhostTypeReduceErrorKind::VoidAsType\b/),
        requiredPattern("no constructor lookup", /constructor lookup は行いません/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_reduce_prefix", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("prefix result owner", /\bSelfhostTypePrefixReducePrefixResult\b/),
        requiredPattern("prefix result free", /\bselfhost_type_prefix_reduce_prefix_result_free\b/),
        requiredPattern("trailing items not returned", /\bTrailingItems\b/),
        requiredPattern("next index boundary", /\bnext_index\b/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_reduce_with_constructors", ["purpose", "contract", "current", "returns", "complexity", "ownerBoundary", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("constructor table boundary", /\bSelfhostTypeConstructorTable\b/),
        requiredPattern("bound plan owner free", /\bselfhost_type_bound_plan_free\b/),
        requiredPattern("generic argument missing", /\bSelfhostTypeReduceErrorKind::GenericTypeArgumentMissing\b/),
        requiredPattern("linear constructor lookup", /線形検索/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_reduce_prefix_with_constructors", ["purpose", "contract", "current", "returns", "complexity", "ownerBoundary", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("constructor table boundary", /\bSelfhostTypeConstructorTable\b/),
        requiredPattern("prefix result owner", /\bSelfhostTypePrefixReducePrefixResult\b/),
        requiredPattern("generic argument missing", /\bSelfhostTypeReduceErrorKind::GenericTypeArgumentMissing\b/),
        requiredPattern("trailing items not returned", /\bTrailingItems\b/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_reduce_with_constructors_and_type_parameters", ["purpose", "contract", "current", "returns", "complexity", "ownerBoundary", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("constructor table boundary", /\bSelfhostTypeConstructorTable\b/),
        requiredPattern("type parameter env boundary", /\bSelfhostTypeParameterEnv\b/),
        requiredPattern("conflict error", /\bSelfhostTypeReduceErrorKind::TypeParameterConstructorNameConflict\b/),
        requiredPattern("trailing items error", /\bSelfhostTypeReduceErrorKind::TrailingItems\b/),
        requiredPattern("resolver local id current", /\bSelfhostTypeParameterId\b/),
    ]),
    typeReduceRequirement("selfhost_type_prefix_list_reduce_prefix_with_constructors_and_type_parameters", ["purpose", "contract", "current", "returns", "complexity", "ownerBoundary", "authorityBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("constructor table boundary", /\bSelfhostTypeConstructorTable\b/),
        requiredPattern("type parameter env boundary", /\bSelfhostTypeParameterEnv\b/),
        requiredPattern("prefix result owner", /\bSelfhostTypePrefixReducePrefixResult\b/),
        requiredPattern("conflict error", /\bSelfhostTypeReduceErrorKind::TypeParameterConstructorNameConflict\b/),
        requiredPattern("linear type parameter lookup", /線形検索/),
    ]),
    typeReduceBuildRequirement("selfhost_type_reduce_build_state_push_node", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("build state owner", /\bSelfhostTypeReduceBuildState\b/),
        requiredPattern("reduce step owner", /\bSelfhostTypeReduceStep\b/),
        requiredPattern("out of memory branch", /\bSelfhostTypeReduceErrorKind::OutOfMemory\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_reduce_single_param_vec", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"], [
        requiredPattern("single param vector", /\bVec SelfhostResolvedTypeNodeId\b/),
        requiredPattern("push error vector cleanup", /\bvec_push_error_vec\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_reduce_append_inner_args_loop", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("inner function arg range", /inner function.*argument range|function argument range/),
        requiredPattern("index out of bounds branch", /\bStdErrorKind::IndexOutOfBounds\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_reduce_copy_function_args", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "errorVariant"], [
        requiredPattern("function arg table", /function argument table/),
        requiredPattern("index out of bounds branch", /\bStdErrorKind::IndexOutOfBounds\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_reduce_add_function_from_params", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("function type payload", /\bSelfhostResolvedFunctionType\b/),
        requiredPattern("function node variant", /\bSelfhostResolvedTypeNode::Function\b/),
        requiredPattern("empty range for void function", /\bSelfhostResolvedFunctionArgRange::Empty\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_reduce_add_function_empty_params", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("zero argument function", /0 引数 function|fn void T/),
        requiredPattern("void marker no type node", /void.*型 node ではなく/),
        requiredPattern("out of memory branch", /\bSelfhostTypeReduceErrorKind::OutOfMemory\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_reduce_add_function_nonempty_params", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("function flattening", /flatten/),
        requiredPattern("nested void result not flattened", /fn A fn void C|flatten せず/),
        requiredPattern("internal invariant branch", /\bSelfhostTypeReduceErrorKind::InternalInvariant\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_reduce_atom_node_unchecked", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "typeBoundary"], [
        requiredPattern("primitive node", /\bSelfhostResolvedTypeNode::Primitive\b/),
        requiredPattern("named node", /\bSelfhostResolvedTypeNode::Named\b/),
        requiredPattern("plan authority", /\bSelfhostTypeReducePlanItem\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_prefix_list_build_at", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("plain plan input", /\bSelfhostTypeReducePlan\b/),
        requiredPattern("validate before build", /\bselfhost_type_prefix_list_validate_at\b/),
        requiredPattern("unexpected end error", /\bSelfhostTypeReduceErrorKind::UnexpectedEnd\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_prefix_list_build_function_nonvoid_arg", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("non-void argument", /non-void|void でない/),
        requiredPattern("function flattening helper", /\bselfhost_type_reduce_add_function_nonempty_params\b/),
    ]),
    typeReduceBuildRequirement("selfhost_type_prefix_list_build_function", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "typeBoundary", "errorVariant"], [
        requiredPattern("void function split", /\bfn void T\b|void.*argument subtree/),
        requiredPattern("function missing argument", /\bSelfhostTypeReduceErrorKind::FunctionMissingArgument\b/),
    ]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentMatchErrorKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentMatchError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "SelfhostExprArgumentOwnedMatch", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_match", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_checked_argument", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_owned_match_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_expected_type_is_function", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_function_value_error_from_candidate_collect", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_function_value_candidate_is_monomorphic", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_candidate", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_candidates", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_function_value_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_range_from_prefix", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_find_prefix_item_by_token_loop", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_find_prefix_item_by_token", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_validate_ascription_expected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_span_from_ascription_error", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_ascribed_with_projection", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_ascribed_at_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/argument.nepl", "selfhost_expr_argument_match_at_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionProjection", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "SelfhostExprAscriptionHeadProjection", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_expectation", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_tail", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_type_id", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_expectation", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_expression_first_token", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_projection_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_head_projection_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_first_token_is_percent", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_push_type_item", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_type_items_loop", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_type_prefix_list_from_range", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_type_span_from_range", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_tail_span_from_tokens", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_expression_tail_range", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_expression_first_token", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_reduced", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_head_reduced", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_expectation", ["purpose", "contract", "returns", "complexity", "doctest"], {
        doctestUses: [
            "selfhost_expr_ascription_project_expectation",
            "selfhost_expr_ascription_projection_tail",
            "selfhost_expr_ascription_projection_free",
        ],
    }),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_head_expectation", ["purpose", "contract", "returns", "complexity", "doctest"], {
        doctestUses: [
            "selfhost_expr_ascription_project_head_expectation",
            "selfhost_expr_ascription_head_projection_expression_first_token",
            "selfhost_expr_ascription_head_projection_free",
        ],
    }),
    requirement("stdlib/neplg2/core/check/expr/ascription.nepl", "selfhost_expr_ascription_project_expectation_with_constructors", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/block_body.nepl", "selfhost_block_body_result_segment_span", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/block_body.nepl", "selfhost_block_body_result_from_expression_segment", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/block_body.nepl", "selfhost_block_body_result_from_single_segment", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/block_body.nepl", "selfhost_block_body_result_from_segment_list", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/body_line.nepl", "selfhost_check_expr_syntax_range_span", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/body_line.nepl", "selfhost_check_expr_head_starts_with_percent", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/body_line.nepl", "selfhost_check_expr_reduce_body_segment_with_projected_ascription", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_make_prefix", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_make_prefix_with_first_arg", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_make_prefix_with_ascribed_first_arg", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_make_candidate_vec", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_reduce", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_error_is", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_direct_ok", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_partial_rejected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_expected_rejected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_argument_type_rejected", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_ascribed_argument_unsupported", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_generic_rejected", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_ambiguous_rejected", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0_add_two_i32_function", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage0.nepl", "selfhost_check_expr_stage0", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_scope", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_value_types", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_signatures", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_empty_value_context", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_with_binding_only", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_with_typed_value", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_with_function", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_value_context_with_shadowed_function_value", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_make_named_candidate_vec", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_make_candidate_vec", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_collect_candidates_from_fixture_scope", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_add_two_i32_function", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_add_one_i32_function", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_add_function_value_consumer", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_function_value_argument_segment", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_implicit_function_value_argument_segment", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_make_function_value_argument_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_make_implicit_function_value_argument_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_shadowed_function_argument_uses_value_evidence", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_success_is_two_arg_direct_call", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_success_is_one_arg_direct_call", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_success_has_function_value_argument", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_segment_span", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_reduce_body_segment_with_value_context", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_reduce_block_intro_with_value_context", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_reduce_body_segment_with_empty_values", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_reduce_block_intro_with_empty_values", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_function_value_argument_ok_with_scope", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_bare_function_value_argument_rejected", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_with_candidate", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascription_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascription_conflict_with_types", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_argument_conflict_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascription_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascription_conflict_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_argument_conflict_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_named_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_named_argument_missing_evidence_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_named_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_nested_call_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_trailing_block_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_shadowed_function_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_function_value_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_bare_function_value_argument_with_i32", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_function_value_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_bare_function_value_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_named_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_named_argument_missing_evidence_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_ascribed_named_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_nested_call_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_trailing_block_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_run_shadowed_function_argument_with_tokens", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_ascription_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_ascribed_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_named_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_nested_call_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_function_value_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_trailing_block_argument_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/stage1.nepl", "selfhost_check_expr_stage1_body_line", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_error_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_existing_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_argument_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "SelfhostCallReduceArgumentCheckState", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_free", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_check_state_into_arena", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_free_argument_state_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_push_checked_argument", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_error_from_candidate_collect", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_error_from_block_body_result", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_generic_state_error", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_expected_result", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_match_direct_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_consume_loop_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_nested_single_named_candidate_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_nested_named_candidates_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_argument_match_at_with_source_or_nested", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_single_named_candidate", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_single_named_candidate_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix_with_source_and_trailing_block", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_named_prefix_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_prefix_with_source_and_trailing_block", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/expr/call_reduce.nepl", "selfhost_call_reduce_prefix_with_source", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_directive_item_kind", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_directive_item_fact", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_directive_fact", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_directive_state", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_span", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_declaration_item_fact", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "ownerBoundary"]),
    requirement("stdlib/neplg2/core/check/module/declaration_adapter.nepl", "selfhost_module_check_item_declaration_header", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_diag", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_labeled_diag", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_raw_block_empty_diag", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_unexpected_proof_diag", ["purpose", "contract", "returns", "complexity", "errorVariant"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_directive_duplicate_message", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_directive_duplicate_label", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_directive_duplicate_diag", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_declaration_header_missing_diag", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_declaration_header_invalid_diag", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_index_unavailable_diag", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/diagnostic.nepl", "selfhost_module_check_refutation_diag", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_raw_backend_item_kind", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_raw_backend_item_fact", ["purpose", "contract", "returns", "complexity", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_raw_backend_fact", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_item_raw_state", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/raw_backend_adapter.nepl", "selfhost_module_check_finish_raw_state", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "SelfhostModuleCheckSummary", ["purpose"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_item_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_doc_comment_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_directive_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_entry_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_target_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_import_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_declaration_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_function_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_type_declaration_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_impl_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_raw_block_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary.nepl", "selfhost_module_check_summary_raw_text_count", ["purpose", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary_update.nepl", "selfhost_module_check_summary_new", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/summary_update.nepl", "selfhost_module_check_summary_record", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "SelfhostModuleCheckStep", ["purpose", "contract", "complexity", "ownerBoundary"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_module_check_step_new", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_module_check_item", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_check_module_ast_loop", ["purpose", "contract", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/check/module/orchestrate.nepl", "selfhost_check_module_ast", ["purpose", "returns", "complexity"]),
    moduleSolverRequirement("selfhost_proof_solve_raw_backend_transition", ["purpose", "contract", "current", "returns", "complexity", "doctest", "errorVariant", "authorityBoundary", "rawBoundary"], [
        requiredPattern("raw backend obligation", /\bSelfhostProofObligation::RawBackendTransition\b/),
        requiredPattern("raw backend item fact", /\bSelfhostRawBackendItemFact\b/),
        requiredPattern("raw backend state", /\bSelfhostRawBackendState::(Normal|OpenEmpty|OpenReady)\b/),
        requiredPattern("raw backend item kind", /\bSelfhostRawBackendItemKind::(WasmBlock|LlvmIrBlock|WasmText|LlvmIrText|NonRaw|StreamEnd)\b/),
        requiredPattern("raw backend kind", /\bSelfhostRawBackendKind::(Wasm|LlvmIr)\b/),
        requiredPattern("raw backend evidence", /\bSelfhostProofEvidence::RawBackendTransition\b/),
        requiredPattern("text without block refutation", /\bSelfhostProofRefutation::RawBackendTextWithoutBlock\b/),
        requiredPattern("empty block refutation", /\bSelfhostProofRefutation::RawBackendBlockEmpty\b/),
        requiredPattern("representative solver call", /\bselfhost_proof_solve_raw_backend_transition\b/),
    ], {
        doctestUses: [
            "selfhost_proof_solve_raw_backend_transition",
            "SelfhostRawBackendItemKind::WasmBlock",
            "SelfhostRawBackendItemKind::LlvmIrText",
        ],
    }),
    moduleSolverRequirement("selfhost_proof_solve_module_directive_transition", ["purpose", "contract", "current", "returns", "complexity", "doctest", "errorVariant", "authorityBoundary", "directiveBoundary"], [
        requiredPattern("module directive obligation", /\bSelfhostProofObligation::ModuleDirectiveTransition\b/),
        requiredPattern("module directive fact", /\bSelfhostModuleDirectiveFact\b/),
        requiredPattern("module directive state", /\bSelfhostModuleDirectiveState::(NoneSeen|EntrySeen|TargetSeen|EntryAndTargetSeen)\b/),
        requiredPattern("module directive kind", /\bSelfhostModuleDirectiveKind::(Other|Entry|Target)\b/),
        requiredPattern("module directive evidence", /\bSelfhostProofEvidence::ModuleDirectiveTransition\b/),
        requiredPattern("module directive duplicate refutation", /\bSelfhostProofRefutation::ModuleDirectiveDuplicate\b/),
        requiredPattern("module directive duplicate payload", /\bSelfhostModuleDirectiveDuplicate\b/),
        requiredPattern("representative solver call", /\bselfhost_proof_solve_module_directive_transition\b/),
    ], {
        doctestUses: [
            "selfhost_proof_solve_module_directive_transition",
            "SelfhostModuleDirectiveKind::Entry",
            "SelfhostModuleDirectiveState::EntrySeen",
        ],
    }),
    moduleSolverRequirement("selfhost_proof_span_contains_span", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("outer file authority", /\bouter\.file_id\b/),
        requiredPattern("inner file authority", /\binner\.file_id\b/),
        requiredPattern("outer start boundary", /\bouter\.start\b/),
        requiredPattern("inner start boundary", /\binner\.start\b/),
        requiredPattern("inner end boundary", /\binner\.end\b/),
        requiredPattern("outer end boundary", /\bouter\.end\b/),
        requiredPattern("span validity remains caller authority", /\bsource_span_is_valid\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_module_declaration_head_allowed", ["purpose", "contract", "current", "returns", "complexity", "moduleBoundary"], [
        requiredPattern("function declaration kind", /\bSelfhostModuleDeclarationKind::Function\b/),
        requiredPattern("struct declaration kind", /\bSelfhostModuleDeclarationKind::Struct\b/),
        requiredPattern("enum declaration kind", /\bSelfhostModuleDeclarationKind::Enum\b/),
        requiredPattern("trait declaration kind", /\bSelfhostModuleDeclarationKind::Trait\b/),
        requiredPattern("impl declaration kind", /\bSelfhostModuleDeclarationKind::Impl\b/),
        requiredPattern("name head kind", /\bSelfhostModuleDeclarationHeadKind::Name\b/),
        requiredPattern("type-label head kind", /\bSelfhostModuleDeclarationHeadKind::TypeLabel\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_module_declaration_header_spans_valid", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("header span authority", /\bheader\.header_span\b/),
        requiredPattern("keyword span authority", /\bheader\.keyword_span\b/),
        requiredPattern("source span validation", /\bsource_span_is_valid\b/),
        requiredPattern("span containment helper", /\bselfhost_proof_span_contains_span\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_module_declaration_header_head_valid", ["purpose", "contract", "current", "returns", "complexity", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("head some branch", /\bOption::Some\b/),
        requiredPattern("head none branch", /\bOption::None\b/),
        requiredPattern("head span authority", /\bhead\.span\b/),
        requiredPattern("source span validation", /\bsource_span_is_valid\b/),
        requiredPattern("span containment helper", /\bselfhost_proof_span_contains_span\b/),
        requiredPattern("head allowed helper", /\bselfhost_proof_module_declaration_head_allowed\b/),
        requiredPattern("name head kind", /\bSelfhostModuleDeclarationHeadKind::Name\b/),
        requiredPattern("type-label head kind", /\bSelfhostModuleDeclarationHeadKind::TypeLabel\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_module_declaration_item_kind_supports", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("fact item kind authority", /\bfact\.item_kind\b/),
        requiredPattern("obligation kind", /\bSelfhostProofObligation::ModuleDeclarationHeaderAvailable\b/),
        requiredPattern("module item kind authority", /\bSelfhostModuleItemKind\b/),
        requiredPattern("item-kind declaration mapper", /\bselfhost_module_item_kind_declaration\b/),
        requiredPattern("declaration mapping present branch", /\bOption::Some\b/),
        requiredPattern("declaration mapping absent branch", /\bOption::None\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_module_declaration_header_supports", ["purpose", "contract", "current", "returns", "complexity", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("declaration kind equality", /\bselfhost_proof_module_declaration_kind_eq\b/),
        requiredPattern("span validity helper", /\bselfhost_proof_module_declaration_header_spans_valid\b/),
        requiredPattern("head validity helper", /\bselfhost_proof_module_declaration_header_head_valid\b/),
        requiredPattern("range validity helper", /\bselfhost_proof_module_declaration_header_ranges_valid\b/),
        requiredPattern("range allowed helper", /\bselfhost_proof_module_declaration_header_ranges_allowed\b/),
        requiredPattern("visibility allowed helper", /\bselfhost_proof_module_declaration_visibility_allowed\b/),
        requiredPattern("function requires ranges", /\bSelfhostModuleDeclarationKind::Function\b/),
        requiredPattern("public visibility branch", /\bSelfhostModuleDeclarationVisibility::Public\b/),
        requiredPattern("invalid refutation limitation", /\bSelfhostProofRefutation::ModuleDeclarationHeaderInvalid\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_module_declaration_header_missing", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("missing header refutation", /\bSelfhostProofRefutation::ModuleDeclarationHeaderMissing\b/),
        requiredPattern("header issue payload", /\bSelfhostModuleDeclarationHeaderIssue\b/),
        requiredPattern("refuted proof result", /\bSelfhostProofResult::Refuted\b/),
        requiredPattern("none branch source", /\bOption::None\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_module_declaration_header_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("invalid header refutation", /\bSelfhostProofRefutation::ModuleDeclarationHeaderInvalid\b/),
        requiredPattern("header issue payload", /\bSelfhostModuleDeclarationHeaderIssue\b/),
        requiredPattern("header span authority", /\bheader\.header_span\b/),
        requiredPattern("refuted proof result", /\bSelfhostProofResult::Refuted\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_module_declaration_header_proven", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("declaration header evidence", /\bSelfhostProofEvidence::ModuleDeclarationHeaderAvailable\b/),
        requiredPattern("proven proof result", /\bSelfhostProofResult::Proven\b/),
        requiredPattern("declaration header payload", /\bSelfhostModuleDeclarationHeader\b/),
    ]),
    moduleSolverRequirement("selfhost_proof_solve_module_declaration_header", ["purpose", "contract", "current", "returns", "complexity", "doctest", "errorVariant", "authorityBoundary", "moduleBoundary"], [
        requiredPattern("module declaration header obligation", /\bSelfhostProofObligation::ModuleDeclarationHeaderAvailable\b/),
        requiredPattern("declaration fact authority", /\bSelfhostModuleDeclarationFact\b/),
        requiredPattern("fact item kind authority", /\bfact\.item_kind\b/),
        requiredPattern("fact declaration authority", /\bfact\.declaration\b/),
        requiredPattern("some declaration branch", /\bOption::Some\b/),
        requiredPattern("none declaration branch", /\bOption::None\b/),
        requiredPattern("available header evidence", /\bSelfhostProofEvidence::ModuleDeclarationHeaderAvailable\b/),
        requiredPattern("mismatch refutation", /\bSelfhostProofRefutation::FactObligationMismatch\b/),
        requiredPattern("missing refutation", /\bSelfhostProofRefutation::ModuleDeclarationHeaderMissing\b/),
        requiredPattern("invalid refutation", /\bSelfhostProofRefutation::ModuleDeclarationHeaderInvalid\b/),
        requiredPattern("match exhaustiveness", /match .*網羅性検査/),
    ], {
        doctestUses: [
            "selfhost_proof_solve_module_declaration_header",
            "SelfhostModuleDeclarationHeader",
            "SelfhostModuleDeclarationKind::Function",
        ],
    }),
    typeSolverRequirement("selfhost_proof_type_kind_compatible_result", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "typeBoundary"], [
        requiredPattern("type kind compatibility evidence", /\bSelfhostProofEvidence::TypeKindCompatible\b/),
        requiredPattern("typed proof result", /\bSelfhostProofResult::Proven\b/),
    ]),
    typeSolverRequirement("selfhost_proof_type_kind_mismatch", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "typeBoundary"], [
        requiredPattern("type kind mismatch refutation", /\bSelfhostProofRefutation::TypeKindMismatch\b/),
        requiredPattern("type kind mismatch payload", /\bSelfhostTypeKindMismatch\b/),
        requiredPattern("expected type kind", /\bexpected\b/),
        requiredPattern("actual type kind from fact", /\bfact\.kind\b/),
    ]),
    typeSolverRequirement("selfhost_proof_solve_type_kind_compatible", ["purpose", "contract", "current", "returns", "complexity", "errorVariant", "authorityBoundary", "typeBoundary"], [
        requiredPattern("type kind equality authority", /\bselfhost_type_kind_eq\b/),
        requiredPattern("type kind enum authority", /\bSelfhostTypeKind\b/),
        requiredPattern("type kind compatibility evidence", /\bSelfhostProofEvidence::TypeKindCompatible\b/),
        requiredPattern("type kind mismatch refutation", /\bSelfhostProofRefutation::TypeKindMismatch\b/),
        requiredPattern("named type current limitation", /\bSelfhostTypeKind::Named\b/),
        requiredPattern("type parameter current limitation", /\bSelfhostTypeKind::Parameter\b/),
        requiredPattern("never special case is not implemented here", /\bSelfhostTypeKind::Never\b/),
        requiredPattern("match exhaustiveness", /match .*網羅性検査/),
    ]),
    typeSolverRequirement("selfhost_proof_trait_impl_non_overlapping_result", ["purpose", "contract", "returns", "complexity", "typeBoundary"], [
        requiredPattern("trait impl non-overlap evidence", /\bSelfhostProofEvidence::TraitImplNonOverlapping\b/),
        requiredPattern("different trait success relation", /\bDifferentTrait\b/),
        requiredPattern("different self type success relation", /\bSameTraitDifferentSelfType\b/),
    ]),
    typeSolverRequirement("selfhost_proof_trait_impl_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "typeBoundary"], [
        requiredPattern("trait impl coherence refutation", /\bSelfhostProofRefutation::TraitImplCoherenceInvalid\b/),
        requiredPattern("trait impl coherence issue payload", /\bSelfhostTraitImplCoherenceIssue\b/),
        requiredPattern("invalid candidate key error", /\bSelfhostTraitImplCoherenceError::InvalidCandidateKey\b/),
        requiredPattern("invalid existing key error", /\bSelfhostTraitImplCoherenceError::InvalidExistingKey\b/),
        requiredPattern("duplicate impl error", /\bSelfhostTraitImplCoherenceError::DuplicateImpl\b/),
    ]),
    typeSolverRequirement("selfhost_proof_solve_trait_impl_non_overlapping", ["purpose", "contract", "current", "returns", "complexity", "errorVariant", "authorityBoundary", "typeBoundary"], [
        requiredPattern("invalid candidate relation", /\bSelfhostTraitImplRelation::InvalidCandidate\b/),
        requiredPattern("invalid existing relation", /\bSelfhostTraitImplRelation::InvalidExisting\b/),
        requiredPattern("different trait relation", /\bSelfhostTraitImplRelation::DifferentTrait\b/),
        requiredPattern("same trait different self type relation", /\bSelfhostTraitImplRelation::SameTraitDifferentSelfType\b/),
        requiredPattern("same trait same self type relation", /\bSelfhostTraitImplRelation::SameTraitSameSelfType\b/),
        requiredPattern("invalid candidate key error", /\bSelfhostTraitImplCoherenceError::InvalidCandidateKey\b/),
        requiredPattern("invalid existing key error", /\bSelfhostTraitImplCoherenceError::InvalidExistingKey\b/),
        requiredPattern("duplicate impl error", /\bSelfhostTraitImplCoherenceError::DuplicateImpl\b/),
        requiredPattern("trait impl non-overlap evidence", /\bSelfhostProofEvidence::TraitImplNonOverlapping\b/),
        requiredPattern("trait impl coherence refutation", /\bSelfhostProofRefutation::TraitImplCoherenceInvalid\b/),
        requiredPattern("generic overlap current limitation", /generic overlap/),
        requiredPattern("blanket impl current limitation", /blanket impl/),
        requiredPattern("match exhaustiveness", /match .*網羅性検査/),
    ]),
    resourceSolverRequirement("selfhost_proof_resource_cell_proven", ["purpose", "contract", "returns", "complexity", "resourceBoundary"], [
        requiredPattern("resource cell transition evidence", /\bSelfhostProofEvidence::ResourceCellTransition\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_resource_cell_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "resourceBoundary"], [
        requiredPattern("resource cell transition refutation", /\bSelfhostProofRefutation::ResourceCellTransitionInvalid\b/),
        requiredPattern("resource cell transition error", /\bSelfhostResourceCellTransitionError\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_uninitialized", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("uninitialized state", /\bSelfhostResourceCellState::Uninitialized\b/),
        requiredPattern("initialize event", /\bSelfhostResourceCellEventKind::Initialize\b/),
        requiredPattern("move uninitialized error", /\bSelfhostResourceCellTransitionError::MoveUninitialized\b/),
        requiredPattern("drop uninitialized error", /\bSelfhostResourceCellTransitionError::DropUninitialized\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_initialized", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("initialized state", /\bSelfhostResourceCellState::Initialized\b/),
        requiredPattern("already initialized error", /\bSelfhostResourceCellTransitionError::InitializeAlreadyInitialized\b/),
        requiredPattern("move target state", /\bSelfhostResourceCellState::Moved\b/),
        requiredPattern("drop target state", /\bSelfhostResourceCellState::Dropped\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_moved", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("moved state", /\bSelfhostResourceCellState::Moved\b/),
        requiredPattern("move after move error", /\bSelfhostResourceCellTransitionError::MoveAfterMove\b/),
        requiredPattern("drop after move error", /\bSelfhostResourceCellTransitionError::DropAfterMove\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_dropped", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("dropped state", /\bSelfhostResourceCellState::Dropped\b/),
        requiredPattern("initialize after drop error", /\bSelfhostResourceCellTransitionError::InitializeAfterDrop\b/),
        requiredPattern("move after drop error", /\bSelfhostResourceCellTransitionError::MoveAfterDrop\b/),
        requiredPattern("double drop error", /\bSelfhostResourceCellTransitionError::DoubleDrop\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_resource_cell_transition", ["purpose", "contract", "returns", "complexity", "errorVariant", "resourceBoundary"], [
        requiredPattern("all resource cell states", /\bSelfhostResourceCellState::Uninitialized\b[\s\S]*\bSelfhostResourceCellState::Initialized\b[\s\S]*\bSelfhostResourceCellState::Moved\b[\s\S]*\bSelfhostResourceCellState::Dropped\b/),
        requiredPattern("resource cell evidence", /\bSelfhostProofEvidence::ResourceCellTransition\b/),
        requiredPattern("resource cell refutation", /\bSelfhostProofRefutation::ResourceCellTransitionInvalid\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_owner_transition_proven", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("owner transition evidence", /\bSelfhostProofEvidence::OwnerTransition\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_owner_transition_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("owner transition refutation", /\bSelfhostProofRefutation::OwnerTransitionInvalid\b/),
        requiredPattern("owner transition error", /\bSelfhostOwnerTransitionError\b/),
        requiredPattern("owner storage authority", /\bfact\.storage\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_owner_transition_storage_mismatch", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("storage mismatch error", /\bSelfhostOwnerTransitionError::StorageIdMismatch\b/),
        requiredPattern("no owner mismatch branch", /\bSelfhostOwnerState::NoOwner\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_owner_transition_with_storage", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("owned state", /\bSelfhostOwnerState::Owned\b/),
        requiredPattern("moved state", /\bSelfhostOwnerState::Moved\b/),
        requiredPattern("released state", /\bSelfhostOwnerState::Released\b/),
        requiredPattern("acquire event", /\bSelfhostOwnerEventKind::Acquire\b/),
        requiredPattern("move event", /\bSelfhostOwnerEventKind::MoveOut\b/),
        requiredPattern("release event", /\bSelfhostOwnerEventKind::Release\b/),
        requiredPattern("borrow view event", /\bSelfhostOwnerEventKind::BorrowView\b/),
        requiredPattern("acquire while owned error", /\bSelfhostOwnerTransitionError::AcquireWhileOwned\b/),
        requiredPattern("view after release error", /\bSelfhostOwnerTransitionError::ViewAfterRelease\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_owner_transition_no_owner", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("no owner state", /\bSelfhostOwnerState::NoOwner\b/),
        requiredPattern("acquire event", /\bSelfhostOwnerEventKind::Acquire\b/),
        requiredPattern("move without owner error", /\bSelfhostOwnerTransitionError::MoveWithoutOwner\b/),
        requiredPattern("release without owner error", /\bSelfhostOwnerTransitionError::ReleaseWithoutOwner\b/),
        requiredPattern("view without owner error", /\bSelfhostOwnerTransitionError::ViewWithoutOwner\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_owner_transition", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("invalid storage id error", /\bSelfhostOwnerTransitionError::InvalidStorageId\b/),
        requiredPattern("owner transition evidence", /\bSelfhostProofEvidence::OwnerTransition\b/),
        requiredPattern("owner transition refutation", /\bSelfhostProofRefutation::OwnerTransitionInvalid\b/),
        requiredPattern("no-owner state", /\bSelfhostOwnerState::NoOwner\b/),
        requiredPattern("owned state", /\bSelfhostOwnerState::Owned\b/),
        requiredPattern("moved state", /\bSelfhostOwnerState::Moved\b/),
        requiredPattern("released state", /\bSelfhostOwnerState::Released\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_borrow_access_proven", ["purpose", "contract", "returns", "complexity", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("borrow access evidence", /\bSelfhostProofEvidence::ResourceBorrowAccess\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_borrow_access_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("borrow access refutation", /\bSelfhostProofRefutation::BorrowAccessInvalid\b/),
        requiredPattern("borrow access error", /\bSelfhostBorrowAccessError\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_borrow_access_invalid_shared_count", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("invalid shared count error", /\bSelfhostBorrowAccessError::InvalidSharedBorrowCount\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access_unborrowed", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("unborrowed state", /\bSelfhostBorrowState::Unborrowed\b/),
        requiredPattern("start shared request", /\bSelfhostBorrowRequestKind::StartShared\b/),
        requiredPattern("start mutable request", /\bSelfhostBorrowRequestKind::StartMutable\b/),
        requiredPattern("end shared without shared error", /\bSelfhostBorrowAccessError::EndSharedWithoutSharedBorrow\b/),
        requiredPattern("end mutable without mutable error", /\bSelfhostBorrowAccessError::EndMutableWithoutMutableBorrow\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access_shared_valid", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("shared state", /\bSelfhostBorrowState::Shared\b/),
        requiredPattern("mutable while shared error", /\bSelfhostBorrowAccessError::MutableBorrowWhileShared\b/),
        requiredPattern("shared count one transition", /count が 1/),
        requiredPattern("end mutable without mutable error", /\bSelfhostBorrowAccessError::EndMutableWithoutMutableBorrow\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access_shared", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("shared state", /\bSelfhostBorrowState::Shared\b/),
        requiredPattern("shared count validity helper", /\bselfhost_borrow_shared_count_is_valid\b/),
        requiredPattern("invalid shared count error", /\bSelfhostBorrowAccessError::InvalidSharedBorrowCount\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access_mutable", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("mutable state", /\bSelfhostBorrowState::Mutable\b/),
        requiredPattern("shared while mutable error", /\bSelfhostBorrowAccessError::SharedBorrowWhileMutable\b/),
        requiredPattern("mutable while mutable error", /\bSelfhostBorrowAccessError::MutableBorrowWhileMutable\b/),
        requiredPattern("end shared without shared error", /\bSelfhostBorrowAccessError::EndSharedWithoutSharedBorrow\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_borrow_access", ["purpose", "contract", "returns", "complexity", "errorVariant", "ownerBoundary", "resourceBoundary"], [
        requiredPattern("all borrow states", /\bSelfhostBorrowState::Unborrowed\b[\s\S]*\bShared count\b[\s\S]*\bMutable\b/),
        requiredPattern("borrow access evidence", /\bSelfhostProofEvidence::ResourceBorrowAccess\b/),
        requiredPattern("borrow access refutation", /\bSelfhostProofRefutation::BorrowAccessInvalid\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_lifetime_outlives_proven", ["purpose", "contract", "returns", "complexity", "resourceBoundary"], [
        requiredPattern("lifetime outlives evidence", /\bSelfhostProofEvidence::LifetimeOutlives\b/),
        requiredPattern("same lifetime success relation", /\bSelfhostLifetimeRelation::SameLifetime\b/),
        requiredPattern("subject outlives success relation", /\bSubjectOutlivesRequired\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_lifetime_outlives_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "resourceBoundary"], [
        requiredPattern("lifetime outlives refutation", /\bSelfhostProofRefutation::LifetimeOutlivesInvalid\b/),
        requiredPattern("lifetime outlives error", /\bSelfhostLifetimeOutlivesError\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_lifetime_outlives_relation", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "resourceBoundary"], [
        requiredPattern("invalid subject relation", /\bSelfhostLifetimeRelation::InvalidSubject\b/),
        requiredPattern("invalid required relation", /\bSelfhostLifetimeRelation::InvalidRequired\b/),
        requiredPattern("same lifetime relation", /\bSelfhostLifetimeRelation::SameLifetime\b/),
        requiredPattern("subject outlives relation", /\bSelfhostLifetimeRelation::SubjectOutlivesRequired\b/),
        requiredPattern("subject shorter error", /\bSelfhostLifetimeOutlivesError::SubjectDoesNotOutliveRequired\b/),
        requiredPattern("unrelated error", /\bSelfhostLifetimeOutlivesError::UnrelatedLifetimes\b/),
    ]),
    resourceSolverRequirement("selfhost_proof_solve_lifetime_outlives", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "resourceBoundary"], [
        requiredPattern("required lifetime mismatch error", /\bSelfhostLifetimeOutlivesError::RequiredLifetimeMismatch\b/),
        requiredPattern("lifetime id equality authority", /\bselfhost_lifetime_id_eq\b/),
        requiredPattern("lifetime evidence", /\bSelfhostProofEvidence::LifetimeOutlives\b/),
        requiredPattern("lifetime refutation", /\bSelfhostProofRefutation::LifetimeOutlivesInvalid\b/),
    ]),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_effect_allowed_result", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("typed effect evidence", /\bSelfhostProofEvidence::EffectAllowed\b/),
            requiredPattern("effect context authority", /\bSelfhostEffectContext\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_effect_invalid", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("typed effect refutation", /\bSelfhostProofRefutation::EffectBoundaryInvalid\b/),
            requiredPattern("typed effect boundary error", /\bSelfhostEffectBoundaryError\b/),
            requiredPattern("unsafe memory boundary error", /\bSelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary\b/),
            requiredPattern("impure effect in pure context error", /\bSelfhostEffectBoundaryError::ImpureEffectInPureContext\b/),
            requiredPattern("internal allocation escape error", /\bSelfhostEffectBoundaryError::InternalAllocEscapeNotProven\b/),
            requiredPattern("observed effect kind payload", /\bSelfhostEffectKind\b/),
            requiredPattern("observed escape-state payload", /\bSelfhostEffectEscapeState\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_internal_alloc_allowed", ["purpose", "contract", "returns", "complexity", "errorVariant", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("internal allocation effect kind", /\bSelfhostEffectKind::InternalAlloc\b/),
            requiredPattern("no-escape success state", /\bSelfhostEffectEscapeState::NoEscapeProven\b/),
            requiredPattern("not-applicable escape failure state", /\bSelfhostEffectEscapeState::NotApplicable\b/),
            requiredPattern("may-escape failure state", /\bSelfhostEffectEscapeState::MayEscape\b/),
            requiredPattern("internal allocation escape error", /\bSelfhostEffectBoundaryError::InternalAllocEscapeNotProven\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_solve_effect_pure_context", ["purpose", "contract", "returns", "complexity", "errorVariant", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("pure context authority", /\bSelfhostEffectContext::PureContext\b/),
            requiredPattern("internal allocation branch", /\bSelfhostEffectKind::InternalAlloc\b/),
            requiredPattern("unsafe memory branch", /\bSelfhostEffectKind::UnsafeMemory\b/),
            requiredPattern("external io branch", /\bSelfhostEffectKind::ExternalIo\b/),
            requiredPattern("nondeterminism branch", /\bSelfhostEffectKind::Nondet\b/),
            requiredPattern("unsafe memory boundary error", /\bSelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary\b/),
            requiredPattern("impure effect in pure context error", /\bSelfhostEffectBoundaryError::ImpureEffectInPureContext\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_solve_effect_impure_context", ["purpose", "contract", "returns", "complexity", "errorVariant", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("impure context authority", /\bSelfhostEffectContext::ImpureContext\b/),
            requiredPattern("internal allocation branch", /\bSelfhostEffectKind::InternalAlloc\b/),
            requiredPattern("unsafe memory branch", /\bSelfhostEffectKind::UnsafeMemory\b/),
            requiredPattern("external io branch", /\bSelfhostEffectKind::ExternalIo\b/),
            requiredPattern("nondeterminism branch", /\bSelfhostEffectKind::Nondet\b/),
            requiredPattern("unsafe memory boundary error", /\bSelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary\b/),
            requiredPattern("internal allocation escape error", /\bSelfhostEffectBoundaryError::InternalAllocEscapeNotProven\b/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_solve_effect_unsafe_boundary", ["purpose", "contract", "returns", "complexity", "authorityBoundary", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("unsafe boundary authority", /\bSelfhostEffectContext::UnsafeBoundary\b/),
            requiredPattern("pure effect branch", /\bSelfhostEffectKind::Pure\b/),
            requiredPattern("internal allocation branch", /\bSelfhostEffectKind::InternalAlloc\b/),
            requiredPattern("unsafe memory branch", /\bSelfhostEffectKind::UnsafeMemory\b/),
            requiredPattern("external io branch", /\bSelfhostEffectKind::ExternalIo\b/),
            requiredPattern("nondeterminism branch", /\bSelfhostEffectKind::Nondet\b/),
            requiredPattern("unsafe-boundary evidence payload", /\bSelfhostProofEvidence::EffectAllowed UnsafeBoundary\b/),
            requiredPattern("effect kind is not preserved in evidence", /effect kind .*evidence payload .*保存されません/),
        ],
    }),
    requirement("stdlib/neplg2/core/proof/solver/effect.nepl", "selfhost_proof_solve_effect_allowed", ["purpose", "contract", "returns", "complexity", "errorVariant", "authorityBoundary", "effectBoundary"], {
        requiredPatterns: [
            requiredPattern("pure context dispatch", /\bSelfhostEffectContext::PureContext\b/),
            requiredPattern("impure context dispatch", /\bSelfhostEffectContext::ImpureContext\b/),
            requiredPattern("unsafe boundary dispatch", /\bSelfhostEffectContext::UnsafeBoundary\b/),
            requiredPattern("unsafe memory boundary error", /\bSelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary\b/),
            requiredPattern("impure effect in pure context error", /\bSelfhostEffectBoundaryError::ImpureEffectInPureContext\b/),
            requiredPattern("internal allocation escape error", /\bSelfhostEffectBoundaryError::InternalAllocEscapeNotProven\b/),
            requiredPattern("typed effect evidence", /\bSelfhostProofEvidence::EffectAllowed\b/),
            requiredPattern("typed effect refutation", /\bSelfhostProofRefutation::EffectBoundaryInvalid\b/),
            requiredPattern("context match static check", /match .*網羅性検査/),
        ],
    }),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExprKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirFunctionValueIdentityBuildError", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirCallExpr", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirValueIdentity", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExprPayload", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/hir/hir/expr.nepl", "SelfhostHirExpr", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/byte.nepl", "lex_byte_or_end", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/byte.nepl", "lex_is_digit", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/diagnostic.nepl", "LexDiagnostic", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/directive.nepl", "SelfhostLexerDirectiveKind", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/syntax/lexer/directive.nepl", "lex_directive_word_at", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/indent.nepl", "lex_line_indent_width", ["purpose", "contract", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/literal.nepl", "lex_is_hex_digit", ["purpose", "returns", "complexity"]),
    requirement("stdlib/neplg2/core/syntax/lexer/raw_mode.nepl", "SelfhostLexerRawMode", ["purpose", "contract"]),
    requirement("stdlib/neplg2/core/syntax/lexer/token_build.nepl", "lex_token_slice", ["purpose", "contract", "complexity"]),
];

const SECTION_PATTERNS = {
    purpose: /\[目的\/もくてき\]/,
    contract: /\[契約\/けいやく\]/,
    current: /\[現状\/げんじょう\]/,
    returns: /\[戻\/もど\]り\[値\/ち\]/,
    complexity: /\[計算量\/けいさんりょう\]/,
    doctest: /\bneplg2:test\b/,
    errorVariant: /\b(SelfhostCheckerDiagnosticCode::[A-Za-z0-9_]+|SelfhostDiagnosticCode::Checker|SelfhostTypeProjectErrorKind(?:::[A-Za-z0-9_]+)?|SelfhostTypeReduceErrorKind(?:::[A-Za-z0-9_]+)?|StdErrorKind::[A-Za-z0-9_]+|SelfhostProofRefutation::[A-Za-z0-9_]+|SelfhostEffectBoundaryError::[A-Za-z0-9_]+|SelfhostResourceCellTransitionError::[A-Za-z0-9_]+|SelfhostOwnerTransitionError::[A-Za-z0-9_]+|SelfhostBorrowAccessError::[A-Za-z0-9_]+|SelfhostLifetimeOutlivesError::[A-Za-z0-9_]+)\b/,
    authorityBoundary: /\b(authority|typed evidence|parser-provided evidence|parser\/proof|proof layer|source spelling|source text|kind stream|message .*authority|diagnostic kind の authority|表示.*authority)\b/,
    effectBoundary: /\b(SelfhostEffectKind::[A-Za-z0-9_]+|SelfhostEffectContext::[A-Za-z0-9_]+|SelfhostEffectBoundaryError::[A-Za-z0-9_]+|SelfhostProofEvidence::EffectAllowed|SelfhostEffectEscapeState::[A-Za-z0-9_]+)\b/,
    resourceBoundary: /\b(SelfhostResourceCellState::[A-Za-z0-9_]+|SelfhostResourceCellEventKind::[A-Za-z0-9_]+|SelfhostResourceCellTransitionError::[A-Za-z0-9_]+|SelfhostOwnerState::[A-Za-z0-9_]+|SelfhostOwnerEventKind::[A-Za-z0-9_]+|SelfhostOwnerTransitionError::[A-Za-z0-9_]+|SelfhostBorrowState::[A-Za-z0-9_]+|SelfhostBorrowRequestKind::[A-Za-z0-9_]+|SelfhostBorrowAccessError::[A-Za-z0-9_]+|SelfhostLifetimeRelation::[A-Za-z0-9_]+|SelfhostLifetimeOutlivesError::[A-Za-z0-9_]+|SelfhostProofEvidence::(ResourceCellTransition|OwnerTransition|ResourceBorrowAccess|LifetimeOutlives)|SelfhostProofRefutation::(ResourceCellTransitionInvalid|OwnerTransitionInvalid|BorrowAccessInvalid|LifetimeOutlivesInvalid))\b/,
    typeBoundary: /\b(SelfhostTypeKind(?:::)?[A-Za-z0-9_]*|selfhost_type_kind_eq|SelfhostTypeKindMismatch|SelfhostTypeRecord(?:::)?[A-Za-z0-9_]*|SelfhostTypeArenaAlloc|SelfhostTypeArena|SelfhostTypeId|SelfhostTypeParameterBinding|SelfhostTypeParameterEnv|SelfhostPrimitiveTypeKind(?:::)?[A-Za-z0-9_]*|SelfhostResolvedTypeNode(?:::)?[A-Za-z0-9_]*|SelfhostResolvedTypeTreeRoot|SelfhostResolvedTypeTree|SelfhostResolvedTypeNodeId|SelfhostResolvedAppliedType|SelfhostResolvedAppliedTypeArgRange|SelfhostResolvedFunctionType|SelfhostResolvedFunctionArgRange(?:::)?[A-Za-z0-9_]*|SelfhostTypeConstructorKind(?:::)?[A-Za-z0-9_]*|SelfhostTypeConstructorTable|SelfhostTypeProjectErrorKind(?:::)?[A-Za-z0-9_]*|SelfhostTypeReduceDispatchKind(?:::)?[A-Za-z0-9_]*|SelfhostTypeReduceErrorKind(?:::)?[A-Za-z0-9_]*|SelfhostTypeReducePlan|SelfhostTypeReducePlanItem|SelfhostTypeBoundPlan|SelfhostTypeBoundPlanItem|SelfhostTypeReduceBuildState|SelfhostTypeReduceStep|SelfhostTypePrefixReducePrefixResult|SelfhostTraitImplRelation(?:::)?[A-Za-z0-9_]*|SelfhostTraitImplCoherenceError(?:::)?[A-Za-z0-9_]*|SelfhostTraitImplCoherenceIssue|SelfhostProofEvidence::(TypeKindCompatible|TraitImplNonOverlapping)|SelfhostProofRefutation::(TypeKindMismatch|TraitImplCoherenceInvalid))\b/,
    rawBoundary: /\b(SelfhostRawBackendKind(?:::)?[A-Za-z0-9_]*|SelfhostRawBackendItemKind(?:::)?[A-Za-z0-9_]*|SelfhostRawBackendItemFact|SelfhostRawBackendState(?:::)?[A-Za-z0-9_]*|SelfhostRawBackendOpenBlock|SelfhostProofObligation::RawBackendTransition|SelfhostProofEvidence::RawBackendTransition|SelfhostProofRefutation::(RawBackendTextWithoutBlock|RawBackendBlockEmpty)|selfhost_raw_backend_text_matches)\b/,
    directiveBoundary: /\b(SelfhostModuleDirectiveKind(?:::)?[A-Za-z0-9_]*|SelfhostModuleDirectiveFact|SelfhostModuleDirectiveState(?:::)?[A-Za-z0-9_]*|SelfhostModuleDirectiveSeenBoth|SelfhostModuleDirectiveDuplicate|SelfhostProofObligation::ModuleDirectiveTransition|SelfhostProofEvidence::ModuleDirectiveTransition|SelfhostProofRefutation::ModuleDirectiveDuplicate)\b/,
    moduleBoundary: /\b(SelfhostModuleDeclarationKind(?:::)?[A-Za-z0-9_]*|SelfhostModuleDeclarationHeadKind(?:::)?[A-Za-z0-9_]*|SelfhostModuleDeclarationVisibility(?:::)?[A-Za-z0-9_]*|SelfhostModuleDeclarationHeader|SelfhostModuleDeclarationFact|SelfhostModuleDeclarationHeaderIssue|SelfhostModuleItemKind(?:::)?[A-Za-z0-9_]*|SelfhostSyntaxRange(?:::)?[A-Za-z0-9_]*|SelfhostSourceSpan|SelfhostProofObligation::ModuleDeclarationHeaderAvailable|SelfhostProofEvidence::ModuleDeclarationHeaderAvailable|SelfhostProofRefutation::(ModuleDeclarationHeaderMissing|ModuleDeclarationHeaderInvalid|FactObligationMismatch)|selfhost_module_item_kind_declaration|selfhost_syntax_range_is_(?:valid|nonempty)|selfhost_syntax_range_span_is_inside|source_span_is_valid|selfhost_proof_span_contains_span)\b/,
    ownerBoundary: /\b(owner|cleanup obligation|cleanup|borrow|未処理 owner|owner 変換|解放)\b/,
};

function requirement(relPath, name, sections, options = {}) {
    return {
        relPath,
        name,
        sections,
        doctestUses: options.doctestUses || [],
        requiredPatterns: options.requiredPatterns || [],
    };
}

function moduleRequirement(relPath, sections, options = {}) {
    return {
        relPath,
        sections,
        doctestUses: options.doctestUses || [],
        requiredPatterns: options.requiredPatterns || [],
    };
}

function resourceSolverRequirement(name, sections, requiredPatterns = []) {
    return requirement("stdlib/neplg2/core/proof/solver/resource.nepl", name, sections, {
        requiredPatterns,
    });
}

function typeSolverRequirement(name, sections, requiredPatterns = []) {
    return requirement("stdlib/neplg2/core/proof/solver/type.nepl", name, sections, {
        requiredPatterns,
    });
}

function typeProjectRequirement(name, sections, requiredPatterns = []) {
    return requirement("stdlib/neplg2/core/resolve/type_resolver/project.nepl", name, sections, {
        requiredPatterns,
    });
}

function typeReduceModelRequirement(name, sections, requiredPatterns = []) {
    return requirement("stdlib/neplg2/core/resolve/type_resolver/reduce/model.nepl", name, sections, {
        requiredPatterns,
    });
}

function typeReduceRequirement(name, sections, requiredPatterns = []) {
    return requirement("stdlib/neplg2/core/resolve/type_resolver/reduce.nepl", name, sections, {
        requiredPatterns,
    });
}

function typeReducePlanRequirement(name, sections, requiredPatterns = []) {
    return requirement("stdlib/neplg2/core/resolve/type_resolver/reduce/plan.nepl", name, sections, {
        requiredPatterns,
    });
}

function typeReduceBuildRequirement(name, sections, requiredPatterns = []) {
    return requirement("stdlib/neplg2/core/resolve/type_resolver/reduce/build.nepl", name, sections, {
        requiredPatterns,
    });
}

function moduleSolverRequirement(name, sections, requiredPatterns = [], options = {}) {
    return requirement("stdlib/neplg2/core/proof/solver/module.nepl", name, sections, {
        requiredPatterns,
        doctestUses: options.doctestUses || [],
    });
}

function requiredPattern(label, pattern) {
    return { label, pattern };
}

function sectionRequirementKey(relPath, name) {
    return `${relPath}#${name}`;
}

function docHasSection(docLines, section) {
    const pattern = SECTION_PATTERNS[section];
    assert.ok(pattern, `unknown documentation section requirement: ${section}`);
    return docLines.some((line) => pattern.test(line));
}

const docSectionRequirementByKey = new Map(
    DOC_SECTION_REQUIREMENTS.map((item) => [sectionRequirementKey(item.relPath, item.name), item]),
);
const moduleDocSectionRequirementByPath = new Map(
    MODULE_DOC_SECTION_REQUIREMENTS.map((item) => [item.relPath, item]),
);

function walkNeplFiles(dir) {
    const files = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const child = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...walkNeplFiles(child));
        } else if (entry.isFile() && entry.name.endsWith(".nepl")) {
            files.push(child);
        }
    }
    return files;
}

function toRepoPath(filePath) {
    return path.relative(repoRoot, filePath).split(path.sep).join("/");
}

function hasDoctest(docLines) {
    return docLines.some((line) => /\bneplg2:test\b/.test(line));
}

function declarationAt(line) {
    return line.match(/^\s*(pub\s+)?(fn|struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b/);
}

function moduleDocLines(lines) {
    for (let index = 0; index < lines.length; index += 1) {
        const trimmed = lines[index].trim();
        if (trimmed === "" || trimmed === "#indent 4") {
            continue;
        }
        if (declarationAt(lines[index]) || trimmed.startsWith("#import")) {
            return [];
        }
        if (!lines[index].trimStart().startsWith("//:")) {
            return [];
        }
        const doc = [];
        for (let cursor = index; cursor < lines.length; cursor += 1) {
            if (!lines[cursor].trimStart().startsWith("//:")) {
                break;
            }
            doc.push(lines[cursor]);
        }
        if (doc.length > 0 && doc[0].trimStart().startsWith("//: #")) {
            return doc;
        }
        return [];
    }
    return [];
}

function precedingDocLines(lines, index) {
    let cursor = index - 1;
    while (cursor >= 0 && lines[cursor].trim() === "") {
        cursor -= 1;
    }
    const doc = [];
    while (cursor >= 0 && lines[cursor].trimStart().startsWith("//:")) {
        doc.push(lines[cursor]);
        cursor -= 1;
    }
    return doc.reverse();
}

function indentOf(line) {
    const match = line.match(/^(\s*)/);
    return match ? match[1].length : 0;
}

function implHeaderAt(line) {
    return line.match(/^\s*impl(?:\b|<)/);
}

const stats = {
    files: 0,
    moduleNoDoc: 0,
    moduleNoDoctest: 0,
    declarations: 0,
    declarationNoDoc: 0,
    declarationNoDoctest: 0,
    publicNoDoc: 0,
    publicNoDoctest: 0,
    privateNoDoc: 0,
    privateNoDoctest: 0,
};

const samples = [];
const publicDocRequiredPrefixGaps = [];
const moduleDocRequiredPrefixGaps = [];
const docSectionGaps = [];
const seenDocSectionRequirementKeys = new Set();
const seenModuleDocSectionRequirementPaths = new Set();
const seenRepoPaths = new Set();

function sample(message) {
    if (samples.length < 60) {
        samples.push(message);
    }
}

for (const filePath of walkNeplFiles(selfhostRoot).sort()) {
    stats.files += 1;
    const repoPath = toRepoPath(filePath);
    seenRepoPaths.add(repoPath);
    const text = fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
    const lines = text.split("\n");
    const moduleDoc = moduleDocLines(lines);
    if (moduleDoc.length === 0) {
        stats.moduleNoDoc += 1;
        sample(`${repoPath}: module doc is missing`);
        if (PUBLIC_DOC_REQUIRED_PREFIXES.some((prefix) => repoPath.startsWith(prefix))) {
            moduleDocRequiredPrefixGaps.push(`${repoPath}: module doc heading is missing`);
        }
    } else if (!hasDoctest(moduleDoc)) {
        stats.moduleNoDoctest += 1;
    }
    const moduleSectionRequirement = moduleDocSectionRequirementByPath.get(repoPath);
    if (moduleSectionRequirement) {
        seenModuleDocSectionRequirementPaths.add(repoPath);
        if (moduleDoc.length === 0) {
            docSectionGaps.push(`${repoPath}: module doc is missing for fixed Zenn-policy slice`);
        } else {
            for (const section of moduleSectionRequirement.sections) {
                if (!docHasSection(moduleDoc, section)) {
                    docSectionGaps.push(`${repoPath}: module doc is missing [${section}] section`);
                }
            }
            for (const usageName of moduleSectionRequirement.doctestUses) {
                if (!moduleDoc.some((docLine) => docLine.includes(usageName))) {
                    docSectionGaps.push(`${repoPath}: module doc doctest must explain representative use of ${usageName}`);
                }
            }
            for (const requiredDocPattern of moduleSectionRequirement.requiredPatterns) {
                if (!moduleDoc.some((docLine) => requiredDocPattern.pattern.test(docLine))) {
                    docSectionGaps.push(`${repoPath}: module doc must mention ${requiredDocPattern.label}`);
                }
            }
        }
    }

    let implBlockIndent = null;
    for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        const trimmed = line.trim();
        const indentation = indentOf(line);
        const startsImpl = implHeaderAt(line);
        if (
            implBlockIndent !== null
            && trimmed !== ""
            && !trimmed.startsWith("//:")
            && indentation <= implBlockIndent
            && !startsImpl
        ) {
            implBlockIndent = null;
        }
        if (startsImpl) {
            implBlockIndent = indentation;
            continue;
        }
        if (implBlockIndent !== null) {
            continue;
        }
        const declaration = declarationAt(line);
        if (!declaration) {
            continue;
        }

        stats.declarations += 1;
        const isPublic = Boolean(declaration[1]);
        const doc = precedingDocLines(lines, index);
        if (doc.length === 0) {
            stats.declarationNoDoc += 1;
            if (isPublic) {
                stats.publicNoDoc += 1;
            } else {
                stats.privateNoDoc += 1;
            }
            const gap = `${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc is missing`;
            sample(gap);
            if (isPublic && PUBLIC_DOC_REQUIRED_PREFIXES.some((prefix) => repoPath.startsWith(prefix))) {
                publicDocRequiredPrefixGaps.push(gap);
            }
        } else {
            const requirementKey = sectionRequirementKey(repoPath, declaration[3]);
            const sectionRequirement = docSectionRequirementByKey.get(requirementKey);
            if (sectionRequirement) {
                seenDocSectionRequirementKeys.add(requirementKey);
                for (const section of sectionRequirement.sections) {
                    if (!docHasSection(doc, section)) {
                        docSectionGaps.push(`${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc is missing [${section}] section`);
                    }
                }
                for (const usageName of sectionRequirement.doctestUses) {
                    if (!doc.some((docLine) => docLine.includes(usageName))) {
                        docSectionGaps.push(`${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc doctest must explain representative use of ${usageName}`);
                    }
                }
                for (const requiredDocPattern of sectionRequirement.requiredPatterns) {
                    if (!doc.some((docLine) => requiredDocPattern.pattern.test(docLine))) {
                        docSectionGaps.push(`${repoPath}:${index + 1}: ${declaration[2]} ${declaration[3]} doc must mention ${requiredDocPattern.label}`);
                    }
                }
            }
        }
        if (doc.length > 0 && !hasDoctest(doc)) {
            stats.declarationNoDoctest += 1;
            if (isPublic) {
                stats.publicNoDoctest += 1;
            } else {
                stats.privateNoDoctest += 1;
            }
        }
    }
}

for (const repoPath of REQUIRED_SCANNER_SENTINELS) {
    assert.ok(
        seenRepoPaths.has(repoPath),
        `${repoPath} must be included in the selfhost documentation scan`,
    );
}
assert(
    fs.existsSync(path.join(repoRoot, DOC_GAP_TRACKING_ISSUE)),
    `selfhost documentation baseline gaps must be tracked by ${DOC_GAP_TRACKING_ISSUE}`,
);
const docGapTrackingIssueText = fs.readFileSync(path.join(repoRoot, DOC_GAP_TRACKING_ISSUE), "utf8").replace(/\r\n/g, "\n");
assert.match(
    docGapTrackingIssueText,
    /^status:\s*open$/m,
    "selfhost documentation baseline issue must remain open while baseline gaps remain",
);
assert.match(
    docGapTrackingIssueText,
    /^resolved:\s*false$/m,
    "selfhost documentation baseline issue must remain unresolved while baseline gaps remain",
);
assert.ok(
    docGapTrackingIssueText.includes("not an accepted quality level"),
    "selfhost documentation baseline issue must state that the baseline is not an accepted quality level",
);
assert.ok(
    docGapTrackingIssueText.includes("fail-closed debt boundary"),
    "selfhost documentation baseline issue must state that the baseline is a fail-closed debt boundary",
);
for (const [key, value] of Object.entries(BASELINE)) {
    assert.ok(
        docGapTrackingIssueText.includes(`${key}=${value}`),
        `selfhost documentation baseline issue must record ${key}=${value}`,
    );
}
for (const key of HARD_DOC_BASELINE_KEYS) {
    assert(
        stats[key] <= BASELINE[key],
        `selfhost documentation gaps increased for ${key}: ${stats[key]} > ${BASELINE[key]}`,
    );
}
for (const key of REPORT_ONLY_DOCTEST_BASELINE_KEYS) {
    assert.ok(
        Object.hasOwn(BASELINE, key),
        `selfhost doctest debt counter must remain visible in the baseline issue: ${key}`,
    );
}
assert.deepEqual(
    moduleDocRequiredPrefixGaps,
    [],
    `selfhost fixed documentation slices must have explicit module doc headings:\n${moduleDocRequiredPrefixGaps.join("\n")}`,
);
assert.deepEqual(
    publicDocRequiredPrefixGaps,
    [],
    `selfhost fixed public documentation slices must not have public declaration doc gaps:\n${publicDocRequiredPrefixGaps.join("\n")}`,
);
const missingSectionRequirementTargets = [...docSectionRequirementByKey.keys()]
    .filter((key) => !seenDocSectionRequirementKeys.has(key));
assert.deepEqual(
    missingSectionRequirementTargets,
    [],
    `selfhost documentation section requirement targets must be found:\n${missingSectionRequirementTargets.join("\n")}`,
);
const missingModuleDocSectionRequirementTargets = [...moduleDocSectionRequirementByPath.keys()]
    .filter((key) => !seenModuleDocSectionRequirementPaths.has(key));
assert.deepEqual(
    missingModuleDocSectionRequirementTargets,
    [],
    `selfhost module documentation section requirement targets must be found:\n${missingModuleDocSectionRequirementTargets.join("\n")}`,
);
assert.deepEqual(
    docSectionGaps,
    [],
    `selfhost fixed documentation slices must preserve the required Zenn-policy doc sections:\n${docSectionGaps.join("\n")}`,
);

console.log("selfhost documentation contract baseline ok");
console.log(JSON.stringify(stats, null, 2));
if (samples.length > 0) {
    console.log("sample gaps:");
    for (const line of samples) {
        console.log(`- ${line}`);
    }
}
