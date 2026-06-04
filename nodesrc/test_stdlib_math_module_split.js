#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { legacyTypeSyntaxView } = require("./source_policy/nepl_source_view");

const repoRoot = path.resolve(__dirname, "..");

function read(rel) {
    return legacyTypeSyntaxView(fs.readFileSync(path.join(repoRoot, rel), "utf8"));
}

const facade = read("stdlib/core/math.nepl");
const i32Module = read("stdlib/core/math/i32.nepl");
const i32ArithModule = read("stdlib/core/math/i32/arith.nepl");
const i32BitwiseModule = read("stdlib/core/math/i32/bitwise.nepl");
const i32BitwiseBinaryModule = read("stdlib/core/math/i32/bitwise/binary.nepl");
const i32BitwiseShiftModule = read("stdlib/core/math/i32/bitwise/shift.nepl");
const i32BitwiseCountModule = read("stdlib/core/math/i32/bitwise/count.nepl");
const i32CompareModule = read("stdlib/core/math/i32/compare.nepl");
const i64Module = read("stdlib/core/math/i64.nepl");
const i64ArithModule = read("stdlib/core/math/i64/arith.nepl");
const i64BitwiseModule = read("stdlib/core/math/i64/bitwise.nepl");
const i64BitwiseBinaryModule = read("stdlib/core/math/i64/bitwise/binary.nepl");
const i64BitwiseShiftModule = read("stdlib/core/math/i64/bitwise/shift.nepl");
const i64BitwiseCountModule = read("stdlib/core/math/i64/bitwise/count.nepl");
const i64CompareModule = read("stdlib/core/math/i64/compare.nepl");
const f32Module = read("stdlib/core/math/f32.nepl");
const f32BinaryModule = read("stdlib/core/math/f32/binary.nepl");
const f32UnaryModule = read("stdlib/core/math/f32/unary.nepl");
const f32CompareModule = read("stdlib/core/math/f32/compare.nepl");
const f64Module = read("stdlib/core/math/f64.nepl");
const f64BinaryModule = read("stdlib/core/math/f64/binary.nepl");
const f64UnaryModule = read("stdlib/core/math/f64/unary.nepl");
const f64CompareModule = read("stdlib/core/math/f64/compare.nepl");
const convertModule = read("stdlib/core/math/convert.nepl");
const convertWidthModule = read("stdlib/core/math/convert/width.nepl");
const convertFloatModule = read("stdlib/core/math/convert/float.nepl");
const convertFloatIntToFloatModule = read("stdlib/core/math/convert/float/int_to_float.nepl");
const convertFloatToI32Module = read("stdlib/core/math/convert/float/float_to_i32.nepl");
const convertFloatToI64Module = read("stdlib/core/math/convert/float/float_to_i64.nepl");
const convertFloatWidthModule = read("stdlib/core/math/convert/float/float_width.nepl");
const convertReinterpretModule = read("stdlib/core/math/convert/reinterpret.nepl");
const int128Module = read("stdlib/core/math/int128.nepl");
const int128TypesModule = read("stdlib/core/math/int128/types.nepl");
const int128U128Module = read("stdlib/core/math/int128/u128.nepl");
const int128I128Module = read("stdlib/core/math/int128/i128.nepl");
const u8Module = read("stdlib/core/math/u8.nepl");
const u8ArithModule = read("stdlib/core/math/u8/arith.nepl");
const u8CompareModule = read("stdlib/core/math/u8/compare.nepl");
const boolModule = read("stdlib/core/math/bool.nepl");

assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/i32"\s+as\s+@merge/,
    "core/math.nepl must re-export the i32 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/i64"\s+as\s+@merge/,
    "core/math.nepl must re-export the i64 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/f32"\s+as\s+@merge/,
    "core/math.nepl must re-export the f32 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/f64"\s+as\s+@merge/,
    "core/math.nepl must re-export the f64 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/convert"\s+as\s+@merge/,
    "core/math.nepl must re-export the conversion math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/u8"\s+as\s+@merge/,
    "core/math.nepl must re-export the u8 math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/bool"\s+as\s+@merge/,
    "core/math.nepl must re-export the bool math submodule",
);
assert.match(
    facade,
    /pub\s+#import\s+"\.\/math\/int128"\s+as\s+@merge/,
    "core/math.nepl must re-export the 128-bit integer math submodule",
);
for (const [moduleName, pattern] of [
    ["width", /pub\s+#import\s+"\.\/convert\/width"\s+as\s+@merge/],
    ["float", /pub\s+#import\s+"\.\/convert\/float"\s+as\s+@merge/],
    ["reinterpret", /pub\s+#import\s+"\.\/convert\/reinterpret"\s+as\s+@merge/],
]) {
    assert.match(convertModule, pattern, `core/math/convert.nepl must re-export the ${moduleName} conversion submodule`);
}
for (const [moduleName, pattern] of [
    ["int_to_float", /pub\s+#import\s+"\.\/float\/int_to_float"\s+as\s+@merge/],
    ["float_to_i32", /pub\s+#import\s+"\.\/float\/float_to_i32"\s+as\s+@merge/],
    ["float_to_i64", /pub\s+#import\s+"\.\/float\/float_to_i64"\s+as\s+@merge/],
    ["float_width", /pub\s+#import\s+"\.\/float\/float_width"\s+as\s+@merge/],
]) {
    assert.match(convertFloatModule, pattern, `core/math/convert/float.nepl must re-export the ${moduleName} conversion submodule`);
}
assert.doesNotMatch(convertFloatModule, /^fn\s+/m, "core/math/convert/float.nepl must remain a facade without function bodies");
for (const [moduleName, pattern] of [
    ["types", /pub\s+#import\s+"\.\/int128\/types"\s+as\s+@merge/],
    ["u128", /pub\s+#import\s+"\.\/int128\/u128"\s+as\s+@merge/],
    ["i128", /pub\s+#import\s+"\.\/int128\/i128"\s+as\s+@merge/],
]) {
    assert.match(int128Module, pattern, `core/math/int128.nepl must re-export the ${moduleName} int128 submodule`);
}
assert.doesNotMatch(int128Module, /^fn\s+/m, "core/math/int128.nepl must remain a facade without function bodies");
assert.doesNotMatch(int128Module, /^struct\s+/m, "core/math/int128.nepl must remain a facade without structs");
for (const [moduleName, moduleSrc, pattern] of [
    ["u128 field", int128U128Module, /#import\s+"core\/field"\s+as\s+\*/],
    ["u128 i64", int128U128Module, /#import\s+"core\/math\/i64"\s+as\s+\*/],
    ["u128 convert width", int128U128Module, /#import\s+"core\/math\/convert\/width"\s+as\s+\*/],
    ["i128 field", int128I128Module, /#import\s+"core\/field"\s+as\s+\*/],
    ["i128 i64", int128I128Module, /#import\s+"core\/math\/i64"\s+as\s+\*/],
    ["i128 convert width", int128I128Module, /#import\s+"core\/math\/convert\/width"\s+as\s+\*/],
    ["i128 u128 alias", int128I128Module, /#import\s+"\.\/u128"\s+as\s+u128_math/],
]) {
    assert.match(moduleSrc, pattern, `core/math/int128 submodule must import ${moduleName} directly`);
}

for (const [moduleName, pattern] of [
    ["arith", /pub\s+#import\s+"\.\/i32\/arith"\s+as\s+@merge/],
    ["bitwise", /pub\s+#import\s+"\.\/i32\/bitwise"\s+as\s+@merge/],
    ["compare", /pub\s+#import\s+"\.\/i32\/compare"\s+as\s+@merge/],
]) {
    assert.match(i32Module, pattern, `core/math/i32.nepl must re-export the ${moduleName} i32 submodule`);
}
assert.doesNotMatch(i32Module, /^fn\s+/m, "core/math/i32.nepl must remain a facade without function bodies");
for (const [moduleName, pattern] of [
    ["binary", /pub\s+#import\s+"\.\/bitwise\/binary"\s+as\s+@merge/],
    ["shift", /pub\s+#import\s+"\.\/bitwise\/shift"\s+as\s+@merge/],
    ["count", /pub\s+#import\s+"\.\/bitwise\/count"\s+as\s+@merge/],
]) {
    assert.match(i32BitwiseModule, pattern, `core/math/i32/bitwise.nepl must re-export the ${moduleName} i32 bitwise submodule`);
}
assert.doesNotMatch(i32BitwiseModule, /^fn\s+/m, "core/math/i32/bitwise.nepl must remain a facade without function bodies");
for (const [moduleName, pattern] of [
    ["arith", /pub\s+#import\s+"\.\/i64\/arith"\s+as\s+@merge/],
    ["bitwise", /pub\s+#import\s+"\.\/i64\/bitwise"\s+as\s+@merge/],
    ["compare", /pub\s+#import\s+"\.\/i64\/compare"\s+as\s+@merge/],
]) {
    assert.match(i64Module, pattern, `core/math/i64.nepl must re-export the ${moduleName} i64 submodule`);
}
assert.doesNotMatch(i64Module, /^fn\s+/m, "core/math/i64.nepl must remain a facade without function bodies");
for (const [moduleName, pattern] of [
    ["binary", /pub\s+#import\s+"\.\/bitwise\/binary"\s+as\s+@merge/],
    ["shift", /pub\s+#import\s+"\.\/bitwise\/shift"\s+as\s+@merge/],
    ["count", /pub\s+#import\s+"\.\/bitwise\/count"\s+as\s+@merge/],
]) {
    assert.match(i64BitwiseModule, pattern, `core/math/i64/bitwise.nepl must re-export the ${moduleName} i64 bitwise submodule`);
}
assert.doesNotMatch(i64BitwiseModule, /^fn\s+/m, "core/math/i64/bitwise.nepl must remain a facade without function bodies");
for (const [moduleName, pattern] of [
    ["binary", /pub\s+#import\s+"\.\/f32\/binary"\s+as\s+@merge/],
    ["unary", /pub\s+#import\s+"\.\/f32\/unary"\s+as\s+@merge/],
    ["compare", /pub\s+#import\s+"\.\/f32\/compare"\s+as\s+@merge/],
]) {
    assert.match(f32Module, pattern, `core/math/f32.nepl must re-export the ${moduleName} f32 submodule`);
}
assert.doesNotMatch(f32Module, /^fn\s+/m, "core/math/f32.nepl must remain a facade without function bodies");
for (const [moduleName, pattern] of [
    ["binary", /pub\s+#import\s+"\.\/f64\/binary"\s+as\s+@merge/],
    ["unary", /pub\s+#import\s+"\.\/f64\/unary"\s+as\s+@merge/],
    ["compare", /pub\s+#import\s+"\.\/f64\/compare"\s+as\s+@merge/],
]) {
    assert.match(f64Module, pattern, `core/math/f64.nepl must re-export the ${moduleName} f64 submodule`);
}
assert.doesNotMatch(f64Module, /^fn\s+/m, "core/math/f64.nepl must remain a facade without function bodies");

for (const [moduleName, pattern] of [
    ["arith", /pub\s+#import\s+"\.\/u8\/arith"\s+as\s+@merge/],
    ["compare", /pub\s+#import\s+"\.\/u8\/compare"\s+as\s+@merge/],
]) {
    assert.match(u8Module, pattern, `core/math/u8.nepl must re-export the ${moduleName} u8 submodule`);
}
assert.doesNotMatch(u8Module, /^fn\s+/m, "core/math/u8.nepl must remain a facade without function bodies");

for (const fnName of [
    "add_u8",
    "sub_u8",
    "mul_u8",
    "div_u_u8",
    "rem_u_u8",
]) {
    assert.match(u8ArithModule, new RegExp(`\\bfn\\s+${fnName}\\b`), `core/math/u8/arith.nepl must define ${fnName}`);
    assert.doesNotMatch(u8Module, new RegExp(`\\bfn\\s+${fnName}\\b`), `core/math/u8.nepl must not keep ${fnName}`);
    assert.doesNotMatch(facade, new RegExp(`\\bfn\\s+${fnName}\\b`), `core/math.nepl must not keep ${fnName}`);
}

for (const fnName of [
    "eq_u8",
    "ne_u8",
    "lt_u_u8",
    "le_u_u8",
    "gt_u_u8",
    "ge_u_u8",
]) {
    assert.match(u8CompareModule, new RegExp(`\\bfn\\s+${fnName}\\b`), `core/math/u8/compare.nepl must define ${fnName}`);
    assert.doesNotMatch(u8Module, new RegExp(`\\bfn\\s+${fnName}\\b`), `core/math/u8.nepl must not keep ${fnName}`);
    assert.doesNotMatch(facade, new RegExp(`\\bfn\\s+${fnName}\\b`), `core/math.nepl must not keep ${fnName}`);
}

for (const [name, signature] of [
    ["add", "<\\(u8,u8\\)->u8>"],
    ["sub", "<\\(u8,u8\\)->u8>"],
    ["mul", "<\\(u8,u8\\)->u8>"],
    ["div_u", "<\\(u8,u8\\)->u8>"],
    ["rem_u", "<\\(u8,u8\\)->u8>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(u8ArithModule, pattern, `core/math/u8/arith.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(u8Module, pattern, `core/math/u8.nepl must not keep arithmetic overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["eq", "<\\(u8,u8\\)->bool>"],
    ["ne", "<\\(u8,u8\\)->bool>"],
    ["lt_u", "<\\(u8,u8\\)->bool>"],
    ["le_u", "<\\(u8,u8\\)->bool>"],
    ["gt_u", "<\\(u8,u8\\)->bool>"],
    ["ge_u", "<\\(u8,u8\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(u8CompareModule, pattern, `core/math/u8/compare.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(u8Module, pattern, `core/math/u8.nepl must not keep compare overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["add", "<\\(i64,i64\\)->i64>"],
    ["sub", "<\\(i64,i64\\)->i64>"],
    ["mul", "<\\(i64,i64\\)->i64>"],
    ["div_s", "<\\(i64,i64\\)->i64>"],
    ["div_u", "<\\(i64,i64\\)->i64>"],
    ["rem_s", "<\\(i64,i64\\)->i64>"],
    ["rem_u", "<\\(i64,i64\\)->i64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i64ArithModule, pattern, `core/math/i64/arith.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(i64Module, pattern, `core/math/i64.nepl must not keep arithmetic overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["and", "<\\(i64,i64\\)->i64>"],
    ["or", "<\\(i64,i64\\)->i64>"],
    ["xor", "<\\(i64,i64\\)->i64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i64BitwiseBinaryModule, pattern, `core/math/i64/bitwise/binary.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(i64BitwiseModule, pattern, `core/math/i64/bitwise.nepl must not keep binary overload ${name} ${signature}`);
    assert.doesNotMatch(i64Module, pattern, `core/math/i64.nepl must not keep bitwise overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["shl", "<\\(i64,i64\\)->i64>"],
    ["shr_s", "<\\(i64,i64\\)->i64>"],
    ["shr_u", "<\\(i64,i64\\)->i64>"],
    ["rotl", "<\\(i64,i64\\)->i64>"],
    ["rotr", "<\\(i64,i64\\)->i64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i64BitwiseShiftModule, pattern, `core/math/i64/bitwise/shift.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(i64BitwiseModule, pattern, `core/math/i64/bitwise.nepl must not keep shift overload ${name} ${signature}`);
    assert.doesNotMatch(i64Module, pattern, `core/math/i64.nepl must not keep bitwise overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["eq", "<\\(i64,i64\\)->bool>"],
    ["ne", "<\\(i64,i64\\)->bool>"],
    ["lt", "<\\(i64,i64\\)->bool>"],
    ["lt_u", "<\\(i64,i64\\)->bool>"],
    ["le", "<\\(i64,i64\\)->bool>"],
    ["le_u", "<\\(i64,i64\\)->bool>"],
    ["gt", "<\\(i64,i64\\)->bool>"],
    ["gt_u", "<\\(i64,i64\\)->bool>"],
    ["ge", "<\\(i64,i64\\)->bool>"],
    ["ge_u", "<\\(i64,i64\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i64CompareModule, pattern, `core/math/i64/compare.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(i64Module, pattern, `core/math/i64.nepl must not keep compare overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["clz", "<\\(i64\\)->i64>"],
    ["ctz", "<\\(i64\\)->i64>"],
    ["popcnt", "<\\(i64\\)->i64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i64BitwiseCountModule, pattern, `core/math/i64/bitwise/count.nepl must define unary ${name} ${signature}`);
    assert.doesNotMatch(i64BitwiseModule, pattern, `core/math/i64/bitwise.nepl must not keep bit count unary ${name} ${signature}`);
    assert.doesNotMatch(i64Module, pattern, `core/math/i64.nepl must not keep bitwise unary ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep unary ${name} ${signature}`);
}

for (const [name, signature] of [
    ["add", "<\\(f32,f32\\)->f32>"],
    ["sub", "<\\(f32,f32\\)->f32>"],
    ["mul", "<\\(f32,f32\\)->f32>"],
    ["div", "<\\(f32,f32\\)->f32>"],
    ["min", "<\\(f32,f32\\)->f32>"],
    ["max", "<\\(f32,f32\\)->f32>"],
    ["copysign", "<\\(f32,f32\\)->f32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(f32BinaryModule, pattern, `core/math/f32/binary.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(f32Module, pattern, `core/math/f32.nepl must not keep binary overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["eq", "<\\(f32,f32\\)->bool>"],
    ["ne", "<\\(f32,f32\\)->bool>"],
    ["lt", "<\\(f32,f32\\)->bool>"],
    ["le", "<\\(f32,f32\\)->bool>"],
    ["gt", "<\\(f32,f32\\)->bool>"],
    ["ge", "<\\(f32,f32\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(f32CompareModule, pattern, `core/math/f32/compare.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(f32Module, pattern, `core/math/f32.nepl must not keep compare overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["sqrt", "<\\(f32\\)->f32>"],
    ["abs", "<\\(f32\\)->f32>"],
    ["neg", "<\\(f32\\)->f32>"],
    ["ceil", "<\\(f32\\)->f32>"],
    ["floor", "<\\(f32\\)->f32>"],
    ["trunc", "<\\(f32\\)->f32>"],
    ["nearest", "<\\(f32\\)->f32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(f32UnaryModule, pattern, `core/math/f32/unary.nepl must define unary ${name} ${signature}`);
    assert.doesNotMatch(f32Module, pattern, `core/math/f32.nepl must not keep unary ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep unary ${name} ${signature}`);
}

for (const [name, signature] of [
    ["add", "<\\(f64,f64\\)->f64>"],
    ["sub", "<\\(f64,f64\\)->f64>"],
    ["mul", "<\\(f64,f64\\)->f64>"],
    ["div", "<\\(f64,f64\\)->f64>"],
    ["min", "<\\(f64,f64\\)->f64>"],
    ["max", "<\\(f64,f64\\)->f64>"],
    ["copysign", "<\\(f64,f64\\)->f64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(f64BinaryModule, pattern, `core/math/f64/binary.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(f64Module, pattern, `core/math/f64.nepl must not keep binary overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["eq", "<\\(f64,f64\\)->bool>"],
    ["ne", "<\\(f64,f64\\)->bool>"],
    ["lt", "<\\(f64,f64\\)->bool>"],
    ["le", "<\\(f64,f64\\)->bool>"],
    ["gt", "<\\(f64,f64\\)->bool>"],
    ["ge", "<\\(f64,f64\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(f64CompareModule, pattern, `core/math/f64/compare.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(f64Module, pattern, `core/math/f64.nepl must not keep compare overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["sqrt", "<\\(f64\\)->f64>"],
    ["abs", "<\\(f64\\)->f64>"],
    ["neg", "<\\(f64\\)->f64>"],
    ["ceil", "<\\(f64\\)->f64>"],
    ["floor", "<\\(f64\\)->f64>"],
    ["trunc", "<\\(f64\\)->f64>"],
    ["nearest", "<\\(f64\\)->f64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(f64UnaryModule, pattern, `core/math/f64/unary.nepl must define unary ${name} ${signature}`);
    assert.doesNotMatch(f64Module, pattern, `core/math/f64.nepl must not keep unary ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep unary ${name} ${signature}`);
}

for (const [name, signature] of [
    ["and", "<\\(bool,bool\\)->bool>"],
    ["or", "<\\(bool,bool\\)->bool>"],
    ["not", "<\\(bool\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(boolModule, pattern, `core/math/bool.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["add", "<\\(i32,i32\\)->i32>"],
    ["sub", "<\\(i32,i32\\)->i32>"],
    ["mul", "<\\(i32,i32\\)->i32>"],
    ["div_s", "<\\(i32,i32\\)->i32>"],
    ["div_u", "<\\(i32,i32\\)->i32>"],
    ["rem_s", "<\\(i32,i32\\)->i32>"],
    ["mod_s", "<\\(i32,i32\\)->i32>"],
    ["rem_u", "<\\(i32,i32\\)->i32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i32ArithModule, pattern, `core/math/i32/arith.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(i32Module, pattern, `core/math/i32.nepl must not keep arithmetic overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["and", "<\\(i32,i32\\)->i32>"],
    ["or", "<\\(i32,i32\\)->i32>"],
    ["xor", "<\\(i32,i32\\)->i32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i32BitwiseBinaryModule, pattern, `core/math/i32/bitwise/binary.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(i32BitwiseModule, pattern, `core/math/i32/bitwise.nepl must not keep binary overload ${name} ${signature}`);
    assert.doesNotMatch(i32Module, pattern, `core/math/i32.nepl must not keep bitwise overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["shl", "<\\(i32,i32\\)->i32>"],
    ["shr_s", "<\\(i32,i32\\)->i32>"],
    ["shr_u", "<\\(i32,i32\\)->i32>"],
    ["rotl", "<\\(i32,i32\\)->i32>"],
    ["rotr", "<\\(i32,i32\\)->i32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i32BitwiseShiftModule, pattern, `core/math/i32/bitwise/shift.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(i32BitwiseModule, pattern, `core/math/i32/bitwise.nepl must not keep shift overload ${name} ${signature}`);
    assert.doesNotMatch(i32Module, pattern, `core/math/i32.nepl must not keep bitwise overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["eq", "<\\(i32,i32\\)->bool>"],
    ["ne", "<\\(i32,i32\\)->bool>"],
    ["lt", "<\\(i32,i32\\)->bool>"],
    ["lt_u", "<\\(i32,i32\\)->bool>"],
    ["le", "<\\(i32,i32\\)->bool>"],
    ["le_u", "<\\(i32,i32\\)->bool>"],
    ["gt", "<\\(i32,i32\\)->bool>"],
    ["gt_u", "<\\(i32,i32\\)->bool>"],
    ["ge", "<\\(i32,i32\\)->bool>"],
    ["ge_u", "<\\(i32,i32\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i32CompareModule, pattern, `core/math/i32/compare.nepl must define overload ${name} ${signature}`);
    assert.doesNotMatch(i32Module, pattern, `core/math/i32.nepl must not keep compare overload ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep overload ${name} ${signature}`);
}

for (const [name, signature] of [
    ["clz", "<\\(i32\\)->i32>"],
    ["ctz", "<\\(i32\\)->i32>"],
    ["popcnt", "<\\(i32\\)->i32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(i32BitwiseCountModule, pattern, `core/math/i32/bitwise/count.nepl must define unary ${name} ${signature}`);
    assert.doesNotMatch(i32BitwiseModule, pattern, `core/math/i32/bitwise.nepl must not keep bit count unary ${name} ${signature}`);
    assert.doesNotMatch(i32Module, pattern, `core/math/i32.nepl must not keep bitwise unary ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep unary ${name} ${signature}`);
}

for (const [name, signature] of [
    ["extend8_s_i32", "<\\(i32\\)->i32>"],
    ["extend16_s_i32", "<\\(i32\\)->i32>"],
    ["extend_s_i32_to_i64", "<\\(i32\\)->i64>"],
    ["extend_u_i32_to_i64", "<\\(i32\\)->i64>"],
    ["wrap_i64_to_i32", "<\\(i64\\)->i32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(convertWidthModule, pattern, `core/math/convert/width.nepl must define ${name} ${signature}`);
    assert.doesNotMatch(convertModule, pattern, `core/math/convert.nepl must not keep width helper ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep width helper ${name} ${signature}`);
}

for (const [name, signature] of [
    ["convert_s_i32_to_f32", "<\\(i32\\)->f32>"],
    ["convert_u_i32_to_f32", "<\\(i32\\)->f32>"],
    ["convert_s_i64_to_f32", "<\\(i64\\)->f32>"],
    ["convert_u_i64_to_f32", "<\\(i64\\)->f32>"],
    ["convert_s_i32_to_f64", "<\\(i32\\)->f64>"],
    ["convert_u_i32_to_f64", "<\\(i32\\)->f64>"],
    ["convert_s_i64_to_f64", "<\\(i64\\)->f64>"],
    ["convert_u_i64_to_f64", "<\\(i64\\)->f64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(convertFloatIntToFloatModule, pattern, `core/math/convert/float/int_to_float.nepl must define ${name} ${signature}`);
    assert.doesNotMatch(convertFloatModule, pattern, `core/math/convert/float.nepl must not keep int-to-float helper ${name} ${signature}`);
    assert.doesNotMatch(convertModule, pattern, `core/math/convert.nepl must not keep float helper ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep float helper ${name} ${signature}`);
}

for (const [name, signature] of [
    ["trunc_s_f32_to_i32", "<\\(f32\\)->i32>"],
    ["trunc_u_f32_to_i32", "<\\(f32\\)->i32>"],
    ["trunc_sat_s_f32_to_i32", "<\\(f32\\)->i32>"],
    ["trunc_sat_u_f32_to_i32", "<\\(f32\\)->i32>"],
    ["trunc_s_f64_to_i32", "<\\(f64\\)->i32>"],
    ["trunc_u_f64_to_i32", "<\\(f64\\)->i32>"],
    ["trunc_sat_s_f64_to_i32", "<\\(f64\\)->i32>"],
    ["trunc_sat_u_f64_to_i32", "<\\(f64\\)->i32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(convertFloatToI32Module, pattern, `core/math/convert/float/float_to_i32.nepl must define ${name} ${signature}`);
    assert.doesNotMatch(convertFloatModule, pattern, `core/math/convert/float.nepl must not keep float-to-i32 helper ${name} ${signature}`);
    assert.doesNotMatch(convertModule, pattern, `core/math/convert.nepl must not keep float helper ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep float helper ${name} ${signature}`);
}

for (const [name, signature] of [
    ["trunc_s_f32_to_i64", "<\\(f32\\)->i64>"],
    ["trunc_u_f32_to_i64", "<\\(f32\\)->i64>"],
    ["trunc_sat_s_f32_to_i64", "<\\(f32\\)->i64>"],
    ["trunc_sat_u_f32_to_i64", "<\\(f32\\)->i64>"],
    ["trunc_s_f64_to_i64", "<\\(f64\\)->i64>"],
    ["trunc_u_f64_to_i64", "<\\(f64\\)->i64>"],
    ["trunc_sat_s_f64_to_i64", "<\\(f64\\)->i64>"],
    ["trunc_sat_u_f64_to_i64", "<\\(f64\\)->i64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(convertFloatToI64Module, pattern, `core/math/convert/float/float_to_i64.nepl must define ${name} ${signature}`);
    assert.doesNotMatch(convertFloatModule, pattern, `core/math/convert/float.nepl must not keep float-to-i64 helper ${name} ${signature}`);
    assert.doesNotMatch(convertModule, pattern, `core/math/convert.nepl must not keep float helper ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep float helper ${name} ${signature}`);
}

for (const [name, signature] of [
    ["promote_f32_to_f64", "<\\(f32\\)->f64>"],
    ["demote_f64_to_f32", "<\\(f64\\)->f32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(convertFloatWidthModule, pattern, `core/math/convert/float/float_width.nepl must define ${name} ${signature}`);
    assert.doesNotMatch(convertFloatModule, pattern, `core/math/convert/float.nepl must not keep float-width helper ${name} ${signature}`);
    assert.doesNotMatch(convertModule, pattern, `core/math/convert.nepl must not keep float helper ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep float helper ${name} ${signature}`);
}

for (const [name, signature] of [
    ["reinterpret_i32_to_f32", "<\\(i32\\)->f32>"],
    ["reinterpret_f32_to_i32", "<\\(f32\\)->i32>"],
    ["reinterpret_i64_to_f64", "<\\(i64\\)->f64>"],
    ["reinterpret_f64_to_i64", "<\\(f64\\)->i64>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(convertReinterpretModule, pattern, `core/math/convert/reinterpret.nepl must define ${name} ${signature}`);
    assert.doesNotMatch(convertModule, pattern, `core/math/convert.nepl must not keep reinterpret helper ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep reinterpret helper ${name} ${signature}`);
}

for (const [name, signature] of [
    ["extend_s", "<\\(i32\\)->i64>"],
    ["wrap", "<\\(i64\\)->i32>"],
    ["convert_s", "<\\(i32\\)->f64>"],
    ["convert_s", "<\\(i64\\)->f64>"],
    ["convert_s", "<\\(i64\\)->f32>"],
    ["trunc_s", "<\\(f64\\)->i32>"],
    ["trunc_s", "<\\(f64\\)->i64>"],
    ["trunc_s", "<\\(f32\\)->i64>"],
    ["promote", "<\\(f32\\)->f64>"],
    ["demote", "<\\(f64\\)->f32>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(convertModule, pattern, `core/math/convert.nepl must define conversion wrapper ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep conversion wrapper ${name} ${signature}`);
}

for (const structName of ["u128", "i128"]) {
    const pattern = new RegExp(`\\bstruct\\s+${structName}\\b`);
    assert.match(int128TypesModule, pattern, `core/math/int128/types.nepl must define ${structName}`);
    assert.doesNotMatch(int128Module, pattern, `core/math/int128.nepl must not keep ${structName}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep ${structName}`);
}

for (const [name, signature] of [
    ["new", "<\\(i64,i64\\)->u128>"],
    ["to_u128", "<\\(i64\\)->u128>"],
    ["add", "<\\(u128,u128\\)->u128>"],
    ["sub", "<\\(u128,u128\\)->u128>"],
    ["lt", "<\\(u128,u128\\)->bool>"],
    ["mul_wide", "<\\(i64,i64\\)->u128>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(int128U128Module, pattern, `core/math/int128/u128.nepl must define ${name} ${signature}`);
    assert.doesNotMatch(int128Module, pattern, `core/math/int128.nepl must not keep u128 API ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep int128 API ${name} ${signature}`);
}

for (const [name, signature] of [
    ["new", "<\\(i64,i64\\)->i128>"],
    ["to_i128", "<\\(i64\\)->i128>"],
    ["add", "<\\(i128,i128\\)->i128>"],
    ["sub", "<\\(i128,i128\\)->i128>"],
    ["mul", "<\\(i128,i128\\)->i128>"],
    ["lt", "<\\(i128,i128\\)->bool>"],
]) {
    const pattern = new RegExp(`\\bfn\\s+${name}\\s+${signature}`);
    assert.match(int128I128Module, pattern, `core/math/int128/i128.nepl must define ${name} ${signature}`);
    assert.doesNotMatch(int128Module, pattern, `core/math/int128.nepl must not keep i128 API ${name} ${signature}`);
    assert.doesNotMatch(facade, pattern, `core/math.nepl must not keep int128 API ${name} ${signature}`);
}

assert.doesNotMatch(facade, /^#import\s+"core\/field"\s+as\s+\*/m, "core/math.nepl must not depend on core/field after int128 split");
assert.doesNotMatch(facade, /^fn\s+/m, "core/math.nepl must remain a facade without function bodies");
assert.doesNotMatch(facade, /^struct\s+/m, "core/math.nepl must remain a facade without structs");

console.log("stdlib math module split regression passed");
