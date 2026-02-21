const assert = require('node:assert/strict');
const path = require('node:path');
const { candidateDistDirs } = require('../../nodesrc/util_paths');
const { loadCompilerFromCandidates } = require('../../nodesrc/compiler_loader');

async function loadApi(distHint = '') {
    const loaded = await loadCompilerFromCandidates(candidateDistDirs(distHint));
    return loaded.api;
}

function findFnDef(parseResult, name) {
    const items = parseResult?.module?.root?.items;
    if (!Array.isArray(items)) return null;
    return items.find((it) => it && it.kind === 'FnDef' && it.name === name) || null;
}

function getStmtExpr(stmt) {
    if (!stmt || typeof stmt !== 'object') return null;
    return stmt.expr && typeof stmt.expr === 'object' ? stmt.expr : null;
}

function firstSymbolDebug(expr) {
    if (!expr || !Array.isArray(expr.items) || expr.items.length === 0) return '';
    const first = expr.items[0];
    return String(first?.kind === 'Symbol' ? (first.debug || '') : '');
}

function collectExprsFromBlock(block, out = []) {
    const items = block?.items;
    if (!Array.isArray(items)) return out;
    for (const stmt of items) {
        const expr = getStmtExpr(stmt);
        if (!expr) continue;
        out.push(expr);
        if (Array.isArray(expr.items)) {
            for (const it of expr.items) {
                if (it?.kind === 'Block') {
                    collectExprsFromBlock(it.block, out);
                }
            }
        }
    }
    return out;
}

module.exports = {
    assert,
    loadApi,
    findFnDef,
    getStmtExpr,
    firstSymbolDebug,
    collectExprsFromBlock,
    path,
};
