// nodesrc/util_paths.js
// 目的: Trunk の dist 出力位置（dist / web/dist）などの環境差を吸収する探索ユーティリティ。
//
// 想定:
// - GitHub Actions: リポジトリルートで trunk build -> ./dist
// - ローカル: web/ で trunk build -> ./web/dist
// - どちらでも動かすため、候補を列挙して最初に見つかったものを採用する。

const fs = require('node:fs');
const path = require('node:path');

function uniq(arr) {
    const out = [];
    const seen = new Set();
    for (const x of arr) {
        const k = String(x);
        if (seen.has(k)) continue;
        seen.add(k);
        out.push(x);
    }
    return out;
}

function isDir(p) {
    try {
        return fs.statSync(p).isDirectory();
    } catch {
        return false;
    }
}

function candidateDistDirs(hintDir) {
    // ユーザが明示指定したい場合
    const env = process.env.NEPL_DIST;

    const cwd = process.cwd();
    const here = __dirname;

    const base = [];

    if (env) {
        base.push(path.resolve(env));
    }

    if (hintDir) {
        const r = path.resolve(hintDir);
        base.push(r);
        base.push(path.join(r, 'dist'));
        base.push(path.join(r, 'web', 'dist'));

        // hint が dist そのもののケース
        if (path.basename(r) === 'dist') {
            base.push(path.join(path.dirname(r), 'web', 'dist'));
        }
        // hint が web/dist のケース
        if (r.endsWith(path.join('web', 'dist'))) {
            base.push(path.join(path.dirname(path.dirname(r)), 'dist'));
        }
    }

    // 実行ディレクトリ由来
    base.push(path.join(cwd, 'dist'));
    base.push(path.join(cwd, 'web', 'dist'));
    base.push(path.join(cwd, '..', 'dist'));
    base.push(path.join(cwd, '..', 'web', 'dist'));

    // nodesrc 配下からの実行
    base.push(path.join(here, '..', 'dist'));
    base.push(path.join(here, '..', 'web', 'dist'));
    base.push(path.join(here, '..', '..', 'dist'));
    base.push(path.join(here, '..', '..', 'web', 'dist'));

    return uniq(base.map(p => path.resolve(p)));
}

function findFirstExistingDir(dirs) {
    for (const d of dirs) {
        if (isDir(d)) return d;
    }
    return null;
}

module.exports = {
    candidateDistDirs,
    findFirstExistingDir,
    isDir,
};
