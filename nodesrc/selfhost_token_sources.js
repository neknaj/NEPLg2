"use strict";

const fs = require("node:fs");
const path = require("node:path");

const TOKEN_FACADE = "stdlib/neplg2/core/syntax/token.nepl";
const TOKEN_SPLIT_FILES = [
    "stdlib/neplg2/core/syntax/token/kind.nepl",
    "stdlib/neplg2/core/syntax/token/value.nepl",
    "stdlib/neplg2/core/syntax/token/name.nepl",
    "stdlib/neplg2/core/syntax/token/predicate/eof.nepl",
    "stdlib/neplg2/core/syntax/token/predicate/error.nepl",
    "stdlib/neplg2/core/syntax/token/predicate/newline.nepl",
    "stdlib/neplg2/core/syntax/token/predicate/expr_start.nepl",
    "stdlib/neplg2/core/syntax/token/stage0.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readTokenSource(repoRoot) {
    return [TOKEN_FACADE, ...TOKEN_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    TOKEN_FACADE,
    TOKEN_SPLIT_FILES,
    readRepoFile,
    readTokenSource,
};
