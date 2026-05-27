#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const commentTypeFiles = [
    "stdlib/neplg2/cli/args/parse.nepl",
    "stdlib/neplg2/core/infra/diag.nepl",
    "stdlib/neplg2/core/infra/outcome.nepl",
    "stdlib/neplg2/core/module/import_spec.nepl",
    "stdlib/neplg2/core/module/import_scan.nepl",
    "stdlib/neplg2/core/module/graph.nepl",
    "stdlib/neplg2/core/syntax/ast/module_ast.nepl",
    "stdlib/neplg2/core/syntax/token/value.nepl",
    "stdlib/neplg2/core/syntax/lexer/tokenize.nepl",
    "stdlib/std/fs/dir/read_fd.nepl",
    "stdlib/std/fs/dir/path.nepl",
    "stdlib/std/fs/fd.nepl",
    "stdlib/std/fs/stat.nepl",
    "stdlib/std/fs/path/entry.nepl",
    "stdlib/std/fs/path/normalize/range_stack.nepl",
    "stdlib/std/fs/path/normalize/build.nepl",
    "stdlib/std/fs/read/fd.nepl",
    "stdlib/std/fs/raw/llvm.nepl",
    "stdlib/std/fs/write/fd.nepl",
];

const lexerCallFiles = [
    "stdlib/neplg2/core/syntax/lexer/tokenize.nepl",
    "stdlib/neplg2/core/syntax/lexer/indent.nepl",
];

const outcomeFile = "stdlib/neplg2/core/infra/outcome.nepl";

const oldCommentTypeApplication = /\b(?:Vec|Result|Option|Selfhost[A-Za-z0-9_]*|RegionToken|MemPtr)<[^>]+>/;
const oldSelfhostCommentHelperPostfix = /\b(?:unwrap|unwrap_ok|unwrap_err)<Selfhost[A-Za-z0-9_]*>/;
const oldLexerHelperPostfix = /\b(?:push|new|drop_last|vec_push_error_vec)<[A-Za-z_.]/;
const oldOutcomeHelperPostfix = /\bselfhost_outcome_[A-Za-z0-9_]+<[A-Za-z_.]/;
const oldOutcomeConstructorApplication = /\bSelfhostOutcome<\s*\./;

const violations = [];

function readLines(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").split(/\r?\n/);
}

for (const relPath of commentTypeFiles) {
    const lines = readLines(relPath);
    lines.forEach((line, index) => {
        if (
            line.trimStart().startsWith("//") &&
            (oldCommentTypeApplication.test(line) || oldSelfhostCommentHelperPostfix.test(line))
        ) {
            violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
        }
    });
}

for (const relPath of lexerCallFiles) {
    const lines = readLines(relPath);
    lines.forEach((line, index) => {
        if (oldLexerHelperPostfix.test(line)) {
            violations.push(`${relPath}:${index + 1}: ${line.trim()}`);
        }
    });
}

readLines(outcomeFile).forEach((line, index) => {
    const declarationLine = /^\s*pub\s+(?:struct|enum|trait)\s+/.test(line);
    if (oldOutcomeHelperPostfix.test(line) || (!declarationLine && oldOutcomeConstructorApplication.test(line))) {
        violations.push(`${outcomeFile}:${index + 1}: ${line.trim()}`);
    }
});

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 selfhost prose/type postfix cleanup must not reintroduce the migrated old syntax:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 selfhost prose/type postfix cleanup regression passed");
