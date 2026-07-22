#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_command_driver_budget.nepl"), "utf8");

assert.match(source, /struct GuiFontWebRegisteredCommandDriverBudgetOwner:\s*\n\s*total_remaining %i32\s*\n\s*slice_remaining %i32\s*\n\s*slice_limit %i32/);
assert.doesNotMatch(source, /pub struct GuiFontWebRegisteredCommandDriver(?:BudgetOwner|SliceExhaustedOwner)/);
assert.match(source, /budget_start[\s\S]*gt total_limit 0[\s\S]*gt slice_limit 0/);
assert.match(source, /GuiFontWebRegisteredCommandDriverBudgetOwner total_limit if lt total_limit slice_limit then total_limit else slice_limit slice_limit/);
assert.match(source, /budget_take[\s\S]*le total_remaining 0 then[\s\S]*Exhausted[\s\S]*le slice_remaining 0 then[\s\S]*SliceExhausted[\s\S]*sub total_remaining 1 sub slice_remaining 1/);
assert.match(source, /budget_resume_slice[\s\S]*field::get owner "slice_limit"[\s\S]*if lt total_remaining slice_limit then total_remaining else slice_limit/);
assert.doesNotMatch(source, /budget_resume_slice[^\n]*i32|set |add total_remaining|add slice_remaining/);

process.stdout.write("registered command driver budget contract passed\n");
