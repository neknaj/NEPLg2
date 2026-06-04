"use strict";

const fs = require("node:fs");
const path = require("node:path");

const NAME_RESOLVER_FACADE = "stdlib/neplg2/core/resolve/name_resolver.nepl";
const NAME_RESOLVER_SPLIT_FILES = [
    "stdlib/neplg2/core/resolve/name_resolver/id.nepl",
    "stdlib/neplg2/core/resolve/name_resolver/kind.nepl",
    "stdlib/neplg2/core/resolve/name_resolver/binding.nepl",
    "stdlib/neplg2/core/resolve/name_resolver/scope.nepl",
    "stdlib/neplg2/core/resolve/name_resolver/hoist.nepl",
    "stdlib/neplg2/core/resolve/name_resolver/stage0.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readNameResolverSource(repoRoot) {
    return [NAME_RESOLVER_FACADE, ...NAME_RESOLVER_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    NAME_RESOLVER_FACADE,
    NAME_RESOLVER_SPLIT_FILES,
    readNameResolverSource,
    readRepoFile,
};
