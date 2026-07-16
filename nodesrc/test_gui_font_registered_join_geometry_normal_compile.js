#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { loadCompilerFromDist } = require("./compiler_loader");
const { compileWithLocalStdlib } = require("./cli");

const testOnlyNames = [
    "gui_sfnt_simple_glyph_render_stroke_join_geometry_test_policy_matrix",
    "gui_sfnt_simple_glyph_render_stroke_closure_test_style_projection",
    "gui_sfnt_simple_glyph_render_stroke_test_neutral_line_side_edge",
    "gui_font_registered_face_simple_glyph_indexed_stroke_join_geometry_test_completed_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_force_push_failure",
    "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_normal_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_work_bound_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_coordinate_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_coverage_scan_test_quadratic_direction_contract",
];

const probe = (testOnlyName) => `#entry main
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
    for (const testOnlyName of testOnlyNames) {
        try {
            compileWithLocalStdlib(api, { source: probe(testOnlyName) });
        } catch (error) {
            const message = String(error?.message || error);
            if (message.includes("resolve.identifier.undefined") && message.includes(testOnlyName)) {
                continue;
            }
            throw error;
        }
        throw new Error(`F5nxp/F5nxq test-only helper unexpectedly compiled in normal mode: ${testOnlyName}`);
    }
    console.log("F5nxp/F5nxq normal compile excludes test-only join geometry helpers");
}

main().catch((error) => {
    console.error(String(error?.stack || error?.message || error));
    process.exitCode = 1;
});
