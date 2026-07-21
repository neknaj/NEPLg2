#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const source = fs.readFileSync(path.join(root, "stdlib/platforms/gui/web/font_registered_two_run_supplied_owner_test.nepl"), "utf8");
const fixture = fs.readFileSync(path.join(root, "stdlib/alloc/gui/font/registered_face/simple_glyph/indexed/stroke_compositor_tile_rle_begin_frame_virtual_drain_test.nepl"), "utf8");

assert.match(source, /two_run_fixture_test_completed 473/);
assert.match(source, /begin_frame_virtual_drain_test_owner_from_completed completed/);
assert.match(source, /virtual_drain_owner_into_schedule/);
assert.match(source, /schedule_owner_into_host_request/);
assert.match(source, /host_request_owner_into_dispatch/);
assert.match(source, /dispatch_owner_into_loop/);
assert.match(source, /dispatch_loop_owner_into_host_execution_driver/);
assert.match(source, /host_execution_driver_owner_into_host_action_executor_session_request/);
assert.match(source, /host_action_executor_session_complete GuiRgba8888CompositorTileRlePresentHostExecutorSupport::Window/);
assert.match(source, /retry_pending_decide retry GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCompositorTileRleBeginFrameHostActionRetryBudget::OneRemaining/);
assert.match(source, /gui_font_web_registered_begin_frame_retry_execute ready/);
assert.match(source, /GuiFontWebRegisteredBeginFrameRetrySuccessPhaseOwner::Yield/);
assert.doesNotMatch(source, /GuiFontRegisteredFaceSimpleGlyphIndexedStrokeCompositorTileRleBeginFrameHostActionRetryReadyOwner [A-Z]/);
assert.doesNotMatch(source, /GuiRgba8888CompositorTileRlePresentCommand::Run /);
assert.match(fixture, /owner_from_completed[\s\S]*completed_owner_into_compositor_tile_rle_begin_frame_record_budget completed surface frame_config tile_config 0 64/);

process.stdout.write("registered two-run supplied owner contract passed\n");
