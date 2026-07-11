#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { loadCompilerFromDist } = require("./compiler_loader");
const { compileWithLocalStdlib } = require("./cli");

const helperNames = [
    "gui_font_registered_face_simple_glyph_indexed_path_command_stream_test_force_span_lookup_failure",
    "gui_font_registered_face_simple_glyph_indexed_path_command_stream_test_force_tag_read_failure",
    "gui_font_registered_face_simple_glyph_indexed_path_command_stream_test_force_event_read_failure",
];

function probeSource(helperName) {
    return `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/path_command_stream" as *

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
            if (message.includes("resolve.identifier.undefined") && message.includes(helperName)) continue;
            throw error;
        }
        throw new Error(`F5nxi test-only helper unexpectedly compiled in normal mode: ${helperName}`);
    }
    console.log("F5nxi normal compile excludes all three #test entries");
}

main().catch((error) => {
    console.error(String(error?.stack || error?.message || error));
    process.exitCode = 1;
});
