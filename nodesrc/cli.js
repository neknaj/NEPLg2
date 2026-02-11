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
            count += genOne(inPath, rel, outRootHtml, outRootHtmlPlay, htmlPlayAssets, null);
            continue;
        }
        if (!isDir(inPath)) {
            console.error(`input not found: ${input}`);
            continue;
        }

        const files = walkFiles(inPath, excludeDirs).filter(p => p.endsWith('.n.md') || p.endsWith('.nepl'));
        const tocEntries = buildTocEntries(inPath, files);
        for (const f of files) {
            const rel = path.relative(inPath, f);
            count += genOne(f, rel, outRootHtml, outRootHtmlPlay, htmlPlayAssets, tocEntries);
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
    const hasIndex = files.some(f => toPosix(path.relative(inputRoot, f)) === '00_index.n.md');
    const allOutRels = files.map(f => toPosix(path.relative(inputRoot, f))
        .replace(/\.n\.md$/i, '.html')
        .replace(/\.nepl$/i, '.html'))
        .filter(outRel => outRel !== '00_index.html');
    allOutRels.sort();

    const indexPath = path.join(inputRoot, '00_index.n.md');
    if (!isFile(indexPath)) {
        const flat = allOutRels.map(outRel => ({
            outRel,
            label: humanizeDocName(outRel),
            isGroup: false,
            depth: 0,
        }));
        if (hasIndex) {
            flat.unshift({
                outRel: '00_index.html',
                label: '00 index',
                isGroup: false,
                depth: 0,
            });
        }
        return flat;
    }

    const known = new Set(allOutRels);
    const outRelToTitle = new Map();
    for (const f of files) {
        const outRel = toPosix(path.relative(inputRoot, f))
            .replace(/\.n\.md$/i, '.html')
            .replace(/\.nepl$/i, '.html');
        const title = readFirstHeadingTitle(f);
        if (title && title.length > 0) {
            outRelToTitle.set(outRel, title);
        }
    }
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
            .replace(/\.nepl$/i, '.html');
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
        const indexLabel = outRelToTitle.get('00_index.html') || '00 index';
        entries.unshift({
            outRel: '00_index.html',
            label: indexLabel,
            isGroup: false,
            depth: 0,
        });
    }
    if (remaining.length > 0) {
        entries.push({
            label: 'Other',
            isGroup: true,
            depth: 0,
        });
        for (const outRel of remaining) {
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
        if (n.type === 'ruby') return inlinesToPlainText(n.base);
        if (n.type === 'gloss') return inlinesToPlainText(n.base);
        if (n.type === 'link') return inlinesToPlainText(n.text);
        return '';
    }).join('');
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

function buildTutorialMeta(relPath, ast) {
    const baseNoExt = path.basename(relPath).replace(/\.n\.md$/i, '').replace(/\.nepl$/i, '');
    const extracted = extractMetaFromAst(ast);

    let title = `NEPLg2 tutorial - ${baseNoExt}`;
    if (extracted.title) {
        const prefixMatch = baseNoExt.match(/^(\d+)/);
        const prefix = prefixMatch ? prefixMatch[1] : baseNoExt;
        title = `NEPLg2 tutorial - ${prefix} - ${extracted.title}`;
    }

    let description = `NEPLg2 Getting Started tutorial: ${baseNoExt}`;
    if (extracted.description) {
        description = `NEPLg2 Getting Started tutorial - ${extracted.description}`;
    }

    return { title, description };
}

function genOne(filePath, relPath, outRootHtml, outRootHtmlPlay, htmlPlayAssets, tocEntries) {
    const md = extractMarkdownForHtml(filePath);
    if (!md || md.trim().length === 0) {
        return 0;
    }

    const ast = parseNmdAst(md);
    const { title, description } = buildTutorialMeta(relPath, ast);

    const outRel = relPath
        .replace(/\.n\.md$/i, '.html')
        .replace(/\.nepl$/i, '.html');

    let wrote = 0;

    if (outRootHtml) {
        const html = renderHtml(ast, { title, description, rewriteLinks: true });
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
        const tocLinks = makePageTocLinks(outRel, tocEntries);
        const htmlPlay = renderHtmlPlayground(ast, {
            title,
            description,
            rewriteLinks: true,
            moduleJsPath,
            tocLinks,
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
