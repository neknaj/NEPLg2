#!/usr/bin/env node
// nodesrc/issues.js
// 目的:
// - issues/items/*.md を単一の正とし、衝突しにくい issue ID の生成、検証、索引生成を行う。
// - 旧 doc/review20260425 の連番 Issue を legacy_id 付きの新 Issue へ移行する。

const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');

const ROOT = process.cwd();
const DEFAULT_ISSUES_DIR = path.join(ROOT, 'issues');
const ITEMS_DIRNAME = 'items';
const INDEX_JSON = 'index.json';
const INDEX_MD = 'index.md';
const SCHEMA = 'neplg2-issues/v1';

const REQUIRED_FIELDS = ['id', 'title', 'area', 'status', 'resolved', 'priority', 'type', 'created', 'updated'];
const VALID_STATUS = new Set(['open', 'investigating', 'fixed', 'verified', 'wontfix']);
const VALID_PRIORITY = new Set(['P0', 'P1', 'P2', 'P3']);
const VALID_TYPE = new Set(['bug', 'performance', 'architecture', 'test', 'doc', 'security', 'maintenance']);
const ID_RE = /^ISS-\d{8}T\d{6}(?:\d{3})?Z-[A-Z0-9][A-Z0-9-]{3,80}$/;

class UsageError extends Error {
    constructor(message) {
        super(message);
        this.name = 'UsageError';
    }
}

function usage() {
    return [
        'Usage:',
        '  node nodesrc/issues.js check [--dir issues]',
        '  node nodesrc/issues.js index [--dir issues]',
        '  node nodesrc/issues.js new --area <area> --title <title> [--priority P1] [--type architecture] [--target <path>] [--problem <text>] [--impact <text>] [--fix <text>] [--verify <text>]',
        '  node nodesrc/issues.js migrate-review20260425 [--dir issues] [--force]',
    ].join('\n');
}

function parseArgs(argv) {
    const command = argv[0] || '';
    const opts = { _: [] };
    for (let i = 1; i < argv.length; i += 1) {
        const arg = argv[i];
        if (arg === '--force') {
            opts.force = true;
            continue;
        }
        if (arg.startsWith('--')) {
            const key = arg.slice(2);
            if (i + 1 >= argv.length || argv[i + 1].startsWith('--')) {
                throw new UsageError(`${arg} requires a value`);
            }
            opts[key] = argv[i + 1];
            i += 1;
            continue;
        }
        opts._.push(arg);
    }
    return { command, opts };
}

function ensureDir(dir) {
    fs.mkdirSync(dir, { recursive: true });
}

function toPosixPath(value) {
    return String(value).replace(/\\/g, '/');
}

function todayUtc() {
    return new Date().toISOString().slice(0, 10);
}

function idTimestamp(date = new Date()) {
    const pad = (n, width = 2) => String(n).padStart(width, '0');
    return [
        date.getUTCFullYear(),
        pad(date.getUTCMonth() + 1),
        pad(date.getUTCDate()),
        'T',
        pad(date.getUTCHours()),
        pad(date.getUTCMinutes()),
        pad(date.getUTCSeconds()),
        pad(date.getUTCMilliseconds(), 3),
        'Z',
    ].join('');
}

function slugifyUpper(text, fallback = 'ISSUE') {
    const slug = String(text)
        .normalize('NFKD')
        .replace(/[^a-zA-Z0-9]+/g, '-')
        .replace(/^-+|-+$/g, '')
        .replace(/-+/g, '-')
        .toUpperCase();
    return slug || fallback;
}

function randomIssueId(title) {
    const stamp = idTimestamp();
    const slug = slugifyUpper(title).slice(0, 36);
    const suffix = crypto.randomBytes(4).toString('hex').toUpperCase();
    return `ISS-${stamp}-${slug}-${suffix}`;
}

function legacyIssueId(legacyId, title) {
    const legacySlug = slugifyUpper(legacyId, 'LEGACY');
    const titleHash = crypto.createHash('sha256').update(`${legacyId}\n${title}`).digest('hex').slice(0, 8).toUpperCase();
    return `ISS-20260425T000000Z-${legacySlug}-${titleHash}`;
}

