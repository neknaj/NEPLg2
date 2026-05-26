#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const repoRoot = path.resolve(__dirname, "..");

const checks = [
    {
        relPath: "stdlib/alloc/collections/adjacency_matrix/storage.nepl",
        forbiddenPatterns: [/Vec<u8>/, /vec::(?:get|replace|filled|free)<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/adjacency_matrix/api/cleanup.nepl",
        forbiddenPatterns: [/Vec<u8>/, /vec::(?:get|replace|filled|free)<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/bitset/types.nepl",
        forbiddenPatterns: [/Vec<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/bitset/storage.nepl",
        forbiddenPatterns: [/Vec<u8>/, /vec::(?:get|replace|filled|free)<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/bitset/api/cleanup.nepl",
        forbiddenPatterns: [/Vec<u8>/, /vec::(?:get|replace|filled|free)<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/bloom_filter/storage.nepl",
        forbiddenPatterns: [/Vec<u8>/, /vec::(?:get|replace|filled|free)<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/bloom_filter/api.nepl",
        forbiddenPatterns: [/Vec<u8>/, /vec::(?:get|replace|filled|free)<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/counting_bloom_filter/storage.nepl",
        forbiddenPatterns: [/Vec<u8>/, /vec::(?:get|replace|filled|free)<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/counting_bloom_filter/api.nepl",
        forbiddenPatterns: [/Vec<u8>/, /vec::(?:get|replace|filled|free)<u8>/],
    },
    {
        relPath: "stdlib/alloc/collections/disjoint_set/storage.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/disjoint_set/query.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/disjoint_set/api/create.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/disjoint_set/api/cleanup.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/fenwick/storage.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/fenwick/api/create.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/fenwick/api/cleanup.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/sparse_set/types.nepl",
        forbiddenPatterns: [/Vec<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/sparse_set/storage.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/sparse_set/api/create.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/sparse_set/api/cleanup.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/segment_tree/layout.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/segment_tree/storage.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/segment_tree/api/create.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
    {
        relPath: "stdlib/alloc/collections/segment_tree/api/cleanup.nepl",
        forbiddenPatterns: [/Vec<i32>/, /vec::(?:get|replace|filled|free|len)<i32>/],
    },
];

const violations = [];

for (const check of checks) {
    const filePath = path.join(repoRoot, check.relPath);
    const lines = fs.readFileSync(filePath, "utf8").split(/\r?\n/);
    lines.forEach((line, index) => {
        for (const pattern of check.forbiddenPatterns) {
            if (pattern.test(line)) {
                violations.push(`${check.relPath}:${index + 1}: ${line.trim()}`);
            }
        }
    });
}

assert.deepEqual(
    violations,
    [],
    `NEPLg2.1 collection storage Vec cleanup must not reintroduce selected old type applications:\n${violations.join("\n")}`,
);

console.log("NEPLg2.1 collection storage Vec postfix cleanup regression passed");
