#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const repoRoot = path.resolve(__dirname, "..");

const usage = [
    "usage: node nodesrc/selfhost_zenn_review_response_check.js --input <review-response.md>",
    "   or: node nodesrc/selfhost_zenn_review_response_check.js --stdin",
    "  optional: --record <note-or-issue.md>",
].join("\n");

const requiredSections = [
    "review_scope",
    "decision",
    "policy/spec",
    "implementation/test",
    "zenn_check",
    "evidence_to_record",
    "summary",
];

const sectionFields = new Map([
    ["review_scope", ["branch", "base", "head", "files_read", "not_reviewed", "subagent_review_ids", "subagent_review_count"]],
    ["policy/spec", [
        "classification",
        "file/function",
        "finding",
        "root_cause",
        "reason",
        "recommended_fix",
        "source_policy",
        "source_policy_reason",
        "doc_issue_note",
        "verify",
    ]],
    ["implementation/test", [
        "classification",
        "file/function",
        "finding",
        "root_cause",
        "reason",
        "recommended_fix",
        "source_policy",
        "source_policy_reason",
        "doc_issue_note",
        "verify",
    ]],
    ["zenn_check", [
        "Result/Option",
        "enum error/display separation",
        "match exhaustiveness",
        "pure/impure boundary",
        "authority boundary",
        "owner/free",
        "zero-cost/performance",
        "doc comment",
        "prototype/fail-closed",
    ]],
    ["evidence_to_record", ["note", "issue", "source policy", "tests"]],
    ["summary", [
        "blockers",
        "non_blockers",
        "questions",
        "approve",
        "residual_risk",
        "unexecuted_verification",
    ]],
]);

try {
    const args = parseArgs(process.argv.slice(2));
    const source = readReviewResponse(args);
    const errors = validateReviewResponse(source);
    const recordPath = args.get("record");
    if (recordPath) {
        const resolvedRecordPath = resolveRecordPath(recordPath, errors);
        if (resolvedRecordPath) {
            const record = fs.readFileSync(resolvedRecordPath, "utf8").replace(/\r\n/g, "\n");
            validateReviewRecordEvidence(source, record, errors);
        }
    }
    if (errors.length > 0) {
        for (const error of errors) {
            console.error(`${error.code}: ${error.message}`);
        }
        process.exitCode = 1;
    } else {
        process.stdout.write("selfhost Zenn review response contract passed\n");
    }
} catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
}

function parseArgs(argv) {
    if (argv.includes("--help")) {
        process.stdout.write(`${usage}\n`);
        process.exit(0);
    }
    const parsed = new Map();
    for (let index = 0; index < argv.length; index += 1) {
        const name = argv[index];
        if (name === "--stdin") {
            parsed.set("stdin", "true");
            continue;
        }
        if (!name.startsWith("--")) {
            throw new Error(`unexpected positional argument: ${name}\n${usage}`);
        }
        const value = argv[index + 1];
        if (value === undefined || value.startsWith("--")) {
            throw new Error(`missing value for ${name}\n${usage}`);
        }
        parsed.set(name.slice(2), value);
        index += 1;
    }
    return parsed;
}

function readReviewResponse(args) {
    const input = args.get("input");
    const stdin = args.get("stdin") === "true";
    if (input && stdin) {
        throw new Error(`use either --input or --stdin, not both\n${usage}`);
    }
    if (input) {
        return fs.readFileSync(input, "utf8").replace(/\r\n/g, "\n");
    }
    if (stdin) {
        return fs.readFileSync(0, "utf8").replace(/\r\n/g, "\n");
    }
    throw new Error(`review response input is required\n${usage}`);
}

function validateReviewResponse(source) {
    const errors = [];
    const sections = parseSections(source);
    for (const section of requiredSections) {
        if (!sections.has(section)) {
            errors.push(reviewError("missing_section", `missing required section: ## ${section}`));
        }
    }
    if (errors.length > 0) {
        return errors;
    }

    validateDecision(sections.get("decision"), errors);
    for (const [section, fields] of sectionFields.entries()) {
        for (const field of fields) {
            const value = fieldValue(sections.get(section), field);
            if (!value || isPlaceholderValue(value)) {
                errors.push(reviewError("missing_field", `## ${section} must contain a non-empty ${field}: field`));
            }
        }
    }
    validateFilesRead(sections.get("review_scope"), errors);
    validateSubagentReviewEvidence(sections.get("review_scope"), errors);
    validateClassificationValue(sections.get("policy/spec"), "policy/spec", errors);
    validateClassificationValue(sections.get("implementation/test"), "implementation/test", errors);
    validateSourcePolicyValue(sections.get("policy/spec"), "policy/spec", errors);
    validateSourcePolicyValue(sections.get("implementation/test"), "implementation/test", errors);
    validateMergeApproval(
        sections.get("decision"),
        sections.get("summary"),
        sections.get("policy/spec"),
        sections.get("implementation/test"),
        errors,
    );
    validateReviewDoesNotAcceptWarnings(sections.get("summary"), errors);
    return errors;
}

