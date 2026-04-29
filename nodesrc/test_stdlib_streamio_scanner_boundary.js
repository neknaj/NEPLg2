#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/streamio.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');

assert.match(
    code,
    /\bfn\s+stream_scanner_byte_at\s+<\(MemPtr<u8>,i32,i32\)->i32>/,
    'StreamScanner byte access must go through stream_scanner_byte_at',
);

assert.doesNotMatch(
    code,
    /\bfn\s+stream_scanner_header_ptr\b/,
    'StreamScanner must not reintroduce a RegionToken header pointer helper',
);
const scannerHeaderMatch = code.match(
    /fn\s+stream_scanner_header_off\b([\s\S]*?)\nfn\s+stream_scanner_byte_at\b/,
);
assert.ok(scannerHeaderMatch, 'StreamScanner header helper section must be found');
assert.doesNotMatch(
    scannerHeaderMatch[1],
    /\bmatch\s+load_i32\s+p\b/,
    'StreamScanner header load must not read through an unproven RegionToken pointer',
);

const scannerMatch = code.match(
    /fn\s+stream_scanner_skip_ws_header\b([\s\S]*?)\nfn\s+scan_f32_impl\b/,
);
assert.ok(scannerMatch, 'StreamScanner scanner implementation section must be found');
const scannerCode = scannerMatch[1];

for (const pattern of [
    /\bload_u8\s+add\s+buf\b/,
    /\bload_u8\s+buf\b/,
    /\bstore_u8\s+add\s+s\b/,
    /\bstring_from_addr_unchecked\s+s\b/,
]) {
    assert.doesNotMatch(
        scannerCode,
        pattern,
        'StreamScanner scanner code must not directly load raw buffer bytes or rebuild string layout',
    );
}

for (const fnName of [
    'stream_scanner_skip_ws_header',
    'skip',
    'scan_token_impl',
    'scan_i32_impl',
    'scan_u32_impl',
    'scan_u64_impl',
    'scan_i64_impl',
    'scan_f64_impl',
]) {
    const re = new RegExp(`fn\\s+${fnName}\\b[\\s\\S]*?(?=\\nfn\\s+|$)`);
    const match = code.match(re);
    assert.ok(match, `${fnName} body must be found`);
    assert.match(
        match[0],
        /\bstream_scanner_byte_at\b/,
        `${fnName} must use stream_scanner_byte_at for buffer reads`,
    );
}

assert.match(
    scannerCode,
    /\bstring_from_mem_unchecked_result\s+mem_ptr_add\s+buf\s+start\s+tlen\b/,
    'scan_token_impl must delegate token string construction to alloc/string',
);

console.log('stdlib streamio scanner boundary regression passed');
