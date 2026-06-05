"use strict";

const fs = require("node:fs");
const path = require("node:path");

const MODULE_PARSER_FACADE = "stdlib/neplg2/core/syntax/parser/module_parser.nepl";
const MODULE_PARSER_SPLIT_FILES = [
    "stdlib/neplg2/core/syntax/parser/range.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/state.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/token_role.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/token_role_header.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/action.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/diagnostic.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/header_boundary.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/item_kind.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/prefix_range.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/body_range.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/declaration.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/loop.nepl",
    "stdlib/neplg2/core/syntax/parser/module_parser/entry.nepl",
];

function readRepoFile(repoRoot, relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function readModuleParserSource(repoRoot) {
    return [MODULE_PARSER_FACADE, ...MODULE_PARSER_SPLIT_FILES]
        .map((relPath) => readRepoFile(repoRoot, relPath))
        .join("\n");
}

module.exports = {
    MODULE_PARSER_FACADE,
    MODULE_PARSER_SPLIT_FILES,
    readModuleParserSource,
    readRepoFile,
};
