#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const source = fs.readFileSync(path.resolve(__dirname, "..", "stdlib/platforms/gui/web/font_registered_resumed_web_loop.nepl"), "utf8");
const body = source.match(/pub fn gui_font_web_registered_resumed_web_loop_execute[\s\S]*?(?=\npub fn)/)?.[0] || "";
const stages = [
    "yield_scheduler_decide",
    "yield_resume_and_step_next_command",
    "yield_next_command_execute_run",
    "run_execution_success_step_end_frame",
    "end_frame_command_execute",
    "end_frame_execution_into_terminal_completion",
];
let offset = -1;
for (const stage of stages) {
    assert.equal((body.match(new RegExp(stage, "g")) || []).length, 1, stage);
    const next = body.indexOf(stage);
    assert.ok(next > offset, stage);
    offset = next;
}
assert.doesNotMatch(body, /pending_request_new|schedule_step_record|session_start|RetryBudget::|#extern|while|queue|timer|_test/);
assert.match(source, /error_free[\s\S]*run_execution_error_free[\s\S]*end_frame_command_error_free[\s\S]*end_frame_execution_error_free[\s\S]*terminal_completion_error_free/);
const attemptClose = fs.readFileSync(path.resolve(__dirname, "..", "stdlib/std/gui/compositor_tile_present_host_action_attempt_driver.nepl"), "utf8");
assert.match(attemptClose, /attempt_driver_error_close[\s\S]*AttemptActionMismatch[\s\S]*pending_abort[\s\S]*SinkRejected[\s\S]*pending_abort[\s\S]*DriverCompletionFailed/);
console.log("registered resumed Web loop contract passed");
