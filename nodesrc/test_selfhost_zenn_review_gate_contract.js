#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

const design = read("doc/neplg2/self_host_neplg21_compiler_design.md");
const executionPlan = read("doc/neplg2/self_host_execution_plan.md");
const docCommentPolicy = read("doc/stdlib_doc_comment_policy.md");

function assertIncludes(needle, message) {
    assert.ok(design.includes(needle), message);
}

function assertPlanIncludes(needle, message) {
    assert.ok(executionPlan.includes(needle), message);
}

assertIncludes(
    "https://zenn.dev/bem130/articles/1b352797de94e7",
    "selfhost design must name the Zenn policy article as the review authority",
);
assertIncludes(
    "## Zenn 方針 review gate",
    "selfhost design must define a dedicated Zenn policy review gate",
);
assertIncludes(
    "新しい issue、実装 slice、または設計変更に着手する前",
    "selfhost design must require policy confirmation before new issue or slice work starts",
);
assertIncludes(
    "独立した subagent",
    "selfhost design must require independent subagent review",
);
assertIncludes(
    "Blocker は同じ slice 内で修正する",
    "selfhost design must require blocker findings to be fixed in the current slice",
);
assertIncludes(
    "修正できないものは、原因、影響、完了条件を持つ issue として分離する",
    "selfhost design must require unresolved blockers to become issue-scoped work",
);
assertIncludes(
    "commit 前に、今回の差分が Zenn 方針 review gate を通ったことを `note.n.md` の checkpoint に記録する",
    "selfhost design must require a note.n.md checkpoint before commit",
);

for (const needle of [
    "Result",
    "Option",
    "enum error",
    "match",
    "sentinel 値",
    "pure core",
    "host / CLI boundary",
    "parser、checker、HIR、Resource IR、backend",
    "探索範囲",
    "事前検査済み artifact",
    "cache key",
]) {
    assertIncludes(needle, `selfhost review gate must cover ${needle}`);
}

for (const needle of [
    "目的",
    "契約",
    "戻り値",
    "error variant",
    "計算量",
    "制約",
    "典型例",
    "現状の実装詳細",
    "将来も守る契約",
]) {
    assertIncludes(needle, `selfhost review gate must preserve doc comment coverage for ${needle}`);
}

assertIncludes(
    "コメントの量を減らすための行数制限や説明削減の検査が入っていないこと",
    "selfhost design must reject checks that discourage detailed documentation comments",
);
assertIncludes(
    "実装行数やドキュメントコメントの長さそのものを制限してはならない",
    "source policy must not restrict explanation volume instead of structural responsibility",
);

assertPlanIncludes(
    "## 2.1 Zenn 方針 review gate",
    "selfhost execution plan must define a Zenn policy gate",
);
assertPlanIncludes(
    "作業開始時と commit 前に確認する",
    "selfhost execution plan must require policy confirmation before work and before commit",
);
assertPlanIncludes(
    "独立した subagent review",
    "selfhost execution plan must require independent subagent review",
);
assertPlanIncludes(
    "Blocker は同じ branch 内で修正する",
    "selfhost execution plan must require blocker fixes in the current branch",
);
assertPlanIncludes(
    "コメントを減らすための行数制限や説明削減の検査を入れてはならない",
    "selfhost execution plan must reject checks that suppress detailed documentation",
);
assertPlanIncludes(
    "commit の大きさは行数では判定しない",
    "selfhost execution plan must judge commit scope by responsibility boundary",
);
assert.doesNotMatch(
    executionPlan,
    /500 行/,
    "selfhost execution plan must not keep numeric size thresholds for commit acceptability",
);
assert.ok(
    docCommentPolicy.includes("`stdlib/neplg2/` セルフホストコンパイラ実装にも同じ水準を適用"),
    "stdlib documentation policy must explicitly cover the current selfhost compiler implementation",
);

console.log("selfhost Zenn review gate contract passed");
