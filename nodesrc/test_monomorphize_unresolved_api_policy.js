#!/usr/bin/env node

const fs = require('node:fs');
const path = require('node:path');

const ROOT = path.resolve(__dirname, '..');
const monomorphizePath = path.join(ROOT, 'nepl-core', 'src', 'monomorphize.rs');
const source = fs.readFileSync(monomorphizePath, 'utf8').replace(/\r\n/g, '\n');

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

assert(
    source.includes('pub fn monomorphize(ctx: &mut TypeCtx, module: HirModule) -> MonomorphizeResult'),
    'public monomorphize API must return MonomorphizeResult with structured unresolved trait calls',
);
assert(
    !source.includes('monomorphize_with_unresolved_trait_calls'),
    'monomorphize.rs must not keep a second public unresolved-trait entry point',
);
assert(
    !source.includes('assert_no_trait_calls'),
    'monomorphize.rs must not keep the panic-based unresolved trait assertion',
);
assert(
    !source.includes('panic!'),
    'monomorphize.rs must not panic for unresolved trait calls',
);

console.log('monomorphize unresolved API policy ok');
