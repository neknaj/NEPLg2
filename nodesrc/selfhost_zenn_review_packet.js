#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");
const usage = [
    "usage: node nodesrc/selfhost_zenn_review_packet.js",
    "  --issue <issue-id-or-note-checkpoint>",
    "  --slice <implementation-slice-name>",
    "  --accepted <accepted-scope>",
    "  --fail-closed <remaining-fail-closed-scope>",
    "  --zenn-checked-at <YYYY-MM-DD-or-ISO-like-date-time>",
    "  --executed <command-list-or-none-with-reason>",
    "  --not-executed <command-and-reason-list-or-none>",
    "  --existing-warnings <warning-list-or-none>",
    "  --new-warnings <warning-list-or-none>",
    "  [--base <commit>]",
    "  [--head <commit>]",
].join("\n");

if (process.argv.includes("--help")) {
    process.stdout.write(`${usage}\n`);
    process.exit(0);
}

const args = parseArgs(process.argv.slice(2));

try {
    const packet = buildPacket(args);
    process.stdout.write(`${packet}\n`);
} catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
}

function parseArgs(argv) {
    const parsed = new Map();
    for (let index = 0; index < argv.length; index += 1) {
        const name = argv[index];
        if (!name.startsWith("--")) {
            throw new Error(`unexpected positional argument: ${name}`);
        }
        const value = argv[index + 1];
        if (value === undefined || value.startsWith("--")) {
            throw new Error(`missing value for ${name}`);
        }
        parsed.set(name.slice(2), value);
        index += 1;
    }
    return parsed;
}

function buildPacket(parsed) {
    const issue = requiredOption(parsed, "issue");
    const slice = requiredOption(parsed, "slice");
    const accepted = requiredOption(parsed, "accepted");
    const failClosed = requiredOption(parsed, "fail-closed");
    const zennCheckedAt = requiredDateTimeOption(parsed, "zenn-checked-at");
    const executed = requiredOption(parsed, "executed");
    const notExecuted = requiredOption(parsed, "not-executed");
    const existingWarnings = requiredOption(parsed, "existing-warnings");
    const newWarnings = requiredOption(parsed, "new-warnings");
    const designDocs = optionalOption(
        parsed,
        "design-docs",
        [
            "doc/neplg2/self_host_neplg21_compiler_design.md",
            "doc/neplg2/self_host_execution_plan.md",
        ].join(","),
    );
    const related = optionalOption(parsed, "related", "note.n.md");

    const branch = gitOutput(["rev-parse", "--abbrev-ref", "HEAD"]);
    const head = optionalOption(parsed, "head", gitOutput(["rev-parse", "HEAD"]));
    const base = optionalOption(parsed, "base", gitOutput(["merge-base", head, "origin/main"]));
    const files = changedFileGroups(base, head);

    return [
        "Repository: NEPLg2",
        `対象 branch: ${branch}`,
        `base commit: ${base}`,
        `head commit: ${head}`,
        `対象 issue / slice: ${issue} / ${slice}`,
        "変更 file list:",
        renderFileGroups(files),
        "変更目的:",
        `  ${slice}`,
        "今回 accepted にした範囲:",
        `  ${accepted}`,
        "fail-closed に残した範囲:",
        `  ${failClosed}`,
        "Zenn policy:",
        "  https://zenn.dev/bem130/articles/1b352797de94e7",
        `  zenn_checked_at: ${zennCheckedAt}`,
        "  Review owner reopened this article before sending the packet.",
        "Repo policy:",
        "  AGENTS.md",
        "Review checklist:",
        "  doc/neplg2/self_host_zenn_review_checklist.md",
        "Review prompt authority:",
        "  doc/neplg2/self_host_zenn_review_prompt.md",
        "Design docs:",
        renderList(splitCsv(designDocs)),
        "関連 issue / note:",
        renderList(splitCsv(related)),
        "検証:",
        "  executed:",
        renderList(splitCsv(executed), "    "),
        "  not executed:",
        renderList(splitCsv(notExecuted), "    "),
        "  existing warnings:",
        renderList(splitCsv(existingWarnings), "    "),
        "  new warnings:",
        renderList(splitCsv(newWarnings), "    "),
        "",
        "依頼:",
        "  編集しないでレビューのみ行ってください。",
        "  この slice を policy/spec と implementation/test の 2 軸でレビューしてください。",
        "  Zenn policy、AGENTS.md、NEPLg2.1 仕様、設計文書、issue 完了条件、source policy、doc comment、検証結果に照らして確認してください。",
        "  実際に読んだ file list を files_read に列挙してください。",
        "  見ていない範囲は not_reviewed に明記してください。",
        "  行数制限、ファイル長制限、doc comment 長制限、コメント削減を理由にしないでください。",
        "  source token 再読、scope lookup 再実行、cursor-only evidence loss、owner/free、pure/impure、authority boundary を重点確認してください。",
        "  Blocker は同じ branch 内で修正が必要なものとして分類してください。",
        "  Non-blocker は次 slice または issue へ残す改善として分類してください。",
        "  Question は仕様判断や優先順位確認が必要なものとして分類してください。",
        "  Approve は Blocker がない場合だけ出してください。",
        "  返答は `nodesrc/selfhost_zenn_review_response_check.js` で検査します。",
        "",
        "必ず `doc/neplg2/self_host_zenn_review_prompt.md` の response 形式で返してください。",
    ].join("\n");
}

