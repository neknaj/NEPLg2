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
const { parseFile } = require('./parser');
const { markdownToHtml, wrapHtml } = require('./html_gen');

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
    if (help || inputs.length === 0 || !outs.html) {
        console.log('Usage: node nodesrc/cli.js -i <input_dir_or_file> [-i ...] -o html=<output_dir> [--exclude-dir <name>]');
        process.exit(help ? 0 : 2);
    }

    const outRoot = path.resolve(outs.html);
    ensureDir(outRoot);

    let count = 0;

    for (const input of inputs) {
        const inPath = path.resolve(input);
        if (isFile(inPath)) {
            const rel = path.basename(inPath);
            count += genOne(inPath, rel, outRoot);
            continue;
        }
        if (!isDir(inPath)) {
            console.error(`input not found: ${input}`);
            continue;
        }

        const files = walkFiles(inPath, excludeDirs).filter(p => p.endsWith('.n.md') || p.endsWith('.nepl'));
        for (const f of files) {
            const rel = path.relative(inPath, f);
            count += genOne(f, rel, outRoot);
        }
    }

    console.log(`generated ${count} html file(s) into ${outRoot}`);
}

function genOne(filePath, relPath, outRoot) {
    const md = extractMarkdownForHtml(filePath);
    if (!md || md.trim().length === 0) {
        return 0;
    }

    const body = markdownToHtml(md);
    const title = path.basename(filePath);
    const html = wrapHtml(title, body);

    const outRel = relPath
        .replace(/\.n\.md$/i, '.html')
        .replace(/\.nepl$/i, '.html');

    const outPath = path.join(outRoot, outRel);
    ensureDir(path.dirname(outPath));
    fs.writeFileSync(outPath, html);
    return 1;
}

if (require.main === module) {
    try {
        main();
    } catch (e) {
        console.error(String(e?.stack || e?.message || e));
        process.exit(1);
    }
}
