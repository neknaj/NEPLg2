#!/usr/bin/env node
// nodesrc/cli.js
// 目的:
// - -i で指定した入力ディレクトリ（複数可）を走査し、.n.md と .nepl のドキュメントを HTML 化して出力する。
//
// 使い方例:
//   node nodesrc/cli.js -i tutorials/getting_started -o html=dist/tutorials/getting_started
//   node nodesrc/cli.js -i stdlib/core -o html=dist/doc/stdlib/core

const fs = require('node:fs');
const path = require('node:path');
const http = require('node:http');
const https = require('node:https');
const { candidateDistDirs } = require('./util_paths');
const { findCompilerDistDir } = require('./compiler_loader');
const { buildEntriesFromAst } = require('./search');
const { runCases: runPlaygroundEditorCases } = require('./playground_editor_test_runner');
const { loadStdlibVfsFromFs: loadCachedStdlibVfsFromFs } = require('./stdlib_vfs_cache');
const DISCORD_WEBHOOK_USERNAME = 'NEPLg2 dev report';

let parserModuleCache = null;
let htmlGenModuleCache = null;
let htmlPlayModuleCache = null;

class UsageError extends Error {
    constructor(message) {
        super(message);
        this.name = 'UsageError';
    }
}

function getParserModule() {
    if (!parserModuleCache) {
        parserModuleCache = require('./parser');
    }
    return parserModuleCache;
}

function getHtmlGenModule() {
    if (!htmlGenModuleCache) {
        htmlGenModuleCache = require('./html_gen');
    }
    return htmlGenModuleCache;
}

function getHtmlPlayModule() {
    if (!htmlPlayModuleCache) {
        htmlPlayModuleCache = require('./html_gen_playground');
    }
    return htmlPlayModuleCache;
}

function parseArgs(argv) {
    const inputs = [];
    const outs = {};
    const excludeDirs = [];
    let siteName = 'NEPLg2';
    let descriptionPrefix = 'NEPLg2';
    let playgroundEditorTests = false;
    let discordMessage = null;
    let webhookUrl = process.env.NEPL_DISCORD_WEBHOOK_URL || process.env.DISCORD_WEBHOOK_URL || '';
    const positional = [];

    const requireValue = (flag, index) => {
        if (index + 1 >= argv.length) {
            throw new UsageError(`${flag} requires a value`);
        }
        return argv[index + 1];
    };

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === '-i') {
            inputs.push(requireValue(a, i));
            i += 1;
            continue;
        }
        if (a === '-o') {
            const kv = requireValue(a, i);
            const m = kv.match(/^([a-zA-Z0-9_]+)=(.*)$/);
            if (!m) {
                throw new UsageError(`-o expects key=value, got: ${kv}`);
            }
            outs[m[1]] = m[2];
            i += 1;
            continue;
        }
        if (a === '--exclude-dir' || a === '--exclude-dirname') {
            excludeDirs.push(requireValue(a, i));
            i += 1;
            continue;
        }
        if (a === '--site-name') {
            siteName = requireValue(a, i);
            i += 1;
            continue;
        }
        if (a === '--description-prefix') {
            descriptionPrefix = requireValue(a, i);
            i += 1;
            continue;
        }
        if (a === '--playground-editor-tests') {
            playgroundEditorTests = true;
            continue;
        }
        if (a === '--discord') {
            discordMessage = requireValue(a, i);
            i += 1;
            continue;
        }
        if (a === '--discord-webhook-url') {
            webhookUrl = requireValue(a, i);
            i += 1;
            continue;
        }
        if (a === '-h' || a === '--help') {
            return {
                help: true,
                inputs,
                outs,
                excludeDirs,
                siteName,
                descriptionPrefix,
                playgroundEditorTests,
                discordMessage,
                webhookUrl,
            };
        }
        if (!a.startsWith('-')) {
            positional.push(a);
            continue;
        }
        throw new UsageError(`unknown argument: ${a}`);
    }
    if (discordMessage === null && positional.length > 0 && inputs.length === 0 && Object.keys(outs).length === 0 && !playgroundEditorTests) {
        discordMessage = positional.join(' ');
    }
    return {
        help: false,
        inputs,
        outs,
        excludeDirs,
        siteName,
        descriptionPrefix,
        playgroundEditorTests,
        discordMessage,
        webhookUrl,
    };
}

