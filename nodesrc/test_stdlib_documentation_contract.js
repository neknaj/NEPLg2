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
    files: 385,
    moduleNoDoc: 0,
    moduleNoDoctest: 309,
    declarations: 1745,
    declarationNoDoc: 547,
    declarationNoDoctest: 1032,
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

const stats = {
    files: 0,
    moduleNoDoc: 0,
    moduleNoDoctest: 0,
    declarations: 0,
    declarationNoDoc: 0,
    declarationNoDoctest: 0,
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

    for (let index = 0; index < lines.length; index += 1) {
        const declaration = declarationAt(lines[index]);
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
        }
    }
}

assert(stats.files >= BASELINE.files, `stdlib file count decreased unexpectedly: ${stats.files} < ${BASELINE.files}`);
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

console.log("stdlib documentation contract baseline ok");
console.log(JSON.stringify(stats, null, 2));
if (samples.length > 0) {
    console.log("sample gaps:");
    for (const line of samples) {
        console.log(`- ${line}`);
    }
}
