#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const parserPath = 'stdlib/nm/parser.nepl';
const documentPath = 'stdlib/nm/parser/document.nepl';
const scannerPath = 'stdlib/nm/parser/scanner.nepl';
const jsonInlinePath = 'stdlib/nm/parser/json_inline.nepl';
const htmlPath = 'stdlib/nm/html_gen.nepl';
const htmlInlinePath = 'stdlib/nm/html_inline.nepl';
const parserSrc = fs.readFileSync(path.join(repoRoot, parserPath), 'utf8');
const documentSrc = fs.readFileSync(path.join(repoRoot, documentPath), 'utf8');
const scannerSrc = fs.readFileSync(path.join(repoRoot, scannerPath), 'utf8');
const jsonInlineSrc = fs.readFileSync(path.join(repoRoot, jsonInlinePath), 'utf8');
const htmlSrc = fs.readFileSync(path.join(repoRoot, htmlPath), 'utf8');
const htmlInlineSrc = fs.readFileSync(path.join(repoRoot, htmlInlinePath), 'utf8');

function implementationSource(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

const parser = implementationSource(parserSrc);
const document = implementationSource(documentSrc);
const scanner = implementationSource(scannerSrc);
const jsonInline = implementationSource(jsonInlineSrc);
const html = implementationSource(htmlSrc);
const htmlInline = implementationSource(htmlInlineSrc);
const combined = `${parser}\n${document}\n${scanner}\n${jsonInline}\n${html}\n${htmlInline}`;

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
    /Vec<Inline>/,
    /v::push<Inline>/,
    /struct\s+InlinePushRes/,
    /struct\s+StrPushRes/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(combined, pattern, 'nm inline serializers must not use unsafe unwraps or non-Copy Vec-backed inline AST');
}

assert.match(jsonInline, /fn\s+nm_inline_to_json_into\s+<\(StringBuilder,str\)->StringBuilder>\s+\(out,\s*s\):[\s\S]*match\s+ch:/, 'JSON inline serializer must dispatch marker bytes with match');
assert.match(jsonInline, /fn\s+nm_inline_to_json\s+<\(str\)->str>\s+\(s\):[\s\S]*sb_build\s+nm_inline_to_json_into\s+string_builder_new\s+s/, 'JSON inline wrapper must delegate through the builder serializer');
assert.match(htmlInline, /fn\s+nm_inline_to_html\s+<\(str\)->str>\s+\(s\):[\s\S]*match\s+ch:/, 'HTML inline serializer must dispatch marker bytes with match');
assert.match(scanner, /fn\s+nm_find_gloss_slash\s+<\(str,i32,i32\)->i32>/, 'gloss slash scanning must stay string-backed');
assert.match(scanner, /fn\s+nm_find_math_end\s+<\(str,i32,i32\)->i32>/, 'math delimiter scanning must stay string-backed');

console.log('stdlib nm parser inline unsafe unwrap regression passed');
