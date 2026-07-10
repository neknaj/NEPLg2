#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { loadCompilerFromDist } = require("./compiler_loader");
const { compileWithLocalStdlib } = require("./cli");

const helperName = "gui_font_registered_face_simple_glyph_indexed_outline_storage_test_force_scalar_slot_storage_alloc_failed";
const source = `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face_simple_glyph_indexed_outline_storage" as *

fn main %fn void i32 \\void:
    ${helperName}
    0
`;

async function main() {
    const distDir = path.resolve(__dirname, "..", "web", "dist");
    const { api } = await loadCompilerFromDist(distDir);
    try {
        compileWithLocalStdlib(api, { source });
    } catch (error) {
        const message = String(error?.message || error);
        if (message.includes("resolve.identifier.undefined") && message.includes(helperName)) {
            console.log("F5nxc normal compile excludes #test entry");
            return;
        }
        throw error;
    }
    throw new Error("F5nxc test-only helper unexpectedly compiled in normal mode");
}

main().catch((error) => {
    console.error(String(error?.stack || error?.message || error));
    process.exitCode = 1;
});
