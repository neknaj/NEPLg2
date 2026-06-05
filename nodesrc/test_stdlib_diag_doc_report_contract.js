#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function source(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function precedingDoc(code, declarationNeedle) {
    const index = code.indexOf(declarationNeedle);
    assert.notEqual(index, -1, `missing declaration: ${declarationNeedle}`);
    const before = code.slice(0, index).split("\n");
    const doc = [];
    let cursor = before.length - 1;
    while (cursor >= 0 && before[cursor].trim() === "") {
        cursor -= 1;
    }
    while (cursor >= 0 && before[cursor].trimStart().startsWith("//:")) {
        doc.push(before[cursor]);
        cursor -= 1;
    }
    return doc.reverse().join("\n");
}

function assertIncludes(code, needle, message) {
    assert.ok(code.includes(needle), message);
}

function assertReportDoc(code, declarationNeedle, reportName) {
    const doc = precedingDoc(code, declarationNeedle);
    assertIncludes(doc, "neplg2:test[stdio, normalize_newlines]", `${reportName} must use stdout-normalized doctest metadata`);
    assertIncludes(doc, `test_report_new "${reportName}"`, `${reportName} report doctest is missing`);
    assertIncludes(doc, "### [契約/けいやく]", `${reportName} must document stable contract`);
    assertIncludes(doc, "### [現在/げんざい]の[実装/じっそう]", `${reportName} must separate current implementation notes`);
    assertIncludes(doc, "### [計算量/けいさんりょう]", `${reportName} must document complexity`);
    return doc;
}

const renderer = source("stdlib/alloc/diag/diag.nepl");
const diag = source("stdlib/alloc/diag/error/diag.nepl");
const diags = source("stdlib/alloc/diag/error/diags.nepl");

const rendererDocs = [
    [
        "pub fn kind_str %fn Diag str \\d:",
        "diag_kind_str_by_value_doc",
    ],
    [
        "pub fn diag_to_string %fn Diag str \\d:",
        "diag_to_string_by_value_doc",
    ],
    [
        "pub fn diags_to_string %impure fn Diags str \\ds:",
        "diags_to_string_by_value_doc",
    ],
    [
        "pub fn diags_to_string_loop %fn &Vec Diag fn i32 fn i32 fn str str \\items\\n\\i\\acc:",
        "diags_to_string_loop_doc",
    ],
    [
        "pub fn diag_print %impure fn Diag unit \\d:",
        "diag_print_doc",
    ],
    [
        "pub fn diag_println %impure fn Diag unit \\d:",
        "diag_println_doc",
    ],
    [
        "pub fn diags_print %impure fn Diags unit \\ds:",
        "diags_print_doc",
    ],
    [
        "pub fn diags_println %impure fn Diags unit \\ds:",
        "diags_println_doc",
    ],
];

for (const [declaration, reportName] of rendererDocs) {
    assertReportDoc(renderer, declaration, reportName);
}

const diagDocs = [
    [
        "pub fn diag_level %fn Diag DiagLevel \\d:",
        "diag_level_by_value_doc",
    ],
    [
        "pub fn diag_std_error_kind %fn Diag Option StdErrorKind \\d:",
        "diag_std_error_kind_by_value_doc",
    ],
    [
        "pub fn diag_std_error_kind_str %fn Diag str \\d:",
        "diag_std_error_kind_str_by_value_doc",
    ],
];

for (const [declaration, reportName] of diagDocs) {
    assertReportDoc(diag, declaration, reportName);
}

const diagsDocs = [
    [
        "pub fn diags_len %impure fn Diags i32 \\ds:",
        "diags_len_by_value_doc",
    ],
    [
        "pub fn diags_has_errors %impure fn Diags bool \\ds:",
        "diags_has_errors_by_value_doc",
    ],
    [
        "pub fn diags_has_errors_loop %fn &Vec Diag fn i32 fn i32 bool \\items\\n\\i:",
        "diags_has_errors_loop_doc",
    ],
];

for (const [declaration, reportName] of diagsDocs) {
    assertReportDoc(diags, declaration, reportName);
}

for (const snippet of [
    "error の[権威/けんい]は `DiagKind.std_error` の enum payload",
    "typed error [判定/はんてい]には `diag_std_error_kind`",
    "error kind の[権威/けんい]は `DiagKind` / `StdErrorKind` enum",
    "message や kind [文字列/もじれつ]には[依存/いぞん]しません",
]) {
    assertIncludes([renderer, diag, diags].join("\n"), snippet, `diag docs must keep enum authority separated from display strings: ${snippet}`);
}

assert.match(
    renderer,
    /pub\s+fn\s+diags_to_string\s+%impure\s+fn\s+Diags\s+str\s+\\ds:[\s\S]*?let\s+s\s+%str\s+diags_to_string\s+&ds[\s\S]*?diags_free\s+ds[\s\S]*?\bs\b/,
    "by-value diags_to_string must be impure, call borrowed observer first, then close the Diags owner",
);

for (const [name, valueType] of [
    ["diags_len", "i32"],
    ["diags_has_errors", "bool"],
]) {
    const pattern = new RegExp(
        String.raw`pub\s+fn\s+${name}\s+%impure\s+fn\s+Diags\s+${valueType}\s+\\ds:[\s\S]*?let\s+([A-Za-z_][A-Za-z0-9_]*)\s+%${valueType}\s+${name}\s+&ds[\s\S]*?diags_free\s+ds[\s\S]*?\b\1\b`,
    );
    assert.match(diags, pattern, `by-value ${name} must observe through borrowed overload, close owner, and return the observed value`);
}

assert.match(
    diags,
    /pub\s+fn\s+diags_has_errors_loop[\s\S]*?match\s+vec_get::get\s+items\s+i:[\s\S]*?Option::Some\s+d:[\s\S]*?match\s+level:[\s\S]*?DiagLevel::Error:[\s\S]*?DiagLevel::Log:[\s\S]*?DiagLevel::Info:[\s\S]*?DiagLevel::Warn:[\s\S]*?Option::None:[\s\S]*?false/,
    "diags_has_errors_loop must use Vec.get and exhaustive DiagLevel match arms without string severity checks",
);

for (const snippet of [
    "stdio [副作用/ふくさよう]はこの helper に[閉/と]じ",
    "stdio [副作用/ふくさよう]は std target のこの helper に[閉/と]じます",
    "owner cleanup を[伴/ともな]うため、この overload は `impure fn`",
]) {
    assertIncludes(renderer, snippet, `diagnostic renderer docs must pin IO and owner-cleanup effect boundaries: ${snippet}`);
}

assert.doesNotMatch(
    [renderer, diag, diags].join("\n"),
    /message(?:文字列)?を.*(?:error|エラー).*(?:権威|authority)|"None".*(?:権威|authority)/u,
    "diag docs must not make display strings or the None label the diagnostic authority",
);

console.log("stdlib diag doc report contract passed");
