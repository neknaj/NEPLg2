#!/usr/bin/env node
// nodesrc/convert_stdlib_tests.js
// 目的:
// - stdlib/tests/*.nepl（ドキュメントコメントなしのテスト用ソース）を
//   Node 側 doctest ランナーで扱える .n.md に変換する。
//
// 方針:
// - 元の .nepl を「そのまま」コードブロックに埋め込み、最低 1 つの neplg2:test を生成する。
// - 本文側に「[性質/せいしつ]（Property; /ˈprɑːpərti/）の説明」を書ける枠を用意し、
//   以後は .n.md で「なぜその性質が成り立つべきか」を文章で説明できるようにする。
//
// 使い方:
//   node nodesrc/convert_stdlib_tests.js -i stdlib/tests -o stdlib/tests
//
// 出力:
// - <入力>/<name>.nepl  ->  <出力>/<name>.n.md
//
// 注意:
// - 既存の .n.md がある場合は上書きしない（--force で上書き）。
// - 生成された .n.md は「雛形」なので、後からプロパティを追記して育てる想定。

const fs = require('node:fs');
const path = require('node:path');

function parseArgs(argv) {
    let inDir = '';
    let outDir = '';
    let force = false;

    for (let i = 0; i < argv.length; i++) {
        const a = argv[i];
        if (a === '-i' && i + 1 < argv.length) { inDir = argv[++i]; continue; }
        if (a === '-o' && i + 1 < argv.length) { outDir = argv[++i]; continue; }
        if (a === '--force') { force = true; continue; }
        if (a === '-h' || a === '--help') {
            return { help: true, inDir, outDir, force };
        }
    }
    return { help: false, inDir, outDir, force };
}

function isDir(p) {
    try { return fs.statSync(p).isDirectory(); } catch { return false; }
}

function ensureDir(p) {
    fs.mkdirSync(p, { recursive: true });
}

function listNeplFiles(dir) {
    const out = [];
    for (const e of fs.readdirSync(dir, { withFileTypes: true })) {
        if (!e.isFile()) continue;
        if (!e.name.endsWith('.nepl')) continue;
        out.push(path.join(dir, e.name));
    }
    out.sort();
    return out;
}

function escapeForMdFence(s) {
    // ``` を含むとフェンスが壊れるので最小限の回避
    return s.replace(/```/g, '``\\`');
}

function makeNmd(name, relNeplPath, srcText) {
    const body = escapeForMdFence(srcText.replace(/\r\n/g, '\n'));
    return [
        `# ${name} のテスト`,
        ``,
        `このファイルは、\`${relNeplPath}\`（元のテスト用ソース）から自動生成した .n.md です。`,
        ``,
        `ここでは、単なる「入出力が合う」だけでなく、コードが満たすべき[性質/せいしつ]（Property; /ˈprɑːpərti/）を文章で説明し、`,
        `それをテストコードで確認する形に寄せます。`,
        ``,
        `## このテストで確認したい性質（後で追記）`,
        ``,
        `- 例）[関数/かんすう] f が「単調増加」なら、a < b で f(a) <= f(b) になる`,
        `- 例）データ構造が「重複を許さない」なら、同じ要素を 2 回入れても要素数が増えない`,
        `- 例）[逆/ぎゃく]演算 inv が正しいなら、x に対して inv(inv(x)) == x`,
        ``,
        `## doctest`,
        ``,
        `neplg2:test`,
        '```neplg2',
        `// NOTE: ここには元の .nepl をそのまま貼っています。`,
        `// もし #entry main / #target wasi などが不足してコンパイルできない場合は、`,
        `// 元のファイル側に合わせて上部に追記してください。`,
        `// その際、本文では隠したい前置きは | 行を使うと便利です。`,
        body,
        '```',
        ``,
    ].join('\n');
}

function main() {
    const { help, inDir, outDir, force } = parseArgs(process.argv.slice(2));
    if (help || !inDir || !outDir) {
        console.log('Usage: node nodesrc/convert_stdlib_tests.js -i <stdlib/tests> -o <outdir> [--force]');
        process.exit(help ? 0 : 2);
    }

    const inAbs = path.resolve(inDir);
    const outAbs = path.resolve(outDir);

    if (!isDir(inAbs)) {
        console.error(`input dir not found: ${inDir}`);
        process.exit(2);
    }
    ensureDir(outAbs);

    const files = listNeplFiles(inAbs);
    let made = 0;

    for (const f of files) {
        const name = path.basename(f, '.nepl');
        const relNepl = path.relative(process.cwd(), f).replace(/\\/g, '/');
        const outPath = path.join(outAbs, name + '.n.md');

        if (!force && fs.existsSync(outPath)) {
            continue;
        }

        const src = fs.readFileSync(f, 'utf-8');
        const nmd = makeNmd(name, relNepl, src);
        fs.writeFileSync(outPath, nmd, 'utf-8');
        made++;
    }

    console.log(`generated ${made} .n.md file(s) into ${outAbs}`);
}

if (require.main === module) {
    try {
        main();
    } catch (e) {
        console.error(String(e?.stack || e?.message || e));
        process.exit(1);
    }
}
