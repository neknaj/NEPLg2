"use strict";

const fs = require("node:fs");
const path = require("node:path");

const PRELUDE_FACADE = "stdlib/neplg2/core/builtins/prelude.nepl";
const PRELUDE_SPLIT_FILES = [
    "stdlib/neplg2/core/builtins/prelude/model.nepl",
    "stdlib/neplg2/core/builtins/prelude/kind.nepl",
    "stdlib/neplg2/core/builtins/prelude/signature.nepl",
    "stdlib/neplg2/core/builtins/prelude/function_registry.nepl",
    "stdlib/neplg2/core/builtins/prelude/primitive_registry.nepl",
    "stdlib/neplg2/core/builtins/prelude/path.nepl",
    "stdlib/neplg2/core/builtins/prelude/stage0.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readPreludeSource(repoRoot) {
    return [PRELUDE_FACADE, ...PRELUDE_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    PRELUDE_FACADE,
    PRELUDE_SPLIT_FILES,
    readPreludeSource,
    readRepoFile,
};
