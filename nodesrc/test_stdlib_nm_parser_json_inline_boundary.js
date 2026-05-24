#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { legacyTypeSyntaxView } = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
}

const parserPath = 'stdlib/nm/parser.nepl';
const documentPath = 'stdlib/nm/parser/document.nepl';
const jsonInlinePath = 'stdlib/nm/parser/json_inline.nepl';
const parser = legacyTypeSyntaxView(read(parserPath));
const document = legacyTypeSyntaxView(read(documentPath));
const jsonInline = legacyTypeSyntaxView(read(jsonInlinePath));

assert.match(
    jsonInline,
    /pub\s+fn\s+nm_inline_to_json_into\s+<\(StringBuilder,str\)->StringBuilder>\s+\(out,\s*s\):[\s\S]*match\s+ch:/,
    `${jsonInlinePath} must own inline JSON marker dispatch`,
);
assert.match(
    jsonInline,
    /pub\s+fn\s+nm_inline_to_json\s+<\(str\)->str>\s+\(s\):[\s\S]*sb_build\s+nm_inline_to_json_into\s+string_builder_new\s+s/,
    `${jsonInlinePath} must expose the string-building wrapper`,
);
assert.match(
    jsonInline,
    /^#import "\.\.\/json_escape" as json$/m,
    `${jsonInlinePath} must import JSON escape helper directly`,
);

assert.match(
    document,
    /^#import "\.\/json_inline" as json_inline$/m,
    `${documentPath} must import the dedicated inline JSON serializer`,
);
assert.match(
    document,
    /\bjson_inline::nm_inline_to_json_into\b/,
    `${documentPath} must delegate inline JSON output to json_inline`,
);
assert.doesNotMatch(
    parser,
    /\bfn\s+nm_inline_to_json(?:_into)?\b/,
    `${parserPath} must not reintroduce inline JSON serializer ownership`,
);
assert.doesNotMatch(
    document,
    /\bfn\s+nm_inline_to_json(?:_into)?\b/,
    `${documentPath} must not reintroduce inline JSON serializer ownership`,
);

console.log('stdlib nm parser JSON inline boundary regression passed');