function trimTrailingWhitespace(text) {
    return String(text)
        .replace(/\r\n/g, '\n')
        .replace(/\r/g, '\n')
        .split('\n')
        .map((line) => line.replace(/[ \t]+$/g, ''))
        .join('\n');
}

function yamlValue(value) {
    const raw = value == null ? '' : String(value);
    if (/^[A-Za-z0-9_.\/:-]+$/.test(raw)) {
        return raw;
    }
    return JSON.stringify(raw);
}

function issueFileName(id) {
    return `${id}.md`;
}

function formatIssue(meta, body) {
    const keys = [
        'id',
        'title',
        'area',
        'status',
        'resolved',
        'priority',
        'type',
        'created',
        'updated',
        'target',
        'legacy_id',
        'source',
    ];
    const lines = ['---'];
    for (const key of keys) {
        if (Object.prototype.hasOwnProperty.call(meta, key) && meta[key] !== undefined && meta[key] !== '') {
            lines.push(`${key}: ${yamlValue(meta[key])}`);
        }
    }
    lines.push('---');
    lines.push('');
    lines.push(trimTrailingWhitespace(body).replace(/\n*$/g, ''));
    lines.push('');
    return `${lines.join('\n')}`;
}

function parseFrontmatter(filePath) {
    const text = fs.readFileSync(filePath, 'utf8').replace(/\r\n/g, '\n');
    if (!text.startsWith('---\n')) {
        throw new Error(`${filePath}: missing frontmatter`);
    }
    const end = text.indexOf('\n---\n', 4);
    if (end < 0) {
        throw new Error(`${filePath}: unclosed frontmatter`);
    }
    const rawMeta = text.slice(4, end);
    const body = text.slice(end + 5);
    const meta = {};
    for (const line of rawMeta.split('\n')) {
        if (!line.trim()) continue;
        const m = line.match(/^([A-Za-z0-9_]+):\s*(.*)$/);
        if (!m) {
            throw new Error(`${filePath}: invalid frontmatter line: ${line}`);
        }
        const key = m[1];
        let value = m[2].trim();
        if (value.startsWith('"') || value.startsWith("'")) {
            try {
                value = JSON.parse(value);
            } catch {
                value = value.slice(1, -1);
            }
        }
        meta[key] = value;
    }
    return { meta, body };
}

function itemsDir(issuesDir) {
    return path.join(issuesDir, ITEMS_DIRNAME);
}

function listIssueFiles(issuesDir) {
    const dir = itemsDir(issuesDir);
    if (!fs.existsSync(dir)) return [];
    return fs.readdirSync(dir)
        .filter((name) => name.endsWith('.md'))
        .map((name) => path.join(dir, name))
        .sort();
}

function normalizeIssueRecord(issuesDir, filePath, meta) {
    return {
        id: meta.id,
        title: meta.title,
        area: meta.area,
        status: meta.status,
        resolved: String(meta.resolved) === 'true',
        priority: meta.priority,
        type: meta.type,
        created: meta.created,
        updated: meta.updated,
        target: meta.target || '',
        legacy_id: meta.legacy_id || '',
        source: meta.source || '',
        file: toPosixPath(path.relative(issuesDir, filePath)),
    };
}

function readIssues(issuesDir) {
    return listIssueFiles(issuesDir).map((filePath) => {
        const parsed = parseFrontmatter(filePath);
        return {
            ...normalizeIssueRecord(issuesDir, filePath, parsed.meta),
            body: parsed.body,
        };
    });
}

