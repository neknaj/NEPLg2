#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const RESOURCE_DIR = path.join(ROOT, 'nepl-core', 'src', 'resource');

function readResource(name) {
    return fs.readFileSync(path.join(RESOURCE_DIR, name), 'utf8').replace(/\r\n/g, '\n');
}

function lineCount(text) {
    return text.split('\n').length;
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertFile(name) {
    const filePath = path.join(RESOURCE_DIR, name);
    assert(fs.existsSync(filePath), `missing resource module: ${name}`);
    return readResource(name);
}

function assertMissing(name) {
    const filePath = path.join(RESOURCE_DIR, name);
    assert(!fs.existsSync(filePath), `${name} must not be reintroduced as a monolithic checker`);
}

function assertContains(text, needle, source) {
    assert(text.includes(needle), `${source} must contain ${needle}`);
}

function assertNotContains(text, needle, source) {
    assert(!text.includes(needle), `${source} must not contain ${needle}`);
}

function escapeRegExp(text) {
    return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function assertUsesResourceModuleSymbol(text, moduleName, symbolName, source) {
    const directImport = `super::${moduleName}::${symbolName}`;
    const groupedImport = new RegExp(
        `super::${escapeRegExp(moduleName)}::\\{[^}]*\\b${escapeRegExp(symbolName)}\\b[^}]*\\}`,
    );
    assert(
        text.includes(directImport) || groupedImport.test(text),
        `${source} must import ${symbolName} from super::${moduleName}`,
    );
}

const mod = assertFile('mod.rs');
assertMissing('check.rs');

for (const moduleName of [
    'initialized.rs',
    'borrow_check.rs',
    'borrow_summary.rs',
    'owner_check.rs',
    'owner_flow.rs',
    'owner_raw_view.rs',
    'owner_summary.rs',
    'owner_summary_leaf.rs',
    'owner_summary_record.rs',
    'owner_return.rs',
    'summary.rs',
    'effect.rs',
    'effect_check.rs',
    'effect_summary.rs',
    'effect_identity.rs',
    'coverage.rs',
    'coverage_hir.rs',
    'coverage_resource.rs',
    'lower_raw_address.rs',
    'lower_raw_memory.rs',
    'report.rs',
    'shadow.rs',
    'initialized_alias.rs',
    'initialized_alias_flow.rs',
    'initialized_external_io.rs',
    'initialized_raw_memory.rs',
    'initialized_rekey.rs',
    'initialized_summary.rs',
    'initialized_summary_apply.rs',
    'initialized_summary_build.rs',
]) {
    assertFile(moduleName);
}

for (const moduleDecl of [
    'mod initialized;',
    'mod borrow_check;',
    'mod borrow_summary;',
    'mod owner_check;',
    'mod owner_flow;',
    'mod owner_raw_view;',
    'mod owner_summary;',
    'mod owner_summary_leaf;',
    'mod owner_summary_record;',
    'mod owner_return;',
    'mod summary;',
    'mod effect;',
    'mod effect_check;',
    'mod effect_summary;',
    'mod effect_identity;',
    'mod coverage;',
    'mod coverage_hir;',
    'mod coverage_resource;',
    'mod lower_raw_address;',
    'mod lower_raw_memory;',
    'mod report;',
    'mod shadow;',
    'mod initialized_alias;',
    'mod initialized_alias_flow;',
    'mod initialized_external_io;',
    'mod initialized_raw_memory;',
    'mod initialized_rekey;',
    'mod initialized_summary;',
    'mod initialized_summary_apply;',
    'mod initialized_summary_build;',
]) {
    assertContains(mod, moduleDecl, 'resource/mod.rs');
}

assertNotContains(mod, 'mod check;', 'resource/mod.rs');

const initialized = readResource('initialized.rs');
const borrowCheck = readResource('borrow_check.rs');
const borrowSummary = readResource('borrow_summary.rs');
const ownerCheck = readResource('owner_check.rs');
const ownerSummary = readResource('owner_summary.rs');
const ownerReturn = readResource('owner_return.rs');
const summary = readResource('summary.rs');
const effect = readResource('effect.rs');
const effectCheck = readResource('effect_check.rs');
const effectSummary = readResource('effect_summary.rs');
const coverage = readResource('coverage.rs');
const coverageHir = readResource('coverage_hir.rs');
const coverageResource = readResource('coverage_resource.rs');
const lower = readResource('lower.rs');
const lowerRawAddress = readResource('lower_raw_address.rs');
const lowerRawMemory = readResource('lower_raw_memory.rs');

assertContains(initialized, 'struct ResourceCheckEngine', 'initialized.rs');
assertContains(borrowCheck, 'struct ResourceBorrowCheckEngine', 'borrow_check.rs');
assertContains(ownerCheck, 'struct ResourceOwnerCheckEngine', 'owner_check.rs');
assertContains(effectCheck, 'struct ResourceEffectBoundaryEngine', 'effect_check.rs');

assertNotContains(effect, 'struct ResourceEffectBoundaryEngine', 'effect.rs');
assertContains(effect, 'pub fn check_resource_effect_boundaries', 'effect.rs');
assertContains(coverage, 'pub fn compare_hir_resource_lowering_typed', 'coverage.rs');
assertContains(coverageHir, 'pub(super) fn hir_body_coverage', 'coverage_hir.rs');
assertContains(
    coverageResource,
    'pub(super) fn resource_function_coverage',
    'coverage_resource.rs',
);
assertContains(
    lowerRawAddress,
    'pub(super) fn push_user_raw_address_return_semantics',
    'lower_raw_address.rs',
);
assertContains(
    lowerRawMemory,
    'pub(super) fn raw_memory_op_from_name',
    'lower_raw_memory.rs',
);
assertNotContains(lower, 'struct RawAddressSource', 'lower.rs');
assertUsesResourceModuleSymbol(
    borrowSummary,
    'borrow_check',
    'ResourceBorrowCheckEngine',
    'borrow_summary.rs',
);
assertUsesResourceModuleSymbol(
    ownerSummary,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_summary.rs',
);
assertUsesResourceModuleSymbol(
    ownerReturn,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'owner_return.rs',
);
assertUsesResourceModuleSymbol(
    effectSummary,
    'effect_check',
    'ResourceEffectBoundaryEngine',
    'effect_summary.rs',
);

const maxLines = new Map([
    ['effect.rs', 160],
    ['initialized.rs', 750],
    ['borrow_check.rs', 550],
    ['borrow_summary.rs', 120],
    ['owner_check.rs', 800],
    ['owner_flow.rs', 620],
    ['owner_raw_view.rs', 180],
    ['owner_summary.rs', 380],
    ['owner_summary_leaf.rs', 260],
    ['owner_summary_record.rs', 260],
    ['owner_return.rs', 400],
    ['effect_check.rs', 700],
    ['summary.rs', 300],
    ['effect_summary.rs', 250],
    ['coverage.rs', 280],
    ['coverage_hir.rs', 420],
    ['coverage_resource.rs', 520],
    ['lower.rs', 1300],
    ['lower_raw_address.rs', 700],
    ['lower_raw_memory.rs', 120],
    ['initialized_alias.rs', 550],
    ['initialized_alias_flow.rs', 550],
    ['initialized_external_io.rs', 140],
    ['initialized_raw_memory.rs', 300],
    ['initialized_rekey.rs', 160],
    ['initialized_summary.rs', 80],
    ['initialized_summary_apply.rs', 160],
    ['initialized_summary_build.rs', 260],
]);

for (const [name, limit] of maxLines) {
    const lines = lineCount(readResource(name));
    assert(lines <= limit, `${name} has ${lines} lines; responsibility split limit is ${limit}`);
}

console.log('resource checker responsibility ok');
