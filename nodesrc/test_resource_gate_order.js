#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const compilerPath = path.join(ROOT, 'nepl-core', 'src', 'compiler.rs');
const source = fs.readFileSync(compilerPath, 'utf8').replace(/\r\n/g, '\n');

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function extractFunctionBody(text, name) {
    const start = text.indexOf(`fn ${name}(`);
    assert(start >= 0, `missing function ${name}`);
    const open = text.indexOf('{', start);
    assert(open >= 0, `missing body for ${name}`);

    let depth = 0;
    for (let i = open; i < text.length; i += 1) {
        const ch = text[i];
        if (ch === '{') {
            depth += 1;
        } else if (ch === '}') {
            depth -= 1;
            if (depth === 0) {
                return text.slice(open + 1, i);
            }
        }
    }

    throw new Error(`unterminated body for ${name}`);
}

const body = extractFunctionBody(source, 'run_resource_static_check');
assert(
    !body.includes('passes::move_check::run'),
    'run_resource_static_check must not call legacy passes::move_check::run',
);

for (const gate of [
    'crate::resource::lower_hir_module(hir_module, types)',
    'run_resource_lowering_coverage_gate(&lowering_coverage, diagnostics)',
    'crate::resource::check_resource_initialized_moves_with_summary_cache(',
    'crate::resource::check_resource_initialized_moves(&resource, types)',
    'run_resource_cell_gate(&initialized_moves, diagnostics)',
    'crate::resource::compute_resource_drop_elaboration_plan(&resource, &initialized_moves)',
    'run_resource_drop_elaboration_plan_gate(',
    'crate::resource::check_resource_borrow_lifetimes(&resource, types)',
    'run_resource_borrow_lifetime_gate(&borrow_lifetimes, diagnostics)',
    'crate::resource::check_resource_effect_boundaries_typed(&resource, types)',
    'run_resource_effect_boundary_gate(&effect_boundaries, diagnostics, source_map)',
    'crate::resource::check_resource_owner_obligations(&resource, types)',
    'run_resource_owner_obligation_gate(&owner_obligations, diagnostics)',
]) {
    const index = body.indexOf(gate);
    assert(index >= 0, `run_resource_static_check must call ${gate}`);
}

assert(
    !body.includes('run_resource_cell_gate(&initialized_moves, diagnostics, source_map)'),
    'Resource cell gate must not receive SourceMap because raw-memory-boundary suppression belongs to the effect boundary gate',
);
assert(
    !body.includes('run_resource_owner_obligation_gate(&owner_obligations, diagnostics, source_map)'),
    'Resource owner obligation gate must not receive SourceMap because raw-memory-boundary suppression belongs to the effect boundary gate',
);
assert(
    !body.includes('crate::resource::check_resource_effect_boundaries(&resource)'),
    'Resource effect boundary gate must use typed effect summaries from TypeCtx instead of falling back to untyped Resource IR effects',
);

const compilerRelative = source
    .replace(/\r\n/g, '\n')
    .includes('passes::move_check::run');
assert(!compilerRelative, 'compiler.rs must not retain legacy move_check fallback');

const prepareWrapperBody = extractFunctionBody(source, 'prepare_module_for_codegen_with_source_map');
assert(
    prepareWrapperBody.includes('prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash('),
    'prepare_module_for_codegen_with_source_map must delegate to the dependency-surface wrapper',
);
const prepareDependencyWrapperBody = extractFunctionBody(
    source,
    'prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash',
);
assert(
    prepareDependencyWrapperBody.includes('prepare_module_for_codegen_with_source_map_dependency_public_surface_hash_and_resource_summary_value_cache(')
        && prepareDependencyWrapperBody.includes('None'),
    'prepare_module_for_codegen_with_source_map_and_dependency_public_surface_hash must keep the cache-free public wrapper',
);
const prepareBody = extractFunctionBody(
    source,
    'prepare_module_for_codegen_with_source_map_dependency_public_surface_hash_and_resource_summary_value_cache',
);
const resourceTypecheckIndex = prepareBody.indexOf('let resource_tc = run_typecheck(');
const resourceMonomorphizeIndex = prepareBody.indexOf(
    'monomorphize::monomorphize(',
);
const resourcePlanBindingIndex = prepareBody.indexOf(
    'let resource_drop_elaboration_plan =',
);
const resourceCheckIndex = prepareBody.indexOf('run_resource_static_check(');
const hirBridgeGateIndex = prepareBody.indexOf(
    'run_resource_drop_elaboration_hir_bridge_gate(',
);
const dropInsertionIndex = prepareBody.indexOf(
    'passes::insert_resource_drops(&mut hir_module, &mut types, &resource_drop_elaboration_plan)',
);
const finalMonomorphizeIndex = prepareBody.lastIndexOf(
    'monomorphize::monomorphize(',
);
const preparedPlanFieldIndex = prepareBody.lastIndexOf('resource_drop_elaboration_plan,');
assert(
    resourceTypecheckIndex >= 0,
    'prepare_module_for_codegen_with_source_map must typecheck once before Resource IR source checking',
);
assert(
    resourceMonomorphizeIndex >= 0,
    'prepare_module_for_codegen_with_source_map must monomorphize source HIR for Resource IR before generated drops',
);
assert(
    !prepareBody.includes('tc.module.clone()')
        && !prepareBody.includes('resource_tc.module.clone()'),
    'Resource IR source monomorphize must not recursively clone HIR because deep prefix trees can overflow the native stack',
);
assert(
    resourceCheckIndex >= 0,
    'prepare_module_for_codegen_with_source_map must run Resource IR static check on monomorphized source HIR',
);
assert(
    resourcePlanBindingIndex >= 0,
    'prepare_module_for_codegen_with_source_map must retain the checked Resource IR drop elaboration plan',
);
assert(
    !prepareBody.includes('let mut codegen_tc = run_typecheck('),
    'prepare_module_for_codegen_with_source_map must not keep a second legacy HIR typecheck path after Resource IR drop insertion is authoritative',
);
assert(
    hirBridgeGateIndex >= 0,
    'prepare_module_for_codegen_with_source_map must validate the checked Resource IR drop plan before consuming it',
);
assert(
    dropInsertionIndex >= 0,
    'prepare_module_for_codegen_with_source_map must elaborate drops from ResourceDropElaborationPlan',
);
assert(
    !prepareBody.includes('passes::insert_drops'),
    'prepare_module_for_codegen_with_source_map must not call the legacy HIR VarState drop walker',
);
assert(
    finalMonomorphizeIndex > dropInsertionIndex,
    'prepare_module_for_codegen_with_source_map must rerun monomorphize after plan-based drop insertion to resolve generated trait calls',
);
assert(
    resourceTypecheckIndex < resourceMonomorphizeIndex
        && resourceMonomorphizeIndex < resourceCheckIndex
        && resourcePlanBindingIndex < resourceCheckIndex
        && resourceCheckIndex < hirBridgeGateIndex
        && hirBridgeGateIndex < dropInsertionIndex,
    'Resource IR static check must run on drop-free source semantics before plan-based drop insertion so generated drops cannot mask source resource violations',
);
assert(
    preparedPlanFieldIndex > dropInsertionIndex,
    'PreparedProgram must carry the checked Resource IR drop elaboration plan after codegen HIR is prepared',
);

console.log('resource gate authority ok');
