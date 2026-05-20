"use strict";

const fs = require("node:fs");
const path = require("node:path");

const HIR_FACADE = "stdlib/neplg2/core/hir/hir.nepl";
const HIR_SPLIT_FILES = [
    "stdlib/neplg2/core/hir/hir/id.nepl",
    "stdlib/neplg2/core/hir/hir/range.nepl",
    "stdlib/neplg2/core/hir/hir/expr.nepl",
    "stdlib/neplg2/core/hir/hir/function.nepl",
    "stdlib/neplg2/core/hir/hir/module.nepl",
    "stdlib/neplg2/core/hir/hir/arena.nepl",
    "stdlib/neplg2/core/hir/hir/stage0.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readHirSource(repoRoot) {
    return [HIR_FACADE, ...HIR_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    HIR_FACADE,
    HIR_SPLIT_FILES,
    readHirSource,
    readRepoFile,
};