function validateIssue(filePath, meta) {
    const errors = [];
    for (const key of REQUIRED_FIELDS) {
        if (!Object.prototype.hasOwnProperty.call(meta, key) || String(meta[key]).trim() === '') {
            errors.push(`${filePath}: missing required field: ${key}`);
        }
    }
    if (meta.id && !ID_RE.test(meta.id)) {
        errors.push(`${filePath}: invalid id: ${meta.id}`);
    }
    if (meta.status && !VALID_STATUS.has(meta.status)) {
        errors.push(`${filePath}: invalid status: ${meta.status}`);
    }
    if (meta.priority && !VALID_PRIORITY.has(meta.priority)) {
        errors.push(`${filePath}: invalid priority: ${meta.priority}`);
    }
    if (meta.type && !VALID_TYPE.has(meta.type)) {
        errors.push(`${filePath}: invalid type: ${meta.type}`);
    }
    if (meta.resolved && !new Set(['true', 'false']).has(String(meta.resolved))) {
        errors.push(`${filePath}: resolved must be true or false`);
    }
    const expectedName = meta.id ? issueFileName(meta.id) : '';
    if (expectedName && path.basename(filePath) !== expectedName) {
        errors.push(`${filePath}: filename must be ${expectedName}`);
    }
    return errors;
}

function checkIssues(issuesDir) {
    const files = listIssueFiles(issuesDir);
    const errors = [];
    const ids = new Map();
    for (const filePath of files) {
        let parsed;
        try {
            parsed = parseFrontmatter(filePath);
        } catch (e) {
            errors.push(e.message);
            continue;
        }
        errors.push(...validateIssue(filePath, parsed.meta));
        if (parsed.meta.id) {
            if (ids.has(parsed.meta.id)) {
                errors.push(`${filePath}: duplicate id also used by ${ids.get(parsed.meta.id)}`);
            } else {
                ids.set(parsed.meta.id, filePath);
            }
        }
    }
    return { files, errors };
}

function issueSortKey(issue) {
    const resolvedRank = issue.resolved ? 1 : 0;
    const priorityRank = { P0: 0, P1: 1, P2: 2, P3: 3 }[issue.priority] ?? 9;
    return `${resolvedRank}:${priorityRank}:${issue.area}:${issue.id}`;
}

function buildIndex(issuesDir) {
    const issues = readIssues(issuesDir)
        .map(({ body, ...issue }) => issue)
        .sort((a, b) => issueSortKey(a).localeCompare(issueSortKey(b)));
    const counts = {
        total: issues.length,
        open: issues.filter((issue) => !issue.resolved).length,
        resolved: issues.filter((issue) => issue.resolved).length,
        by_area: {},
    };
    for (const issue of issues) {
        const bucket = counts.by_area[issue.area] || { total: 0, open: 0, resolved: 0 };
        bucket.total += 1;
        if (issue.resolved) bucket.resolved += 1;
        else bucket.open += 1;
        counts.by_area[issue.area] = bucket;
    }
    return {
        schema: SCHEMA,
        generated_at: new Date().toISOString(),
        counts,
        issues,
    };
}

function writeIndexFiles(issuesDir) {
    ensureDir(issuesDir);
    const index = buildIndex(issuesDir);
    fs.writeFileSync(path.join(issuesDir, INDEX_JSON), `${JSON.stringify(index, null, 2)}\n`);

    const lines = [
        '# Issue Index',
        '',
        `Generated by \`node nodesrc/issues.js index\`.`,
        '',
        '## Summary',
        '',
        '| Area | Total | Open | Resolved |',
        '|---|---:|---:|---:|',
    ];
    const areas = Object.keys(index.counts.by_area).sort();
    for (const area of areas) {
        const count = index.counts.by_area[area];
        lines.push(`| ${area} | ${count.total} | ${count.open} | ${count.resolved} |`);
    }
    lines.push(`| total | ${index.counts.total} | ${index.counts.open} | ${index.counts.resolved} |`);
    lines.push('');
    lines.push('## Open Issues');
    lines.push('');
    lines.push('| ID | Area | Priority | Type | Title | Legacy |');
    lines.push('|---|---|---|---|---|---|');
    for (const issue of index.issues.filter((item) => !item.resolved)) {
        lines.push(`| [${issue.id}](./${issue.file}) | ${issue.area} | ${issue.priority} | ${issue.type} | ${issue.title} | ${issue.legacy_id || ''} |`);
    }
    lines.push('');
    lines.push('## Resolved Issues');
    lines.push('');
    lines.push('| ID | Area | Status | Priority | Title | Legacy |');
    lines.push('|---|---|---|---|---|---|');
    for (const issue of index.issues.filter((item) => item.resolved)) {
        lines.push(`| [${issue.id}](./${issue.file}) | ${issue.area} | ${issue.status} | ${issue.priority} | ${issue.title} | ${issue.legacy_id || ''} |`);
    }
    fs.writeFileSync(path.join(issuesDir, INDEX_MD), `${lines.join('\n')}\n`);
    return index;
}

