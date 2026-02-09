// nodesrc/compiler_loader.js
// 目的: Trunk が出力した nepl-web-*.js と *_bg.wasm を探して Node から初期化し、コンパイラ API を返す。
//
// 注意:
// - nepl-web-*.js は wasm-bindgen 生成物（ESM）なので、CommonJS からは dynamic import() で読み込む。

const fs = require('node:fs');
const path = require('node:path');
const { pathToFileURL } = require('node:url');

function listFiles(dir) {
    try {
        return fs.readdirSync(dir);
    } catch {
        return [];
    }
}

function pickCompilerPair(distDir) {
    const files = listFiles(distDir);

    const jsCandidates = files.filter(f => /^nepl-web-.*\.js$/.test(f));
    const wasmCandidates = files.filter(f => /^nepl-web-.*_bg\.wasm$/.test(f));

    if (jsCandidates.length === 0 || wasmCandidates.length === 0) {
        return null;
    }

    // 同じプレフィックス（hash 部分）を優先して組にする
    // 例: nepl-web-xxxxx.js と nepl-web-xxxxx_bg.wasm
    for (const js of jsCandidates) {
        const m = js.match(/^(nepl-web-.*)\.js$/);
        if (!m) continue;
        const prefix = m[1];
        const want = prefix + '_bg.wasm';
        if (wasmCandidates.includes(want)) {
            return {
                jsPath: path.join(distDir, js),
                wasmPath: path.join(distDir, want),
                jsFile: js,
                wasmFile: want,
            };
        }
    }

    // 見つからなければ先頭同士で妥協
    return {
        jsPath: path.join(distDir, jsCandidates[0]),
        wasmPath: path.join(distDir, wasmCandidates[0]),
        jsFile: jsCandidates[0],
        wasmFile: wasmCandidates[0],
    };
}

async function loadCompilerFromDist(distDir) {
    const pair = pickCompilerPair(distDir);
    if (!pair) {
        throw new Error(`nepl-web-*.js / *_bg.wasm not found in dist: ${distDir}`);
    }

    const wasmBytes = fs.readFileSync(pair.wasmPath);
    const modUrl = pathToFileURL(pair.jsPath).href;

    // ESM を dynamic import
    const nepl = await import(modUrl);

    // wasm-bindgen init (object 形式)
    if (typeof nepl.initSync !== 'function') {
        throw new Error(`initSync not found in ${pair.jsFile}`);
    }
    nepl.initSync({ module: wasmBytes });

    return {
        api: nepl,
        meta: {
            distDir,
            jsFile: pair.jsFile,
            wasmFile: pair.wasmFile,
        },
    };
}

module.exports = {
    pickCompilerPair,
    loadCompilerFromDist,
};
