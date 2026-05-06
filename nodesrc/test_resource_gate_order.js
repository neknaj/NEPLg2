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

const body = extractFunctionBody(source, 'run_move_check');
const legacyMoveCheck = body.indexOf('passes::move_check::run(hir_module, types)');
assert(legacyMoveCheck >= 0, 'run_move_check must retain legacy move_check as fallback');

for (const gate of [
    'crate::resource::lower_hir_module(hir_module, types)',
    'run_resource_lowering_coverage_gate(&lowering_coverage, diagnostics)',
    'crate::resource::check_resource_initialized_moves(&resource, types)',
    'run_resource_cell_gate(&initialized_moves, diagnostics, source_map)',
    'crate::resource::check_resource_borrow_lifetimes(&resource)',
    'run_resource_borrow_lifetime_gate(&borrow_lifetimes, diagnostics)',
    'crate::resource::check_resource_effect_boundaries(&resource)',
    'run_resource_effect_boundary_gate(&effect_boundaries, diagnostics, source_map)',
    'crate::resource::check_resource_owner_obligations(&resource, types)',
    'run_resource_owner_obligation_gate(&owner_obligations, diagnostics, source_map)',
]) {
    const index = body.indexOf(gate);
    assert(index >= 0, `run_move_check must call ${gate}`);
    assert(
        index < legacyMoveCheck,
        `${gate} must run before legacy passes::move_check::run`,
    );
}

console.log('resource gate order ok');
