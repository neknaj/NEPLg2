#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const source = fs.readFileSync(path.resolve(__dirname, "..", "stdlib/platforms/gui/web/font_registered_unified_web_frame.nepl"), "utf8");
const body = source.match(/pub fn gui_font_web_registered_unified_web_frame_execute[\s\S]*?(?=\npub fn)/)?.[0] || "";
assert.equal((body.match(/gui_font_web_registered_begin_frame_retry_execute/g) || []).length, 1);
assert.equal((body.match(/gui_font_web_registered_begin_frame_retry_execution_success_into_phase/g) || []).length, 1);
assert.equal((body.match(/gui_font_web_registered_resumed_web_loop_execute/g) || []).length, 1);
assert.ok(body.indexOf("begin_frame_retry_execute") < body.indexOf("execution_success_into_phase"));
assert.ok(body.indexOf("execution_success_into_phase") < body.indexOf("resumed_web_loop_execute"));
const yieldArm = body.match(/SuccessPhaseOwner::Yield[\s\S]*$/)?.[0] || "";
assert.match(yieldArm, /resumed_web_loop_execute/);
assert.match(body, /Result::Err failure:\s*Result::Err GuiFontWebRegisteredUnifiedWebFrameError::RetryExecutionFailed failure/);
assert.match(body, /SuccessPhaseOwner::Continue owner:\s*Result::Ok GuiFontWebRegisteredUnifiedWebFrameSuccess::Continue owner/);
assert.match(body, /SuccessPhaseOwner::Completed owner:\s*Result::Ok GuiFontWebRegisteredUnifiedWebFrameSuccess::Completed owner/);
assert.match(yieldArm, /Result::Err failure: Result::Err GuiFontWebRegisteredUnifiedWebFrameError::ResumedLoopFailed failure/);
assert.match(yieldArm, /Result::Ok terminal: Result::Ok GuiFontWebRegisteredUnifiedWebFrameSuccess::Terminal terminal/);
assert.doesNotMatch(body, /retry_pending_decide|session_start|pending_request_new|RetryBudget::|recovered_state.*resume|#extern|while|queue|timer|recursive/);
assert.match(source, /UnifiedWebFrameSuccess:[\s\S]*Continue[\s\S]*Completed[\s\S]*Terminal/);
assert.match(source, /error_free[\s\S]*execution_failure_free[\s\S]*resumed_web_loop_error_free/);
assert.match(source, /error_into_recovery[\s\S]*RetryExecutionFailed failure[\s\S]*ResumedLoopFailed failure/);
console.log("registered unified Web frame contract passed");
