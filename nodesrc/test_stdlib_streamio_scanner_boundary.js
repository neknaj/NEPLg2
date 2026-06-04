#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const {
    stripNeplComments,
    fnSignaturePattern,
    structFieldPattern,
} = require('./source_policy/nepl_source_view');

const repoRoot = path.resolve(__dirname, '..');
const facadeRelPath = 'stdlib/std/streamio.nepl';
const scannerRelPath = 'stdlib/std/streamio/scanner.nepl';
const scannerCursorRelPath = 'stdlib/std/streamio/scanner/cursor.nepl';
const scannerErrorRelPath = 'stdlib/std/streamio/scanner/error.nepl';
const scannerNumberRelPath = 'stdlib/std/streamio/scanner/number.nepl';
const scannerNumberIntRelPath = 'stdlib/std/streamio/scanner/number/int.nepl';
const scannerNumberFloatRelPath = 'stdlib/std/streamio/scanner/number/float.nepl';
const scannerStateRelPath = 'stdlib/std/streamio/scanner/state.nepl';
const facade = fs.readFileSync(path.join(repoRoot, facadeRelPath), 'utf8');
const src = fs.readFileSync(path.join(repoRoot, scannerRelPath), 'utf8');
const cursorSrc = fs.readFileSync(path.join(repoRoot, scannerCursorRelPath), 'utf8');
const errorSrc = fs.readFileSync(path.join(repoRoot, scannerErrorRelPath), 'utf8');
const numberSrc = fs.readFileSync(path.join(repoRoot, scannerNumberRelPath), 'utf8');
const numberIntSrc = fs.readFileSync(path.join(repoRoot, scannerNumberIntRelPath), 'utf8');
const numberFloatSrc = fs.readFileSync(path.join(repoRoot, scannerNumberFloatRelPath), 'utf8');
const stateSrc = fs.readFileSync(path.join(repoRoot, scannerStateRelPath), 'utf8');

const code = stripNeplComments(src);
const stateCode = stripNeplComments(stateSrc);
const cursorCode = stripNeplComments(cursorSrc);
const errorCode = stripNeplComments(errorSrc);
const numberCode = stripNeplComments(numberSrc);
const numberIntCode = stripNeplComments(numberIntSrc);
const numberFloatCode = stripNeplComments(numberFloatSrc);
const facadeCode = stripNeplComments(facade);
const scannerImplementationCode = `${cursorCode}\n${errorCode}\n${code}\n${numberCode}\n${numberIntCode}\n${numberFloatCode}`;

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
    /\bfn\s+stream_scanner_load_pos_result\b/,
    /\bfn\s+stream_scanner_slice_to_str_result\b/,
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
    /\bfn\s+stream_scanner_load_pos_result\b/,
    /\bfn\s+stream_scanner_byte_at\b/,
    /\bfn\s+stream_scanner_slice_to_str_result\b/,
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
    /\bfn\s+stream_scanner_skip_ws_result\b/,
    /\bfn\s+stream_scanner_skip_ws_state\b/,
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
    new RegExp(`struct\\s+StreamScanner:[\\s\\S]*${structFieldPattern('bytes', 'ByteBuf')}[\\s\\S]*${structFieldPattern('cursor', 'Vec i32')}`),
    'StreamScanner state must expose the input ByteBuf owner and typed cursor storage as fields',
);

assert.doesNotMatch(
    stateCode,
    /^\s+header\s+%MemPtr\s+u8/m,
    'StreamScanner must not keep a direct raw MemPtr header owner field',
);

assert.match(
    stateCode,
    new RegExp(fnSignaturePattern('stream_scanner_byte_at', ['&ByteBuf', 'i32'], 'i32')),
    'StreamScanner byte access must go through a borrowed ByteBuf boundary',
);

assert.doesNotMatch(
    stateCode,
    /\bimpl\s+(?:Copy|Clone)\s+for\s+StreamScanner\b/,
    'StreamScanner owns raw-backed header/buffer storage and must not be Copy or Clone',
);

