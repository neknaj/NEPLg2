#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { loadCompilerFromDist } = require("./compiler_loader");
const { compileWithLocalStdlib } = require("./cli");

const helperNames = [
    "gui_font_registered_face_simple_glyph_indexed_point_x_test_force_item_read_failure",
    "gui_font_registered_face_simple_glyph_indexed_point_x_test_force_point_x_push_failure",
];

function probeSource(helperName) {
    return `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/point_x" as *

fn main %fn void i32 \\void:
    ${helperName}
    0
`;
}

async function main() {
    const distDir = path.resolve(__dirname, "..", "web", "dist");
    const { api } = await loadCompilerFromDist(distDir);
    for (const helperName of helperNames) {
        try {
            compileWithLocalStdlib(api, { source: probeSource(helperName) });
        } catch (error) {
            const message = String(error?.message || error);
            if (message.includes("resolve.identifier.undefined") && message.includes(helperName)) {
                continue;
            }
            throw error;
        }
        throw new Error(`F5nxe test-only helper unexpectedly compiled in normal mode: ${helperName}`);
    }
    console.log("F5nxe normal compile excludes both #test entries");
}

main().catch((error) => {
    console.error(String(error?.stack || error?.message || error));
    process.exitCode = 1;
});
