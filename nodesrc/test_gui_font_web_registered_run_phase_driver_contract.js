#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const driver = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_run_phase_driver.nepl"), "utf8");
const command = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_end_frame_command.nepl"), "utf8");

assert.match(driver, /pub enum GuiFontWebRegisteredRunPhaseDriverActionBudget:\s*\n\s*Exhausted\s*\n\s*OneRemaining/);
assert.match(driver, /pub enum GuiFontWebRegisteredRunPhaseDriverYieldResumeBudget:\s*\n\s*Exhausted\s*\n\s*DeferOnce\s*\n\s*OneRemaining/);
assert.match(driver, /pub struct GuiFontWebRegisteredRunPhaseDriverSuspendedOwner:\s*\n\s*phase_owner %GuiFontWebRegisteredRunSuccessPhaseOwner\s*\n\s*action_budget %GuiFontWebRegisteredRunPhaseDriverActionBudget\s*\n\s*yield_resume_budget %GuiFontWebRegisteredRunPhaseDriverYieldResumeBudget/);
assert.match(driver, /pub enum GuiFontWebRegisteredRunPhaseDriverOutcome:\s*\n\s*Terminal[^\n]*\n\s*Suspended %GuiFontWebRegisteredRunPhaseDriverSuspendedOwner\s*\n\s*Completed/);
assert.match(driver, /Completed completed:\s*\n\s*Result::Ok GuiFontWebRegisteredRunPhaseDriverOutcome::Completed completed/);
assert.match(driver, /Continue continuation:[\s\S]*Exhausted:[\s\S]*run_phase_driver_suspended phase_owner GuiFontWebRegisteredRunPhaseDriverActionBudget::Exhausted yield_resume_budget[\s\S]*OneRemaining:[\s\S]*run_success_phase_continue_into_end_frame continuation/);
assert.match(driver, /Yield continuation:[\s\S]*yield_resume_budget[\s\S]*DeferOnce:[\s\S]*run_phase_driver_suspended phase_owner GuiFontWebRegisteredRunPhaseDriverActionBudget::OneRemaining GuiFontWebRegisteredRunPhaseDriverYieldResumeBudget::OneRemaining[\s\S]*OneRemaining:[\s\S]*run_success_phase_yield_resume_into_end_frame continuation/);
assert.match(driver, /run_phase_driver_suspended_owner_into_parts owner[\s\S]*execute_with_budget host policy support field::get parts "action_budget" field::get parts "yield_resume_budget" field::get parts "phase_owner"/);
assert.doesNotMatch(driver, /pub fn gui_font_web_registered_run_phase_driver_suspended_owner_into_parts/);
assert.doesNotMatch(driver, /run_phase_driver_resume[^\n]*RunPhaseDriverPolicy/);
assert.match(driver, /EndFrameCommandFailed[^\n]*end_frame_command_error_free[\s\S]*EndFrameExecutionFailed[^\n]*end_frame_execution_error_free[\s\S]*TerminalCompletionFailed[^\n]*terminal_completion_error_free/);
assert.doesNotMatch(driver, /while |loop |timer|queue|retry_execute|execute_run|session_start|pending_action|state_initial/);

assert.match(command, /run_success_phase_continue_into_end_frame[\s\S]*continue_owner_state &owner[\s\S]*continue_owner_into_parts owner[\s\S]*run_execution_step_end_frame state step/);
assert.match(command, /run_success_phase_yield_resume_into_end_frame[\s\S]*yield_owner_state &owner[\s\S]*state_resume_slice yielded[\s\S]*yield_owner_into_parts owner[\s\S]*run_execution_step_end_frame state step/);
const connector = command.slice(command.indexOf("pub fn gui_font_web_registered_run_success_phase_continue_into_end_frame"));
assert.equal((connector.match(/state_resume_slice yielded/g) || []).length, 1);
assert.doesNotMatch(connector, /retry_ready|session_pending|host_request|execute_run|gui_web_compositor/);

process.stdout.write("registered Run phase driver contract passed\n");