function validateReviewRecordEvidence(source, record, errors) {
    const sections = parseSections(source);
    for (const section of requiredSections) {
        if (!sections.has(section)) {
            return;
        }
    }

    for (const needle of [
        "https://zenn.dev/bem130/articles/1b352797de94e7",
        "AGENTS.md",
        "policy/spec",
        "implementation/test",
        "subagent review",
        "files_read",
        "not_reviewed",
        "Blocker",
        "Non-blocker",
        "Question",
        "Approve",
        "classification",
        "decision",
        "source_policy",
        "verify",
    ]) {
        if (!record.includes(needle)) {
            errors.push(reviewError(
                "missing_record_evidence",
                `record file must include review evidence: ${needle}`,
            ));
        }
    }

    for (const [section, fields] of [
        ["review_scope", ["branch", "base", "head", "subagent_review_ids", "subagent_review_count"]],
        ["summary", ["blockers", "questions", "approve"]],
    ]) {
        for (const field of fields) {
            const value = fieldValue(sections.get(section), field);
            if (field === "subagent_review_ids") {
                for (const id of subagentReviewIds(value)) {
                    if (!record.includes(id)) {
                        errors.push(reviewError(
                            "missing_record_evidence",
                            `record file must include ${section}.${field}: ${id}`,
                        ));
                    }
                }
                continue;
            }
            if (value && !isPlaceholderValue(value) && !record.includes(value)) {
                errors.push(reviewError(
                    "missing_record_evidence",
                    `record file must include ${section}.${field}: ${value}`,
                ));
            }
        }
    }

    const decision = sections.get("decision").match(/\b(MERGE_APPROVED|BLOCKED|QUESTION)\b/);
    if (decision && !record.includes(decision[1])) {
        errors.push(reviewError(
            "missing_record_evidence",
            `record file must include review decision: ${decision[1]}`,
        ));
    }

    if (!recordIncludesAny(record, ["executed", "検証済み", "実行した検証"])) {
        errors.push(reviewError(
            "missing_record_evidence",
            "record file must include executed verification evidence",
        ));
    }
    if (!recordIncludesAny(record, ["not executed", "未実行", "未実行の検証"])) {
        errors.push(reviewError(
            "missing_record_evidence",
            "record file must include unexecuted verification evidence",
        ));
    }
    if (!recordIncludesAny(record, ["existing warnings", "既存 warning"])) {
        errors.push(reviewError(
            "missing_record_evidence",
            "record file must distinguish existing warnings",
        ));
    }
    if (!recordIncludesAny(record, ["new warnings", "今回差分由来 warning"])) {
        errors.push(reviewError(
            "missing_record_evidence",
            "record file must distinguish warnings introduced by the current diff",
        ));
    }
}

function resolveRecordPath(recordPath, errors) {
    const resolvedPath = path.isAbsolute(recordPath)
        ? path.resolve(recordPath)
        : path.resolve(repoRoot, recordPath);
    const normalized = path.relative(repoRoot, resolvedPath).replace(/\\/g, "/");
    if (normalized.startsWith("../") || normalized === ".." || path.isAbsolute(normalized)) {
        errors.push(reviewError(
            "invalid_record_target",
            "--record must point to note.n.md or issues/items/*.md inside the repository",
        ));
        return null;
    }
    if (normalized === "note.n.md" || /^issues\/items\/[^/]+\.md$/.test(normalized)) {
        return resolvedPath;
    }
    errors.push(reviewError(
        "invalid_record_target",
        "--record must point to note.n.md or issues/items/*.md so accepted review evidence is durable",
    ));
    return null;
}