function requiredOption(parsed, name) {
    const value = parsed.get(name);
    if (!value || value.trim() === "") {
        throw new Error(`--${name} is required\n${usage}`);
    }
    return value;
}

function requiredDateTimeOption(parsed, name) {
    const value = requiredOption(parsed, name);
    if (!isReviewEvidenceDateTime(value)) {
        throw new Error(`--${name} must be YYYY-MM-DD or ISO-like date-time\n${usage}`);
    }
    return value.trim();
}

function isReviewEvidenceDateTime(value) {
    const normalized = value.trim();
    if (/^\d{4}-\d{2}-\d{2}$/.test(normalized)) {
        return isValidCalendarDate(normalized);
    }
    const dateTimeMatch = normalized.match(
        /^(\d{4}-\d{2}-\d{2})T(\d{2}):(\d{2})(?::(\d{2}))?(Z|([+-])(\d{2}):(\d{2}))?$/,
    );
    if (!dateTimeMatch) {
        return false;
    }

    const [, datePart, hourText, minuteText, secondText = "00", timezone, , timezoneHourText, timezoneMinuteText] = dateTimeMatch;
    const hour = Number(hourText);
    const minute = Number(minuteText);
    const second = Number(secondText);
    if (
        !isValidCalendarDate(datePart)
        || !isIntegerBetween(hour, 0, 23)
        || !isIntegerBetween(minute, 0, 59)
        || !isIntegerBetween(second, 0, 59)
    ) {
        return false;
    }

    if (!timezone || timezone === "Z") {
        return true;
    }
    return isIntegerBetween(Number(timezoneHourText), 0, 23)
        && isIntegerBetween(Number(timezoneMinuteText), 0, 59);
}

function isValidCalendarDate(value) {
    const [yearText, monthText, dayText] = value.split("-");
    const year = Number(yearText);
    const month = Number(monthText);
    const day = Number(dayText);
    return Number.isInteger(year)
        && isIntegerBetween(month, 1, 12)
        && isIntegerBetween(day, 1, daysInMonth(year, month));
}

function daysInMonth(year, month) {
    if (month === 2) {
        return isLeapYear(year) ? 29 : 28;
    }
    if ([4, 6, 9, 11].includes(month)) {
        return 30;
    }
    return 31;
}

function isLeapYear(year) {
    return year % 400 === 0 || (year % 4 === 0 && year % 100 !== 0);
}

function isIntegerBetween(value, min, max) {
    return Number.isInteger(value) && value >= min && value <= max;
}

function optionalOption(parsed, name, fallback) {
    const value = parsed.get(name);
    if (!value || value.trim() === "") {
        return fallback;
    }
    return value;
}

function changedFileGroups(base, head) {
    const committed = splitLines(gitOutput(["diff", "--name-only", `${base}...${head}`]));
    const staged = splitLines(gitOutput(["diff", "--cached", "--name-only"]));
    const unstaged = splitLines(gitOutput(["diff", "--name-only"]));
    const untracked = splitLines(gitOutput(["ls-files", "--others", "--exclude-standard"]));
    return {
        committed: uniqueSorted(committed),
        staged: uniqueSorted(staged),
        unstaged: uniqueSorted(unstaged),
        untracked: uniqueSorted(untracked),
    };
}

function gitOutput(args) {
    const result = spawnSync("git", args, {
        cwd: repoRoot,
        encoding: "utf8",
    });
    if (result.status !== 0) {
        const stderr = result.stderr.trim();
        throw new Error(`git ${args.join(" ")} failed${stderr ? `: ${stderr}` : ""}`);
    }
    return result.stdout.trim();
}

function splitLines(value) {
    return value.split(/\r?\n/).map((line) => line.trim()).filter(Boolean);
}

function splitCsv(value) {
    return value.split(",").map((item) => item.trim()).filter(Boolean);
}

function uniqueSorted(values) {
    return [...new Set(values)].sort((left, right) => left.localeCompare(right));
}

function renderList(values, indent = "  ", emptyLabel = "(none; fill before final review)") {
    if (values.length === 0) {
        return `${indent}- ${emptyLabel}`;
    }
    return values.map((value) => `${indent}- ${value}`).join("\n");
}

function renderFileGroups(files) {
    return [
        "  committed diff files:",
        renderList(files.committed, "    ", "none"),
        "  staged files:",
        renderList(files.staged, "    ", "none"),
        "  unstaged files:",
        renderList(files.unstaged, "    ", "none"),
        "  untracked files:",
        renderList(files.untracked, "    ", "none"),
    ].join("\n");
}
