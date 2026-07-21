#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const source = fs.readFileSync(path.resolve(__dirname, "../stdlib/platforms/gui/web/font_registered_run_next_command.nepl"), "utf8");
const count = (pattern) => (source.match(pattern) || []).length;

assert.match(source, /run_success_continue_step_next_command/);
assert.match(source, /run_success_yield_resume_and_step_next_command/);
assert.equal(count(/command_cursor_step_finish_owner current_step/g), 1);
assert.equal(count(/command_cursor_step cursor/g), 1);
assert.equal(count(/dispatch_loop_state_resume_slice yielded/g), 1);
assert.match(source, /owner_result[\s\S]*command_cursor_step_result field::get_ref owner "step"/);
assert.match(source, /error_free[\s\S]*command_cursor_step_error_free field::get error "lower"/);
assert.doesNotMatch(source, /pub struct GuiFontWebRegisteredRunNextCommandOwnerParts|pub fn gui_font_web_registered_run_next_command_owner_into_parts/);
assert.match(source, /pub enum GuiFontWebRegisteredRunNextCommandPhaseOwner:\s*\n\s*BeginFrame[\s\S]*Run[\s\S]*EndFrame[\s\S]*Completed/);
assert.match(source, /owner_into_phase[\s\S]*Command command:[\s\S]*BeginFrame _:[\s\S]*Run _:[\s\S]*EndFrame _:[\s\S]*CursorStepResult::Completed:/);
const classifier = source.match(/pub fn gui_font_web_registered_run_next_command_owner_into_phase[^\n]*:\r?\n([\s\S]*?)(?=\r?\npub fn )/);
assert.ok(classifier);
assert.doesNotMatch(classifier[1], /cursor_step cursor|resume_slice|schedule|host_request|session_start|executor_execute/);
process.stdout.write("registered Run next command contract passed\n");
