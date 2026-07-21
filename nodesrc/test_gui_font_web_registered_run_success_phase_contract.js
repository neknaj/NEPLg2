#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_run_success_phase.nepl"), "utf8");

function count(pattern) {
    return (source.match(pattern) || []).length;
}

function structBody(name) {
    const match = source.match(new RegExp(`pub struct ${name}:\\n([\\s\\S]*?)(?=\\npub (?:struct|enum|fn) )`));
    assert.ok(match, `${name} must exist`);
    return match[1];
}

function functionBody(name) {
    const match = source.match(new RegExp(`pub fn ${name}[^\\n]*:\\n([\\s\\S]*?)(?=\\npub fn |$)`));
    assert.ok(match, `${name} must exist`);
    return match[1];
}

for (const phase of ["Continue", "Yield", "Completed"]) {
    for (const suffix of ["Owner", "OwnerParts"]) {
        const body = structBody(`GuiFontWebRegisteredRunSuccessPhase${phase}${suffix}`);
        assert.match(body, /spent_budget[^\n]*\n\s*step %GuiRgba8888CompositorTileRlePresentCommandCursorStep\s*\n\s*completion %GuiRgba8888CompositorTileRlePresentHostActionExecutorSessionCompletion/);
        assert.doesNotMatch(body, /\n\s*state %/);
    }
    const lower = phase.toLowerCase();
    const partsBody = functionBody(`gui_font_web_registered_run_success_phase_${lower}_owner_into_parts`);
    assert.match(partsBody, /field::get owner "step" field::get owner "completion"/);
    const stateBody = functionBody(`gui_font_web_registered_run_success_phase_${lower}_owner_state`);
    assert.match(stateBody, /completion_dispatch_loop_completion field::get_ref owner "completion"/);
    assert.doesNotMatch(stateBody, /field::get_ref owner "state"/);
}

assert.match(source, /pub enum GuiFontWebRegisteredRunSuccessPhaseOwner:\s*\n\s*Continue %GuiFontWebRegisteredRunSuccessPhaseContinueOwner\s*\n\s*Yield %GuiFontWebRegisteredRunSuccessPhaseYieldOwner\s*\n\s*Completed %GuiFontWebRegisteredRunSuccessPhaseCompletedOwner/);
assert.equal(count(/gui_font_web_registered_run_execution_success_into_parts owner/g), 1);
assert.equal(count(/completion_dispatch_loop_completion &completion/g), 1);
assert.match(source, /Continue _:\s*\n\s*GuiFontWebRegisteredRunSuccessPhaseOwner::Continue[^\n]*step completion/);
assert.match(source, /Yield _:\s*\n\s*GuiFontWebRegisteredRunSuccessPhaseOwner::Yield[^\n]*step completion/);
assert.match(source, /Completed _:\s*\n\s*GuiFontWebRegisteredRunSuccessPhaseOwner::Completed[^\n]*step completion/);
assert.match(source, /pub fn gui_font_web_registered_run_success_phase_owner_free[\s\S]*Continue payload:[^\n]*continue_owner_free payload[\s\S]*Yield payload:[^\n]*yield_owner_free payload[\s\S]*Completed payload:[^\n]*completed_owner_free payload/);
assert.doesNotMatch(source, /success_step_end_frame|resume_slice|cursor_step cursor|gui_web_compositor|schedule_owner|host_request|session_start/);
assert.doesNotMatch(source, /\n\s*_:/);

process.stdout.write("registered Run success phase contract passed\n");
