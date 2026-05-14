#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(__dirname, '..');
const facadeRelPath = 'stdlib/std/streamio.nepl';
const scannerRelPath = 'stdlib/std/streamio/scanner.nepl';
const scannerCursorRelPath = 'stdlib/std/streamio/scanner/cursor.nepl';
const scannerNumberRelPath = 'stdlib/std/streamio/scanner/number.nepl';
const scannerNumberIntRelPath = 'stdlib/std/streamio/scanner/number/int.nepl';
const scannerNumberFloatRelPath = 'stdlib/std/streamio/scanner/number/float.nepl';
const scannerStateRelPath = 'stdlib/std/streamio/scanner/state.nepl';
const facade = fs.readFileSync(path.join(repoRoot, facadeRelPath), 'utf8');
const src = fs.readFileSync(path.join(repoRoot, scannerRelPath), 'utf8');
const cursorSrc = fs.readFileSync(path.join(repoRoot, scannerCursorRelPath), 'utf8');
const numberSrc = fs.readFileSync(path.join(repoRoot, scannerNumberRelPath), 'utf8');
const numberIntSrc = fs.readFileSync(path.join(repoRoot, scannerNumberIntRelPath), 'utf8');
const numberFloatSrc = fs.readFileSync(path.join(repoRoot, scannerNumberFloatRelPath), 'utf8');
const stateSrc = fs.readFileSync(path.join(repoRoot, scannerStateRelPath), 'utf8');

const code = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const stateCode = stateSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const cursorCode = cursorSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const numberCode = numberSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const numberIntCode = numberIntSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const numberFloatCode = numberFloatSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const facadeCode = facade
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerImplementationCode = `${cursorCode}\n${code}\n${numberCode}\n${numberIntCode}\n${numberFloatCode}`;

assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/scanner"\s+as\s+\*/,
    'std/streamio.nepl must re-export the scanner submodule',
);

assert.match(
    code,
    /#import\s+"std\/streamio\/scanner\/state"\s+as\s+\*/,
    'std/streamio/scanner.nepl must import the scanner state boundary module',
);
assert.match(
    code,
    /#import\s+"std\/streamio\/scanner\/cursor"\s+as\s+\*/,
    'std/streamio/scanner.nepl must import the scanner cursor boundary module',
);
assert.match(
    code,
    /#import\s+"std\/streamio\/scanner\/number"\s+as\s+\*/,
    'std/streamio/scanner.nepl must import the scanner number parser module',
);

for (const pattern of [
    /\bstruct\s+StreamScanner\b/,
    /\benum\s+StreamScannerHeaderField\b/,
    /\bfn\s+stream_scanner_load_header_result\b/,
    /\bfn\s+scanner_from_bytes\b/,
    /\bfn\s+scan_token_impl\b/,
]) {
    assert.doesNotMatch(
        facadeCode,
        pattern,
        'std/streamio.nepl facade must not keep scanner implementation bodies',
    );
}

for (const pattern of [
    /\bstruct\s+StreamScanner\b/,
    /\benum\s+StreamScannerHeaderField\b/,
    /\bfn\s+stream_scanner_load_header_result\b/,
    /\bfn\s+stream_scanner_byte_at\b/,
    /\bfn\s+scanner_from_bytes\b/,
]) {
    assert.doesNotMatch(
        code,
        pattern,
        'std/streamio/scanner.nepl must not keep scanner state implementation bodies',
    );
    assert.match(
        stateCode,
        pattern,
        'std/streamio/scanner/state.nepl must own scanner state implementation bodies',
    );
}

for (const pattern of [
    /\bfn\s+stream_scanner_is_leading_skip_byte\b/,
    /\bfn\s+stream_scanner_is_token_separator\b/,
    /\bfn\s+stream_scanner_is_ascii_digit\b/,
    /\bfn\s+stream_scanner_skip_ws_header\b/,
]) {
    assert.doesNotMatch(
        code,
        pattern,
        'std/streamio/scanner.nepl must not keep scanner cursor implementation bodies',
    );
    assert.match(
        cursorCode,
        pattern,
        'std/streamio/scanner/cursor.nepl must own scanner cursor implementation bodies',
    );
}

for (const pattern of [
    /\bfn\s+scan_i32_impl\b/,
    /\bfn\s+scan_u32_impl\b/,
    /\bfn\s+scan_u64_impl\b/,
    /\bfn\s+scan_i64_impl\b/,
]) {
    assert.doesNotMatch(
        code,
        pattern,
        'std/streamio/scanner.nepl must not keep scanner number parser implementation bodies',
    );
    assert.doesNotMatch(
        numberCode,
        pattern,
        'std/streamio/scanner/number.nepl facade must not keep integer parser implementation bodies',
    );
    assert.match(
        numberIntCode,
        pattern,
        'std/streamio/scanner/number/int.nepl must own integer parser implementation bodies',
    );
}

