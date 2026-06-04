"use strict";

const fs = require("node:fs");
const path = require("node:path");

const TYPE_RESOLVER_FACADE = "stdlib/neplg2/core/resolve/type_resolver.nepl";
const TYPE_RESOLVER_SPLIT_FILES = [
    "stdlib/neplg2/core/resolve/type_resolver/model.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/input.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/constructor.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/resolved.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/model.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/plan.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce/build.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/reduce.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/project.nepl",
    "stdlib/neplg2/core/resolve/type_resolver/stage0.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readTypeResolverSource(repoRoot) {
    return [TYPE_RESOLVER_FACADE, ...TYPE_RESOLVER_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    TYPE_RESOLVER_FACADE,
    TYPE_RESOLVER_SPLIT_FILES,
    readRepoFile,
    readTypeResolverSource,
};
