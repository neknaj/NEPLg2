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

const bytebuf = source("stdlib/alloc/io/bytebuf.nepl");
const builderTypes = source("stdlib/alloc/io/bytebuilder/types.nepl");
const traits = source("stdlib/alloc/io/traits.nepl");
const documentationContract = source("nodesrc/test_stdlib_documentation_contract.js");

for (const [declaration, reportName] of [
    ["pub fn io_bytebuf_empty %fn void ByteBuf \\void:", "io_bytebuf_empty_doc"],
    ["pub fn io_bytebuf_len %fn &ByteBuf i32 \\buf:", "io_bytebuf_len_doc"],
    ["pub fn io_bytebuf_len_ref %fn &ByteBuf i32 \\buf:", "io_bytebuf_len_ref_doc"],
    ["pub fn io_bytebuf_data_ptr_ref %fn &ByteBuf MemPtr u8 \\buf:", "io_bytebuf_data_ptr_ref_doc"],
    ["pub fn io_bytebuf_ptr_ref %fn &ByteBuf Option MemPtr u8 \\buf:", "io_bytebuf_ptr_ref_doc"],
    ["pub fn io_bytebuf_storage_size %fn i32 i32 \\byte_len:", "io_bytebuf_storage_size_doc"],
    ["pub fn io_bytebuf_byte_at %fn &ByteBuf fn i32 Option i32 \\buf\\idx:", "io_bytebuf_byte_at_doc"],
    ["pub fn io_bytebuf_free %fn ByteBuf unit \\buf:", "io_bytebuf_free_doc"],
]) {
    assertReportDoc(bytebuf, declaration, reportName);
}

for (const [declaration, reportName] of [
    ["pub fn byte_builder_data_ptr_ref %fn &ByteBuilder MemPtr u8 \\builder:", "byte_builder_data_ptr_ref_doc"],
    ["pub fn byte_builder_ptr_ref %fn &ByteBuilder Option MemPtr u8 \\builder:", "byte_builder_ptr_ref_doc"],
]) {
    assertReportDoc(builderTypes, declaration, reportName);
}

for (const [declaration, reportName] of [
    ["trait ByteReader:", "byte_reader_read_all_bytes_doc"],
    ["trait ByteWriter:", "byte_writer_write_bytes_doc"],
    ["trait TextReader:", "text_reader_read_all_text_doc"],
    ["trait TextWriter:", "text_writer_write_str_doc"],
    ["trait Flush:", "flush_trait_doc"],
    ["trait Close:", "close_trait_doc"],
    ["pub fn io_read_all_bytes <.T: ByteReader> %impure fn .T Result ByteBuf StdErrorKind \\stream:", "io_read_all_bytes_doc"],
    ["pub fn io_write_bytes <.T: ByteWriter> %impure fn .T impure fn ByteBuf Result .T StdErrorKind \\stream\\bytes:", "io_write_bytes_doc"],
    ["pub fn io_read_all_text <.T: TextReader> %impure fn .T Result str StdErrorKind \\stream:", "io_read_all_text_doc"],
    ["pub fn io_write_str <.T: TextWriter> %impure fn .T impure fn str Result .T StdErrorKind \\stream\\text:", "io_write_str_doc"],
    ["pub fn io_flush <.T: Flush> %impure fn .T Result .T StdErrorKind \\stream:", "io_flush_doc"],
    ["pub fn io_close <.T: Close> %impure fn .T Result unit StdErrorKind \\stream:", "io_close_doc"],
]) {
    assertReportDoc(traits, declaration, reportName);
}

for (const snippet of [
    "`ByteBufStorage::Empty`",
    "`ByteBufStorage::Owned(region)`",
    "`RegionToken` owner",
    "返った `MemPtr` は所有権を[持/も]ちません",
    "`Option::None`",
    "`Option::Some ptr`",
    "`idx < 0` または `idx >= len`",
    "`dealloc_region<u8>` に owner token",
]) {
    assertIncludes(bytebuf, snippet, `ByteBuf docs must pin owner and Option boundary: ${snippet}`);
}

for (const snippet of [
    "`ByteBuilderStorage::Owned(region)`",
    "返った `MemPtr` は所有権を[持/も]ちません",
    "free obligation は builder owner に残ります",
    "`Option::None`",
    "`Option::Some ptr`",
]) {
    assertIncludes(builderTypes, snippet, `ByteBuilder pointer docs must pin borrowed pointer contract: ${snippet}`);
}

for (const snippet of [
    "失敗時は `StdErrorKind` を `Result::Err`",
    "default implementation は未対応 stream として `StdErrorKind::IoError`",
    "trait module は raw memory を[扱/あつか]わず",
    "静的に解決される trait dispatch",
    "helper は text を raw byte pointer として公開しません",
    "silent no-op は trait contract ではありません",
]) {
    assertIncludes(traits, snippet, `io/traits docs must pin effect and typed error boundary: ${snippet}`);
}

assert.doesNotMatch(
    traits,
    /trait\s+[A-Za-z_][A-Za-z0-9_]*:\s*\n\s+\/\/:/,
    "trait body doc comments are not valid NEPLg2 syntax; method contracts must live on the trait declaration docs",
);

for (const snippet of [
    "let traitBlockIndent = null",
    "startsTraitDeclaration",
    "if (traitBlockIndent !== null && !startsTraitDeclaration)",
]) {
    assertIncludes(documentationContract, snippet, `documentation contract must not count trait body methods as independently documentable declarations: ${snippet}`);
}

console.log("stdlib alloc/io doc report contract passed");