function extractSection(text, heading) {
    const re = new RegExp(`^### ${heading}\\s*\\n([\\s\\S]*?)(?=\\n### |\\n## |$)`, 'm');
    const m = text.match(re);
    return m ? m[1].trim() : '';
}

function cleanLegacyTarget(value) {
    return String(value)
        .replace(/`([^`]+)`/g, '$1')
        .replace(/^`|`$/g, '')
        .trim();
}

function parseLegacyReviewFile(filePath, area) {
    const text = fs.readFileSync(filePath, 'utf8').replace(/\r\n/g, '\n');
    const matches = [...text.matchAll(/^##\s+(RV-[A-Z]+-\d+):\s+(.+)$/gm)];
    const issues = [];
    for (let i = 0; i < matches.length; i += 1) {
        const match = matches[i];
        const start = match.index + match[0].length;
        const end = i + 1 < matches.length ? matches[i + 1].index : text.length;
        const body = text.slice(start, end).trim();
        const meta = {};
        for (const line of body.split('\n')) {
            const item = line.match(/^-\s+([^:]+):\s*(.*)$/);
            if (!item) continue;
            const key = item[1].trim();
            const value = item[2].trim();
            if (key === '解決済') meta.resolved = value;
            else if (key === '状態') meta.status = value;
            else if (key === '優先度') meta.priority = value;
            else if (key === '種別') meta.type = value;
            else if (key === '対象') meta.target = cleanLegacyTarget(value);
        }
        const legacyId = match[1];
        const title = match[2].trim();
        const sourceRel = toPosixPath(path.relative(ROOT, filePath));
        issues.push({
            id: legacyIssueId(legacyId, title),
            title,
            area,
            status: meta.status || 'open',
            resolved: meta.resolved === 'true' ? 'true' : 'false',
            priority: meta.priority || 'P2',
            type: meta.type || 'maintenance',
            target: meta.target || '',
            legacy_id: legacyId,
            source: `${sourceRel}#${legacyId.toLowerCase()}`,
            problem: extractSection(body, '問題'),
            impact: extractSection(body, '影響'),
            fix: extractSection(body, '修正方針'),
            verify: extractSection(body, '検証'),
            originalBody: body,
        });
    }
    return issues;
}

function bodyForLegacyIssue(issue) {
    const lines = [
        `# ${issue.legacy_id}: ${issue.title}`,
        '',
        `旧 \`doc/review20260425\` から移行した Issue。新しい正の ID は \`${issue.id}\`。`,
        '',
        '## 要約',
        '',
        issue.problem || issue.title,
        '',
        '## 影響',
        '',
        issue.impact || '旧レビュー本文を参照。',
        '',
        '## 修正方針',
        '',
        issue.fix || '旧レビュー本文を参照。',
        '',
        '## 検証',
        '',
        issue.verify || '旧レビュー本文を参照。',
        '',
        '## 旧レビュー本文',
        '',
        issue.originalBody,
    ];
    return lines.join('\n');
}

function migrateReview20260425(issuesDir, force) {
    const reviewDir = path.join(ROOT, 'doc', 'review20260425');
    const sources = [
        ['core', path.join(reviewDir, 'core.md')],
        ['cli', path.join(reviewDir, 'cli.md')],
        ['stdlib', path.join(reviewDir, 'stdlib.md')],
        ['examples', path.join(reviewDir, 'examples.md')],
    ];
    ensureDir(itemsDir(issuesDir));
    let written = 0;
    for (const [area, filePath] of sources) {
        const issues = parseLegacyReviewFile(filePath, area);
        for (const issue of issues) {
            const meta = {
                id: issue.id,
                title: issue.title,
                area: issue.area,
                status: issue.status,
                resolved: issue.resolved,
                priority: issue.priority,
                type: issue.type,
                created: '2026-04-25',
                updated: '2026-04-26',
                target: issue.target,
                legacy_id: issue.legacy_id,
                source: issue.source,
            };
            const outPath = path.join(itemsDir(issuesDir), issueFileName(issue.id));
            if (fs.existsSync(outPath) && !force) {
                continue;
            }
            fs.writeFileSync(outPath, formatIssue(meta, bodyForLegacyIssue(issue)));
            written += 1;
        }
    }
    writeIndexFiles(issuesDir);
    return written;
}

