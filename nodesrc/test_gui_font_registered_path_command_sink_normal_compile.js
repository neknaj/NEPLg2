#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { loadCompilerFromDist } = require("./compiler_loader");
const { compileWithLocalStdlib } = require("./cli");

const privateNames = [
    "gui_font_registered_face_simple_glyph_indexed_path_command_sink_checked_add",
    "gui_font_registered_face_simple_glyph_indexed_path_command_sink_test_force_source_read_failure",
    "gui_font_registered_face_simple_glyph_indexed_path_command_sink_test_force_push_failure",
    "gui_font_registered_face_simple_glyph_indexed_path_command_sink_test_completed_force_push_failure_ok",
];

function probeSource(name) {
    return `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/path_command_sink" as *

fn main %fn void i32 \\void:
    ${name}
    0
`;
}

async function main() {
    const distDir = path.resolve(__dirname, "..", "web", "dist");
    const { api } = await loadCompilerFromDist(distDir);
    for (const name of privateNames) {
        try {
            compileWithLocalStdlib(api, { source: probeSource(name) });
        } catch (error) {
            const message = String(error?.message || error);
            if (message.includes("resolve.identifier.undefined") && message.includes(name)) continue;
            throw error;
        }
        throw new Error(`F5nxj private helper unexpectedly compiled in normal mode: ${name}`);
    }
    console.log("F5nxj normal compile excludes private plan/writer helpers");
}

main().catch((error) => {
    console.error(String(error?.stack || error?.message || error));
    process.exitCode = 1;
});
