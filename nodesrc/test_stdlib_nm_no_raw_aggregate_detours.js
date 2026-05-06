#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');

function implementationSource(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), 'utf8')
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join('\n');
}

const parserPath = 'stdlib/nm/parser.nepl';
const htmlPath = 'stdlib/nm/html_gen.nepl';
const htmlInlinePath = 'stdlib/nm/html_inline.nepl';
const parser = implementationSource(parserPath);
const html = implementationSource(htmlPath);
const htmlInline = implementationSource(htmlInlinePath);

const forbiddenParserPatterns = [
    /Vec<Inline>/,
    /Vec<Node>/,
    /v::push<Inline>/,
    /v::push<Node>/,
    /str_split\s+input\s+"\\n"/,
    /alloc_raw\s+size_of<Vec<str>>/,
    /store<Vec<str>>\s+\w+_mem\s+lines/,
    /load<Vec<str>>\s+\w+_mem/,
    /alloc_raw\s+size_of<ParaRes>/,
    /store<ParaRes>\s+\w+_mem/,
    /load<ParaRes>\s+\w+_mem/,
    /alloc_raw\s+size_of<NestSection>/,
    /store<NestSection>\s+\w+_mem/,
    /load<NestSection>\s+\w+_mem/,
    /alloc_raw\s+size_of<Heading>/,
    /store<Heading>\s+\w+_mem/,
    /load<Heading>\s+\w+_mem/,
];

for (const pattern of forbiddenParserPatterns) {
    assert.doesNotMatch(parser, pattern, `${parserPath} must not reintroduce aggregate raw-memory decomposition`);
}

const forbiddenHtmlPatterns = [
    /alloc_raw\s+size_of<NestSection>/,
    /store<NestSection>/,
    /load<NestSection>/,
    /alloc_raw\s+size_of<Heading>/,
    /store<Heading>/,
    /load<Heading>/,
];

for (const pattern of forbiddenHtmlPatterns) {
    assert.doesNotMatch(html, pattern, `${htmlPath} must not reintroduce aggregate raw-memory decomposition`);
    assert.doesNotMatch(htmlInline, pattern, `${htmlInlinePath} must not reintroduce aggregate raw-memory decomposition`);
}

assert.match(parser, /pub\s+struct\s+Document:[\s\S]*source\s+<str>/, 'Document must remain a source view instead of owning a non-Copy AST container');
assert.match(parser, /pub\s+fn\s+parse_markdown\s+<\(str\)->Document>\s+\(input\):\s+Document\s+input/, 'parse_markdown must construct the strict-move-safe Document source view directly');
assert.match(parser, /pub\s+fn\s+document_to_json\s+<\(Document\)->str>\s+\(doc\):[\s\S]*let\s+src\s+<str>\s+get\s+doc\s+"source"/, 'document_to_json must serialize from Document.source');
assert.match(html, /pub\s+fn\s+render_document\s+<\(Document\)->str>\s+\(doc\):\s+nm_render_source_html\s+get\s+doc\s+"source"/, 'render_document must serialize from Document.source');
assert.match(parser, /fn\s+nm_inline_to_json\s+<\(str\)->str>\s+\(s\):/, 'inline JSON serializer must be direct and string-backed');
assert.match(htmlInline, /fn\s+nm_inline_to_html\s+<\(str\)->str>\s+\(s\):/, 'inline HTML serializer must be direct and string-backed');

console.log('stdlib nm raw aggregate detour regression passed');