for (const pattern of [
    /\bfn\s+scan_f64_impl\b/,
    /\bfn\s+scan_f32_impl\b/,
]) {
    assert.doesNotMatch(
        code,
        pattern,
        'std/streamio/scanner.nepl must not keep scanner number parser implementation bodies',
    );
    assert.doesNotMatch(
        numberCode,
        pattern,
        'std/streamio/scanner/number.nepl facade must not keep float parser implementation bodies',
    );
    assert.match(
        numberFloatCode,
        pattern,
        'std/streamio/scanner/number/float.nepl must own float parser implementation bodies',
    );
}

assert.match(
    numberCode,
    /pub\s+#import\s+"\.\/number\/int"\s+as\s+@merge/,
    'std/streamio/scanner/number.nepl must re-export integer parser implementation',
);
assert.match(
    numberCode,
    /pub\s+#import\s+"\.\/number\/float"\s+as\s+@merge/,
    'std/streamio/scanner/number.nepl must re-export float parser implementation',
);

assert.match(
    stateCode,
    /\bfn\s+stream_scanner_byte_at\s+<\(MemPtr<u8>,i32,i32\)->i32>/,
    'StreamScanner byte access must go through stream_scanner_byte_at',
);

assert.doesNotMatch(
    stateCode,
    /\bimpl\s+(?:Copy|Clone)\s+for\s+StreamScanner\b/,
    'StreamScanner owns raw-backed header/buffer storage and must not be Copy or Clone',
);

for (const [fnName, relPath, owner] of [
    ['skip_ws', scannerRelPath, 'scanner root'],
    ['is_eof', scannerRelPath, 'scanner root'],
    ['skip', scannerRelPath, 'scanner root'],
    ['scan_token_impl', scannerRelPath, 'scanner root'],
    ['read', scannerRelPath, 'scanner root'],
    ['scan_i32_impl', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_u32_impl', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_u64_impl', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_i64_impl', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_f64_impl', scannerNumberFloatRelPath, 'scanner float parser'],
    ['scan_f32_impl', scannerNumberFloatRelPath, 'scanner float parser'],
]) {
    const fnCode = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    assert.match(
        fnCode,
        new RegExp(`\\bfn\\s+${fnName}\\s+<\\(\\&StreamScanner\\)\\*>`),
        `StreamScanner ${fnName} in ${owner} must borrow the owning handle instead of copying it`,
    );
}

assert.match(
    code,
    /\bfn\s+close\s+<\(StreamScanner\)\*>/,
    'StreamScanner close must remain the owner-consuming cleanup API',
);

assert.doesNotMatch(
    code,
    /\bfn\s+stream_scanner_header_ptr\b/,
    'StreamScanner must not reintroduce a RegionToken header pointer helper',
);
const scannerHeaderMatch = stateCode.match(
    /(?:pub\s+)?fn\s+stream_scanner_header_off\b([\s\S]*?)\n(?:pub\s+)?fn\s+scanner_from_bytes\b/,
);
assert.ok(scannerHeaderMatch, 'StreamScanner header helper section must be found');
assert.doesNotMatch(
    scannerHeaderMatch[1],
    /\bmatch\s+load_i32\s+p\b/,
    'StreamScanner header load must not read through an unproven RegionToken pointer',
);

for (const pattern of [
    /\bload_u8\s+add\s+buf\b/,
    /\bload_u8\s+buf\b/,
    /\bstore_u8\s+add\s+s\b/,
    /\bstring_from_addr_unchecked\s+s\b/,
]) {
    assert.doesNotMatch(
        scannerImplementationCode,
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
    const re = new RegExp(`(?:pub\\s+)?fn\\s+${fnName}\\b[\\s\\S]*?(?=\\n(?:pub\\s+)?fn\\s+|$)`);
    const match = scannerImplementationCode.match(re);
    assert.ok(match, `${fnName} body must be found`);
    assert.match(
        match[0],
        /\bstream_scanner_byte_at\b/,
        `${fnName} must use stream_scanner_byte_at for buffer reads`,
    );
}

assert.match(
    code,
    /\bstring_from_mem_unchecked_result\s+mem_ptr_add\s+buf\s+start\s+tlen\b/,
    'scan_token_impl must delegate token string construction to alloc/string',
);

console.log('stdlib streamio scanner boundary regression passed');
