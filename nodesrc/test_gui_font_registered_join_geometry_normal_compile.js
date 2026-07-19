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
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_writer_plan_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_storage_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_write_cursor_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_write_step_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_write_completed_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_encoded_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_packet_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_present_frame_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_present_run_cursor_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_present_run_step_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_completed_step_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_present_frame_recovery_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_command_cursor_start_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_command_cursor_step_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_command_cursor_run_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_command_cursor_end_frame_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_command_cursor_completed_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_terminal_record_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_begin_frame_record_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_begin_frame_virtual_drain_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_packed_mask_resource_software_drain_compositor_tile_rle_begin_frame_schedule_bridge_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_request_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_request_test_unsupported_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_dispatch_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_dispatch_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_dispatch_loop_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_dispatch_loop_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_execution_driver_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_execution_driver_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_executor_session_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_executor_session_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_completion_test_success_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_completion_test_unsupported_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_completion_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision_test_resume_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision_test_abort_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_yield_resume_test_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_host_action_yield_resume_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_next_command_test_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_next_command_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_record_test_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_record_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_schedule_test_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_schedule_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_end_frame_test_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_end_frame_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_end_frame_record_test_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_end_frame_schedule_test_contract",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_end_frame_schedule_test_owner",
    "gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_terminal_command_test_contract",
];

const probe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "${testOnlyName.includes("tile_rle_begin_frame_schedule_bridge_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_schedule_test" : testOnlyName.includes("tile_rle_begin_frame_virtual_drain_bridge_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_virtual_drain_test" : testOnlyName.includes("tile_rle_begin_frame_record_bridge_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_record_test" : testOnlyName.includes("tile_rle_terminal_record_bridge_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_terminal_record_test" : testOnlyName.includes("tile_rle_command_cursor_completed_bridge_test") || testOnlyName.includes("tile_rle_command_cursor_end_frame_bridge_test") || testOnlyName.includes("tile_rle_command_cursor_run_bridge_test") || testOnlyName.includes("tile_rle_command_cursor_step_bridge_test") || testOnlyName.includes("tile_rle_command_cursor_start_bridge_test") || testOnlyName.includes("tile_rle_present_frame_recovery_bridge_test") || testOnlyName.includes("tile_rle_completed_step_bridge_test") || testOnlyName.includes("tile_rle_present_run_step_bridge_test") || testOnlyName.includes("tile_rle_present_run_cursor_bridge_test") || testOnlyName.includes("tile_rle_present_frame_bridge_test") || testOnlyName.includes("tile_rle_packet_bridge_test") || testOnlyName.includes("tile_rle_encoded_bridge_test") || testOnlyName.includes("tile_rle_write_completed_bridge_test") || testOnlyName.includes("tile_rle_write_step_bridge_test") || testOnlyName.includes("tile_rle_write_cursor_bridge_test") || testOnlyName.includes("tile_rle_storage_bridge_test") || testOnlyName.includes("tile_rle_writer_plan_bridge_test") || testOnlyName.includes("tile_rle_encode_cursor_bridge_test") || testOnlyName.includes("tile_rle_encode_seed_bridge_test") || testOnlyName.includes("tile_rle_count_completed_bridge_test") || testOnlyName.includes("tile_rle_count_step_bridge_test") || testOnlyName.includes("tile_rle_count_start_bridge_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_count_step_test" : testOnlyName.includes("with_prepared_test_contract") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_source_contour" : "alloc/gui/font"}" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const hostRequestProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_request_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const dispatchProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_dispatch_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const dispatchLoopProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_dispatch_loop_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const hostExecutionDriverProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_execution_driver_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const hostActionExecutorSessionProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_executor_session_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const hostActionCompletionProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_completion_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const hostActionYieldResumeProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_host_action_yield_resume_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const resumedNextCommandProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_next_command_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const resumedRunRecordProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_run_record_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const resumedRunScheduleProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_run_schedule_test" as *

fn main %fn void i32 \\void:
    ${testOnlyName}
    0
`;

const resumedEndFrameProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "${testOnlyName.includes("resumed_terminal_command_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_terminal_command_test" : testOnlyName.includes("end_frame_schedule_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_end_frame_schedule_test" : testOnlyName.includes("end_frame_record_test") ? "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_end_frame_record_test" : "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_end_frame_test"}" as *

fn main %impure fn void i32 \\void:
    ${testOnlyName.includes("resumed_terminal_command_test_contract") ? `let evidence %i32 ${testOnlyName} unit` : testOnlyName}
    0
`;

const recoveredStateSchedulerDecisionProbe = (testOnlyName) => `#entry main
#indent 4
#target std
#import "alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_recovered_state_scheduler_decision_test" as *

fn main %impure fn void i32 \\void:
    let evidence %i32 ${testOnlyName} unit
    evidence
`;

async function main() {
    const distDir = path.resolve(__dirname, "..", "web", "dist");
    const { api } = await loadCompilerFromDist(distDir);
    const requestedName = process.argv[2];
    const selectedNames = requestedName === undefined ? testOnlyNames : testOnlyNames.filter((name) => name === requestedName);
    if (requestedName !== undefined && selectedNames.length === 0) {
        throw new Error(`unknown registered GUI font normal compile helper: ${requestedName}`);
    }
    for (const testOnlyName of selectedNames) {
        try {
            const source = testOnlyName.includes("recovered_state_scheduler_decision_test") ? recoveredStateSchedulerDecisionProbe(testOnlyName) : (testOnlyName.includes("tile_rle_begin_frame_resumed_end_frame_test") || testOnlyName.includes("tile_rle_begin_frame_resumed_end_frame_record_test") || testOnlyName.includes("tile_rle_begin_frame_resumed_end_frame_schedule_test") || testOnlyName.includes("tile_rle_begin_frame_resumed_terminal_command_test")) ? resumedEndFrameProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_resumed_run_schedule_test") ? resumedRunScheduleProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_resumed_run_record_test") ? resumedRunRecordProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_resumed_next_command_test") ? resumedNextCommandProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_host_action_yield_resume_test") ? hostActionYieldResumeProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_host_action_completion_test") ? hostActionCompletionProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_host_action_executor_session_test") ? hostActionExecutorSessionProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_host_execution_driver_test") ? hostExecutionDriverProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_dispatch_loop_test") ? dispatchLoopProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_dispatch_test") ? dispatchProbe(testOnlyName) : testOnlyName.includes("tile_rle_begin_frame_host_request_test") ? hostRequestProbe(testOnlyName) : probe(testOnlyName);
            compileWithLocalStdlib(api, { source });
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
