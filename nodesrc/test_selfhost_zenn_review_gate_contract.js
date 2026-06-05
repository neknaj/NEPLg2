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
const checklist = read("doc/neplg2/self_host_zenn_review_checklist.md");
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
assertIncludes(
    "self_host_zenn_review_checklist.md",
    "selfhost design must link to the detailed subagent review checklist",
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
assertPlanIncludes(
    "self_host_zenn_review_checklist.md",
    "selfhost execution plan must link to the detailed subagent review checklist",
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

for (const needle of [
    "## review の入力",
    "## 必須確認項目",
    "### 静的検査と error model",
    "### pure core と platform boundary",
    "### authority boundary",
    "### documentation comment",
    "### performance と探索範囲",
    "### prototype policy",
    "## 指摘分類",
    "## note checkpoint 形式",
    "## source policy 化の基準",
]) {
    assert.ok(checklist.includes(needle), `selfhost checklist must include ${needle}`);
}

for (const needle of [
    "対象 branch と対象 commit",
    "`AGENTS.md` の関連方針",
    "今回変更した file list",
    "実行した検証、未実行の検証、既存 warning と今回差分由来の warning の区別",
    "review の観点が `policy/spec` と `implementation/test` の 2 軸に分かれていること",
    "### review の 2 軸",
    "`policy/spec`",
    "`implementation/test`",
    "Result",
    "Option",
    "enum diagnostic",
    "match",
    "sentinel",
    "pure / impure",
    "parser が prefix call boundary",
    "HIR lowering が source text や token lexeme を再読して型証拠を作り直していない",
    "目的",
    "契約",
    "戻り値",
    "error variant",
    "計算量",
    "探索範囲",
    "cache key",
    "暫定実装",
    "Blocker",
    "Non-blocker",
    "Question",
    "Approve",
    "classification: Blocker | Non-blocker | Question | Approve",
    "decision: fixed | issue | open | not-applicable",
    "source_policy: added | updated | not-needed | follow-up",
    "AGENTS.md の関連方針を確認した",
    "新規 source policy を追加した場合に `nodesrc/run_source_policy_regressions.js` へ登録されていること",
]) {
    assert.ok(checklist.includes(needle), `selfhost checklist must cover ${needle}`);
}

assert.doesNotMatch(
    checklist,
    /行数制限、ファイル長制限、doc comment 長制限は source policy に入れない。[\s\S]*500 行/,
    "selfhost checklist must reject line-count limits without introducing numeric size thresholds",
);

console.log("selfhost Zenn review gate contract passed");
