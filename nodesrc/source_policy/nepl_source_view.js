#!/usr/bin/env node
"use strict";

function stripNeplComments(src) {
    // Contract-oriented source policy checks often need to inspect executable
    // declarations without being confused by doctest prose. This helper removes
    // comment lines only from that inspection view; it must not be used to
    // enforce a limit on comments or discourage detailed documentation.
    return src
        .split(/\r?\n/)
        .filter((line) => !/^\s*\/\//.test(line))
        .join("\n");
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
    const normalizedParams = params.length === 0 ? ["void"] : params;
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

// Source policy checks assert ownership and API-boundary contracts, not parser
// behavior. This arity table lets the policy view normalize NEPLg2.1 prefix
// type annotations into the legacy shape those semantic assertions already use.
// It is deliberately limited to constructors that appear in policy-covered
// source; adding a constructor here means a policy now depends on its structure.
const TYPE_ARITY = new Map([
    ["Option", 1],
    ["Result", 2],
    ["Vec", 1],
    ["MemPtr", 1],
    ["RegionToken", 1],
    ["OwnedBuffer", 1],
    ["VecStorage", 1],
    ["VecStorageInvariant", 0],
    ["VecCopyInvariant", 0],
    ["VecCopyInvariantInvalid", 0],
    ["VecDataView", 1],
    ["VecPop", 1],
    ["VecPartition", 1],
    ["VecPushRejected", 1],
    ["VecPushError", 1],
    ["VecReplaceRejected", 1],
    ["VecReplaceError", 1],
    ["VecTransformError", 1],
    ["VecSortError", 1],
    ["VecReallocRegionError", 1],
    ["RegionReallocError", 1],
    ["BTreeMap", 2],
    ["BTreeMapStorage", 2],
    ["BTreeMapInsertError", 2],
    ["BTreeSet", 1],
    ["BTreeSetStorage", 1],
    ["BTreeSetInsertError", 1],
    ["HashMap", 3],
    ["HashMapStorage", 2],
    ["HashMapUpdateError", 3],
    ["HashSet", 2],
    ["HashSetStorage", 1],
    ["HashSetUpdateError", 2],
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

// Converts only surface type syntax for source policy assertions. It is not a
// replacement for the compiler parser: it preserves distinctions that matter to
// policy checks, such as pure versus impure function types, and leaves ordinary
// expressions in their original source form.
function legacyTypeSyntaxView(src) {
    let typeBodyIndent = null;
    return stripNeplComments(src)
        .split(/\n/)
        .map((line) => {
            const indent = leadingWhitespaceLength(line);
            if (typeBodyIndent !== null && line.trim().length > 0 && indent <= typeBodyIndent) {
                typeBodyIndent = null;
            }
            const converted = convertLegacyTypeLine(line, { allowField: typeBodyIndent !== null });
            if (/^\s*(?:pub\s+)?(?:struct|enum)\s+[A-Za-z_][A-Za-z0-9_:]*(?:<.*>)?:\s*$/.test(line)) {
                typeBodyIndent = indent;
            }
            return converted;
        })
        .join("\n");
}

function leadingWhitespaceLength(line) {
    return line.match(/^\s*/)[0].length;
}

function convertLegacyTypeLine(line, options = {}) {
    const fnLine = line.match(/^(\s*(?:pub\s+)?fn\s+(?:noshadow\s+)?[A-Za-z_][A-Za-z0-9_:]*(?:\s+<.*>)?\s+)%(.+?)(\s+\\.*:)$/);
    if (fnLine) {
        return `${fnLine[1]}<${legacyTypeExpr(fnLine[2])}> ${legacyLambdaParams(fnLine[3])}:`;
    }

    const fieldLine = line.match(/^(\s+[A-Za-z_][A-Za-z0-9_:]*\s+)%(.+)$/);
    if (options.allowField === true && fieldLine) {
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
    if (trimmed === "\\void:") {
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

    if (tokens[index] === "void") {
        const result = parseLegacyTypeTokens(tokens, index + 1);
        if (!result) {
            return null;
        }
        return {
            text: `()${marker}${result.text}`,
            next: result.next,
            fnEffect: effect,
            fnParams: [],
            fnResult: result.text,
        };
    }

    const firstParam = parseLegacyTypeTokens(tokens, index);
    if (!firstParam) {
        return null;
    }
    const result = parseLegacyTypeTokens(tokens, firstParam.next);
    if (!result) {
        return null;
    }

    const params = [firstParam.text];
    let resultText = result.text;
    if (result.fnEffect === effect && result.fnParams.length > 0) {
        params.push(...result.fnParams);
        resultText = result.fnResult;
    }

    return {
        text: `(${params.join(",")})${marker}${resultText}`,
        next: result.next,
        fnEffect: effect,
        fnParams: params,
        fnResult: resultText,
    };
}

module.exports = {
    stripNeplComments,
    legacyTypeSyntaxView,
    typeExprPattern,
    typeAnnotationPattern,
    fnTypePattern,
    fnSignaturePattern,
    structFieldPattern,
};
