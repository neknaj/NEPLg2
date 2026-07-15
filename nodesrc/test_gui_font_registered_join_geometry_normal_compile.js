#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { loadCompilerFromDist } = require("./compiler_loader");
const { compileWithLocalStdlib } = require("./cli");

const testOnlyName = "gui_sfnt_simple_glyph_render_stroke_join_geometry_test_policy_matrix";

const probe = `#entry main
#indent 4
#target std
#import "alloc/gui/font" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

async function main() {
    const distDir = path.resolve(__dirname, "..", "web", "dist");
    const { api } = await loadCompilerFromDist(distDir);
    try {
        compileWithLocalStdlib(api, { source: probe });
    } catch (error) {
        const message = String(error?.message || error);
        if (message.includes("resolve.identifier.undefined") && message.includes(testOnlyName)) {
            console.log("F5nxp normal compile excludes the test-only policy matrix");
            return;
        }
        throw error;
    }
    throw new Error(`F5nxp test-only policy matrix unexpectedly compiled in normal mode: ${testOnlyName}`);
}

main().catch((error) => {
    console.error(String(error?.stack || error?.message || error));
    process.exitCode = 1;
});
