#!/usr/bin/env node
"use strict";

function stripNeplComments(src) {
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
}

function implementationLineCount(src) {
    return stripNeplComments(src)
        .split(/\r?\n/)
        .filter((line) => line.trim().length > 0)
        .length;
}

function escapeRegExp(value) {
    return String(value).replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function typeExprPattern(typeExpr) {
    return String(typeExpr)
        .trim()
        .split(/\s+/)
        .map(escapeRegExp)
        .join("\\s+");
}

function typeAnnotationPattern(typeExpr) {
    return `%${typeExprPattern(typeExpr)}`;
}

function fnTypePattern(params, result, options = {}) {
    const effect = options.effect === "impure" ? "impure\\s+fn" : "fn";
    const pieces = [`%${effect}`];
    const normalizedParams = params.length === 0 ? ["()"] : params;
    for (const param of normalizedParams) {
        pieces.push(typeExprPattern(param));
        pieces.push(effect);
    }
    pieces.pop();
    pieces.push(typeExprPattern(result));
    return pieces.join("\\s+");
}

function fnSignaturePattern(name, params, result, options = {}) {
    const visibility = options.public === true
        ? "pub\\s+"
        : options.public === false
            ? ""
            : "(?:pub\\s+)?";
    const shadow = options.noshadow === true ? "noshadow\\s+" : "";
    return `\\b${visibility}fn\\s+${shadow}${escapeRegExp(name)}\\s+${fnTypePattern(params, result, options)}`;
}

function structFieldPattern(name, typeExpr) {
    return `\\b${escapeRegExp(name)}\\s+${typeAnnotationPattern(typeExpr)}`;
}

const TYPE_ARITY = new Map([
    ["Option", 1],
    ["Result", 2],
    ["Vec", 1],
    ["MemPtr", 1],
    ["RegionToken", 1],
    ["OwnedBuffer", 1],
    ["VecStorage", 1],
    ["VecStorageInvariant", 1],
    ["VecPushError", 1],
    ["VecReplaceError", 1],
    ["VecTransformError", 1],
    ["VecPartition", 1],
    ["VecSortMergeError", 1],
    ["BTreeMap", 2],
    ["BTreeMapInsertError", 2],
    ["BTreeSet", 1],
    ["HashMap", 2],
    ["HashSet", 1],
    ["BinaryHeap", 1],
    ["BinaryHeapPushError", 1],
    ["BinaryHeapPop", 1],
    ["BloomFilter", 2],
    ["CountingBloomFilter", 2],
    ["Queue", 1],
    ["QueuePushError", 1],
    ["QueuePop", 1],
    ["Deque", 1],
    ["DequePushError", 1],
    ["DequePop", 1],
    ["RingBuffer", 1],
    ["RingBufferPushError", 1],
    ["RingBufferPop", 1],
    ["Stack", 1],
    ["StackPushError", 1],
    ["StackPop", 1],
    ["List", 1],
    ["ListPushError", 1],
    ["ListTransformError", 1],
    ["SparseSetUpdateError", 0],
    ["SegmentTreeUpdateError", 0],
    ["DisjointSetUnionError", 0],
]);

function legacyTypeSyntaxView(src) {
    return stripNeplComments(src)
        .split(/\n/)
        .map(convertLegacyTypeLine)
        .join("\n");
}

function convertLegacyTypeLine(line) {
    const fnLine = line.match(/^(\s*(?:pub\s+)?fn\s+(?:noshadow\s+)?[A-Za-z_][A-Za-z0-9_:]*(?:\s+<.*>)?\s+)%(.+?)(\s+\\.*:)$/);
    if (fnLine) {
        return `${fnLine[1]}<${legacyTypeExpr(fnLine[2])}> ${legacyLambdaParams(fnLine[3])}:`;
    }

    const fieldLine = line.match(/^(\s+[A-Za-z_][A-Za-z0-9_:]*\s+)%(.+)$/);
    if (fieldLine) {
        return `${fieldLine[1]}<${legacyTypeExpr(fieldLine[2])}>`;
    }

    return line.replace(/\blet(\s+mut)?\s+([A-Za-z_][A-Za-z0-9_]*)\s+%([^\s;]+(?:\s+[^\s;]+)*)/, (match, mut, name, tail) => {
        const parsed = parseLegacyTypeTokens(tokenizeTypeExpr(tail), 0);
        if (!parsed || parsed.next === 0) {
            return match;
        }
        const tokens = tokenizeTypeExpr(tail);
        const rest = tokens.slice(parsed.next).join(" ");
        return `let${mut || ""} ${name} <${parsed.text}>${rest ? ` ${rest}` : ""}`;
    });
}

function legacyLambdaParams(text) {
    const trimmed = text.trim();
    if (trimmed === "\\():") {
        return "()";
    }
    const params = [...trimmed.matchAll(/\\([A-Za-z_][A-Za-z0-9_]*)/g)].map((match) => match[1]);
    return `(${params.join(",")})`;
}

function legacyTypeExpr(typeExpr) {
    const tokens = tokenizeTypeExpr(typeExpr);
    const parsed = parseLegacyTypeTokens(tokens, 0);
    return parsed && parsed.next === tokens.length ? parsed.text : tokens.join(" ");
}

function tokenizeTypeExpr(typeExpr) {
    return String(typeExpr).trim().split(/\s+/).filter((token) => token.length > 0);
}

function parseLegacyTypeTokens(tokens, index) {
    if (index >= tokens.length) {
        return null;
    }

    const token = tokens[index];
    if (token === "impure" && tokens[index + 1] === "fn") {
        return parseLegacyFnType(tokens, index + 2, "impure");
    }
    if (token === "fn") {
        return parseLegacyFnType(tokens, index + 1, "pure");
    }
    if (token.startsWith("&") && token.length > 1) {
        const parsed = parseLegacyTypeTokens([token.slice(1), ...tokens.slice(index + 1)], 0);
        return parsed ? { text: `&${parsed.text}`, next: index + parsed.next } : null;
    }

    const arity = TYPE_ARITY.get(token);
    if (arity === undefined || arity === 0) {
        return { text: token, next: index + 1 };
    }

    const args = [];
    let next = index + 1;
    for (let i = 0; i < arity; i += 1) {
        const parsed = parseLegacyTypeTokens(tokens, next);
        if (!parsed) {
            return { text: token, next: index + 1 };
        }
        args.push(parsed.text);
        next = parsed.next;
    }
    return { text: `${token}<${args.join(",")}>`, next };
}

function parseLegacyFnType(tokens, index, effect) {
    const marker = effect === "impure" ? "*>" : "->";
    const params = [];
    let next = index;
    while (next < tokens.length) {
        const param = parseLegacyTypeTokens(tokens, next);
        if (!param) {
            return null;
        }
        next = param.next;
        if ((effect === "impure" && tokens[next] === "impure" && tokens[next + 1] === "fn") || (effect === "pure" && tokens[next] === "fn")) {
            params.push(param.text);
            next += effect === "impure" ? 2 : 1;
            continue;
        }
        const result = parseLegacyTypeTokens(tokens, next);
        if (!result) {
            return null;
        }
        const oldParams = param.text === "()" && params.length === 0 ? "" : [...params, param.text].join(",");
        return { text: `(${oldParams})${marker}${result.text}`, next: result.next };
    }
    return null;
}

module.exports = {
    stripNeplComments,
    implementationLineCount,
    legacyTypeSyntaxView,
    typeExprPattern,
    typeAnnotationPattern,
    fnTypePattern,
    fnSignaturePattern,
    structFieldPattern,
};