function bodyForNewIssue(meta, opts) {
    const target = opts.target ? `\`${opts.target}\`` : '未確定';
    return [
        `# ${meta.id}: ${meta.title}`,
        '',
        '## 概要',
        '',
        opts.summary || opts.problem || meta.title,
        '',
        '## 対象',
        '',
        `- ${target}`,
        '',
        '## 根拠',
        '',
        opts.evidence || '- 未記入',
        '',
        '## 問題',
        '',
        opts.problem || '未記入',
        '',
        '## 影響',
        '',
        opts.impact || '未記入',
        '',
        '## 修正方針',
        '',
        opts.fix || '未記入',
        '',
        '## 検証',
        '',
        opts.verify || '未記入',
    ].join('\n');
}

function createIssue(issuesDir, opts) {
    if (!opts.area) throw new UsageError('--area is required');
    if (!opts.title) throw new UsageError('--title is required');
    const now = todayUtc();
    const meta = {
        id: '',
        title: opts.title,
        area: opts.area,
        status: opts.status || 'open',
        resolved: opts.resolved || 'false',
        priority: opts.priority || 'P2',
        type: opts.type || 'maintenance',
        created: opts.created || now,
        updated: opts.updated || now,
        target: opts.target || '',
        source: opts.source || '',
    };
    let outPath = '';
    for (let attempt = 0; attempt < 10; attempt += 1) {
        meta.id = randomIssueId(opts.title);
        outPath = path.join(itemsDir(issuesDir), issueFileName(meta.id));
        if (!fs.existsSync(outPath)) break;
    }
    if (fs.existsSync(outPath)) {
        throw new Error('failed to allocate a unique issue id after 10 attempts');
    }
    const validationErrors = validateIssue(outPath, meta);
    if (validationErrors.length > 0) {
        throw new UsageError(validationErrors.join('\n'));
    }
    ensureDir(path.dirname(outPath));
    fs.writeFileSync(outPath, formatIssue(meta, bodyForNewIssue(meta, opts)));
    writeIndexFiles(issuesDir);
    return outPath;
}

function main() {
    const { command, opts } = parseArgs(process.argv.slice(2));
    const issuesDir = path.resolve(opts.dir || DEFAULT_ISSUES_DIR);
    if (!command || command === '-h' || command === '--help') {
        console.log(usage());
        return;
    }
    if (command === 'migrate-review20260425') {
        const written = migrateReview20260425(issuesDir, Boolean(opts.force));
        const check = checkIssues(issuesDir);
        if (check.errors.length > 0) {
            console.error(check.errors.join('\n'));
            process.exit(1);
        }
        console.log(`migrated review20260425 issues: ${written}`);
        return;
    }
    if (command === 'new') {
        const filePath = createIssue(issuesDir, opts);
        console.log(`created issue: ${toPosixPath(path.relative(ROOT, filePath))}`);
        return;
    }
    if (command === 'index') {
        const index = writeIndexFiles(issuesDir);
        console.log(`indexed issues: total=${index.counts.total} open=${index.counts.open} resolved=${index.counts.resolved}`);
        return;
    }
    if (command === 'check') {
        const check = checkIssues(issuesDir);
        if (check.errors.length > 0) {
            console.error(check.errors.join('\n'));
            process.exit(1);
        }
        console.log(`issues check ok: files=${check.files.length}`);
        return;
    }
    throw new UsageError(`unknown command: ${command}`);
}

try {
    main();
} catch (e) {
    if (e instanceof UsageError) {
        console.error(e.message);
        console.error(usage());
        process.exit(2);
    }
    console.error(e && e.stack ? e.stack : String(e));
    process.exit(1);
}
