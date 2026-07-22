#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_two_run_supplied_owner_test.nepl"), "utf8");
const fixture = fs.readFileSync(path.join(root, "stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_virtual_drain_test.nepl"), "utf8");
const executor = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_run_executor.nepl"), "utf8");
const phaseExecutor = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_run_phase_executor.nepl"), "utf8");
const handoff = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_run_phase_handoff.nepl"), "utf8");

assert.match(source, /two_run_fixture_test_completed 473/);
assert.match(source, /begin_frame_virtual_drain_test_owner_from_completed completed/);
assert.match(source, /virtual_drain_owner_into_schedule/);
assert.match(source, /schedule_owner_into_host_request/);
assert.match(source, /host_request_owner_into_dispatch/);
assert.match(source, /dispatch_owner_into_loop/);
assert.match(source, /dispatch_loop_owner_into_host_execution_driver/);
assert.match(source, /host_execution_driver_owner_into_host_action_executor_session_request/);
assert.match(source, /host_action_executor_session_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window/);
assert.match(source, /retry_pending_decide retry GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCompositorTileRleBeginFrameHostActionRetryBudget::OneRemaining/);
assert.match(source, /gui_font_web_registered_begin_frame_retry_execute retry_owner/);
assert.match(source, /GuiFontWebRegisteredBeginFrameRetrySuccessPhaseOwner::Yield/);
assert.match(source, /begin_frame_retry_yield_scheduler_decide owner GuiFontWebRegisteredBeginFrameRetryYieldSchedulerDecision::ResumeSlice/);
assert.match(source, /begin_frame_retry_yield_resume_and_step_next_command pending/);
assert.match(source, /retry_yield_next_command_execute_run &host &policy GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Offscreen next/);
assert.match(source, /run_execution_success_into_phase success/);
assert.match(source, /run_success_continue_step_next_command owner/);
assert.match(source, /GuiFontWebRegisteredRunNextCommandPhaseOwner::Run owner/);
assert.match(source, /row_tile_rle_run_pixel_offset &run 15/);
assert.match(source, /row_tile_rle_run_pixel_count &run 1/);
assert.match(source, /run_phase_handoff_execute host policy/);
assert.match(source, /RunPhaseHandoffResult::Continue continuation:[\s\S]*run_success_phase_continue_owner_free continuation/);
assert.match(source, /RunPhaseHandoffResult::Yield continuation:[\s\S]*run_success_phase_yield_owner_free continuation/);
assert.match(source, /RunPhaseHandoffResult::Completed completed:[\s\S]*run_success_phase_completed_owner_free completed/);
assert.doesNotMatch(executor, /#import "platforms\/gui\/web\/font_registered_run_next_command"/);
assert.equal((executor.match(/gui_font_web_registered_run_record_execute/g) || []).length, 2);
assert.match(executor, /run_record_owner_into_schedule policy record_owner[\s\S]*run_schedule_owner_into_host_request host scheduled[\s\S]*host_action_executor_session_request session[\s\S]*gui_web_compositor_host_executor_step support action_pending/);
assert.equal((phaseExecutor.match(/run_next_command_phase_run_owner_into_parts owner/g) || []).length, 1);
assert.equal((phaseExecutor.match(/gui_font_web_registered_run_record_execute/g) || []).length, 1);
assert.equal((handoff.match(/run_next_command_phase_run_execute host policy support owner/g) || []).length, 1);
assert.match(handoff, /pub enum GuiFontWebRegisteredRunPhaseHandoffResult:\s*\n\s*Continue %GuiFontWebRegisteredRunSuccessPhaseContinueOwner\s*\n\s*Yield %GuiFontWebRegisteredRunSuccessPhaseYieldOwner\s*\n\s*Completed %GuiFontWebRegisteredRunSuccessPhaseCompletedOwner\s*\n\s*Failed %GuiFontWebRegisteredRunExecutionError/);
assert.match(handoff, /let phase %GuiFontWebRegisteredRunSuccessPhaseOwner gui_font_web_registered_run_execution_success_into_phase success[\s\S]*RunSuccessPhaseOwner::Continue continuation: GuiFontWebRegisteredRunPhaseHandoffResult::Continue continuation[\s\S]*RunSuccessPhaseOwner::Yield continuation: GuiFontWebRegisteredRunPhaseHandoffResult::Yield continuation[\s\S]*RunSuccessPhaseOwner::Completed completed: GuiFontWebRegisteredRunPhaseHandoffResult::Completed completed/);
assert.doesNotMatch(handoff, /field::get|state_initial|command_cursor_step |RetryBudget::|Result GuiFontWebRegisteredRunSuccessPhaseOwner/);
assert.match(source, /present_frame_descriptor_expected_run_count &descriptor 2/);
assert.match(source, /present_frame_descriptor_expected_pixel_count &descriptor 16/);
assert.doesNotMatch(source, /GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCompositorTileRleBeginFrameHostActionRetryReadyOwner [A-Z]/);
assert.match(fixture, /owner_from_completed[\s\S]*completed_owner_into_compositor_tile_rle_begin_frame_record_budget completed surface frame_config tile_config 0 64/);

process.stdout.write("registered two-run supplied owner contract passed\n");
