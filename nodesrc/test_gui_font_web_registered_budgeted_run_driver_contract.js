#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_budgeted_run_driver.nepl"), "utf8");

assert.match(source, /pub struct GuiFontWebRegisteredBudgetedRunDriverReadyOwner:\s*\n\s*run_owner %GuiFontWebRegisteredRunNextCommandPhaseRunOwner\s*\n\s*budget %GuiFontWebRegisteredCommandDriverBudgetOwner/);
assert.match(source, /pub struct GuiFontWebRegisteredBudgetedRunDriverSuspendedOwner:\s*\n\s*run_owner %GuiFontWebRegisteredRunNextCommandPhaseRunOwner\s*\n\s*budget %GuiFontWebRegisteredCommandDriverSliceExhaustedOwner/);
assert.match(source, /budgeted_run_driver_execute[\s\S]*budget_take budget[\s\S]*TotalExhausted[\s\S]*SliceExhausted exhausted[\s\S]*Suspended[\s\S]*Granted next_budget[\s\S]*run_phase_handoff_execute host policy support owner/);
assert.match(source, /RunPhaseHandoffResult::Continue continuation: gui_font_web_registered_budgeted_run_driver_continue host policy support continuation next_budget/);
assert.match(source, /RunPhaseHandoffResult::Yield continuation: gui_font_web_registered_budgeted_run_driver_yield host policy support continuation next_budget/);
assert.match(source, /RunNextCommandPhaseOwner::Run owner: GuiFontWebRegisteredBudgetedRunDriverOutcome::Ready/);
assert.doesNotMatch(source, /RunNextCommandPhaseOwner::Run owner:\s*gui_font_web_registered_budgeted_run_driver_execute/);
assert.match(source, /budgeted_run_driver_outcome_free[\s\S]*Outcome::Ready[\s\S]*Outcome::Suspended[\s\S]*Outcome::ExecutionFailed[\s\S]*Outcome::NextCommandFailed/);

process.stdout.write("registered budgeted Run driver contract passed\n");
