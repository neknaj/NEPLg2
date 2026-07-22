#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const source = fs.readFileSync(path.resolve(__dirname, "..", "stdlib", "platforms", "gui", "web", "font_registered_fresh_run_owner.nepl"), "utf8");

assert.match(source, /owner_result field::get_ref &owner "next_owner"/);
assert.match(source, /Command::Run _:[\s\S]*owner_into_parts owner/);
assert.match(source, /PhaseRunOwner field::get parts "previous_category" field::get parts "previous_diagnostic" field::get parts "spent_budget" field::get next_parts "state" field::get next_parts "step"/);
assert.match(source, /_: GuiFontWebRegisteredFreshRunOwnerResult::NotRun owner/g);
assert.doesNotMatch(source, /run_executor|execution_success/);

const testSource = fs.readFileSync(path.resolve(__dirname, "..", "stdlib", "platforms", "gui", "web", "font_registered_fresh_run_owner_test.nepl"), "utf8");
assert.match(testSource, /FreshRunOwnerResult::NotRun owner:[\s\S]*next_command_owner_free owner/);
assert.match(testSource, /PresentCommand::EndFrame _:[\s\S]*PresentCommandCursorStepResult::Completed/);

process.stdout.write(`${JSON.stringify({ ok: true, contract: "fresh-run-owner" })}\n`);