function parseIntEnv(name, fallback) {
    const raw = process.env[name];
    if (raw === undefined) {
        return fallback;
    }
    const value = Number.parseInt(raw, 10);
    if (!Number.isFinite(value) || value <= 0) {
        return fallback;
    }
    return value;
}

function sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
}

function ensureWebhookUrl(rawUrl) {
    if (!rawUrl || String(rawUrl).trim().length === 0) {
        throw new Error('Discord webhook URL is not set. Set NEPL_DISCORD_WEBHOOK_URL (or DISCORD_WEBHOOK_URL) or pass --discord-webhook-url.');
    }
    let url;
    try {
        url = new URL(rawUrl);
    } catch (e) {
        throw new Error(`invalid Discord webhook URL: ${rawUrl}`);
    }
    const pathname = url.pathname;
    if (!/^\/api\/webhooks\/[^/]+\/[^/]+/.test(pathname)) {
        throw new Error(`invalid Discord webhook URL: ${rawUrl}`);
    }
    return url;
}

function splitDiscordMessage(message, maxChunkLength) {
    const source = String(message);
    if (source.length === 0) return [];
    const limit = Math.max(1, Math.floor(maxChunkLength));
    const chunks = [];
    let remaining = source;
    while (remaining.length > limit) {
        let cut = -1;
        let idx = remaining.lastIndexOf('\n', limit + 1);
        if (idx >= 1) {
            cut = idx;
        }
        if (cut < 0) {
            idx = remaining.lastIndexOf(' ', limit + 1);
            if (idx >= 1) {
                cut = idx;
            }
        }
        if (cut < 0) {
            cut = limit;
        }
        const head = remaining.slice(0, cut);
        const tail = remaining.slice(cut).replace(/^\s+/, '');
        chunks.push(head);
        remaining = tail;
    }
    if (remaining.length > 0) {
        chunks.push(remaining);
    }
    return chunks;
}

function parseRetryDelayMs(res, body) {
    const header = res.headers['retry-after'];
    if (header) {
        const parsed = Number.parseFloat(header);
        if (Number.isFinite(parsed) && parsed >= 0) {
            return Math.ceil(parsed * 1000);
        }
    }
    const bodyRetry = Number.parseFloat(body && body.retry_after);
    if (Number.isFinite(bodyRetry) && bodyRetry >= 0) {
        return Math.ceil(bodyRetry * 1000);
    }
    return 1000;
}

async function postDiscordChunk(url, content, attempt, maxAttempts, timeoutMs) {
    const payload = JSON.stringify({
        content,
        username: DISCORD_WEBHOOK_USERNAME,
        allowed_mentions: { parse: [] },
    });

    const requestUrl = url;
    const client = requestUrl.protocol === 'https:' ? https : http;
    const bodyChunks = [];

    const response = await new Promise((resolve, reject) => {
        const req = client.request(
            {
                method: 'POST',
                hostname: requestUrl.hostname,
                port: requestUrl.port || undefined,
                path: `${requestUrl.pathname}${requestUrl.search}`,
                headers: {
                    'Content-Type': 'application/json',
                    'Content-Length': Buffer.byteLength(payload),
                },
            },
            (res) => {
                res.setEncoding('utf8');
                res.on('data', (chunk) => bodyChunks.push(chunk));
                res.on('end', () => {
                    resolve({
                        statusCode: res.statusCode || 0,
                        headers: res.headers,
                        body: bodyChunks.join(''),
                    });
                });
            },
        );
        req.on('error', reject);
        req.setTimeout(timeoutMs, () => {
            req.destroy(new Error('Discord webhook request timeout'));
        });
        req.write(payload);
        req.end();
    });

    const responseBody = response.body.trim().length > 0 ? response.body : '';
    let responseJson = null;
    if (responseBody) {
        try {
            responseJson = JSON.parse(responseBody);
        } catch (e) {
            responseJson = { raw: responseBody };
        }
    }

    if (response.statusCode === 204 || (response.statusCode >= 200 && response.statusCode < 300)) {
        return responseJson || { status: 'ok' };
    }

    if ((response.statusCode === 429 || response.statusCode >= 500) && attempt < maxAttempts) {
        const retryAfterMs = parseRetryDelayMs(response, responseJson || {});
        await sleep(retryAfterMs);
        return postDiscordChunk(url, content, attempt + 1, maxAttempts, timeoutMs);
    }

    if (response.statusCode === 429) {
        const reason = responseJson && responseJson.message ? responseJson.message : 'rate limited';
        const global = responseJson && responseJson.global ? ' (global)' : '';
        throw new Error(`Discord webhook request failed with 429${global}: ${reason}`);
    }
    const message = responseJson && responseJson.message ? responseJson.message : responseBody || `${response.statusCode}`;
    throw new Error(`Discord webhook request failed with ${response.statusCode}: ${message}`);
}

