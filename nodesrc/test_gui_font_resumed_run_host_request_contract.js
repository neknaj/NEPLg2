#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const dispatchLoop = read("stdlib/std/gui/compositor_tile_present_dispatch_loop.nepl");
const schedule = read("stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_run_schedule.nepl");
const hostRequest = read("stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_run_host_request.nepl");
const webExecutor = read("stdlib/platforms/gui/web/font_registered_run_executor.nepl");
const webRuntime = read("nodesrc/test_gui_font_web_registered_run_executor_runtime.js");

assert.match(dispatchLoop, /GuiRgba8888CompositorTileRlePresentDispatchLoopScheduleOnlyStep:\s*\n\s*previous .*\n\s*state .*\n\s*phase .*\n\s*record /);
assert.match(dispatchLoop, /schedule_step_record[\s\S]*ScheduleOnlyStep state next phase record/);
assert.match(dispatchLoop, /schedule_only_step_into_host_request_pending[\s\S]*schedule_only_step_previous[\s\S]*schedule_only_step_record[\s\S]*host_import_request host record[\s\S]*pending_request_new previous next request post_phase/);
assert.doesNotMatch(dispatchLoop.match(/pub fn gui_rgba8888_compositor_tile_rle_present_dispatch_loop_schedule_only_step_into_host_request_pending[\s\S]*?(?=\n\/\/:|\n\n\/\/:)/)?.[0] || "", /schedule_step_record|dispatch_step_record|host_executor|#extern/);

assert.match(schedule, /run_record_owner_into_schedule[\s\S]*schedule_step_record policy state record/);
assert.match(schedule, /RunScheduleOwnerParts[\s\S]*step .*\n\s*scheduled /);
assert.doesNotMatch(schedule, /impl (Clone|Copy) for GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCompositorTileRleBeginFrameResumedRunScheduleOwner/);

assert.match(hostRequest, /run_schedule_owner_into_host_request[\s\S]*run_schedule_owner_into_parts owner[\s\S]*host_command_step_result &step[\s\S]*RunRecord[\s\S]*schedule_only_step_into_host_request_pending host scheduled/);
assert.doesNotMatch(hostRequest, /schedule_step_record|dispatch_step_record|pending_request_new|#extern|executor|while|queue|timer|fallback/);
assert.match(hostRequest, /owner_into_parts/);
assert.match(hostRequest, /error_into_parts/);
assert.match(hostRequest, /owner_free[\s\S]*command_cursor_step_free/);
assert.match(hostRequest, /error_free[\s\S]*command_cursor_step_free/);
assert.doesNotMatch(hostRequest, /impl (Clone|Copy) for GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCompositorTileRleBeginFrameResumedRunHostRequest/);

assert.match(webExecutor, /yield_next_command_owner_into_parts owner[\s\S]*next_command_owner_into_run_record next_owner[\s\S]*run_record_owner_into_schedule policy record_owner[\s\S]*run_schedule_owner_into_host_request host scheduled/);
assert.match(webExecutor, /host_execution_driver_prepare pending[\s\S]*executor_session_start driver[\s\S]*executor_session_request session[\s\S]*gui_web_compositor_host_executor_step support action_pending/);
assert.doesNotMatch(webExecutor, /dispatch_loop_step_record|schedule_only_step_into_host_request_pending|host_import_request host|pending_request_new|execute_action|#extern|while|queue|timer|fallback/);
assert.match(webExecutor, /GuiFontWebRegisteredRunExecutionSuccessParts/);
assert.match(webExecutor, /run_execution_error_free/);
assert.doesNotMatch(webExecutor, /impl (Clone|Copy) for GuiFontWebRegisteredRunExecution/);

assert.match(webRuntime, /compositor_tile_present_begin\(\)[\s\S]*return 0/);
assert.match(webRuntime, /compositor_tile_present_run\(\)[\s\S]*return 0/);
assert.match(webRuntime, /compositor_tile_present_end\(\)[\s\S]*return -1/);
assert.match(webRuntime, /begin: 1, run: 1, end: 0/);

console.log("registered resumed Run host request contract passed");
