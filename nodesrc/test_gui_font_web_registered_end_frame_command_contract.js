#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const transition = read("stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_end_frame.nepl");
const adapter = read("stdlib/platforms/gui/web/font_registered_end_frame_command.nepl");

assert.match(transition, /fn gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_completion_step_end_frame[\s\S]*command_cursor_step_finish_owner run_step[\s\S]*command_cursor_step cursor/);
assert.doesNotMatch(transition, /pub fn gui_font_registered_face_simple_glyph_indexed_stroke_compositor_tile_rle_begin_frame_resumed_run_completion_step_end_frame/);
assert.match(transition, /run_schedule_owner_step_end_frame[\s\S]*SchedulePhase::Continue:[\s\S]*resumed_run_completion_step_end_frame state run_step/);
assert.doesNotMatch(transition, /pub (struct|fn) .*RunScheduleOwner.*new/);

assert.match(adapter, /run_execution_success_into_parts owner[\s\S]*completion_dispatch_loop_completion &completion[\s\S]*Completion::Continue state:[\s\S]*gui_font_web_registered_run_execution_step_end_frame state step/);
assert.match(adapter, /Completion::Yield state:[\s\S]*ContinueExpected[\s\S]*Completion::Completed state:[\s\S]*ContinueExpected/);
assert.doesNotMatch(adapter, /run_record_owner_into_schedule|schedule_step_record|schedule_only_step_into_host_request_pending|host_import_request|executor_step|#extern|while|queue|timer|fallback/);
assert.doesNotMatch(adapter, /impl (Clone|Copy) for GuiFontWebRegisteredEndFrameCommand/);
assert.match(adapter, /error_into_recovery[\s\S]*phase_error_into_parts[\s\S]*lower_error_into_parts/);

console.log("Web registered EndFrame command contract passed");
