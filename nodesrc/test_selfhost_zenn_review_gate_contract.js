#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

const design = read("doc/neplg2/self_host_neplg21_compiler_design.md");
const executionPlan = read("doc/neplg2/self_host_execution_plan.md");
const checklist = read("doc/neplg2/self_host_zenn_review_checklist.md");
const prompt = read("doc/neplg2/self_host_zenn_review_prompt.md");
const docCommentPolicy = read("doc/stdlib_doc_comment_policy.md");
const note = read("note.n.md");
const docGapIssue = read("issues/items/ISS-20260605T150033175Z-SELFHOST-COMPILER-DOC-COMMENTS-NEED--FF439E41.md");
const packetHelper = read("nodesrc/selfhost_zenn_review_packet.js");
const responseHelper = read("nodesrc/selfhost_zenn_review_response_check.js");
const sourcePolicyRunner = read("nodesrc/run_source_policy_regressions.js");

function assertIncludes(needle, message) {
    assert.ok(design.includes(needle), message);
}

function assertPlanIncludes(needle, message) {
    assert.ok(executionPlan.includes(needle), message);
}

function sourcePolicyRunnerChecks() {
    const match = sourcePolicyRunner.match(/const checks = \[\s*([\s\S]*?)\s*\];/);
    assert.ok(match, "source policy runner must expose a checks array");
    return new Set([...match[1].matchAll(/"([^"]+)"/g)].map((item) => item[1]));
}

function latestSelfhostCheckpoint(source) {
    const checkpoints = source
        .split(/(?=^# \d{4}-\d{2}-\d{2} Agent )/m)
        .filter((section) => /^# \d{4}-\d{2}-\d{2} Agent selfhost\b/m.test(section));
    assert.ok(
        checkpoints.length > 0,
        "note.n.md must contain at least one selfhost checkpoint for Zenn-policy review evidence",
    );
    return checkpoints[0];
}

function assertCheckpointIncludes(checkpoint, needle, message) {
    assert.ok(checkpoint.includes(needle), message);
}

const currentSelfhostCheckpoint = latestSelfhostCheckpoint(note);

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
assertIncludes(
    "selfhost_zenn_review_response_check.js",
    "selfhost design must require machine validation of subagent review responses",
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
assertPlanIncludes(
    "selfhost_zenn_review_response_check.js",
    "selfhost execution plan must require machine validation of subagent review responses",
);

for (const needle of [
    "Zenn 記事",
    "https://zenn.dev/bem130/articles/1b352797de94e7",
    "AGENTS.md",
    "対象 branch",
    "対象 issue / slice",
    "policy/spec",
    "implementation/test",
    "subagent review",
    "subagent_review_ids",
    "subagent_review_count",
    "Blocker",
    "Non-blocker",
    "Question",
    "Approve",
    "classification",
    "decision",
    "source_policy",
    "verify",
    "nodesrc/selfhost_zenn_review_response_check.js",
    "source policy",
    "既存 warning",
    "今回差分由来 warning",
    "検証済み",
    "行数制限",
    "doc comment 長制限",
    "次 slice",
]) {
    assertCheckpointIncludes(
        currentSelfhostCheckpoint,
        needle,
        `latest selfhost note checkpoint must record ${needle}`,
    );
}
assert.doesNotMatch(
    currentSelfhostCheckpoint,
    /実行予定/,
    "latest selfhost note checkpoint must record completed verification, not planned verification",
);

for (const needle of [
    "## 完了条件",
    "## review 証跡",
    "root cause",
    "impact",
    "completion conditions",
    "verification plan",
    "fail-closed boundary",
    "--record <note-or-issue.md>",
    "byte count",
    "file size",
    "doc comment length",
    "comment line count",
]) {
    assert.ok(docGapIssue.includes(needle), `selfhost documentation issue must record ${needle}`);
}
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
assert.ok(
    checklist.includes("最新の `note.n.md` selfhost checkpoint は `nodesrc/test_selfhost_zenn_review_gate_contract.js` で検査する"),
    "selfhost checklist must state that the latest note checkpoint is machine-checked",
);
assert.ok(
    checklist.includes("`note.n.md` または `issues/items/*.md`"),
    "selfhost checklist must restrict durable review records to note.n.md or issues/items/*.md",
);

for (const needle of [
    "対象 branch と対象 commit",
    "self_host_zenn_review_prompt.md",
    "`doc/neplg2/self_host_zenn_review_prompt.md` の request template",
    "`AGENTS.md` の関連方針",
    "今回変更した file list",
    "実行した検証、未実行の検証、既存 warning と今回差分由来の warning の区別",
    "個別 subagent review response を `nodesrc/selfhost_zenn_review_response_check.js --review-kind individual` で検査",
    "最終受理時は、2 件以上の個別 subagent review response を集約",
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
    "個別 subagent review response の必須 section / field を `nodesrc/selfhost_zenn_review_response_check.js --review-kind individual` で検査していること",
    "最終集約 response と durable record を `nodesrc/selfhost_zenn_review_response_check.js --review-kind final --record <note-or-issue.md>` で検査していること",
    "2 件以上の独立 subagent review",
    "`MERGE_APPROVED` は、`blockers` と `questions` が空",
    "source policy 不足、今回差分由来 warning、未実行検証",
    "`subagent_review_ids` と `subagent_review_count`",
    "新規 source policy を追加した場合に `nodesrc/run_source_policy_regressions.js` へ登録されていること",
]) {
    assert.ok(checklist.includes(needle), `selfhost checklist must cover ${needle}`);
}

assert.doesNotMatch(
    checklist,
    /行数制限、ファイル長制限、doc comment 長制限は source policy に入れない。[\s\S]*500 行/,
    "selfhost checklist must reject line-count limits without introducing numeric size thresholds",
);

for (const needle of [
    "## review request template",
    "## response の扱い",
    "## 禁止事項",
    "Repository:",
    "対象 branch:",
    "base commit:",
    "head commit:",
    "対象 issue / slice:",
    "変更 file list:",
    "変更目的:",
    "今回 accepted にした範囲:",
    "fail-closed に残した範囲:",
    "Zenn policy:",
    "https://zenn.dev/bem130/articles/1b352797de94e7",
    "zenn_checked_at: <YYYY-MM-DD-or-ISO-like-date-time>",
    "YYYY-MM-DDTHH:mm",
    "YYYY-MM-DDTHH:mm:ss",
    "+09:00",
    "Repo policy:",
    "AGENTS.md",
    "Review checklist:",
    "doc/neplg2/self_host_zenn_review_checklist.md",
    "編集しないでレビューのみ行ってください",
    "policy/spec と implementation/test の 2 軸",
    "最終受理には 2 件以上の独立 subagent review が必要です",
    "files_read",
    "not_reviewed",
    "行数制限、ファイル長制限、doc comment 長制限、コメント削減を理由にしないでください",
    "返答は `nodesrc/selfhost_zenn_review_response_check.js` で検査します",
    "source token 再読",
    "scope lookup 再実行",
    "cursor-only evidence loss",
    "## review_scope",
    "subagent_review_ids",
    "subagent_review_count",
    "## decision",
    "MERGE_APPROVED | BLOCKED | QUESTION",
    "## policy/spec",
    "## implementation/test",
    "classification:",
    "file/function:",
    "finding:",
    "root_cause:",
    "reason:",
    "recommended_fix:",
    "source_policy:",
    "source_policy_reason:",
    "doc_issue_note:",
    "verify:",
    "## zenn_check",
    "Result/Option:",
    "enum error/display separation:",
    "match exhaustiveness:",
    "pure/impure boundary:",
    "authority boundary:",
    "owner/free:",
    "zero-cost/performance:",
    "prototype/fail-closed:",
    "## evidence_to_record",
    "## warnings",
    "existing_warnings:",
    "new_warnings:",
    "## summary",
    "blockers:",
    "non_blockers:",
    "questions:",
    "approve:",
    "residual_risk:",
    "unexecuted_verification:",
    "node nodesrc/selfhost_zenn_review_response_check.js --review-kind individual --input <review-response.md>",
    "node nodesrc/selfhost_zenn_review_response_check.js --review-kind final --input <aggregate-review-response.md> --record <note-or-issue.md>",
    "--review-kind individual",
    "--review-kind final",
    "response checker が失敗した返答は review 記録として扱わず",
    "`MERGE_APPROVED` は、`blockers` と `questions` が空",
    "2 件以上の独立 `subagent_review_ids`",
    "`existing_warnings` と `new_warnings` を response から省いてはならない",
    "`source_policy: required` または `source_policy: follow-up` が残る `MERGE_APPROVED` は受理しない",
    "`subagent_review_ids` と `subagent_review_count`",
]) {
    assert.ok(prompt.includes(needle), `selfhost review prompt must include ${needle}`);
}

for (const needle of [
    "Blocker` は同じ branch 内で修正する",
    "同じ branch 内で修正できない `Blocker` は、原因、影響、完了条件、検証予定を持つ issue へ分離する",
    "`Question` は仕様確認として扱い、勝手な回避実装で進めない",
    "`Approve` があっても、`files_read`、`not_reviewed`、`subagent_review_ids`、`subagent_review_count`、`zenn_check`、`residual_risk`、`unexecuted_verification` が空の場合は review 記録として扱わない",
    "`source_policy: not-needed` の場合も、`source_policy_reason` に理由を残す",
    "--record <note-or-issue.md>",
    "`note.n.md` または `issues/items/*.md`",
    "一時ファイルや repo 外ファイルを指定してはならない",
    "Zenn 記事 URL、`AGENTS.md`、checklist、対象 branch / commit / issue を省いた依頼を出してはならない",
    "`policy/spec` と `implementation/test` のどちらか片方だけで approve してはならない",
    "2 件以上の独立 `subagent_review_ids`",
    "1 件だけの subagent review または同一 subagent id の重複を最終受理してはならない",
    "`source_policy: required` または `source_policy: follow-up` を残したまま `MERGE_APPROVED` にしてはならない",
    "`source_policy: not-needed` の理由を省いてはならない",
    "行数制限、ファイル長制限、doc comment 長制限を review 条件にしてはならない",
    "warning を既存か今回差分由来か分けずに扱ってはならない",
]) {
    assert.ok(prompt.includes(needle), `selfhost review prompt must enforce ${needle}`);
}

for (const needle of [
    "spawnSync",
    "merge-base",
    "origin/main",
    "diff",
    "ls-files",
    "--issue",
    "--slice",
    "--accepted",
    "--fail-closed",
    "--zenn-checked-at",
    "--executed",
    "--not-executed",
    "--existing-warnings",
    "--new-warnings",
    "requiredDateTimeOption",
    "isReviewEvidenceDateTime",
    "must be YYYY-MM-DD or ISO-like date-time",
    "YYYY-MM-DD-or-ISO-like-date-time",
    "process.exitCode = 1",
    "https://zenn.dev/bem130/articles/1b352797de94e7",
    "zenn_checked_at:",
    "Review owner reopened this article before sending the packet.",
    "AGENTS.md",
    "doc/neplg2/self_host_zenn_review_checklist.md",
    "doc/neplg2/self_host_zenn_review_prompt.md",
    "doc/neplg2/self_host_neplg21_compiler_design.md",
    "doc/neplg2/self_host_execution_plan.md",
    "policy/spec と implementation/test の 2 軸",
    "files_read",
    "not_reviewed",
    "existing warnings",
    "new warnings",
    "最終受理には 2 件以上の独立 subagent review が必要です",
    "`zenn_check` は `yes` や `確認済み` だけで済ませず",
    "Blocker は同じ branch 内で修正が必要なものとして分類してください",
    "返答は `nodesrc/selfhost_zenn_review_response_check.js` で検査します",
    "必ず `doc/neplg2/self_host_zenn_review_prompt.md` の response 形式で返してください",
]) {
    assert.ok(packetHelper.includes(needle), `selfhost review packet helper must include ${needle}`);
}

for (const needle of [
    "対象 branch:",
    "base commit:",
    "head commit:",
    "対象 issue / slice:",
    "変更 file list:",
    "committed diff files:",
    "staged files:",
    "unstaged files:",
    "untracked files:",
    "今回 accepted にした範囲:",
    "fail-closed に残した範囲:",
    "検証:",
    "executed:",
    "not executed:",
]) {
    assert.ok(packetHelper.includes(needle), `selfhost review packet helper must render ${needle}`);
}

for (const needle of [
    "--input <review-response.md>",
    "--stdin",
    "--review-kind final|individual",
    "requiredSections",
    "review_scope",
    "policy/spec",
    "implementation/test",
    "zenn_check",
    "evidence_to_record",
    "warnings",
    "summary",
    "missing_section",
    "missing_field",
    "invalid_decision",
    "invalid_classification",
    "invalid_source_policy",
    "invalid_record_target",
    "missing_files_read",
    "missing_subagent_review_ids",
    "invalid_subagent_review_count",
    "too_few_subagent_reviews",
    "individual_subagent_review_count",
    "individual_review_record",
    "invalid_review_kind",
    "subagent_review_count_mismatch",
    "duplicate_subagent_review_id",
    "missing_record_evidence",
    "--record <note-or-issue.md>",
    "approved_with_blockers",
    "approved_with_questions",
    "approved_with_blocker_classification",
    "approved_with_question_classification",
    "approved_with_required_source_policy",
    "approved_with_follow_up_source_policy",
    "approved_with_residual_risk",
    "approved_with_unexecuted_verification",
    "approved_with_new_warnings",
    "weak_zenn_check",
    "record_has_residual_risk",
    "record_has_unexecuted_verification",
    "record_has_required_source_policy",
    "record_has_follow_up_source_policy",
    "record_has_new_warnings",
    "weak_approval",
    "selfhost Zenn review response contract passed",
]) {
    assert.ok(responseHelper.includes(needle), `selfhost review response checker must include ${needle}`);
}

const selfhostPolicyTests = fs.readdirSync(path.join(repoRoot, "nodesrc"))
    .filter((name) => /^test_selfhost.*\.js$/.test(name))
    .map((name) => `nodesrc/${name}`)
    .sort();
const registeredSourcePolicyChecks = sourcePolicyRunnerChecks();
const missingSelfhostPolicyTests = selfhostPolicyTests.filter((relPath) => !registeredSourcePolicyChecks.has(relPath));
assert.deepEqual(
    missingSelfhostPolicyTests,
    [],
    [
        "selfhost source-policy tests must be registered in nodesrc/run_source_policy_regressions.js",
        "so Zenn-policy review gates cannot be bypassed by adding an unrun test file",
        ...missingSelfhostPolicyTests,
    ].join("\n"),
);

const helperHelp = spawnSync(process.execPath, ["nodesrc/selfhost_zenn_review_packet.js", "--help"], {
    cwd: repoRoot,
    encoding: "utf8",
});
assert.equal(helperHelp.status, 0, "selfhost review packet helper --help must succeed");
assert.ok(
    helperHelp.stdout.includes("--issue") && helperHelp.stdout.includes("--fail-closed"),
    "selfhost review packet helper --help must show required review packet arguments",
);
assert.ok(
    helperHelp.stdout.includes("--zenn-checked-at") && helperHelp.stdout.includes("--existing-warnings"),
    "selfhost review packet helper --help must show review evidence arguments",
);
assert.ok(
    helperHelp.stdout.includes("YYYY-MM-DD-or-ISO-like-date-time"),
    "selfhost review packet helper --help must show the exact Zenn review timestamp format",
);

const helperMissingRequired = spawnSync(process.execPath, ["nodesrc/selfhost_zenn_review_packet.js"], {
    cwd: repoRoot,
    encoding: "utf8",
});
assert.notEqual(helperMissingRequired.status, 0, "selfhost review packet helper must fail when required review context is missing");
assert.ok(
    helperMissingRequired.stderr.includes("--issue is required"),
    "selfhost review packet helper must report the first missing required review context",
);

const untrackedProbeName = `__selfhost_zenn_review_packet_contract_untracked_${process.pid}.probe`;
const untrackedProbeRelPath = path.posix.join("nodesrc", untrackedProbeName);
const untrackedProbe = path.join(repoRoot, "nodesrc", untrackedProbeName);
try {
    fs.writeFileSync(untrackedProbe, "review packet contract probe\n", "utf8");
    const helperPacket = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_packet.js",
        "--issue",
        "note.n.md",
        "--slice",
        "review-packet-contract",
        "--accepted",
        "contract execution check",
        "--fail-closed",
        "none for this packet",
        "--zenn-checked-at",
        "2026-06-05T12:30:00+09:00",
        "--executed",
        "node nodesrc/test_selfhost_zenn_review_gate_contract.js",
        "--not-executed",
        "none",
        "--existing-warnings",
        "none",
        "--new-warnings",
        "none",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.equal(helperPacket.status, 0, "selfhost review packet helper must generate a complete packet");
    for (const needle of [
        "Repository: NEPLg2",
        "zenn_checked_at: 2026-06-05T12:30:00+09:00",
        "committed diff files:",
        "staged files:",
        "unstaged files:",
        "untracked files:",
        untrackedProbeRelPath,
        "今回 accepted にした範囲:",
        "fail-closed に残した範囲:",
        "existing warnings:",
        "new warnings:",
    ]) {
        assert.ok(helperPacket.stdout.includes(needle), `selfhost review packet helper output must include ${needle}`);
    }
} finally {
    if (fs.existsSync(untrackedProbe)) {
        fs.unlinkSync(untrackedProbe);
    }
}

const helperMissingVerification = spawnSync(process.execPath, [
    "nodesrc/selfhost_zenn_review_packet.js",
    "--issue",
    "note.n.md",
    "--slice",
    "review-packet-contract",
    "--accepted",
    "contract execution check",
    "--fail-closed",
    "none for this packet",
    "--zenn-checked-at",
    "2026-06-05",
], {
    cwd: repoRoot,
    encoding: "utf8",
});
assert.notEqual(
    helperMissingVerification.status,
    0,
    "selfhost review packet helper must fail when verification evidence is missing",
);
assert.ok(
    helperMissingVerification.stderr.includes("--executed is required"),
    "selfhost review packet helper must name the missing verification evidence",
);

for (const invalidZennCheckedAt of [
    "today",
    "2026/06/05",
    "06-05",
    "2026-02-30",
    "2026-04-31",
    "2026-06-05T24:00:00+09:00",
]) {
    const helperInvalidZennCheckedAt = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_packet.js",
        "--issue",
        "note.n.md",
        "--slice",
        "review-packet-contract",
        "--accepted",
        "contract execution check",
        "--fail-closed",
        "none for this packet",
        "--zenn-checked-at",
        invalidZennCheckedAt,
        "--executed",
        "node nodesrc/test_selfhost_zenn_review_gate_contract.js",
        "--not-executed",
        "none",
        "--existing-warnings",
        "none",
        "--new-warnings",
        "none",
        "--base",
        "HEAD",
        "--head",
        "HEAD",
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        helperInvalidZennCheckedAt.status,
        0,
        `selfhost review packet helper must reject invalid Zenn review timestamp ${invalidZennCheckedAt}`,
    );
    assert.ok(
        helperInvalidZennCheckedAt.stderr.includes("--zenn-checked-at must be YYYY-MM-DD or ISO-like date-time"),
        "selfhost review packet helper must explain the accepted Zenn review timestamp format",
    );
}

const responseCheckTempDir = fs.mkdtempSync(path.join(os.tmpdir(), "selfhost-zenn-review-response-"));
const validReviewRecordRelPath = `issues/items/__selfhost_zenn_review_record_${process.pid}.md`;
const missingReviewRecordRelPath = `issues/items/__selfhost_zenn_review_missing_record_${process.pid}.md`;
const validReviewRecordPath = path.join(repoRoot, validReviewRecordRelPath);
const missingReviewRecordPath = path.join(repoRoot, missingReviewRecordRelPath);
try {
    const validReviewResponsePath = path.join(responseCheckTempDir, "valid.md");
    fs.writeFileSync(validReviewResponsePath, [
        "## review_scope",
        "- branch: selfhost/example",
        "- base: 0000000",
        "- head: 1111111",
        "- files_read:",
        "  - nodesrc/selfhost_zenn_review_packet.js",
        "- not_reviewed: unrelated stdlib files",
        "- subagent_review_ids:",
        "  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e",
        "  - 019e9936-d0de-75ae-923b-996ad05bf1d3",
        "- subagent_review_count: 2",
        "",
        "## decision",
        "- MERGE_APPROVED",
        "",
        "## policy/spec",
        "- classification: Approve",
        "- file/function: doc/neplg2/self_host_zenn_review_prompt.md",
        "- finding: response format is complete",
        "- root_cause: previous review evidence could be too weak",
        "- reason: all required sections are present",
        "- recommended_fix: none",
        "- source_policy: updated",
        "- source_policy_reason: response checker is covered by source policy",
        "- doc_issue_note: not-needed",
        "- verify: node nodesrc/selfhost_zenn_review_response_check.js --input valid.md",
        "",
        "## implementation/test",
        "- classification: Approve",
        "- file/function: nodesrc/selfhost_zenn_review_response_check.js",
        "- finding: checker accepts complete responses",
        "- root_cause: review response needed machine validation",
        "- reason: required fields are non-empty",
        "- recommended_fix: none",
        "- source_policy: updated",
        "- source_policy_reason: valid and invalid responses are tested",
        "- doc_issue_note: not-needed",
        "- verify: node nodesrc/test_selfhost_zenn_review_gate_contract.js",
        "",
        "## zenn_check",
        "- Result/Option: `nodesrc/selfhost_zenn_review_response_check.js` returns non-zero for invalid response evidence.",
        "- enum error/display separation: `nodesrc/selfhost_zenn_review_response_check.js` emits stable `reviewError` code strings before display text.",
        "- match exhaustiveness: `nodesrc/selfhost_zenn_review_response_check.js` keeps `requiredSections` and `sectionFields` as the accepted response shape.",
        "- pure/impure boundary: file IO stays in the Node helper boundary, not in `stdlib/neplg2` compiler core.",
        "- authority boundary: `--record` validates durable `note.n.md` or `issues/items/*.md` evidence before acceptance.",
        "- owner/free: not-applicable because `nodesrc/selfhost_zenn_review_response_check.js` does not manipulate NEPL owner/free state.",
        "- zero-cost/performance: `nodesrc/selfhost_zenn_review_response_check.js` uses a linear text scan over the review response and record.",
        "- doc comment: `doc/neplg2/self_host_zenn_review_prompt.md` keeps the review contract explicit.",
        "- prototype/fail-closed: `nodesrc/test_selfhost_zenn_review_gate_contract.js` rejects weak approvals and weak `zenn_check` evidence.",
        "",
        "## evidence_to_record",
        "- note: record response checker pass",
        "- issue: not-needed",
        "- source policy: updated",
        "- tests: node nodesrc/test_selfhost_zenn_review_gate_contract.js",
        "",
        "## warnings",
        "- existing_warnings: none",
        "- new_warnings: none",
        "",
        "## summary",
        "- blockers: 0",
        "- non_blockers: 0",
        "- questions: 0",
        "- approve: yes",
        "- residual_risk: none",
        "- unexecuted_verification: none",
        "",
    ].join("\n"), "utf8");
    fs.writeFileSync(validReviewRecordPath, [
        "# 2026-06-06 Agent selfhost review response record checkpoint",
        "",
        "- Zenn 記事 `https://zenn.dev/bem130/articles/1b352797de94e7` と AGENTS.md を確認した。",
        "- 対象 branch: selfhost/example",
        "- base commit: 0000000",
        "- head commit: 1111111",
        "- subagent review response を `nodesrc/selfhost_zenn_review_response_check.js --input valid.md --record record.md` で検査した。",
        "- subagent_review_ids:",
        "  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e",
        "  - 019e9936-d0de-75ae-923b-996ad05bf1d3",
        "- subagent_review_count: 2",
        "- files_read: nodesrc/selfhost_zenn_review_packet.js",
        "- not_reviewed: unrelated stdlib files",
        "- decision: MERGE_APPROVED",
        "- policy/spec classification: Approve, source_policy: updated, verify: node nodesrc/selfhost_zenn_review_response_check.js --input valid.md",
        "- implementation/test classification: Approve, source_policy: updated, verify: node nodesrc/test_selfhost_zenn_review_gate_contract.js",
        "- summary Blocker: 0 / Non-blocker: 0 / Question: 0 / Approve: yes",
        "- executed: node nodesrc/test_selfhost_zenn_review_gate_contract.js",
        "- not executed: none",
        "- residual_risk: none",
        "- unexecuted_verification: none",
        "- existing warnings: none",
        "- new warnings: none",
        "- 次 slice: none",
        "",
    ].join("\n"), "utf8");

    const responseCheckValid = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        validReviewRecordPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.equal(responseCheckValid.status, 0, "selfhost review response checker must accept complete review responses");
    assert.ok(
        responseCheckValid.stdout.includes("selfhost Zenn review response contract passed"),
        "selfhost review response checker must report success for complete review responses",
    );
    const responseCheckValidFromNodesrcCwd = spawnSync(process.execPath, [
        path.join(repoRoot, "nodesrc", "selfhost_zenn_review_response_check.js"),
        "--input",
        validReviewResponsePath,
        "--record",
        validReviewRecordPath,
    ], {
        cwd: path.join(repoRoot, "nodesrc"),
        encoding: "utf8",
    });
    assert.equal(
        responseCheckValidFromNodesrcCwd.status,
        0,
        "selfhost review response checker must validate absolute durable record paths independent of cwd",
    );

    const individualReviewResponsePath = path.join(responseCheckTempDir, "individual.md");
    fs.writeFileSync(
        individualReviewResponsePath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace(
            "- subagent_review_ids:\n  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e\n  - 019e9936-d0de-75ae-923b-996ad05bf1d3\n- subagent_review_count: 2\n",
            "- subagent_review_ids:\n  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e\n- subagent_review_count: 1\n",
        ),
        "utf8",
    );
    const responseCheckIndividual = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        individualReviewResponsePath,
        "--review-kind",
        "individual",
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.equal(
        responseCheckIndividual.status,
        0,
        "selfhost review response checker must accept one concrete reviewer in individual review mode",
    );

    const responseCheckIndividualWithRecord = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        individualReviewResponsePath,
        "--review-kind",
        "individual",
        "--record",
        validReviewRecordPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckIndividualWithRecord.status,
        0,
        "selfhost review response checker must not treat a single individual review as final durable acceptance",
    );
    assert.ok(
        responseCheckIndividualWithRecord.stderr.includes("individual_review_record"),
        "selfhost review response checker must explain individual/final review record separation",
    );

    const responseCheckInvalidReviewKind = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--review-kind",
        "summary",
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckInvalidReviewKind.status,
        0,
        "selfhost review response checker must reject unknown review kinds",
    );
    assert.ok(
        responseCheckInvalidReviewKind.stderr.includes("invalid_review_kind"),
        "selfhost review response checker must explain invalid review kinds",
    );

    const weakReviewResponsePath = path.join(responseCheckTempDir, "weak.md");
    fs.writeFileSync(weakReviewResponsePath, "Approve\n", "utf8");
    fs.writeFileSync(missingReviewRecordPath, "Approve\n", "utf8");
    const responseCheckMissingRecordEvidence = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        missingReviewRecordPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckMissingRecordEvidence.status,
        0,
        "selfhost review response checker must reject missing note/issue evidence records",
    );
    assert.ok(
        responseCheckMissingRecordEvidence.stderr.includes("missing_record_evidence"),
        "selfhost review response checker must explain missing record evidence",
    );

    const responseCheckInvalidRecordTarget = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        weakReviewResponsePath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckInvalidRecordTarget.status,
        0,
        "selfhost review response checker must reject non-durable review record targets",
    );
    assert.ok(
        responseCheckInvalidRecordTarget.stderr.includes("invalid_record_target"),
        "selfhost review response checker must explain invalid review record targets",
    );

    const responseCheckWeak = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        weakReviewResponsePath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(responseCheckWeak.status, 0, "selfhost review response checker must reject approve-only responses");
    assert.ok(
        responseCheckWeak.stderr.includes("missing_section"),
        "selfhost review response checker must explain missing required sections",
    );

    const missingSubagentReviewIdPath = path.join(responseCheckTempDir, "missing-subagent-review-id.md");
    fs.writeFileSync(
        missingSubagentReviewIdPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace(
            "- subagent_review_ids:\n  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e\n  - 019e9936-d0de-75ae-923b-996ad05bf1d3\n- subagent_review_count: 2\n",
            "- subagent_review_ids:\n- subagent_review_count: 2\n",
        ),
        "utf8",
    );
    const responseCheckMissingSubagentReviewId = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        missingSubagentReviewIdPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckMissingSubagentReviewId.status,
        0,
        "selfhost review response checker must reject reviews without concrete subagent ids",
    );
    assert.ok(
        responseCheckMissingSubagentReviewId.stderr.includes("missing_subagent_review_ids"),
        "selfhost review response checker must explain missing subagent review ids",
    );

    const tooFewSubagentReviewsPath = path.join(responseCheckTempDir, "too-few-subagent-reviews.md");
    fs.writeFileSync(
        tooFewSubagentReviewsPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace(
            "- subagent_review_ids:\n  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e\n  - 019e9936-d0de-75ae-923b-996ad05bf1d3\n- subagent_review_count: 2\n",
            "- subagent_review_ids:\n  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e\n- subagent_review_count: 1\n",
        ),
        "utf8",
    );
    const responseCheckTooFewSubagentReviews = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        tooFewSubagentReviewsPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckTooFewSubagentReviews.status,
        0,
        "selfhost review response checker must reject single-review acceptance",
    );
    assert.ok(
        responseCheckTooFewSubagentReviews.stderr.includes("too_few_subagent_reviews"),
        "selfhost review response checker must explain the two independent review requirement",
    );

    const mismatchedSubagentReviewCountPath = path.join(responseCheckTempDir, "mismatched-subagent-review-count.md");
    fs.writeFileSync(
        mismatchedSubagentReviewCountPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- subagent_review_count: 2", "- subagent_review_count: 3"),
        "utf8",
    );
    const responseCheckMismatchedSubagentReviewCount = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        mismatchedSubagentReviewCountPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckMismatchedSubagentReviewCount.status,
        0,
        "selfhost review response checker must reject mismatched subagent review counts",
    );
    assert.ok(
        responseCheckMismatchedSubagentReviewCount.stderr.includes("subagent_review_count_mismatch"),
        "selfhost review response checker must explain mismatched subagent review counts",
    );

    const invalidSubagentReviewCountPath = path.join(responseCheckTempDir, "invalid-subagent-review-count.md");
    fs.writeFileSync(
        invalidSubagentReviewCountPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- subagent_review_count: 2", "- subagent_review_count: 2abc"),
        "utf8",
    );
    const responseCheckInvalidSubagentReviewCount = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        invalidSubagentReviewCountPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckInvalidSubagentReviewCount.status,
        0,
        "selfhost review response checker must reject non-integer subagent review counts",
    );
    assert.ok(
        responseCheckInvalidSubagentReviewCount.stderr.includes("invalid_subagent_review_count"),
        "selfhost review response checker must explain invalid subagent review counts",
    );

    const duplicateSubagentReviewIdPath = path.join(responseCheckTempDir, "duplicate-subagent-review-id.md");
    fs.writeFileSync(
        duplicateSubagentReviewIdPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace(
            "- subagent_review_ids:\n  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e\n  - 019e9936-d0de-75ae-923b-996ad05bf1d3\n- subagent_review_count: 2\n",
            "- subagent_review_ids:\n  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e\n  - 019e9935-ca9d-72d2-aba5-2f1be90bfd5e\n- subagent_review_count: 2\n",
        ),
        "utf8",
    );
    const responseCheckDuplicateSubagentReviewId = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        duplicateSubagentReviewIdPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckDuplicateSubagentReviewId.status,
        0,
        "selfhost review response checker must reject duplicate subagent review ids",
    );
    assert.ok(
        responseCheckDuplicateSubagentReviewId.stderr.includes("duplicate_subagent_review_id"),
        "selfhost review response checker must explain duplicate subagent review ids",
    );

    const missingFieldReviewResponsePath = path.join(responseCheckTempDir, "missing-field.md");
    fs.writeFileSync(
        missingFieldReviewResponsePath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- files_read:\n  - nodesrc/selfhost_zenn_review_packet.js\n", "- files_read:\n"),
        "utf8",
    );
    const responseCheckMissingField = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        missingFieldReviewResponsePath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckMissingField.status,
        0,
        "selfhost review response checker must reject responses with empty required fields",
    );
    assert.ok(
        responseCheckMissingField.stderr.includes("missing_field"),
        "selfhost review response checker must explain missing required fields",
    );

    const invalidClassificationReviewResponsePath = path.join(responseCheckTempDir, "invalid-classification.md");
    fs.writeFileSync(
        invalidClassificationReviewResponsePath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- classification: Approve", "- classification: OK"),
        "utf8",
    );
    const responseCheckInvalidClassification = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        invalidClassificationReviewResponsePath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckInvalidClassification.status,
        0,
        "selfhost review response checker must reject undefined classification values",
    );
    assert.ok(
        responseCheckInvalidClassification.stderr.includes("invalid_classification"),
        "selfhost review response checker must explain invalid classification values",
    );

    const approvedWithBlockerReviewResponsePath = path.join(responseCheckTempDir, "approved-with-blocker.md");
    fs.writeFileSync(
        approvedWithBlockerReviewResponsePath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- blockers: 0", "- blockers: 1"),
        "utf8",
    );
    const responseCheckApprovedWithBlocker = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithBlockerReviewResponsePath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithBlocker.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED responses with blockers",
    );
    assert.ok(
        responseCheckApprovedWithBlocker.stderr.includes("approved_with_blockers"),
        "selfhost review response checker must explain approval/blocker conflicts",
    );

    const approvedWithBlockerClassificationPath = path.join(responseCheckTempDir, "approved-with-blocker-classification.md");
    fs.writeFileSync(
        approvedWithBlockerClassificationPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- classification: Approve", "- classification: Blocker"),
        "utf8",
    );
    const responseCheckApprovedWithBlockerClassification = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithBlockerClassificationPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithBlockerClassification.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED responses with Blocker classification",
    );
    assert.ok(
        responseCheckApprovedWithBlockerClassification.stderr.includes("approved_with_blocker_classification"),
        "selfhost review response checker must explain approval/classification conflicts",
    );

    const approvedWithQuestionClassificationPath = path.join(responseCheckTempDir, "approved-with-question-classification.md");
    fs.writeFileSync(
        approvedWithQuestionClassificationPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- classification: Approve", "- classification: Question"),
        "utf8",
    );
    const responseCheckApprovedWithQuestionClassification = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithQuestionClassificationPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithQuestionClassification.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED responses with Question classification",
    );
    assert.ok(
        responseCheckApprovedWithQuestionClassification.stderr.includes("approved_with_question_classification"),
        "selfhost review response checker must explain approval/question classification conflicts",
    );

    const approvedWithImplementationBlockerClassificationPath = path.join(responseCheckTempDir, "approved-with-implementation-blocker-classification.md");
    fs.writeFileSync(
        approvedWithImplementationBlockerClassificationPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace(
            "- classification: Approve\n- file/function: nodesrc/selfhost_zenn_review_response_check.js",
            "- classification: Blocker\n- file/function: nodesrc/selfhost_zenn_review_response_check.js",
        ),
        "utf8",
    );
    const responseCheckApprovedWithImplementationBlockerClassification = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithImplementationBlockerClassificationPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithImplementationBlockerClassification.status,
        0,
        "selfhost review response checker must reject implementation/test Blocker classification under MERGE_APPROVED",
    );
    assert.ok(
        responseCheckApprovedWithImplementationBlockerClassification.stderr.includes("approved_with_blocker_classification"),
        "selfhost review response checker must cover implementation/test approval/classification conflicts",
    );

    const approvedWithImplementationQuestionClassificationPath = path.join(responseCheckTempDir, "approved-with-implementation-question-classification.md");
    fs.writeFileSync(
        approvedWithImplementationQuestionClassificationPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace(
            "- classification: Approve\n- file/function: nodesrc/selfhost_zenn_review_response_check.js",
            "- classification: Question\n- file/function: nodesrc/selfhost_zenn_review_response_check.js",
        ),
        "utf8",
    );
    const responseCheckApprovedWithImplementationQuestionClassification = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithImplementationQuestionClassificationPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithImplementationQuestionClassification.status,
        0,
        "selfhost review response checker must reject implementation/test Question classification under MERGE_APPROVED",
    );
    assert.ok(
        responseCheckApprovedWithImplementationQuestionClassification.stderr.includes("approved_with_question_classification"),
        "selfhost review response checker must cover implementation/test approval/question conflicts",
    );

    const approvedWithRequiredSourcePolicyPath = path.join(responseCheckTempDir, "approved-with-required-source-policy.md");
    fs.writeFileSync(
        approvedWithRequiredSourcePolicyPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- source_policy: updated", "- source_policy: required"),
        "utf8",
    );
    const responseCheckApprovedWithRequiredSourcePolicy = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithRequiredSourcePolicyPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithRequiredSourcePolicy.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED responses with required source policy work",
    );
    assert.ok(
        responseCheckApprovedWithRequiredSourcePolicy.stderr.includes("approved_with_required_source_policy"),
        "selfhost review response checker must explain required source policy approval conflicts",
    );

    const approvedWithFollowUpSourcePolicyPath = path.join(responseCheckTempDir, "approved-with-follow-up-source-policy.md");
    fs.writeFileSync(
        approvedWithFollowUpSourcePolicyPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- source_policy: updated", "- source_policy: follow-up"),
        "utf8",
    );
    const responseCheckApprovedWithFollowUpSourcePolicy = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithFollowUpSourcePolicyPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithFollowUpSourcePolicy.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED responses with follow-up source policy work",
    );
    assert.ok(
        responseCheckApprovedWithFollowUpSourcePolicy.stderr.includes("approved_with_follow_up_source_policy"),
        "selfhost review response checker must explain follow-up source policy approval conflicts",
    );

    const approvedWithResidualRiskPath = path.join(responseCheckTempDir, "approved-with-residual-risk.md");
    fs.writeFileSync(
        approvedWithResidualRiskPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- residual_risk: none", "- residual_risk: doc policy evidence is incomplete"),
        "utf8",
    );
    const responseCheckApprovedWithResidualRisk = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithResidualRiskPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithResidualRisk.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED responses with residual risk",
    );
    assert.ok(
        responseCheckApprovedWithResidualRisk.stderr.includes("approved_with_residual_risk"),
        "selfhost review response checker must explain residual risk approval conflicts",
    );

    const approvedWithMisleadingNoResidualRiskPath = path.join(responseCheckTempDir, "approved-with-misleading-no-residual-risk.md");
    fs.writeFileSync(
        approvedWithMisleadingNoResidualRiskPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- residual_risk: none", "- residual_risk: none, but documentation evidence is incomplete"),
        "utf8",
    );
    const responseCheckApprovedWithMisleadingNoResidualRisk = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithMisleadingNoResidualRiskPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithMisleadingNoResidualRisk.status,
        0,
        "selfhost review response checker must reject misleading residual-risk no-work prefixes",
    );
    assert.ok(
        responseCheckApprovedWithMisleadingNoResidualRisk.stderr.includes("approved_with_residual_risk"),
        "selfhost review response checker must explain misleading residual-risk approval conflicts",
    );

    const approvedWithUnexecutedVerificationPath = path.join(responseCheckTempDir, "approved-with-unexecuted-verification.md");
    fs.writeFileSync(
        approvedWithUnexecutedVerificationPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- unexecuted_verification: none", "- unexecuted_verification: trunk build"),
        "utf8",
    );
    const responseCheckApprovedWithUnexecutedVerification = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithUnexecutedVerificationPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithUnexecutedVerification.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED responses with unexecuted verification",
    );
    assert.ok(
        responseCheckApprovedWithUnexecutedVerification.stderr.includes("approved_with_unexecuted_verification"),
        "selfhost review response checker must explain unexecuted verification approval conflicts",
    );

    const approvedWithMisleadingNoUnexecutedVerificationPath = path.join(responseCheckTempDir, "approved-with-misleading-no-unexecuted-verification.md");
    fs.writeFileSync(
        approvedWithMisleadingNoUnexecutedVerificationPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- unexecuted_verification: none", "- unexecuted_verification: none, but trunk build was skipped"),
        "utf8",
    );
    const responseCheckApprovedWithMisleadingNoUnexecutedVerification = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithMisleadingNoUnexecutedVerificationPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithMisleadingNoUnexecutedVerification.status,
        0,
        "selfhost review response checker must reject misleading unexecuted-verification no-work prefixes",
    );
    assert.ok(
        responseCheckApprovedWithMisleadingNoUnexecutedVerification.stderr.includes("approved_with_unexecuted_verification"),
        "selfhost review response checker must explain misleading unexecuted-verification approval conflicts",
    );

    const approvedWithNewWarningsPath = path.join(responseCheckTempDir, "approved-with-new-warnings.md");
    fs.writeFileSync(
        approvedWithNewWarningsPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- new_warnings: none", "- new_warnings: selfhost documentation warning from this diff"),
        "utf8",
    );
    const responseCheckApprovedWithNewWarnings = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithNewWarningsPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithNewWarnings.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED responses with new warnings",
    );
    assert.ok(
        responseCheckApprovedWithNewWarnings.stderr.includes("approved_with_new_warnings"),
        "selfhost review response checker must explain current-diff warning approval conflicts in the response",
    );

    const approvedWithMisleadingNoNewWarningsPath = path.join(responseCheckTempDir, "approved-with-misleading-no-new-warnings.md");
    fs.writeFileSync(
        approvedWithMisleadingNoNewWarningsPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace("- new_warnings: none", "- new_warnings: none, but a selfhost documentation warning remains"),
        "utf8",
    );
    const responseCheckApprovedWithMisleadingNoNewWarnings = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        approvedWithMisleadingNoNewWarningsPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckApprovedWithMisleadingNoNewWarnings.status,
        0,
        "selfhost review response checker must reject misleading current-diff warning no-work prefixes",
    );
    assert.ok(
        responseCheckApprovedWithMisleadingNoNewWarnings.stderr.includes("approved_with_new_warnings"),
        "selfhost review response checker must explain misleading current-diff warning approval conflicts",
    );

    const weakZennCheckPath = path.join(responseCheckTempDir, "weak-zenn-check.md");
    fs.writeFileSync(
        weakZennCheckPath,
        fs.readFileSync(validReviewResponsePath, "utf8").replace(
            "- Result/Option: `nodesrc/selfhost_zenn_review_response_check.js` returns non-zero for invalid response evidence.",
            "- Result/Option: yes",
        ),
        "utf8",
    );
    const responseCheckWeakZennCheck = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        weakZennCheckPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckWeakZennCheck.status,
        0,
        "selfhost review response checker must reject empty Zenn-policy confirmations",
    );
    assert.ok(
        responseCheckWeakZennCheck.stderr.includes("weak_zenn_check"),
        "selfhost review response checker must explain weak Zenn-policy evidence",
    );

    const keywordOnlyZennCheckPath = path.join(responseCheckTempDir, "keyword-only-zenn-check.md");
    fs.writeFileSync(
        keywordOnlyZennCheckPath,
        fs.readFileSync(validReviewResponsePath, "utf8")
            .replace(
                "- Result/Option: `nodesrc/selfhost_zenn_review_response_check.js` returns non-zero for invalid response evidence.",
                "- Result/Option: Result",
            )
            .replace(
                "- enum error/display separation: `nodesrc/selfhost_zenn_review_response_check.js` emits stable `reviewError` code strings before display text.",
                "- enum error/display separation: enum",
            )
            .replace(
                "- match exhaustiveness: `nodesrc/selfhost_zenn_review_response_check.js` keeps `requiredSections` and `sectionFields` as the accepted response shape.",
                "- match exhaustiveness: match",
            )
            .replace(
                "- pure/impure boundary: file IO stays in the Node helper boundary, not in `stdlib/neplg2` compiler core.",
                "- pure/impure boundary: pure",
            )
            .replace(
                "- authority boundary: `--record` validates durable `note.n.md` or `issues/items/*.md` evidence before acceptance.",
                "- authority boundary: authority",
            )
            .replace(
                "- owner/free: not-applicable because `nodesrc/selfhost_zenn_review_response_check.js` does not manipulate NEPL owner/free state.",
                "- owner/free: owner",
            )
            .replace(
                "- zero-cost/performance: `nodesrc/selfhost_zenn_review_response_check.js` uses a linear text scan over the review response and record.",
                "- zero-cost/performance: performance",
            )
            .replace(
                "- doc comment: `doc/neplg2/self_host_zenn_review_prompt.md` keeps the review contract explicit.",
                "- doc comment: doc comment",
            )
            .replace(
                "- prototype/fail-closed: `nodesrc/test_selfhost_zenn_review_gate_contract.js` rejects weak approvals and weak `zenn_check` evidence.",
                "- prototype/fail-closed: fail-closed",
            ),
        "utf8",
    );
    const responseCheckKeywordOnlyZennCheck = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        keywordOnlyZennCheckPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckKeywordOnlyZennCheck.status,
        0,
        "selfhost review response checker must reject keyword-only Zenn-policy confirmations",
    );
    assert.ok(
        responseCheckKeywordOnlyZennCheck.stderr.includes("weak_zenn_check"),
        "selfhost review response checker must explain keyword-only Zenn-policy evidence",
    );

    const quotedKeywordOnlyZennCheckPath = path.join(responseCheckTempDir, "quoted-keyword-only-zenn-check.md");
    fs.writeFileSync(
        quotedKeywordOnlyZennCheckPath,
        fs.readFileSync(validReviewResponsePath, "utf8")
            .replace(
                "- Result/Option: `nodesrc/selfhost_zenn_review_response_check.js` returns non-zero for invalid response evidence.",
                "- Result/Option: `Result`",
            )
            .replace(
                "- enum error/display separation: `nodesrc/selfhost_zenn_review_response_check.js` emits stable `reviewError` code strings before display text.",
                "- enum error/display separation: `enum`",
            )
            .replace(
                "- match exhaustiveness: `nodesrc/selfhost_zenn_review_response_check.js` keeps `requiredSections` and `sectionFields` as the accepted response shape.",
                "- match exhaustiveness: `match`",
            )
            .replace(
                "- pure/impure boundary: file IO stays in the Node helper boundary, not in `stdlib/neplg2` compiler core.",
                "- pure/impure boundary: `pure`",
            )
            .replace(
                "- authority boundary: `--record` validates durable `note.n.md` or `issues/items/*.md` evidence before acceptance.",
                "- authority boundary: `authority`",
            )
            .replace(
                "- owner/free: not-applicable because `nodesrc/selfhost_zenn_review_response_check.js` does not manipulate NEPL owner/free state.",
                "- owner/free: `owner`",
            )
            .replace(
                "- zero-cost/performance: `nodesrc/selfhost_zenn_review_response_check.js` uses a linear text scan over the review response and record.",
                "- zero-cost/performance: `performance`",
            )
            .replace(
                "- doc comment: `doc/neplg2/self_host_zenn_review_prompt.md` keeps the review contract explicit.",
                "- doc comment: `doc comment`",
            )
            .replace(
                "- prototype/fail-closed: `nodesrc/test_selfhost_zenn_review_gate_contract.js` rejects weak approvals and weak `zenn_check` evidence.",
                "- prototype/fail-closed: `fail-closed`",
            ),
        "utf8",
    );
    const responseCheckQuotedKeywordOnlyZennCheck = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        quotedKeywordOnlyZennCheckPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckQuotedKeywordOnlyZennCheck.status,
        0,
        "selfhost review response checker must reject code-quoted keyword-only Zenn-policy confirmations",
    );
    assert.ok(
        responseCheckQuotedKeywordOnlyZennCheck.stderr.includes("weak_zenn_check"),
        "selfhost review response checker must explain code-quoted keyword-only Zenn-policy evidence",
    );

    const reviewRecordWithResidualRiskPath = missingReviewRecordPath;
    fs.writeFileSync(
        reviewRecordWithResidualRiskPath,
        fs.readFileSync(validReviewRecordPath, "utf8").replace("- residual_risk: none", "- residual_risk: documentation evidence is incomplete"),
        "utf8",
    );
    const responseCheckRecordWithResidualRisk = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        reviewRecordWithResidualRiskPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckRecordWithResidualRisk.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED records with residual risk",
    );
    assert.ok(
        responseCheckRecordWithResidualRisk.stderr.includes("record_has_residual_risk"),
        "selfhost review response checker must explain residual risk conflicts in durable records",
    );

    const reviewRecordWithUnexecutedVerificationPath = missingReviewRecordPath;
    fs.writeFileSync(
        reviewRecordWithUnexecutedVerificationPath,
        fs.readFileSync(validReviewRecordPath, "utf8").replace("- unexecuted_verification: none", "- unexecuted_verification: trunk build"),
        "utf8",
    );
    const responseCheckRecordWithUnexecutedVerification = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        reviewRecordWithUnexecutedVerificationPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckRecordWithUnexecutedVerification.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED records with unexecuted verification",
    );
    assert.ok(
        responseCheckRecordWithUnexecutedVerification.stderr.includes("record_has_unexecuted_verification"),
        "selfhost review response checker must explain unexecuted verification conflicts in durable records",
    );

    const reviewRecordWithSourcePolicyTextOnlyPath = missingReviewRecordPath;
    fs.writeFileSync(
        reviewRecordWithSourcePolicyTextOnlyPath,
        fs.readFileSync(validReviewRecordPath, "utf8").replaceAll("source_policy: updated", "source_policy checked"),
        "utf8",
    );
    const responseCheckRecordWithSourcePolicyTextOnly = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        reviewRecordWithSourcePolicyTextOnlyPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckRecordWithSourcePolicyTextOnly.status,
        0,
        "selfhost review response checker must reject records that mention source_policy without machine-readable fields",
    );
    assert.ok(
        responseCheckRecordWithSourcePolicyTextOnly.stderr.includes("missing_record_evidence"),
        "selfhost review response checker must explain missing machine-readable source policy fields",
    );

    const reviewRecordWithRequiredSourcePolicyPath = missingReviewRecordPath;
    fs.writeFileSync(
        reviewRecordWithRequiredSourcePolicyPath,
        fs.readFileSync(validReviewRecordPath, "utf8").replace("source_policy: updated", "source_policy: required"),
        "utf8",
    );
    const responseCheckRecordWithRequiredSourcePolicy = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        reviewRecordWithRequiredSourcePolicyPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckRecordWithRequiredSourcePolicy.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED records with required source policy work",
    );
    assert.ok(
        responseCheckRecordWithRequiredSourcePolicy.stderr.includes("record_has_required_source_policy"),
        "selfhost review response checker must explain required source policy conflicts in durable records",
    );

    const reviewRecordWithFollowUpSourcePolicyPath = missingReviewRecordPath;
    fs.writeFileSync(
        reviewRecordWithFollowUpSourcePolicyPath,
        fs.readFileSync(validReviewRecordPath, "utf8").replace("source_policy: updated", "source_policy: follow-up"),
        "utf8",
    );
    const responseCheckRecordWithFollowUpSourcePolicy = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        reviewRecordWithFollowUpSourcePolicyPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckRecordWithFollowUpSourcePolicy.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED records with follow-up source policy work",
    );
    assert.ok(
        responseCheckRecordWithFollowUpSourcePolicy.stderr.includes("record_has_follow_up_source_policy"),
        "selfhost review response checker must explain follow-up source policy conflicts in durable records",
    );

    const reviewRecordWithNewWarningsPath = missingReviewRecordPath;
    fs.writeFileSync(
        reviewRecordWithNewWarningsPath,
        fs.readFileSync(validReviewRecordPath, "utf8").replace("- new warnings: none", "- new warnings: selfhost documentation warning from this diff"),
        "utf8",
    );
    const responseCheckRecordWithNewWarnings = spawnSync(process.execPath, [
        "nodesrc/selfhost_zenn_review_response_check.js",
        "--input",
        validReviewResponsePath,
        "--record",
        reviewRecordWithNewWarningsPath,
    ], {
        cwd: repoRoot,
        encoding: "utf8",
    });
    assert.notEqual(
        responseCheckRecordWithNewWarnings.status,
        0,
        "selfhost review response checker must reject MERGE_APPROVED records with new warnings",
    );
    assert.ok(
        responseCheckRecordWithNewWarnings.stderr.includes("record_has_new_warnings"),
        "selfhost review response checker must explain current-diff warning approval conflicts",
    );
} finally {
    fs.rmSync(responseCheckTempDir, { recursive: true, force: true });
    for (const relPath of [validReviewRecordRelPath, missingReviewRecordRelPath]) {
        const filePath = path.join(repoRoot, relPath);
        if (fs.existsSync(filePath)) {
            fs.unlinkSync(filePath);
        }
    }
}

console.log("selfhost Zenn review gate contract passed");