async function sendDiscordMessage(message, webhookUrl) {
    const url = ensureWebhookUrl(webhookUrl);
    const maxChunkLength = parseIntEnv('NEPL_DISCORD_WEBHOOK_MESSAGE_MAX', 2000);
    const timeoutMs = parseIntEnv('NEPL_DISCORD_WEBHOOK_TIMEOUT_MS', 15000);
    const maxRetries = parseIntEnv('NEPL_DISCORD_WEBHOOK_RETRIES', 3);
    const chunks = splitDiscordMessage(message, maxChunkLength);
    if (chunks.length === 0) {
        throw new Error('Discord message must not be empty.');
    }
    const posted = [];
    for (let i = 0; i < chunks.length; i++) {
        const content = chunks[i];
        const result = await postDiscordChunk(url, content, 0, maxRetries, timeoutMs);
        posted.push({
            index: i + 1,
            total: chunks.length,
            contentLength: content.length,
            result,
        });
    }
    return {
        url,
        chunks: posted.length,
        pieces: posted,
    };
}

function ensureDir(p) {
    fs.mkdirSync(p, { recursive: true });
}

function copyFile(src, dst) {
    ensureDir(path.dirname(dst));
    fs.copyFileSync(src, dst);
}

function isFile(p) {
    try {
        return fs.statSync(p).isFile();
    } catch {
        return false;
    }
}

function isDir(p) {
    try {
        return fs.statSync(p).isDirectory();
    } catch {
        return false;
    }
}

function toPosixPath(p) {
    return String(p).replace(/\\/g, '/');
}

function loadStdlibVfsFromFs(stdlibRootDir) {
    return loadCachedStdlibVfsFromFs(stdlibRootDir, { missing: 'throw' });
}

function compileWithLocalStdlib(api, {
    entryPath = '/virtual/entry.nepl',
    source,
    vfs = {},
    stdlibRootDir = path.resolve(process.cwd(), 'stdlib'),
    profile = 'debug',
}) {
    const stdlibVfs = loadStdlibVfsFromFs(stdlibRootDir);
    if (typeof api.compile_source_with_vfs_stdlib_and_profile === 'function') {
        return api.compile_source_with_vfs_stdlib_and_profile(
            entryPath,
            source,
            vfs,
            stdlibVfs,
            profile,
        );
    }
    if (typeof api.compile_source_with_vfs_and_stdlib === 'function') {
        return api.compile_source_with_vfs_and_stdlib(
            entryPath,
            source,
            vfs,
            stdlibVfs,
        );
    }
    if (typeof api.compile_source_with_vfs_and_profile === 'function') {
        return api.compile_source_with_vfs_and_profile(
            entryPath,
            source,
            { ...stdlibVfs, ...vfs },
            profile,
        );
    }
    if (typeof api.compile_source_with_vfs === 'function') {
        return api.compile_source_with_vfs(entryPath, source, { ...stdlibVfs, ...vfs });
    }
    throw new Error('compiler API not found: compile_source_with_*');
}

function loadBundledStdlibVfs(api) {
    if (typeof api.get_bundled_stdlib_vfs === 'function') {
        return api.get_bundled_stdlib_vfs();
    }
    if (typeof api.get_stdlib_files === 'function') {
        const vfs = {};
        const entries = api.get_stdlib_files();
        for (const [rel, content] of entries) {
            vfs[`/stdlib/${toPosixPath(rel)}`] = String(content);
        }
        return vfs;
    }
    throw new Error('compiler API not found: get_bundled_stdlib_vfs/get_stdlib_files');
}

