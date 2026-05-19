#!/usr/bin/env node

const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { implementationLineCount } = require('./source_policy/stdlib_builder_owner');

const repoRoot = path.resolve(__dirname, '..');
const relPath = 'stdlib/std/streamio.nepl';
const writerRelPath = 'stdlib/std/streamio/writer.nepl';
const writerStateRelPath = 'stdlib/std/streamio/writer/state.nepl';
const writerAppendRelPath = 'stdlib/std/streamio/writer/append.nepl';
const writerAppendTextRelPath = 'stdlib/std/streamio/writer/append/text.nepl';
const writerAppendByteBufRelPath = 'stdlib/std/streamio/writer/append/bytebuf.nepl';
const writerAppendIntegerRelPath = 'stdlib/std/streamio/writer/append/integer.nepl';
const writerAppendFloatRelPath = 'stdlib/std/streamio/writer/append/float.nepl';
const inputRelPath = 'stdlib/std/streamio/input.nepl';
const outputRelPath = 'stdlib/std/streamio/output.nepl';
const outputTypesRelPath = 'stdlib/std/streamio/output/types.nepl';
const outputStdoutRelPath = 'stdlib/std/streamio/output/stdout.nepl';
const outputStderrRelPath = 'stdlib/std/streamio/output/stderr.nepl';
const bytesRelPath = 'stdlib/std/streamio/bytes.nepl';
const scannerRelPath = 'stdlib/std/streamio/scanner.nepl';
const scannerCursorRelPath = 'stdlib/std/streamio/scanner/cursor.nepl';
const scannerNumberRelPath = 'stdlib/std/streamio/scanner/number.nepl';
const scannerNumberIntRelPath = 'stdlib/std/streamio/scanner/number/int.nepl';
const scannerNumberFloatRelPath = 'stdlib/std/streamio/scanner/number/float.nepl';
const scannerStateRelPath = 'stdlib/std/streamio/scanner/state.nepl';
const src = fs.readFileSync(path.join(repoRoot, relPath), 'utf8');
const writerSrc = fs.readFileSync(path.join(repoRoot, writerRelPath), 'utf8');
const writerStateSrc = fs.readFileSync(path.join(repoRoot, writerStateRelPath), 'utf8');
const writerAppendSrc = fs.readFileSync(path.join(repoRoot, writerAppendRelPath), 'utf8');
const writerAppendTextSrc = fs.readFileSync(path.join(repoRoot, writerAppendTextRelPath), 'utf8');
const writerAppendByteBufSrc = fs.readFileSync(path.join(repoRoot, writerAppendByteBufRelPath), 'utf8');
const writerAppendIntegerSrc = fs.readFileSync(path.join(repoRoot, writerAppendIntegerRelPath), 'utf8');
const writerAppendFloatSrc = fs.readFileSync(path.join(repoRoot, writerAppendFloatRelPath), 'utf8');
const inputSrc = fs.readFileSync(path.join(repoRoot, inputRelPath), 'utf8');
const outputSrc = fs.readFileSync(path.join(repoRoot, outputRelPath), 'utf8');
const outputTypesSrc = fs.readFileSync(path.join(repoRoot, outputTypesRelPath), 'utf8');
const outputStdoutSrc = fs.readFileSync(path.join(repoRoot, outputStdoutRelPath), 'utf8');
const outputStderrSrc = fs.readFileSync(path.join(repoRoot, outputStderrRelPath), 'utf8');
const bytesSrc = fs.readFileSync(path.join(repoRoot, bytesRelPath), 'utf8');
const scannerSrc = fs.readFileSync(path.join(repoRoot, scannerRelPath), 'utf8');
const scannerCursorSrc = fs.readFileSync(path.join(repoRoot, scannerCursorRelPath), 'utf8');
const scannerNumberSrc = fs.readFileSync(path.join(repoRoot, scannerNumberRelPath), 'utf8');
const scannerNumberIntSrc = fs.readFileSync(path.join(repoRoot, scannerNumberIntRelPath), 'utf8');
const scannerNumberFloatSrc = fs.readFileSync(path.join(repoRoot, scannerNumberFloatRelPath), 'utf8');
const scannerStateSrc = fs.readFileSync(path.join(repoRoot, scannerStateRelPath), 'utf8');

