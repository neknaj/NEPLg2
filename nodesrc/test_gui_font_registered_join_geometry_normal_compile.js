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
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_frame_entry_bridge_test_normal_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_frame_entry_bridge_test_aggregation_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_frame_entry_bridge_test_prepare_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_batch_drain_test_continuation_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_batch_drain_test_complete_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_batch_drain_test_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_batch_range_bridge_test_normal_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_batch_range_bridge_test_entry_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_byte_storage_bridge_test_normal_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_byte_storage_bridge_test_entry_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_plan_bridge_test_normal_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_plan_bridge_test_entry_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_plan_bridge_test_tile_rows_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_payload_bridge_test_normal_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_payload_bridge_test_entry_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_payload_bridge_test_index_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_with_prepared_test_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_start_bridge_test_normal_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_start_bridge_test_entry_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_start_bridge_test_index_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_step_bridge_test_continuation_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_step_bridge_test_budget_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_step_bridge_test_entry_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_completed_bridge_test_completed_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_completed_bridge_test_pending_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_completed_bridge_test_budget_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_count_completed_bridge_test_entry_recovery_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_encode_seed_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_encode_cursor_bridge_test_success_contract",
];

const probe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "${testOnlyName.includes("tile_rle_encode_cursor_bridge_test") || testOnlyName.includes("tile_rle_encode_seed_bridge_test") || testOnlyName.includes("tile_rle_count_completed_bridge_test") || testOnlyName.includes("tile_rle_count_step_bridge_test") || testOnlyName.includes("tile_rle_count_start_bridge_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_count_step_test" : testOnlyName.includes("with_prepared_test_contract") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" : "alloc/gui/font"}" as *

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
    console.log("registered GUI font normal compile excludes test-only geometry and compositor bridge helpers");
}

main().catch((error) => {
    console.error(String(error?.stack || error?.message || error));
    process.exitCode = 1;
});