function walkFiles(root, excludeDirs) {
    const out = [];
    function rec(cur) {
        const ents = fs.readdirSync(cur, { withFileTypes: true });
        for (const e of ents) {
            const p = path.join(cur, e.name);
            if (e.isDirectory()) {
                if (excludeDirs && excludeDirs.includes(e.name)) continue;
                rec(p);
            }
            else if (e.isFile()) out.push(p);
        }
    }
    rec(root);
    return out;
}

function extractMarkdownForHtml(filePath) {
    const { parseFile } = getParserModule();
    const p = parseFile(filePath);
    if (p.kind === 'nmd') {
        // Strip YAML frontmatter if present
        return p.rawText.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n/, '');
    }
    if (p.kind === 'nepl') {
        // //: が無ければ、先頭の // ブロックを拾う（暫定）
        if (p.docText && p.docText.trim().length > 0) {
            return p.docText;
        }
        const lines = p.rawText.replace(/\r\n/g, '\n').split('\n');
        const head = [];
        for (const ln of lines) {
            const m = ln.match(/^\s*\/\/\s?(.*)$/);
            if (!m) break;
            head.push(m[1]);
        }
        return head.join('\n') + '\n';
    }
    return '';
}

async function runPlaygroundEditorCli(inputs, outPath) {
    const summary = await runPlaygroundEditorCases(inputs);
    const json = JSON.stringify(summary, null, 2);
    ensureDir(path.dirname(outPath));
    fs.writeFileSync(outPath, json);
    console.log(`generated json into ${outPath}`);
    console.log(`playground editor cases: ${summary.passedCount}/${summary.caseCount} passed`);
    if (summary.failedCount > 0) {
        process.exitCode = 1;
    }
}

async function main() {
    const {
        help,
        inputs,
        outs,
        excludeDirs,
        siteName,
        descriptionPrefix,
        playgroundEditorTests,
        discordMessage,
        webhookUrl,
    } = parseArgs(process.argv.slice(2));
    const hasHtml = Boolean(outs.html);
    const hasHtmlPlay = Boolean(outs.html_play);
    const hasJson = Boolean(outs.json);
    const hasDiscord = typeof discordMessage === 'string' && discordMessage.length > 0;
    const hasDiscordTarget = hasDiscord;

    if (help || (inputs.length === 0 && !hasDiscordTarget) || (!hasHtml && !hasHtmlPlay && !(playgroundEditorTests && hasJson) && !hasDiscordTarget)) {
        console.log('Usage: node nodesrc/cli.js -i <input_dir_or_file> [-i ...] -o html=<output_dir> [-o html_play=<output_dir>] [-o json=<output_file>] [--exclude-dir <name>] [--site-name <name>] [--description-prefix <prefix>] [--playground-editor-tests] [--discord <message>] [--discord-webhook-url <url>]');
        process.exit(help ? 0 : 2);
    }
    if (hasDiscordTarget) {
        if (hasHtml || hasHtmlPlay || playgroundEditorTests) {
            throw new Error('Discord mode cannot be used together with html/html_play or playground-editor-tests options.');
        }
        if (inputs.length > 0) {
            throw new Error('Discord mode cannot be used with -i inputs. Use only --discord or positional message text.');
        }
        const result = await sendDiscordMessage(discordMessage, webhookUrl);
        const maskedToken = `${result.url.pathname.split('/').slice(0, 4).join('/')}...`;
        console.log(`discord sent: chunks=${result.chunks}, url=${maskedToken}`);
        return;
    }
    if (playgroundEditorTests) {
        if (!hasJson) {
            throw new Error('--playground-editor-tests requires -o json=<output_file>');
        }
        await runPlaygroundEditorCli(inputs, path.resolve(outs.json));
        return;
    }

    const outRootHtml = hasHtml ? path.resolve(outs.html) : null;
    const outRootHtmlPlay = hasHtmlPlay ? path.resolve(outs.html_play) : null;
    if (outRootHtml) ensureDir(outRootHtml);
    if (outRootHtmlPlay) ensureDir(outRootHtmlPlay);
    const htmlPlayAssets = outRootHtmlPlay ? prepareHtmlPlayAssets(outRootHtmlPlay) : null;

    let count = 0;

    for (const input of inputs) {
        const inPath = path.resolve(input);
        if (isFile(inPath)) {
            const rel = path.basename(inPath);
            count += genOne(inPath, rel, outRootHtml, outRootHtmlPlay, htmlPlayAssets, null, { siteName, descriptionPrefix }, null);
            continue;
        }
        if (!isDir(inPath)) {
            console.error(`input not found: ${input}`);
            continue;
        }

        const files = walkFiles(inPath, excludeDirs).filter(p => p.endsWith('.n.md') || p.endsWith('.nepl') || p.endsWith('.md'));
        const tocEntries = buildTocEntries(inPath, files);
        const tocTitle = siteName.toLowerCase().includes('tutorial') ? 'Getting Started' : 'Contents';

        // スコープ全体の検索インデックスを事前ビルドする（ファイル横断検索用）
        const scopeSearchIndex = buildScopeSearchIndex(inPath, files, excludeDirs);

        for (const f of files) {
            const rel = path.relative(inPath, f);
            count += genOne(f, rel, outRootHtml, outRootHtmlPlay, htmlPlayAssets, tocEntries, { siteName, descriptionPrefix, tocTitle }, scopeSearchIndex);
        }
    }

    if (outRootHtml) {
        console.log(`generated html into ${outRootHtml}`);
    }
    if (outRootHtmlPlay) {
        console.log(`generated html_play into ${outRootHtmlPlay}`);
    }
    console.log(`generated ${count} html file(s)`);
}

