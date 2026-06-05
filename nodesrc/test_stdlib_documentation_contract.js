#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");

const ROOT = path.resolve(__dirname, "..");
const STDLIB_ROOTS = [
    path.join(ROOT, "stdlib", "core"),
    path.join(ROOT, "stdlib", "alloc"),
    path.join(ROOT, "stdlib", "std"),
];

const BASELINE = {
    files: 456,
    moduleNoDoc: 0,
    moduleNoDoctest: 293,
    declarations: 2488,
    declarationNoDoc: 161,
    declarationNoDoctest: 1651,
    publicDeclarationNoDoctest: 1498,
    privateDeclarationNoDoctest: 153,
};

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function walkNeplFiles(dir) {
    const files = [];
    if (!fs.existsSync(dir)) {
        return files;
    }
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const child = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...walkNeplFiles(child));
        } else if (entry.isFile() && entry.name.endsWith(".nepl")) {
            files.push(child);
        }
    }
    return files;
}

function toRepoPath(filePath) {
    return path.relative(ROOT, filePath).split(path.sep).join("/");
}

function hasDoctest(docLines) {
    return docLines.some((line) => /\bneplg2:test\b/.test(line));
}

function moduleDocLines(lines) {
    for (let index = 0; index < Math.min(lines.length, 40); index += 1) {
        if (declarationAt(lines[index])) {
            return [];
        }
        if (!lines[index].trimStart().startsWith("//:")) {
            continue;
        }
        const doc = [];
        for (let cursor = index; cursor < lines.length; cursor += 1) {
            if (!lines[cursor].trimStart().startsWith("//:")) {
                break;
            }
            doc.push(lines[cursor]);
        }
        return doc;
    }
    return [];
}

function precedingDocLines(lines, index) {
    let cursor = index - 1;
    while (cursor >= 0 && lines[cursor].trim() === "") {
        cursor -= 1;
    }
    const doc = [];
    while (cursor >= 0 && lines[cursor].trimStart().startsWith("//:")) {
        doc.push(lines[cursor]);
        cursor -= 1;
    }
    return doc.reverse();
}

function declarationAt(line) {
    return line.match(/^\s*(?:pub\s+)?(fn|struct|enum|trait)\s+([A-Za-z_][A-Za-z0-9_]*)\b/);
}

function indentOf(line) {
    const match = line.match(/^(\s*)/);
    return match ? match[1].length : 0;
}

function implHeaderAt(line) {
    return line.match(/^\s*impl(?:\b|<)/);
}

const stats = {
    files: 0,
    moduleNoDoc: 0,
    moduleNoDoctest: 0,
    declarations: 0,
    declarationNoDoc: 0,
    declarationNoDoctest: 0,
    publicDeclarationNoDoctest: 0,
    privateDeclarationNoDoctest: 0,
};
const samples = [];

function sample(message) {
    if (samples.length < 40) {
        samples.push(message);
    }
}

for (const filePath of STDLIB_ROOTS.flatMap(walkNeplFiles).sort()) {
    stats.files += 1;
    const text = fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
    const lines = text.split("\n");
    const moduleDoc = moduleDocLines(lines);
    if (moduleDoc.length === 0) {
        stats.moduleNoDoc += 1;
        sample(`${toRepoPath(filePath)}: module doc is missing`);
    } else if (!hasDoctest(moduleDoc)) {
        stats.moduleNoDoctest += 1;
    }

    let implBlockIndent = null;
    let traitBlockIndent = null;
    for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        const trimmed = line.trim();
        const indentation = indentOf(line);
        const startsImpl = implHeaderAt(line);
        const startsTraitDeclaration = /^\s*(?:pub\s+)?trait\b/.test(line);
        if (
            implBlockIndent !== null
            && trimmed !== ""
            && !trimmed.startsWith("//:")
            && indentation <= implBlockIndent
            && !startsImpl
        ) {
            implBlockIndent = null;
        }
        if (startsImpl) {
            implBlockIndent = indentation;
            continue;
        }
        if (
            traitBlockIndent !== null
            && trimmed !== ""
            && !trimmed.startsWith("//:")
            && indentation <= traitBlockIndent
            && !startsTraitDeclaration
        ) {
            traitBlockIndent = null;
        }
        if (traitBlockIndent !== null && !startsTraitDeclaration) {
            continue;
        }
        if (implBlockIndent !== null) {
            continue;
        }
        const declaration = declarationAt(line);
        if (!declaration) {
            continue;
        }
        stats.declarations += 1;
        const doc = precedingDocLines(lines, index);
        if (doc.length === 0) {
            stats.declarationNoDoc += 1;
            sample(
                `${toRepoPath(filePath)}:${index + 1}: ${declaration[1]} ${declaration[2]} doc is missing`,
            );
        } else if (!hasDoctest(doc)) {
            stats.declarationNoDoctest += 1;
            if (line.trimStart().startsWith("pub ")) {
                stats.publicDeclarationNoDoctest += 1;
            } else {
                stats.privateDeclarationNoDoctest += 1;
            }
        }
        if (declaration[1] === "trait") {
            traitBlockIndent = indentation;
        }
    }
}

assert(stats.files >= BASELINE.files, `stdlib file count decreased unexpectedly: ${stats.files} < ${BASELINE.files}`);
assert(
    stats.declarations >= BASELINE.declarations,
    `stdlib declaration count decreased unexpectedly: ${stats.declarations} < ${BASELINE.declarations}`,
);
assert(stats.moduleNoDoc === 0, `stdlib module docs must not be missing: ${stats.moduleNoDoc}`);
assert(
    stats.moduleNoDoctest <= BASELINE.moduleNoDoctest,
    `stdlib module doctest gaps increased: ${stats.moduleNoDoctest} > ${BASELINE.moduleNoDoctest}`,
);
assert(
    stats.declarationNoDoc <= BASELINE.declarationNoDoc,
    `stdlib declaration doc gaps increased: ${stats.declarationNoDoc} > ${BASELINE.declarationNoDoc}`,
);
assert(
    stats.declarationNoDoctest <= BASELINE.declarationNoDoctest,
    `stdlib declaration doctest gaps increased: ${stats.declarationNoDoctest} > ${BASELINE.declarationNoDoctest}`,
);
assert(
    stats.publicDeclarationNoDoctest <= BASELINE.publicDeclarationNoDoctest,
    `stdlib public declaration doctest gaps increased: ${stats.publicDeclarationNoDoctest} > ${BASELINE.publicDeclarationNoDoctest}`,
);
assert(
    stats.privateDeclarationNoDoctest <= BASELINE.privateDeclarationNoDoctest,
    `stdlib private declaration doctest gaps increased: ${stats.privateDeclarationNoDoctest} > ${BASELINE.privateDeclarationNoDoctest}`,
);

console.log("stdlib documentation contract baseline ok");
console.log(JSON.stringify(stats, null, 2));
if (samples.length > 0) {
    console.log("sample gaps:");
    for (const line of samples) {
        console.log(`- ${line}`);
    }
}
