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
    'crate::resource::check_resource_initialized_moves(&resource, types)',
    'run_resource_cell_gate(&initialized_moves, diagnostics, source_map)',
    'crate::resource::check_resource_borrow_lifetimes(&resource, types)',
    'run_resource_borrow_lifetime_gate(&borrow_lifetimes, diagnostics)',
    'crate::resource::check_resource_effect_boundaries(&resource)',
    'run_resource_effect_boundary_gate(&effect_boundaries, diagnostics, source_map)',
    'crate::resource::check_resource_owner_obligations(&resource, types)',
    'run_resource_owner_obligation_gate(&owner_obligations, diagnostics, source_map)',
]) {
    const index = body.indexOf(gate);
    assert(index >= 0, `run_resource_static_check must call ${gate}`);
}

const compilerRelative = source
    .replace(/\r\n/g, '\n')
    .includes('passes::move_check::run');
assert(!compilerRelative, 'compiler.rs must not retain legacy move_check fallback');

const prepareBody = extractFunctionBody(source, 'prepare_module_for_codegen_with_source_map');
const resourceTypecheckIndex = prepareBody.indexOf('let resource_tc = run_typecheck(');
const resourceMonomorphizeIndex = prepareBody.indexOf(
    'monomorphize::monomorphize_with_unresolved_trait_calls(',
);
const resourceCheckIndex = prepareBody.indexOf(
    '&resource_hir_module,\n        &resource_types,\n        &mut diagnostics,\n        source_map,',
);
const codegenTypecheckIndex = prepareBody.indexOf('let mut codegen_tc = run_typecheck(');
const dropInsertionIndex = prepareBody.indexOf(
    'passes::insert_drops(&mut codegen_tc.module, &mut codegen_tc.types);',
);
assert(
    resourceTypecheckIndex >= 0,
    'prepare_module_for_codegen_with_source_map must build a dedicated typed HIR for Resource IR source checking',
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
    codegenTypecheckIndex >= 0,
    'prepare_module_for_codegen_with_source_map must build a separate typed HIR for legacy HIR drop elaboration until Resource IR drop insertion replaces it',
);
assert(
    dropInsertionIndex >= 0,
    'prepare_module_for_codegen_with_source_map must still elaborate drops before monomorphize/codegen',
);
assert(
    resourceTypecheckIndex < resourceMonomorphizeIndex
        && resourceMonomorphizeIndex < resourceCheckIndex
        && resourceCheckIndex < codegenTypecheckIndex
        && codegenTypecheckIndex < dropInsertionIndex,
    'Resource IR static check must run on drop-free source semantics before HIR drop insertion so generated drops cannot mask source resource violations',
);

console.log('resource gate authority ok');