function prepareHtmlPlayAssets(outRootHtmlPlay) {
    const candidates = candidateDistDirs(null);
    const found = findCompilerDistDir(candidates);
    if (!found || !found.pair) {
        const listed = candidates.map(d => `- ${d}`).join('\n');
        throw new Error(
            'html_play requires nepl-web compiler artifacts, but none were found.\n'
            + `searched:\n${listed}\n`
            + 'run trunk build first.'
        );
    }
    const { pair } = found;
    const jsOut = path.join(outRootHtmlPlay, pair.jsFile);
    const wasmOut = path.join(outRootHtmlPlay, pair.wasmFile);
    copyFile(pair.jsPath, jsOut);
    copyFile(pair.wasmPath, wasmOut);

    // wasm-bindgen 生成 JS が既定で参照する名前の互換ファイルも置く。
    const wasmCompatOut = path.join(outRootHtmlPlay, 'nepl-web_bg.wasm');
    if (path.basename(wasmOut) !== 'nepl-web_bg.wasm') {
        copyFile(pair.wasmPath, wasmCompatOut);
    }

    // playground_runtime.js と search.js をコピー
    const runtimeSrc = path.join(__dirname, 'static', 'playground_runtime.js');
    const runtimeOut = path.join(outRootHtmlPlay, 'playground_runtime.js');
    copyFile(runtimeSrc, runtimeOut);

    const searchSrc = path.join(__dirname, 'search.js');
    const searchOut = path.join(outRootHtmlPlay, 'search.js');
    copyFile(searchSrc, searchOut);

    const cssSrc = path.join(__dirname, 'static', 'playground.css');
    const cssOut = path.join(outRootHtmlPlay, 'playground.css');
    copyFile(cssSrc, cssOut);

    return {
        jsFile: pair.jsFile,
        wasmFile: pair.wasmFile,
        wasmCompatFile: 'nepl-web_bg.wasm',
        sourceDistDir: found.distDir,
        runtimeFile: 'playground_runtime.js',
        searchFile: 'search.js',
        cssFile: 'playground.css',
    };
}

/**
 * スコープ（入力ディレクトリ）全体の検索インデックスを構築する
 * 各ファイルの AST を解析しエントリを収集する
 * @param {string} inputRoot - 入力ディレクトリの絶対パス
 * @param {string[]} files - 対象ファイルパスの配列
 * @param {string[]} excludeDirs - 除外ディレクトリ名
 * @returns {object[]} SearchEntry[]
 */
