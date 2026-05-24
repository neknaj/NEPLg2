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
    /let\s+e\s+<BitSetUpdateError>\s+BitSetUpdateError\s+bs\s+d/,
    "legacyTypeSyntaxView must render typed local annotations without swallowing the initializer",
);
assert.doesNotMatch(legacy, /comment must not be visible/, "legacyTypeSyntaxView must strip NEPL comments");

console.log("source policy NEPL source view regression passed");
