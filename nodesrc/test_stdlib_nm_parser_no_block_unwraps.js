#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const parserPath = 'stdlib/nm/parser.nepl';
const scannerPath = 'stdlib/nm/parser/scanner.nepl';
const htmlPath = 'stdlib/nm/html_gen.nepl';
const htmlInlinePath = 'stdlib/nm/html_inline.nepl';
const parserSrc = fs.readFileSync(path.join(repoRoot, parserPath), 'utf8');
const scannerSrc = fs.readFileSync(path.join(repoRoot, scannerPath), 'utf8');
const htmlSrc = fs.readFileSync(path.join(repoRoot, htmlPath), 'utf8');
const htmlInlineSrc = fs.readFileSync(path.join(repoRoot, htmlInlinePath), 'utf8');

function implementationSource(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

const parser = implementationSource(parserSrc);
const scanner = implementationSource(scannerSrc);
const html = implementationSource(htmlSrc);
const htmlInline = implementationSource(htmlInlineSrc);
const combined = `${parser}\n${scanner}\n${html}\n${htmlInline}`;

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
    /Vec<Node>/,
    /v::push<Node>/,
    /struct\s+NodePushRes/,
    /alloc_raw\s+size_of<NestSection>/,
    /store<NestSection>/,
    /load<NestSection>/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(combined, pattern, 'nm block serializers must not use unsafe unwraps or non-Copy Vec-backed block AST');
}

assert.match(parser, /pub\s+struct\s+Document:[\s\S]*source\s+<str>/, 'Document must be a Copy source view');
assert.match(parser, /pub\s+fn\s+parse_markdown\s+<\(str\)->Document>\s+\(input\):\s+Document\s+input/, 'parse_markdown must not allocate a non-Copy block AST');
assert.match(parser, /pub\s+fn\s+document_to_json\s+<\(Document\)->str>\s+\(doc\):[\s\S]*let\s+src\s+<str>\s+get\s+doc\s+"source"/, 'document_to_json must direct-serialize from source');
assert.match(html, /fn\s+nm_render_source_html\s+<\(str\)->str>\s+\(src\):[\s\S]*while\s+lt\s+pos\s+n:/, 'HTML renderer must direct-scan source lines');
assert.match(scanner, /fn\s+nm_deepest_level\s+<\(bool,bool,bool,bool,bool,bool\)->i32>/, 'section stack must remain Copy bool state');

console.log('stdlib nm parser block unsafe unwrap regression passed');
