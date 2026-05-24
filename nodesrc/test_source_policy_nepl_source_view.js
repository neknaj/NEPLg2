#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const {
    fnSignaturePattern,
    legacyTypeSyntaxView,
} = require("./source_policy/nepl_source_view");

assert.match(
    "pub fn debug %impure fn str () \\s:",
    new RegExp(fnSignaturePattern("debug", ["str"], "()", { effect: "impure" })),
    "fnSignaturePattern must preserve impure function signatures",
);

const legacy = legacyTypeSyntaxView(`
// comment must not be visible to source policy checks
pub struct BloomFilter<.T,.H>:
    bits %Vec u8
    hasher %.H

pub fn scan %impure fn &StreamScanner str \\sc:
    sc

pub fn fold <.T: Copy,.U> %fn &List .T fn .U fn fn .U fn .T .U .U \\lst\\acc\\f:
    acc

pub fn data_mem_view <.T: Copy> %fn &Vec .T VecDataView .T \\v:
    VecDataView::Empty

pub fn realloc_region_bytes_keep <.T> %fn RegionToken .T fn i32 Result RegionToken .T RegionReallocError .T \\region\\new_size:
    Result::Ok region

pub fn vec_push_rejected_with <.T,.R> %impure fn VecPushRejected .T impure fn impure fn Vec .T impure fn .T .R .R \\rejected\\callback:
    callback

pub fn read_byte %fn i32 i32 \\addr:
    load_u8 %i32 add addr 4

let invariant %VecStorageInvariant vec_buffer_current_storage_invariant<.T> v_buffer_ref
let e %BitSetUpdateError BitSetUpdateError bs d
`);

assert.match(
    legacy,
    /bits\s+<Vec<u8>>/,
    "legacyTypeSyntaxView must render prefix field type annotations as angle-bracket fields",
);
assert.match(
    legacy,
    /pub\s+fn\s+scan\s+<\(&StreamScanner\)\*>str>\s+\(sc\):/,
    "legacyTypeSyntaxView must render impure single-argument signatures without losing effect",
);
assert.match(
    legacy,
    /pub\s+fn\s+fold\s+<\.T:\s*Copy,\.U>\s+<\(&List<\.T>,\.U,\(\.U,\.T\)->\.U\)->\.U>\s+\(lst,acc,f\):/,
    "legacyTypeSyntaxView must render nested callback function types inside source policy signatures",
);
assert.match(
    legacy,
    /pub\s+fn\s+data_mem_view\s+<\.T:\s*Copy>\s+<\(&Vec<\.T>\)->VecDataView<\.T>>\s+\(v\):/,
    "legacyTypeSyntaxView must render unary functions whose argument and result are both generic types",
);
assert.match(
    legacy,
    /pub\s+fn\s+realloc_region_bytes_keep\s+<\.T>\s+<\(RegionToken<\.T>,i32\)->Result<RegionToken<\.T>,RegionReallocError<\.T>>>\s+\(region,new_size\):/,
    "legacyTypeSyntaxView must keep owner-preserving Result payload boundaries inside generic signatures",
);
assert.match(
    legacy,
    /pub\s+fn\s+vec_push_rejected_with\s+<\.T,\.R>\s+<\(VecPushRejected<\.T>,\(Vec<\.T>,\.T\)\*>\.R\)\*>\.R>\s+\(rejected,callback\):/,
    "legacyTypeSyntaxView must render nested impure owner-recovery callback signatures",
);
assert.match(
    legacy,
    /let\s+invariant\s+<VecStorageInvariant>\s+vec_buffer_current_storage_invariant<\.T>\s+v_buffer_ref/,
    "legacyTypeSyntaxView must not consume initializer expressions after zero-arity policy types",
);
assert.match(
    legacy,
    /load_u8\s+%i32\s+add\s+addr\s+4/,
    "legacyTypeSyntaxView must not treat expression-local type ascriptions as struct fields",
);
assert.match(
    legacy,
    /let\s+e\s+<BitSetUpdateError>\s+BitSetUpdateError\s+bs\s+d/,
    "legacyTypeSyntaxView must render typed local annotations without swallowing the initializer",
);
assert.doesNotMatch(legacy, /comment must not be visible/, "legacyTypeSyntaxView must strip NEPL comments");

console.log("source policy NEPL source view regression passed");