const facadeCode = src
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writerCode = writerSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writerStateCode = writerStateSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writerAppendCode = writerAppendSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writerAppendTextCode = writerAppendTextSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writerAppendByteBufCode = writerAppendByteBufSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writerAppendIntegerCode = writerAppendIntegerSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const writerAppendFloatCode = writerAppendFloatSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const inputCode = inputSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const outputCode = outputSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const outputTypesCode = outputTypesSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const outputStdoutCode = outputStdoutSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const outputStderrCode = outputStderrSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const bytesCode = bytesSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerCode = scannerSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerStateCode = scannerStateSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerCursorCode = scannerCursorSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerNumberCode = scannerNumberSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerNumberIntCode = scannerNumberIntSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const scannerNumberFloatCode = scannerNumberFloatSrc
    .split(/\r?\n/)
    .filter((line) => !/^\s*\/\//.test(line))
    .join('\n');
const code = `${facadeCode}\n${inputCode}\n${outputCode}\n${outputTypesCode}\n${outputStdoutCode}\n${outputStderrCode}\n${writerCode}\n${writerStateCode}\n${writerAppendCode}\n${writerAppendTextCode}\n${writerAppendByteBufCode}\n${writerAppendIntegerCode}\n${writerAppendFloatCode}\n${bytesCode}\n${scannerCode}\n${scannerCursorCode}\n${scannerNumberCode}\n${scannerNumberIntCode}\n${scannerNumberFloatCode}\n${scannerStateCode}`;

for (const [modulePath, srcText, maxLines] of [
    [relPath, src, 90],
    [inputRelPath, inputSrc, 220],
    [outputRelPath, outputSrc, 80],
    [outputTypesRelPath, outputTypesSrc, 90],
    [outputStdoutRelPath, outputStdoutSrc, 180],
    [outputStderrRelPath, outputStderrSrc, 180],
    [writerAppendRelPath, writerAppendSrc, 80],
    [writerAppendTextRelPath, writerAppendTextSrc, 80],
    [writerAppendByteBufRelPath, writerAppendByteBufSrc, 110],
    [writerAppendIntegerRelPath, writerAppendIntegerSrc, 180],
    [writerAppendFloatRelPath, writerAppendFloatSrc, 130],
    [scannerNumberRelPath, scannerNumberSrc, 80],
    [scannerNumberIntRelPath, scannerNumberIntSrc, 240],
    [scannerNumberFloatRelPath, scannerNumberFloatSrc, 220],
]) {
    const lineCount = implementationLineCount(srcText);
    assert.ok(lineCount <= maxLines, `${modulePath} must stay within its responsibility boundary (${lineCount}/${maxLines})`);
}

assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/input"\s+as\s+\*/,
    `${relPath} must re-export the input stream module`,
);
assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/output"\s+as\s+\*/,
    `${relPath} must re-export the output stream module`,
);
assert.match(
    outputCode,
    /pub\s+#import\s+"std\/streamio\/output\/types"\s+as\s+\*/,
    `${outputRelPath} must re-export the output stream types module`,
);
assert.match(
    outputCode,
    /pub\s+#import\s+"std\/streamio\/output\/stdout"\s+as\s+\*/,
    `${outputRelPath} must re-export the stdout stream module`,
);
assert.match(
    outputCode,
    /pub\s+#import\s+"std\/streamio\/output\/stderr"\s+as\s+\*/,
    `${outputRelPath} must re-export the stderr stream module`,
);
assert.doesNotMatch(
    outputCode,
    /^\s*(struct|trait|impl|fn)\s/m,
    `${outputRelPath} must stay a facade without output implementation bodies`,
);
assert.match(
    outputTypesCode,
    /struct\s+StdoutStream:[\s\S]*impl\s+Copy\s+for\s+StdoutStream[\s\S]*struct\s+StderrStream:[\s\S]*impl\s+Copy\s+for\s+StderrStream/,
    `${outputTypesRelPath} must own lightweight output handle types`,
);
assert.doesNotMatch(
    outputTypesCode,
    /\b(?:stdio_write_bytes_result|stdio_write_stderr_bytes_result|io_write_str)\b/,
    `${outputTypesRelPath} must not own stdout/stderr write behavior`,
);
assert.match(
    outputStdoutCode,
    /impl\s+ByteWriter\s+for\s+StdoutStream:[\s\S]*trait\s+StdoutWritable:[\s\S]*fn\s+write\s+<\.T:\s+StdoutWritable>[\s\S]*fn\s+writeln\s+<\(StdoutStream,\s*str\)/,
    `${outputStdoutRelPath} must own StdoutStream writer behavior and convenience overloads`,
);
assert.doesNotMatch(
    outputStdoutCode,
    /\bStderrStream\b/,
    `${outputStdoutRelPath} must not own stderr behavior`,
);
assert.match(
    outputStderrCode,
    /impl\s+ByteWriter\s+for\s+StderrStream:[\s\S]*trait\s+StderrWritable:[\s\S]*fn\s+write\s+<\.T:\s+StderrWritable>[\s\S]*fn\s+writeln\s+<\(StderrStream,\s*str\)/,
    `${outputStderrRelPath} must own StderrStream writer behavior and convenience overloads`,
);
assert.doesNotMatch(
    outputStderrCode,
    /\bStdoutStream\b/,
    `${outputStderrRelPath} must not own stdout behavior`,
);
assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/writer"\s+as\s+\*/,
    `${relPath} must re-export the writer module`,
);
assert.match(
    writerCode,
    /#import\s+"std\/streamio\/writer\/state"\s+as\s+\*/,
    `${writerRelPath} must import the writer state module`,
);
assert.match(
    writerCode,
    /#import\s+"std\/streamio\/writer\/append"\s+as\s+\*/,
    `${writerRelPath} must import the writer append module`,
);
assert.match(
    writerCode,
    /\bfn\s+close\s+<\(StreamWriter\)\*>\(\)>\s+\(w\):\s*stream_writer_close_impl\s+w\b/,
    `${writerRelPath} must expose StreamWriter owner cleanup through the root close facade`,
);
assert.match(
    writerStateCode,
    /\bfn\s+stream_writer_close_impl\s+<\(StreamWriter\)\*>\(\)>/,
    `${writerStateRelPath} must keep the StreamWriter cleanup implementation helper`,
);
assert.doesNotMatch(
    writerStateCode,
    /\bfn\s+close\s+<\(StreamWriter\)\*>\(\)>/,
    `${writerStateRelPath} must not expose the public common-name close overload`,
);
assert.match(
    writerAppendCode,
    /pub\s+#import\s+"std\/streamio\/writer\/append\/text"\s+as\s+\*/,
    `${writerAppendRelPath} must re-export text append helpers`,
);
assert.match(
    writerAppendCode,
    /pub\s+#import\s+"std\/streamio\/writer\/append\/bytebuf"\s+as\s+\*/,
    `${writerAppendRelPath} must re-export ByteBuf append helpers`,
);
assert.match(
    writerAppendCode,
    /pub\s+#import\s+"std\/streamio\/writer\/append\/integer"\s+as\s+\*/,
    `${writerAppendRelPath} must re-export integer append helpers`,
);
assert.match(
    writerAppendCode,
    /pub\s+#import\s+"std\/streamio\/writer\/append\/float"\s+as\s+\*/,
    `${writerAppendRelPath} must re-export float append helpers`,
);
assert.doesNotMatch(
    writerAppendCode,
    /^\s*(struct|trait|impl|fn)\s/m,
    `${writerAppendRelPath} must stay a facade without append implementation bodies`,
);
assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/bytes"\s+as\s+\*/,
    `${relPath} must re-export the bytes module`,
);
assert.match(
    facadeCode,
    /pub\s+#import\s+"\.\/streamio\/scanner"\s+as\s+\*/,
    `${relPath} must re-export the scanner module`,
);

