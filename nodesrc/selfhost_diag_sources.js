"use strict";

const fs = require("node:fs");
const path = require("node:path");

const DIAG_FACADE = "stdlib/neplg2/core/infra/diag.nepl";
const DIAG_SPLIT_FILES = [
    "stdlib/neplg2/core/infra/diag/code.nepl",
    "stdlib/neplg2/core/infra/diag/value.nepl",
    "stdlib/neplg2/core/infra/diag/collection.nepl",
    "stdlib/neplg2/core/infra/diag/stage0.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readDiagSource(repoRoot) {
    return [DIAG_FACADE, ...DIAG_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    DIAG_FACADE,
    DIAG_SPLIT_FILES,
    readDiagSource,
    readRepoFile,
};
