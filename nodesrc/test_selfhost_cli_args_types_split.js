#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const argsRel = 'stdlib/neplg2/cli/args.nepl';
const typesRel = 'stdlib/neplg2/cli/args/types.nepl';
const classifyRel = 'stdlib/neplg2/cli/args/classify.nepl';
const emitRel = 'stdlib/neplg2/cli/args/emit.nepl';
const argsSrc = fs.readFileSync(path.join(repoRoot, argsRel), 'utf8');
const typesSrc = fs.readFileSync(path.join(repoRoot, typesRel), 'utf8');
const classifySrc = fs.readFileSync(path.join(repoRoot, classifyRel), 'utf8');
const emitSrc = fs.readFileSync(path.join(repoRoot, emitRel), 'utf8');

assert.match(
    argsSrc,
    /pub\s+#import\s+"\.(?:\/|\\)args(?:\/|\\)types"\s+as\s+\*/,
    'neplg2/cli/args must re-export cli/args/types as the compatibility facade',
);

assert.match(
    argsSrc,
    /#import\s+"\.(?:\/|\\)args(?:\/|\\)classify"\s+as\s+\*/,
    'neplg2/cli/args must import cli/args/classify for parser-private token classification',
);

assert.match(
    argsSrc,
    /pub\s+#import\s+"\.(?:\/|\\)args(?:\/|\\)emit"\s+as\s+\*/,
    'neplg2/cli/args must re-export cli/args/emit as the emit option facade',
);

for (const name of [
    'SelfhostCliTarget',
    'SelfhostCliEmit',
    'SelfhostCliEmitSet',
    'SelfhostCliProfile',
    'SelfhostCliErrorKind',
    'SelfhostCliOptions',
]) {
    assert.match(
        typesSrc,
        new RegExp(`\\b(?:pub\\s+)?(?:enum|struct)\\s+${name}\\b`),
        `${name} must live in cli/args/types.nepl`,
    );
    assert.doesNotMatch(
        argsSrc,
        new RegExp(`\\bpub\\s+(?:enum|struct)\\s+${name}\\b`),
        `${name} must not be reintroduced into cli/args.nepl`,
    );
}

assert.match(
    classifySrc,
    /\bpub\s+enum\s+SelfhostCliArgKind\b/,
    'parser-only SelfhostCliArgKind must live in cli/args/classify.nepl',
);

assert.doesNotMatch(
    argsSrc,
    /\benum\s+SelfhostCliArgKind\b/,
    'SelfhostCliArgKind must not be reintroduced into cli/args.nepl',
);

for (const name of [
    'selfhost_cli_emit_set_default',
    'selfhost_cli_parse_emit_set_value',
    'selfhost_cli_emit_is_wasm',
    'selfhost_cli_emit_set_has_wat',
    'selfhost_cli_emit_set_has_wat_min',
    'selfhost_cli_emit_set_has_llvm',
    'selfhost_cli_emit_set_has_llvm_min',
]) {
    assert.match(
        emitSrc,
        new RegExp(`\\bpub\\s+fn\\s+${name}\\b`),
        `${name} must live in cli/args/emit.nepl`,
    );
    assert.doesNotMatch(
        argsSrc,
        new RegExp(`\\bpub\\s+fn\\s+${name}\\b`),
        `${name} must not be reintroduced into cli/args.nepl`,
    );
}

console.log('selfhost CLI args module split regression passed');