function buildScopeSearchIndex(inputRoot, files, excludeDirs) {
    const { parseNmdAst } = getParserModule();
    const allEntries = [];
    for (const f of files) {
        try {
            const md = extractMarkdownForHtml(f);
            if (!md || md.trim().length === 0) continue;
            const ast = parseNmdAst(md);
            const relFilePath = toPosix(path.relative(inputRoot, f));
            const outRel = relFilePath
                .replace(/\.n\.md$/i, '.html')
                .replace(/\.nepl$/i, '.html')
                .replace(/\.md$/i, '.html');
            const pageTitle = readFirstHeadingTitle(f) || path.basename(f, path.extname(f));
            const entries = buildEntriesFromAst(ast, outRel, pageTitle, relFilePath);
            allEntries.push(...entries);
        } catch (e) {
            // 個別ファイルのエラーは無視してインデックス構築を続行
        }
    }
    return allEntries;
}

function humanizeDocName(outRel) {
    const base = path.basename(outRel, '.html');
    return base.replace(/^\d+[_-]?/, '').replace(/_/g, ' ');
}

function readFirstHeadingTitle(filePath) {
    try {
        const md = extractMarkdownForHtml(filePath);
        if (!md) return null;
        const lines = md.replace(/\r\n/g, '\n').split('\n');
        for (const ln of lines) {
            const t = ln.trim();
            if (!t) continue;
            const h = t.match(/^#\s+(.+?)\s*$/);
            if (h) return h[1].trim();
            break;
        }
        return null;
    } catch {
        return null;
    }
}

function toPosix(p) {
    return String(p).replace(/\\/g, '/');
}

function buildTocEntries(inputRoot, files) {
    const hasIndex = files.some(f => {
        const rel = toPosix(path.relative(inputRoot, f));
        return rel === 'index.n.md' || rel === '00_index.n.md';
    });
    const allOutRels = files.map(f => toPosix(path.relative(inputRoot, f))
        .replace(/\.n\.md$/i, '.html')
        .replace(/\.nepl$/i, '.html')
        .replace(/\.md$/i, '.html'))
        .filter(outRel => outRel !== 'index.html' && outRel !== '00_index.html');
    allOutRels.sort();

    let indexPath = path.join(inputRoot, 'index.n.md');
    let indexOutRel = 'index.html';
    if (!isFile(indexPath)) {
        indexPath = path.join(inputRoot, '00_index.n.md');
        indexOutRel = '00_index.html';
    }

    // Title map — built here so the flat-fallback can also use it
    const outRelToTitle = new Map();
    for (const f of files) {
        const outRel = toPosix(path.relative(inputRoot, f))
            .replace(/\.n\.md$/i, '.html')
            .replace(/\.nepl$/i, '.html')
            .replace(/\.md$/i, '.html');
        const title = readFirstHeadingTitle(f);
        if (title && title.length > 0) outRelToTitle.set(outRel, title);
    }

    if (!isFile(indexPath)) {
        // Group by first path segment (top-level directory)
        const byDir = new Map();
        const dirOrder = [];
        for (const outRel of allOutRels) {
            const parts = outRel.split('/');
            const dir = parts.length > 1 ? parts[0] : '.';
            if (!byDir.has(dir)) { byDir.set(dir, []); dirOrder.push(dir); }
            byDir.get(dir).push(outRel);
        }
        const result = [];
        if (hasIndex) {
            const idxLabel = outRelToTitle.get(indexOutRel) || 'Index';
            result.push({ outRel: indexOutRel, label: idxLabel, isGroup: false, depth: 0 });
        }
        for (const outRel of (byDir.get('.') || [])) {
            result.push({ outRel, label: outRelToTitle.get(outRel) || humanizeDocName(outRel), isGroup: false, depth: 0 });
        }
        for (const dir of dirOrder) {
            if (dir === '.') continue;
            result.push({ label: dir, isGroup: true, depth: 0 });
            for (const outRel of byDir.get(dir)) {
                result.push({ outRel, label: outRelToTitle.get(outRel) || humanizeDocName(outRel), isGroup: false, depth: 1 });
            }
        }
        return result;
    }

    const known = new Set(allOutRels);
    // outRelToTitle already populated above
    const used = new Set();
    const entries = [];
    const text = fs.readFileSync(indexPath, 'utf8').replace(/\r\n/g, '\n');
    const lines = text.split('\n');

    for (const ln of lines) {
        const h3 = ln.match(/^###\s+(.+)\s*$/);
        if (h3) {
            entries.push({
                label: h3[1].trim(),
                isGroup: true,
                depth: 0,
            });
            continue;
        }

        const item = ln.match(/^(\s*)-\s+\[([^\]]+)\]\(([^)]+)\)\s*$/);
        if (!item) continue;
        const indent = item[1] || '';
        const indexLabel = item[2].trim();
        const rawHref = item[3].trim();
        if (!rawHref || /^https?:\/\//i.test(rawHref)) continue;

        const outRel = toPosix(rawHref)
            .replace(/^\.\//, '')
            .replace(/\.n\.md$/i, '.html')
            .replace(/\.nepl$/i, '.html')
            .replace(/\.md$/i, '.html');
        if (!known.has(outRel)) continue;
        const label = outRelToTitle.get(outRel) || indexLabel;

        const depth = Math.floor(indent.length / 2) + 1;
        entries.push({
            outRel,
            label,
            isGroup: false,
            depth,
        });
        used.add(outRel);
    }

    const remaining = allOutRels.filter(r => !used.has(r));
    if (hasIndex) {
        const indexLabel = outRelToTitle.get(indexOutRel) || (indexOutRel === 'index.html' ? 'Index' : '00 index');
        entries.unshift({
            outRel: indexOutRel,
            label: indexLabel,
            isGroup: false,
            depth: 0,
        });
    }
    if (remaining.length > 0) {
        remaining.sort();
        let lastDir = null;
        for (const outRel of remaining) {
            const dir = path.dirname(outRel);
            if (dir !== lastDir) {
                // Hierarchical grouping for "remaining" files
                if (dir === '.') {
                    entries.push({ label: 'Other', isGroup: true, depth: 0 });
                } else {
                    const parts = dir.split('/');
                    for (let i = 0; i < parts.length; i++) {
                        const subDir = parts.slice(0, i + 1).join('/');
                        // We only add group if it's not the same as last one processed
                        // Actually, let's keep it simple for now: group by the deepest dir
                    }
                    entries.push({
                        label: dir,
                        isGroup: true,
                        depth: 0,
                    });
                }
                lastDir = dir;
            }
            entries.push({
                outRel,
                label: outRelToTitle.get(outRel) || humanizeDocName(outRel),
                isGroup: false,
                depth: 1,
            });
        }
    }

    return entries;
}

function makePageTocLinks(currentOutRel, tocEntries) {
    if (!Array.isArray(tocEntries) || tocEntries.length === 0) return [];
    const curDir = path.posix.dirname(toPosix(currentOutRel));
    return tocEntries.map(e => {
        if (e.isGroup || !e.outRel) {
            return {
                href: '',
                label: e.label,
                active: false,
                isGroup: true,
                depth: Number.isFinite(e.depth) ? e.depth : 0,
            };
        }
        const rel = path.posix.relative(curDir === '.' ? '' : curDir, e.outRel);
        return {
            href: rel === '' ? path.posix.basename(e.outRel) : rel,
            label: e.label,
            active: e.outRel === toPosix(currentOutRel),
            isGroup: false,
            depth: Number.isFinite(e.depth) ? e.depth : 0,
        };
    });
}

function inlinesToPlainText(inlines) {
    if (!Array.isArray(inlines)) return '';
    return inlines.map(n => {
        if (n.type === 'text') return n.text;
        if (n.type === 'code_inline') return n.text;
        if (n.type === 'math') return n.text;
        if (n.type === 'ruby') return inlinesToPlainText(n.base); // drop n.ruby
        if (n.type === 'gloss') return inlinesToPlainText(n.base); // drop n.notes
        if (n.type === 'link') return inlinesToPlainText(n.text);
        return '';
    }).join('').replace(/\s+/g, " ").trim();
}

function extractMetaFromAst(ast) {
    let title = '';
    let description = '';

    function visit(nodes) {
        for (const node of nodes) {
            if (!title && node.type === 'section' && node.level === 1) {
                title = inlinesToPlainText(node.heading);
            }
            if (!description && node.type === 'paragraph') {
                description = inlinesToPlainText(node.inlines);
            }
            if (title && description) return;

            if (node.type === 'section' || node.type === 'document') {
                if (Array.isArray(node.children)) {
                    visit(node.children);
                }
            }
            if (title && description) return;
        }
    }

    if (ast) {
        if (ast.type === 'document') visit(ast.children);
        else visit([ast]);
    }
    
    if (description) {
        description = description.replace(/\s+/g, ' ').trim();
        if (description.length > 300) {
            description = description.slice(0, 297) + '...';
        }
    }

    return { title, description };
}

function buildPageMeta(relPath, ast, { siteName, descriptionPrefix }) {
    const baseNoExt = path.basename(relPath).replace(/\.n\.md$/i, '').replace(/\.nepl$/i, '').replace(/\.md$/i, '');
    const extracted = extractMetaFromAst(ast);

    let title = `${siteName} - ${baseNoExt}`;
    if (extracted.title) {
        const prefixMatch = baseNoExt.match(/^(\d+)/);
        const prefix = prefixMatch ? prefixMatch[1] : baseNoExt;
        title = `${siteName} - ${prefix} - ${extracted.title}`;
    } else if (baseNoExt === '00_index' || baseNoExt === 'index') {
        title = siteName;
    }

    let description = `${descriptionPrefix}: ${baseNoExt}`;
    if (extracted.description) {
        description = `${descriptionPrefix} - ${extracted.description}`;
    }

    return { title, description };
}

function genOne(filePath, relPath, outRootHtml, outRootHtmlPlay, htmlPlayAssets, tocEntries, { siteName, descriptionPrefix, tocTitle }, scopeSearchIndex) {
    const { parseNmdAst } = getParserModule();
    const md = extractMarkdownForHtml(filePath);
    if (!md || md.trim().length === 0) {
        return 0;
    }

    const ast = parseNmdAst(md);
    const { title, description } = buildPageMeta(relPath, ast, { siteName, descriptionPrefix });

    const outRel = relPath
        .replace(/\.n\.md$/i, '.html')
        .replace(/\.nepl$/i, '.html')
        .replace(/\.md$/i, '.html');

    let wrote = 0;

    if (outRootHtml) {
        const { renderHtml } = getHtmlGenModule();
        const html = renderHtml(ast, { title, description, rewriteLinks: true });
        const outPath = path.join(outRootHtml, outRel);
        ensureDir(path.dirname(outPath));
        fs.writeFileSync(outPath, html);
        wrote += 1;
    }

    if (outRootHtmlPlay) {
        const { renderHtmlPlayground } = getHtmlPlayModule();
        if (!htmlPlayAssets || !htmlPlayAssets.jsFile) {
            throw new Error('internal error: html_play assets not prepared');
        }
        const depth = outRel.split('/').length - 1;
        const prefix = depth > 0 ? '../'.repeat(depth) : './';
        const runtimeJsPath = `${prefix}${htmlPlayAssets.runtimeFile}`;
        const searchJsPath = `${prefix}${htmlPlayAssets.searchFile}`;
        const playgroundCssPath = `${prefix}${htmlPlayAssets.cssFile}`;
        const moduleJsPath = `${prefix}${htmlPlayAssets.jsFile}`;
        const htmlPlay = renderHtmlPlayground(ast, {
            title,
            description,
            rewriteLinks: true,
            moduleJsPath,
            runtimeJsPath,
            searchJsPath,
            playgroundCssPath,
            tocLinks: makePageTocLinks(outRel, tocEntries),
            tocTitle,
            searchIndex: scopeSearchIndex || [],
            rootPrefix: prefix,
        });
        const outPathPlay = path.join(outRootHtmlPlay, outRel);
        ensureDir(path.dirname(outPathPlay));
        fs.writeFileSync(outPathPlay, htmlPlay);
        wrote += 1;
    }

    return wrote;
}

if (require.main === module) {
    Promise.resolve()
        .then(() => main())
        .catch((e) => {
            console.error(String(e?.message || e));
            process.exit(e instanceof UsageError ? 2 : 1);
        });
}

module.exports = {
    compileWithLocalStdlib,
    loadBundledStdlibVfs,
    loadStdlibVfsFromFs,
};