const forbidden = [
    /\bunwrap\b/,
    /\bunwrap_ok\b/,
    /\bunwrap_err\b/,
    /\buwok\b/,
    /\buwerr\b/,
    /#intrinsic\s+"unreachable"/,
];

for (const pattern of forbidden) {
    assert.doesNotMatch(code, pattern, `${relPath} must not use unsafe unwrap helpers in implementation code`);
}

assert.doesNotMatch(code, /trait\s+ScannerReadable/, 'StreamScanner read must use concrete overloads, not an unreachable generic trait default');
assert.doesNotMatch(scannerStateCode, /impl\s+(?:Copy|Clone)\s+for\s+StreamScanner/, 'StreamScanner owns scanner storage and must not be Copy or Clone');
assert.match(code, /fn\s+read\s+<\(&StreamScanner\)\*>str>\s+\(sc\):[\s\S]*scan_token_impl\s+sc/, 'StreamScanner must keep a borrowed str read overload');
assert.match(code, /fn\s+read\s+<\(&StreamScanner\)\*>i32>\s+\(sc\):[\s\S]*scan_i32_impl\s+sc/, 'StreamScanner must keep a borrowed i32 read overload');
assert.match(code, /fn\s+read\s+<\(&StreamScanner\)\*>f64>\s+\(sc\):[\s\S]*scan_f64_impl\s+sc/, 'StreamScanner must keep a borrowed f64 read overload');
assert.match(code, /fn\s+close\s+<\(StreamScanner\)\*>/, 'StreamScanner close must remain owner-consuming');

