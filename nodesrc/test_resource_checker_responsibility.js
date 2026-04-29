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
    'owner_check.rs',
    'owner_flow.rs',
    'summary.rs',
    'effect.rs',
    'effect_check.rs',
    'effect_summary.rs',
    'effect_identity.rs',
    'report.rs',
    'shadow.rs',
    'initialized_alias.rs',
    'initialized_alias_flow.rs',
    'initialized_raw_memory.rs',
]) {
    assertFile(moduleName);
}

for (const moduleDecl of [
    'mod initialized;',
    'mod borrow_check;',
    'mod owner_check;',
    'mod owner_flow;',
    'mod summary;',
    'mod effect;',
    'mod effect_check;',
    'mod effect_summary;',
    'mod effect_identity;',
    'mod report;',
    'mod shadow;',
    'mod initialized_alias;',
    'mod initialized_alias_flow;',
    'mod initialized_raw_memory;',
]) {
    assertContains(mod, moduleDecl, 'resource/mod.rs');
}

assertNotContains(mod, 'mod check;', 'resource/mod.rs');

const initialized = readResource('initialized.rs');
const borrowCheck = readResource('borrow_check.rs');
const ownerCheck = readResource('owner_check.rs');
const summary = readResource('summary.rs');
const effect = readResource('effect.rs');
const effectCheck = readResource('effect_check.rs');
const effectSummary = readResource('effect_summary.rs');

assertContains(initialized, 'struct ResourceCheckEngine', 'initialized.rs');
assertContains(borrowCheck, 'struct ResourceBorrowCheckEngine', 'borrow_check.rs');
assertContains(ownerCheck, 'struct ResourceOwnerCheckEngine', 'owner_check.rs');
assertContains(effectCheck, 'struct ResourceEffectBoundaryEngine', 'effect_check.rs');

assertNotContains(effect, 'struct ResourceEffectBoundaryEngine', 'effect.rs');
assertContains(effect, 'pub fn check_resource_effect_boundaries', 'effect.rs');
assertUsesResourceModuleSymbol(
    summary,
    'borrow_check',
    'ResourceBorrowCheckEngine',
    'summary.rs',
);
assertUsesResourceModuleSymbol(
    summary,
    'owner_check',
    'ResourceOwnerCheckEngine',
    'summary.rs',
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
    ['owner_check.rs', 800],
    ['owner_flow.rs', 620],
    ['effect_check.rs', 700],
    ['summary.rs', 300],
    ['effect_summary.rs', 250],
    ['initialized_alias.rs', 550],
    ['initialized_alias_flow.rs', 550],
    ['initialized_raw_memory.rs', 300],
]);

for (const [name, limit] of maxLines) {
    const lines = lineCount(readResource(name));
    assert(lines <= limit, `${name} has ${lines} lines; responsibility split limit is ${limit}`);
}

console.log('resource checker responsibility ok');
