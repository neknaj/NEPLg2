#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const lower = fs.readFileSync(path.join(root, "stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_resumed_terminal_command.nepl"), "utf8");
const adapter = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_terminal_completion.nepl"), "utf8");

assert.match(lower, /resumed_end_frame_step_into_terminal_command/);
assert.match(lower, /PresentCommand::EndFrame/);
assert.equal((lower.match(/command_cursor_step_finish_owner step/g) || []).length, 1);
assert.match(adapter, /end_frame_execution_success_into_parts success/);
assert.match(adapter, /DispatchLoopCompletion::Completed state/);
assert.match(adapter, /resumed_end_frame_step_into_terminal_command state step/);
assert.match(adapter, /resumed_terminal_parts_complete terminal_parts/);
assert.match(adapter, /pub enum GuiFontWebRegisteredTerminalCompletionErrorRecovery/);
assert.match(adapter, /terminal_completion_error_free/);
assert.doesNotMatch(adapter, /end_frame_schedule|host_request|session_complete|retry_budget_new/);
assert.doesNotMatch(adapter, /impl (Clone|Copy) for GuiFontWebRegisteredTerminalCompletion/);

console.log("Web registered terminal completion contract passed");