assert.match(
    code,
    /fn\s+stream_scanner_load_pos_result\s+<\(&StreamScanner\)->Result<i32,str>>\s+\(sc\):[\s\S]*let\s+cursor\s+<&Vec<i32>>\s+get_ref\s+sc\s+"cursor"[\s\S]*match\s+vec::get<i32>\s+cursor\s+0:[\s\S]*Option::Some\s+pos:[\s\S]*Result<i32,str>::Ok\s+pos[\s\S]*Result<i32,str>::Err\s+"streamio\.stream_scanner_load_pos failed"/,
    'stream scanner cursor loads must return Result through typed cursor storage instead of trapping',
);

assert.match(
    code,
    /fn\s+stream_scanner_store_pos_result\s+<\(&StreamScanner,i32\)\*>Result<\(\),str>>\s+\(sc,\s*pos\):[\s\S]*let\s+cursor\s+<&Vec<i32>>\s+get_ref\s+sc\s+"cursor"[\s\S]*match\s+vec::get<i32>\s+cursor\s+0:[\s\S]*Option::Some\s+_old:[\s\S]*vec::replace<i32>\s+cursor\s+0\s+pos[\s\S]*Result<\(\),str>::Ok\s+\(\)[\s\S]*Result<\(\),str>::Err\s+"streamio\.stream_scanner_store_pos failed"/,
    'stream scanner cursor stores must return Result through typed cursor storage instead of trapping',
);

assert.match(
    code,
    /fn\s+scanner_from_bytes\s+<\(ByteBuf\)\*>Result<StreamScanner,str>>\s+\(bytes\):[\s\S]*match\s+io_bytebuf_ptr_ref\s+&bytes:[\s\S]*Option::None:[\s\S]*eq\s+len\s+0[\s\S]*stream_scanner_cursor_new[\s\S]*Result<StreamScanner,str>::Ok\s+StreamScanner\s+bytes\s+cursor[\s\S]*Option::Some\s+_buf:[\s\S]*stream_scanner_cursor_new/,
    'scanner_from_bytes must keep the ByteBuf owner in StreamScanner and allocate only cursor storage separately',
);

assert.match(
    code,
    /fn\s+push_u8_impl\s+<\(StreamWriter,i32\)\*>StreamWriter>\s+\(w,\s*b\):[\s\S]*let\s+w1\s+<StreamWriter>\s+reserve_impl\s+w\s+1[\s\S]*let\s+builder\s+<ByteBuilder>\s+get\s+w1\s+"builder"[\s\S]*match\s+byte_builder_push_u8\s+builder\s+b:[\s\S]*Result::Ok\s+next_builder:[\s\S]*StreamWriter\s+next_builder\s+target\s+@stream_writer_noncopy_marker[\s\S]*Result::Err\s+e:[\s\S]*byte_builder_error_free\s+e[\s\S]*StreamWriter\s+byte_builder_empty\s+target\s+@stream_writer_noncopy_marker/,
    'push_u8_impl must delegate byte storage and length advance to ByteBuilder after reserve_impl',
);

assert.doesNotMatch(
    code,
    /store_u8\s+mem_ptr_add\s+\*get_ref\s+&w1\s+"buf"/,
    'push_u8_impl must not directly store through a StreamWriter MemPtr field',
);

assert.match(
    code,
    /fn\s+append_str_impl\s+<\(StreamWriter,str\)\*>StreamWriter>\s+\(w,\s*s\):[\s\S]*while\s+lt\s+i\s+n:[\s\S]*match\s+checked_string_byte_at\s+s\s+i:[\s\S]*Option::Some\s+ch:[\s\S]*set\s+ww\s+push_u8_impl\s+ww\s+ch/,
    'append_str_impl must stream through the alloc/string byte boundary and push_u8_impl',
);

assert.match(
    code,
    /fn\s+append_bytebuf_impl\s+<\(StreamWriter,ByteBuf\)\*>StreamWriter>\s+\(w,\s*bytes\):[\s\S]*while\s+and\s+eq\s+done\s+0\s+lt\s+i\s+n:[\s\S]*match\s+stream_writer_bytebuf_byte_at\s+&bytes\s+i:[\s\S]*Option::Some\s+ch:[\s\S]*set\s+ww\s+push_u8_impl\s+ww\s+ch[\s\S]*io_bytebuf_free\s+bytes/,
    'append_bytebuf_impl must stream through the borrowed ByteBuf byte boundary, preserve ownership cleanup, and avoid unsafe unwrap',
);

assert.doesNotMatch(code, /store_u8\s+mem_ptr_add\s+buf\s+off/, 'numeric writer helpers must not bypass push_u8_impl with direct buffer stores');
assert.match(code, /fn\s+append_u64_digits_impl\s+<\(StreamWriter,i64\)\*>StreamWriter>[\s\S]*push_u8_impl/, 'numeric writer helpers must funnel digits through push_u8_impl');

console.log('stdlib streamio unsafe unwrap regression passed');