assert.match(
    stateCode,
    /\bstring_from_utf8_mem_result\s+mem_ptr_add\s+ptr\s+start\s+tlen\b/,
    'StreamScanner token slice construction must validate UTF-8 before constructing str',
);
assert.doesNotMatch(
    stateCode,
    /\bstring_from_mem_unchecked_result\s+mem_ptr_add\s+ptr\s+start\s+tlen\b/,
    'StreamScanner token slice construction must not use unchecked string construction for external bytes',
);

for (const [fnName, relPath, owner] of [
    ['skip_ws', scannerRelPath, 'scanner root'],
    ['skip_ws_result', scannerRelPath, 'scanner root'],
    ['is_eof', scannerRelPath, 'scanner root'],
    ['is_eof_result', scannerRelPath, 'scanner root'],
    ['skip', scannerRelPath, 'scanner root'],
    ['skip_result', scannerRelPath, 'scanner root'],
    ['scan_token_result', scannerRelPath, 'scanner root'],
    ['scan_token_impl', scannerRelPath, 'scanner root'],
    ['read', scannerRelPath, 'scanner root'],
    ['read_result', scannerRelPath, 'scanner root'],
    ['scan_i32_result', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_i32_impl', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_u32_result', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_u32_impl', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_u64_result', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_u64_impl', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_i64_result', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_i64_impl', scannerNumberIntRelPath, 'scanner integer parser'],
    ['scan_f64_result', scannerNumberFloatRelPath, 'scanner float parser'],
    ['scan_f64_impl', scannerNumberFloatRelPath, 'scanner float parser'],
    ['scan_f32_result', scannerNumberFloatRelPath, 'scanner float parser'],
    ['scan_f32_impl', scannerNumberFloatRelPath, 'scanner float parser'],
]) {
    const fnCode = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
    assert.match(
        fnCode,
        new RegExp(`\\bfn\\s+${fnName}\\s+%impure\\s+fn\\s+&StreamScanner\\b`),
        `StreamScanner ${fnName} in ${owner} must borrow the owning handle instead of copying it`,
    );
}

assert.match(
    code,
    new RegExp(fnSignaturePattern('close', ['StreamScanner'], 'unit', { effect: 'impure' })),
    'StreamScanner close must remain the owner-consuming cleanup API',
);

assert.doesNotMatch(
    code,
    /\bfn\s+stream_scanner_header_ptr\b/,
    'StreamScanner must not reintroduce a RegionToken header pointer helper',
);
const scannerHeaderMatch = stateCode.match(
    /(?:pub\s+)?fn\s+stream_scanner_load_pos_result\b([\s\S]*?)\n(?:pub\s+)?fn\s+scanner_from_bytes\b/,
);
assert.ok(scannerHeaderMatch, 'StreamScanner cursor helper section must be found');
assert.doesNotMatch(
    scannerHeaderMatch[1],
    /\b(?:load_i32|store_i32)\b/,
    'StreamScanner cursor state must not use raw i32 memory load/store',
);
assert.match(
    scannerHeaderMatch[1],
    /\bvec::get\s+cursor\s+0[\s\S]*\bvec::replace\s+cursor\s+0\s+pos/,
    'StreamScanner cursor stores must use typed Vec cursor storage instead of a raw header owner field',
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
    'stream_scanner_skip_ws_result',
    'skip_result',
    'scan_token_result',
    'scan_i32_result',
    'scan_u32_result',
    'scan_u64_result',
    'scan_i64_result',
    'scan_f64_result',
]) {
    const re = new RegExp(`(?:pub\\s+)?fn\\s+${fnName}\\b[\\s\\S]*?(?=\\n(?:pub\\s+)?fn\\s+|$)`);
    const match = scannerImplementationCode.match(re);
    assert.ok(match, `${fnName} body must be found`);
    assert.match(
        match[0],
        /\bstream_scanner_byte_at_result\b/,
        `${fnName} must use stream_scanner_byte_at_result for typed buffer reads`,
    );
}

assert.match(
    code,
    /\bstream_scanner_slice_to_str_result\s+sc\s+start\s+tlen\b/,
    'scan_token_result must delegate token string construction to the scanner state ByteBuf slice boundary',
);

console.log('stdlib streamio scanner boundary regression passed');