function parseSections(source) {
    const lines = source.split("\n");
    const sections = new Map();
    let current = null;
    let body = [];
    for (const line of lines) {
        const match = line.match(/^## ([^\r\n]+)\s*$/);
        if (match) {
            if (current) {
                sections.set(current, body.join("\n"));
            }
            current = match[1].trim();
            body = [];
        } else if (current) {
            body.push(line);
        }
    }
    if (current) {
        sections.set(current, body.join("\n"));
    }
    return sections;
}

function validateDecision(section, errors) {
    const decision = section.match(/\b(MERGE_APPROVED|BLOCKED|QUESTION)\b/);
    if (!decision) {
        errors.push(reviewError("invalid_decision", "## decision must contain MERGE_APPROVED, BLOCKED, or QUESTION"));
    }
}

function fieldValue(section, field) {
    const lines = section.split("\n");
    const fieldPattern = new RegExp(`^-\\s*${escapeRegExp(field)}\\s*:\\s*(.*)$`);
    const nextFieldPattern = /^-\s*[^:]+:\s*/;
    for (let index = 0; index < lines.length; index += 1) {
        const match = lines[index].match(fieldPattern);
        if (!match) {
            continue;
        }
        const inline = match[1].trim();
        if (inline !== "") {
            return inline;
        }
        const continuation = [];
        for (let next = index + 1; next < lines.length; next += 1) {
            const line = lines[next];
            if (nextFieldPattern.test(line)) {
                break;
            }
            if (line.trim() !== "") {
                continuation.push(line.trim());
            }
        }
        return continuation.join("\n").trim();
    }
    return "";
}

function validateSourcePolicyValue(section, sectionName, errors) {
    const value = fieldValue(section, "source_policy");
    if (!/^(added|updated|required|not-needed|follow-up)\b/.test(value)) {
        errors.push(reviewError(
            "invalid_source_policy",
            `## ${sectionName} source_policy must be added, updated, required, not-needed, or follow-up`,
        ));
    }
}

function validateClassificationValue(section, sectionName, errors) {
    const value = fieldValue(section, "classification");
    if (!/^(Blocker|Non-blocker|Question|Approve)\b/.test(value)) {
        errors.push(reviewError(
            "invalid_classification",
            `## ${sectionName} classification must be Blocker, Non-blocker, Question, or Approve`,
        ));
    }
    if (/^Blocker\b/.test(value) && isNoWorkValue(fieldValue(section, "recommended_fix"))) {
        errors.push(reviewError(
            "missing_blocker_fix",
            `## ${sectionName} Blocker findings must include a concrete recommended_fix`,
        ));
    }
}

function validateFilesRead(section, errors) {
    const value = fieldValue(section, "files_read");
    if (isNoWorkValue(value)) {
        errors.push(reviewError("missing_files_read", "## review_scope files_read must list at least one reviewed file"));
    }
}

function validateSubagentReviewEvidence(section, errors) {
    const idsValue = fieldValue(section, "subagent_review_ids");
    const countValue = fieldValue(section, "subagent_review_count");
    const ids = subagentReviewIds(idsValue);
    if (ids.length === 0) {
        errors.push(reviewError(
            "missing_subagent_review_ids",
            "## review_scope subagent_review_ids must list at least one UUID-like subagent id",
        ));
    }
    const uniqueIds = new Set(ids.map((id) => id.toLowerCase()));
    if (uniqueIds.size !== ids.length) {
        errors.push(reviewError(
            "duplicate_subagent_review_id",
            "## review_scope subagent_review_ids must not contain duplicate subagent ids",
        ));
    }
    if (!/^[1-9][0-9]*$/.test(countValue)) {
        errors.push(reviewError(
            "invalid_subagent_review_count",
            "## review_scope subagent_review_count must be a positive integer",
        ));
        return;
    }
    const count = Number.parseInt(countValue, 10);
    if (uniqueIds.size !== count) {
        errors.push(reviewError(
            "subagent_review_count_mismatch",
            `## review_scope subagent_review_count must match unique listed ids: ${count} != ${uniqueIds.size}`,
        ));
    }
}

function subagentReviewIds(value) {
    const matches = value.match(/\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/gi);
    return matches || [];
}

function validateMergeApproval(decisionSection, summarySection, policySection, implementationSection, errors) {
    if (!/\bMERGE_APPROVED\b/.test(decisionSection)) {
        return;
    }
    if (!isNoWorkValue(fieldValue(summarySection, "blockers"))) {
        errors.push(reviewError("approved_with_blockers", "MERGE_APPROVED responses must have no blockers"));
    }
    if (!isNoWorkValue(fieldValue(summarySection, "questions"))) {
        errors.push(reviewError("approved_with_questions", "MERGE_APPROVED responses must have no open questions"));
    }
    if (!isAffirmative(fieldValue(summarySection, "approve"))) {
        errors.push(reviewError("missing_approval_summary", "MERGE_APPROVED responses must have an affirmative approve summary"));
    }
    for (const [sectionName, section] of [
        ["policy/spec", policySection],
        ["implementation/test", implementationSection],
    ]) {
        const classification = fieldValue(section, "classification");
        if (/^Blocker\b/.test(classification)) {
            errors.push(reviewError(
                "approved_with_blocker_classification",
                `MERGE_APPROVED responses must not leave ## ${sectionName} classified as Blocker`,
            ));
        }
        if (/^Question\b/.test(classification)) {
            errors.push(reviewError(
                "approved_with_question_classification",
                `MERGE_APPROVED responses must not leave ## ${sectionName} classified as Question`,
            ));
        }
    }
}

function validateReviewDoesNotAcceptWarnings(section, errors) {
    const approve = fieldValue(section, "approve");
    const unexecuted = fieldValue(section, "unexecuted_verification");
    if (/\byes\b|\btrue\b|はい|承認/i.test(approve) && isPlaceholderValue(unexecuted)) {
        errors.push(reviewError(
            "weak_approval",
            "approved review responses must explicitly state unexecuted_verification",
        ));
    }
}

function isPlaceholderValue(value) {
    const normalized = value.trim();
    return normalized === ""
        || normalized === "-"
        || normalized === "TODO"
        || normalized === "TBD"
        || normalized === "<fill>"
        || normalized === "<command>";
}

function isNoWorkValue(value) {
    return /^(0|none|なし|無し|no|not-applicable|n\/a)\b/i.test(value.trim());
}

function isAffirmative(value) {
    return /\b(yes|true|approved)\b|はい|承認/i.test(value.trim());
}

function recordIncludesAny(record, needles) {
    return needles.some((needle) => record.includes(needle));
}

function reviewError(code, message) {
    return { code, message };
}

function escapeRegExp(value) {
    return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
