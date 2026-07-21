#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const lower = fs.readFileSync(path.join(root, "stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_end_frame_host_request.nepl"), "utf8");
const adapter = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_end_frame_executor.nepl"), "utf8");

assert.match(lower, /resumed_end_frame_schedule_owner_into_host_request/);
assert.match(lower, /HostCommandRecord::EndFrame/);
assert.equal((lower.match(/schedule_only_step_into_host_request_pending/g) || []).length, 1);
assert.match(adapter, /end_frame_owner_into_record/);
assert.match(adapter, /end_frame_record_owner_into_schedule/);
assert.match(adapter, /end_frame_schedule_owner_into_host_request/);
assert.match(adapter, /gui_web_compositor_host_executor_step/);
assert.match(adapter, /pub enum GuiFontWebRegisteredEndFrameExecutionErrorRecovery/);
assert.match(adapter, /end_frame_execution_error_into_recovery/);
assert.match(adapter, /end_frame_execution_error_free/);
assert.doesNotMatch(adapter, /command_cursor_step_finish_owner|resumed_terminal_command|retry_budget_new/);
assert.doesNotMatch(adapter, /impl (Clone|Copy) for GuiFontWebRegisteredEndFrameExecution/);

console.log("Web registered EndFrame executor contract passed");
