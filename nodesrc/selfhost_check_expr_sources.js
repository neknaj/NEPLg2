"use strict";

const fs = require("node:fs");
const path = require("node:path");

const CHECK_EXPR_FACADE = "stdlib/neplg2/core/check/expr.nepl";
const CHECK_EXPR_SPLIT_FILES = [
    "stdlib/neplg2/core/check/expr/expectation.nepl",
    "stdlib/neplg2/core/check/expr/call_candidate.nepl",
    "stdlib/neplg2/core/check/expr/candidate_collection.nepl",
    "stdlib/neplg2/core/check/expr/value_evidence.nepl",
    "stdlib/neplg2/core/check/expr/model.nepl",
    "stdlib/neplg2/core/check/expr/argument.nepl",
    "stdlib/neplg2/core/check/expr/block_body.nepl",
    "stdlib/neplg2/core/check/expr/call_reduce.nepl",
    "stdlib/neplg2/core/check/expr/ascription.nepl",
    "stdlib/neplg2/core/check/expr/body_line.nepl",
    "stdlib/neplg2/core/check/expr/stage0.nepl",
    "stdlib/neplg2/core/check/expr/stage1.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readCheckExprSource(repoRoot) {
    return [CHECK_EXPR_FACADE, ...CHECK_EXPR_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    CHECK_EXPR_FACADE,
    CHECK_EXPR_SPLIT_FILES,
    readCheckExprSource,
    readRepoFile,
};
