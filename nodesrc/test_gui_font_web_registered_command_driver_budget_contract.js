#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_command_driver_budget.nepl"), "utf8");

assert.match(source, /struct GuiFontWebRegisteredCommandDriverBudgetActiveProof:/);
assert.match(source, /struct GuiFontWebRegisteredCommandDriverBudgetSliceExhaustedProof:/);
assert.match(source, /pub struct GuiFontWebRegisteredCommandDriverBudgetOwner:\s*\n\s*proof %GuiFontWebRegisteredCommandDriverBudgetActiveProof\s*\n\s*\n/);
assert.match(source, /pub struct GuiFontWebRegisteredCommandDriverSliceExhaustedOwner:\s*\n\s*proof %GuiFontWebRegisteredCommandDriverBudgetSliceExhaustedProof\s*\n\s*\n/);
assert.doesNotMatch(source, /pub struct GuiFontWebRegisteredCommandDriverBudget(?:Active|SliceExhausted)Proof|pub fn gui_font_web_registered_command_driver_budget_(?:active|slice_exhausted)_proof/);
assert.match(source, /budget_start[\s\S]*gt total_limit 0[\s\S]*gt slice_limit 0/);
assert.match(source, /budget_owner %fn i32 fn i32 fn i32[\s\S]*BudgetOwner \(gui_font_web_registered_command_driver_budget_active_proof total_remaining slice_remaining slice_limit\)/);
assert.match(source, /SliceExhausted GuiFontWebRegisteredCommandDriverSliceExhaustedOwner \(gui_font_web_registered_command_driver_budget_slice_exhausted_proof total_remaining slice_limit\)/);
assert.match(source, /budget_take[\s\S]*le total_remaining 0 then[\s\S]*Exhausted[\s\S]*le slice_remaining 0 then[\s\S]*SliceExhausted[\s\S]*budget_owner \(sub total_remaining 1\) \(sub slice_remaining 1\)/);
assert.match(source, /budget_resume_slice[\s\S]*field::get owner "proof"[\s\S]*field::get proof "slice_limit"[\s\S]*if lt total_remaining slice_limit then total_remaining else slice_limit/);
assert.doesNotMatch(source, /budget_resume_slice[^\n]*i32|set |add total_remaining|add slice_remaining/);

process.stdout.write("registered command driver budget contract passed\n");
