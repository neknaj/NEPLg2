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
const { parseFile, parseNmdAst } = require('./parser');
const { renderHtml } = require('./html_gen');
const { renderHtmlPlayground } = require('./html_gen_playground');
const { candidateDistDirs } = require('./util_paths');
const { findCompilerDistDir } = require('./compiler_loader');

function parseArgs(argv) {
    const inputs = [];
    const outs = {};
    const excludeDirs = [];

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === '-i' && i + 1 < argv.length) {
            inputs.push(argv[++i]);
            continue;
        }
        if (a === '-o' && i + 1 < argv.length) {
            const kv = argv[++i];
            const m = kv.match(/^([a-zA-Z0-9_]+)=(.*)$/);
            if (!m) {
                throw new Error(`-o expects key=value, got: ${kv}`);
            }
            outs[m[1]] = m[2];
            continue;
        }
        if ((a === '--exclude-dir' || a === '--exclude-dirname') && i + 1 < argv.length) {
            excludeDirs.push(argv[++i]);
            continue;
        }
        if (a === '-h' || a === '--help') {
            return { help: true, inputs, outs, excludeDirs };
        }
    }
    return { help: false, inputs, outs, excludeDirs };
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
    const p = parseFile(filePath);
    if (p.kind === 'nmd') {
        return p.rawText;
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

function main() {
    const { help, inputs, outs, excludeDirs } = parseArgs(process.argv.slice(2));
    const hasHtml = Boolean(outs.html);
    const hasHtmlPlay = Boolean(outs.html_play);
    if (help || inputs.length === 0 || (!hasHtml && !hasHtmlPlay)) {
        console.log('Usage: node nodesrc/cli.js -i <input_dir_or_file> [-i ...] -o html=<output_dir> [-o html_play=<output_dir>] [--exclude-dir <name>]');
        process.exit(help ? 0 : 2);
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
            count += genOne(inPath, rel, outRootHtml, outRootHtmlPlay, htmlPlayAssets);
            continue;
        }
        if (!isDir(inPath)) {
            console.error(`input not found: ${input}`);
            continue;
        }

        const files = walkFiles(inPath, excludeDirs).filter(p => p.endsWith('.n.md') || p.endsWith('.nepl'));
        for (const f of files) {
            const rel = path.relative(inPath, f);
            count += genOne(f, rel, outRootHtml, outRootHtmlPlay, htmlPlayAssets);
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
    // 例: nepl-web-<hash>.js が内部で "nepl-web_bg.wasm" を fetch するケース。
    const wasmCompatOut = path.join(outRootHtmlPlay, 'nepl-web_bg.wasm');
    if (path.basename(wasmOut) !== 'nepl-web_bg.wasm') {
        copyFile(pair.wasmPath, wasmCompatOut);
    }
    return {
        jsFile: pair.jsFile,
        wasmFile: pair.wasmFile,
        wasmCompatFile: 'nepl-web_bg.wasm',
        sourceDistDir: found.distDir,
    };
}

function genOne(filePath, relPath, outRootHtml, outRootHtmlPlay, htmlPlayAssets) {
    const md = extractMarkdownForHtml(filePath);
    if (!md || md.trim().length === 0) {
        return 0;
    }

    const ast = parseNmdAst(md);
    const title = path.basename(filePath);

    const outRel = relPath
        .replace(/\.n\.md$/i, '.html')
        .replace(/\.nepl$/i, '.html');

    let wrote = 0;

    if (outRootHtml) {
        const html = renderHtml(ast, { title, rewriteLinks: true });
        const outPath = path.join(outRootHtml, outRel);
        ensureDir(path.dirname(outPath));
        fs.writeFileSync(outPath, html);
        wrote += 1;
    }

    if (outRootHtmlPlay) {
        if (!htmlPlayAssets || !htmlPlayAssets.jsFile) {
            throw new Error('internal error: html_play assets not prepared');
        }
        const depth = outRel.split('/').length - 1;
        const prefix = depth > 0 ? '../'.repeat(depth) : './';
        const moduleJsPath = `${prefix}${htmlPlayAssets.jsFile}`;
        const htmlPlay = renderHtmlPlayground(ast, {
            title,
            description: `${title} - NEPLg2 tutorial runnable document`,
            rewriteLinks: true,
            moduleJsPath,
        });
        const outPathPlay = path.join(outRootHtmlPlay, outRel);
        ensureDir(path.dirname(outPathPlay));
        fs.writeFileSync(outPathPlay, htmlPlay);
        wrote += 1;
    }

    return wrote;
}

if (require.main === module) {
    try {
        main();
    } catch (e) {
        console.error(String(e?.stack || e?.message || e));
        process.exit(1);
    }
}
