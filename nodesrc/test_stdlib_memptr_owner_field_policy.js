#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");
const stdlibRoot = path.join(repoRoot, "stdlib");

const transitionalFields = new Map([
    [
        "stdlib/core/mem/types.nepl::RegionToken.ptr::MemPtr<.T>",
        "core owner token before compiler-issued OwnedRegion replaces forgeable RegionToken construction",
    ],
    [
        "stdlib/alloc/collections/vec/types.nepl::Vec.data::MemPtr<.T>",
        "Vec backing storage until OwnedBuffer<T> carries the free obligation",
    ],
    [
        "stdlib/alloc/io/bytebuf.nepl::ByteBuf.ptr::Option<MemPtr<u8>>",
        "ByteBuf owned bytes until OwnedBytes replaces Option<MemPtr<u8>>",
    ],
    [
        "stdlib/alloc/io/bytebuilder/types.nepl::ByteBuilder.ptr::Option<MemPtr<u8>>",
        "ByteBuilder owned bytes until OwnedBytesBuilder replaces Option<MemPtr<u8>>",
    ],
]);

const observedFields = collectMemPtrStructFields(stdlibRoot);
const observedKeys = observedFields.map((field) => field.key);
const allowedKeys = [...transitionalFields.keys()];

const unexpected = observedFields.filter((field) => !transitionalFields.has(field.key));
if (unexpected.length > 0) {
    fail(
        "New direct MemPtr struct fields must not be introduced as owner state. " +
            "Use an explicit owner type such as OwnedRegion/OwnedBuffer, or update the migration design first.\n" +
            unexpected.map(formatObservedField).join("\n"),
    );
}

const stale = allowedKeys.filter((key) => !observedKeys.includes(key));
if (stale.length > 0) {
    fail(
        "MemPtr owner-field migration allowlist is stale. Remove resolved transitional entries instead of keeping dead exceptions.\n" +
            stale.map((key) => `- ${key}: ${transitionalFields.get(key)}`).join("\n"),
    );
}

console.log(`stdlib MemPtr owner-field migration policy ok (${observedFields.length} transitional field(s))`);

function collectMemPtrStructFields(root) {
    const fields = [];
    for (const relPath of walkNeplFiles(root)) {
        const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
        let currentStruct = null;
        const lines = source.split("\n");
        for (let i = 0; i < lines.length; i += 1) {
            const rawLine = lines[i];
            const line = rawLine.replace(/\s*\/\/.*$/, "");
            if (/^\s*$/.test(line)) {
                continue;
            }

            const structMatch = line.match(/^\s*(?:pub\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)(?:<[^>\n]+>)?:\s*$/);
            if (structMatch) {
                currentStruct = structMatch[1];
                continue;
            }

            if (currentStruct && /^\S/.test(line)) {
                currentStruct = null;
            }

            if (!currentStruct) {
                continue;
            }

            const fieldMatch = line.match(/^\s+([A-Za-z_][A-Za-z0-9_]*)\s+<(.+)>\s*$/);
            if (!fieldMatch) {
                continue;
            }

            const fieldName = fieldMatch[1];
            const fieldType = normalizeType(fieldMatch[2]);
            if (!/\bMemPtr\s*</.test(fieldType)) {
                continue;
            }

            fields.push({
                relPath,
                line: i + 1,
                structName: currentStruct,
                fieldName,
                fieldType,
                key: `${relPath}::${currentStruct}.${fieldName}::${fieldType}`,
            });
        }
    }
    return fields;
}

function walkNeplFiles(root, out = []) {
    for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
        const fullPath = path.join(root, entry.name);
        if (entry.isDirectory()) {
            walkNeplFiles(fullPath, out);
            continue;
        }
        if (!entry.isFile() || !entry.name.endsWith(".nepl")) {
            continue;
        }
        out.push(path.relative(repoRoot, fullPath).replace(/\\/g, "/"));
    }
    return out.sort();
}

function normalizeType(typeText) {
    return typeText.replace(/\s+/g, "");
}

function formatObservedField(field) {
    return `- ${field.relPath}:${field.line} ${field.structName}.${field.fieldName} <${field.fieldType}>`;
}

function fail(message) {
    console.error(message);
    process.exit(1);
}
