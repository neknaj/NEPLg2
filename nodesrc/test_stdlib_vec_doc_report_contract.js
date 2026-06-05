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

function assertPrecedingReport(code, declarationNeedle, reportName) {
    const doc = precedingDoc(code, declarationNeedle);
    assertIncludes(
        doc,
        "neplg2:test[stdio, normalize_newlines]",
        `${reportName} must use stdio report doctest metadata`,
    );
    assertIncludes(doc, `test_report_new "${reportName}"`, `${reportName} report doctest is missing`);
    assertIncludes(doc, "### [契約/けいやく]", `${reportName} must document stable contract`);
    assertIncludes(doc, "### [現在/げんざい]の[実装/じっそう]", `${reportName} must separate current implementation notes`);
    return doc;
}

const invariant = source("stdlib/alloc/collections/vec/invariant.nepl");
const push = source("stdlib/alloc/collections/vec/mutation/push.nepl");
const fill = source("stdlib/alloc/collections/vec/storage/fill.nepl");
const filter = source("stdlib/alloc/collections/vec/transform/filter/select.nepl");

const adapterDoc = precedingDoc(
    invariant,
    "fn vec_storage_invalid_to_copy_invalid %fn VecStorageInvariantInvalid VecCopyInvariantInvalid \\reason:",
);
for (const snippet of [
    "test_report_new \"vec_storage_invalid_to_copy_invalid_doc\"",
    "VecStorageInvariantInvalid",
    "VecCopyInvariantInvalid",
    "1 対 1",
    "enum payload",
    "match` の網羅性",
    "payload を読み書きせず",
    "O(1)",
]) {
    assertIncludes(adapterDoc, snippet, `Vec invariant adapter docs must keep typed enum proof mapping contract: ${snippet}`);
}
assert.doesNotMatch(
    adapterDoc,
    /bool.*変換|message.*変換/u,
    "Vec invariant adapter docs must not describe error proof as bool or message conversion",
);

const pushCopyDoc = assertPrecedingReport(
    push,
    "pub fn push <.T: Copy> %impure fn Vec .T impure fn .T Result Vec .T VecPushError .T \\v\\item:",
    "vec_push_copy_doc",
);
const pushDropDoc = assertPrecedingReport(
    push,
    "pub fn push <.T: Drop> %impure fn Vec .T impure fn .T Result Vec .T VecPushError .T \\v\\item:",
    "vec_push_drop_doc",
);
for (const snippet of [
    "Result::Ok next_vec",
    "Result::Err VecPushError",
    "VecPushRejected .T",
    "typed `RegionToken .T`",
    "VecStorageInvariant",
    "償却 O(1)",
    "grow 時は O(n)",
]) {
    assertIncludes(pushCopyDoc, snippet, `Vec push Copy docs must keep owner and grow contract: ${snippet}`);
}
for (const snippet of [
    ".T: Drop",
    "Copy せず",
    "rejected `item` owner",
    "vec_push_rejected_with",
    "owner loss なし",
    "VecStorageInvariant",
    "vec_push_storage_checked",
]) {
    assertIncludes(pushDropDoc, snippet, `Vec push Drop docs must keep owner recovery contract: ${snippet}`);
}

const filledDoc = assertPrecedingReport(
    fill,
    "pub fn filled <.T: Copy> %fn i32 fn .T Result Vec .T StdErrorKind \\n\\value:",
    "vec_filled_doc",
);
for (const snippet of [
    "n <= 0",
    "VecStorage::Empty",
    "長さ、初期化済み長、capacity",
    "StdErrorKind::OutOfMemory",
    ".T: Copy",
    "collection_slot_initialize_empty",
    "O(n)",
]) {
    assertIncludes(filledDoc, snippet, `Vec filled docs must keep initialized storage contract: ${snippet}`);
}

const filterCopyDoc = assertPrecedingReport(
    filter,
    "pub fn filter <.T: Copy> %impure fn Vec .T impure fn fn .T bool Result Vec .T VecTransformError .T \\v\\p:",
    "vec_filter_copy_doc",
);
for (const snippet of [
    "入力順",
    "VecTransformError .T",
    "入力 `Vec` owner",
    "(.T)->bool",
    "Drop payload には使いません",
    "StdErrorKind::InvalidOperation",
    "1 回目の traversal",
    "2 回目の traversal",
    "O(n)",
]) {
    assertIncludes(filterCopyDoc, snippet, `Vec filter Copy docs must keep transform contract: ${snippet}`);
}

assert.doesNotMatch(
    [pushCopyDoc, pushDropDoc, filledDoc, filterCopyDoc].join("\n"),
    /rollback を(?:行います|保証します)|rollback is guaranteed/u,
    "Vec public docs must not claim rollback for storage invariant failure",
);

console.log("stdlib vec doc report contract passed");
