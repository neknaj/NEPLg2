#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..");

function read(relPath) {
    return fs.readFileSync(path.join(repoRoot, relPath), "utf8").replace(/\r\n/g, "\n");
}

function withoutComments(text) {
    return text
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//"))
        .join("\n");
}

function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

function assertMatch(text, pattern, message) {
    assert(pattern.test(text), `${message}: expected ${pattern}`);
}

function assertNoMatch(text, pattern, message) {
    assert(!pattern.test(text), `${message}: forbidden ${pattern}`);
}

const spec = read("doc/neplg2/gui_redesign_spec.md");
const detailedDesign = read("doc/neplg2/gui_redesign_detailed_design.md");
const implementationPlan = read("doc/neplg2/gui_redesign_implementation_plan.md");
const offscreen = read("stdlib/std/gui/offscreen.nepl");
const offscreenImpl = withoutComments(offscreen);
const virtualEvent = read("stdlib/std/gui/virtual_event.nepl");
const virtualEventImpl = withoutComments(virtualEvent);
const virtualTimer = read("stdlib/std/gui/virtual_timer.nepl");
const virtualTimerImpl = withoutComments(virtualTimer);
const turnVirtualTimer = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.nepl");
const turnVirtualTimerImpl = withoutComments(turnVirtualTimer);
const stdGuiFacade = read("stdlib/std/gui.nepl");
const guiStdTests = read("tests/stdlib/gui_std.n.md");
const guiStdVirtualTimerTests = read("tests/stdlib/gui_std_virtual_timer.n.md");
const guiStdTurnVirtualTimerTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md");

assertMatch(
    spec,
    /OffscreenPixel[\s\S]*fallback[\s\S]*Headless[\s\S]*Unsupported/,
    "GUI spec must state that offscreen capture is explicit and headless does not fallback",
);
assertMatch(
    detailedDesign,
    /GuiOffscreenSnapshot:[\s\S]*pixel_hash\s+i32[\s\S]*0\s+や\s+-1\s+を\s+sentinel\s+にしない/,
    "detailed design must define pixel_hash as an opaque signed value without sentinel meanings",
);
assertMatch(
    detailedDesign,
    /GuiVirtualEventScript:[\s\S]*first\s+Option\s+GuiEvent[\s\S]*second\s+Option\s+GuiEvent/,
    "detailed design must use Option GuiEvent slots for virtual events",
);
assertMatch(
    detailedDesign,
    /GuiVirtualTimerState:[\s\S]*request\s+Option\s+TimerRequest[\s\S]*Repeating timer[\s\S]*advance state 0/,
    "detailed design must define virtual timer state, zero-delta drain, and repeating semantics",
);
assertMatch(
    implementationPlan,
    /Phase 5\.1:[\s\S]*stdlib\/std\/gui\/offscreen\.nepl[\s\S]*stdlib\/std\/gui\/virtual_event\.nepl/,
    "implementation plan must track the offscreen and virtual event implementation slice",
);
assertMatch(
    implementationPlan,
    /Phase 5\.2:[\s\S]*stdlib\/std\/gui\/virtual_timer\.nepl[\s\S]*tests\/stdlib\/gui_std_virtual_timer\.n\.md/,
    "implementation plan must track the virtual timer scheduler implementation slice",
);
assertMatch(
    implementationPlan,
    /Phase 5\.3:[\s\S]*tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer\.nepl[\s\S]*gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer\.n\.md/,
    "implementation plan must track the virtual timer turn bridge implementation slice",
);

assertMatch(
    offscreenImpl,
    /pub\s+struct\s+GuiOffscreenSnapshot:[\s\S]*surface\s+%SurfaceId[\s\S]*frame\s+%FrameId[\s\S]*pixel_hash\s+%i32/,
    "std/gui/offscreen must publish a typed snapshot with opaque pixel hash",
);
assertMatch(
    offscreenImpl,
    /pub\s+fn\s+gui_offscreen_snapshot_from_runtime_command\s+%fn\s+&GuiHost\s+fn\s+GuiRuntimeCommand\s+fn\s+i32\s+Result\s+GuiOffscreenSnapshot\s+GuiError/,
    "std/gui/offscreen must expose a Result-returning snapshot constructor",
);
assertMatch(
    offscreenImpl,
    /surface_kind_is_offscreen_pixel\s+gui_capabilities_surface_kind\s+&capabilities/,
    "std/gui/offscreen must require an OffscreenPixel host",
);
assertMatch(
    offscreenImpl,
    /GuiRuntimeCommand::PresentSurface[\s\S]*GuiSurfacePresentCommand::PresentPixelFrame/,
    "std/gui/offscreen must accept only present surface pixel frame commands",
);
assertMatch(
    offscreenImpl,
    /Result::Err\s+GuiError::Unsupported/,
    "std/gui/offscreen must reject unsupported hosts or commands through GuiError",
);
assertNoMatch(
    offscreenImpl,
    /\b(?:DOM|Canvas|HTMLCanvasElement|ImageData|document\.|window\.|minifb|HWND|stdout)\b/i,
    "std/gui/offscreen must not expose concrete platform transport details",
);
assertNoMatch(
    offscreenImpl,
    /SurfaceKind::(?:WindowPixel|DevicePixel|Headless)[\s\S]*Result::Ok/,
    "std/gui/offscreen must not fallback from non-offscreen surfaces to successful snapshots",
);

assertMatch(
    virtualEventImpl,
    /pub\s+struct\s+GuiVirtualEventScript:[\s\S]*first\s+%Option\s+GuiEvent[\s\S]*second\s+%Option\s+GuiEvent/,
    "std/gui/virtual_event must store replay slots as Option GuiEvent",
);
assertNoMatch(
    virtualEventImpl,
    /GuiEvent::None/,
    "std/gui/virtual_event must not invent a GuiEvent sentinel",
);
assertMatch(
    virtualEventImpl,
    /Result::Err\s+GuiError::ResourceExhausted/,
    "std/gui/virtual_event must reject replay script overflow explicitly",
);
assertMatch(
    virtualEventImpl,
    /Result::Err\s+GuiError::InvalidCommand/,
    "std/gui/virtual_event must reject malformed counts and clock overflow explicitly",
);
assertMatch(
    virtualEventImpl,
    /gui_virtual_clock_advance[\s\S]*gt\s+delta_ms\s+max_delta[\s\S]*ge\s+tick\s+gui_virtual_i32_max/,
    "std/gui/virtual_event must check both time and tick overflow",
);
assertNoMatch(
    virtualEventImpl,
    /\b(?:DOM|Canvas|KeyboardEvent|MouseEvent|PointerEvent|EventTarget|document\.|window\.)\b/,
    "std/gui/virtual_event must not store platform raw events",
);

assertMatch(
    virtualTimerImpl,
    /pub\s+struct\s+GuiVirtualTimerState:[\s\S]*request\s+%Option\s+TimerRequest[\s\S]*elapsed_ms\s+%i32[\s\S]*tick\s+%i32/,
    "std/gui/virtual_timer must store active timer as Option TimerRequest plus deterministic counters",
);
assertMatch(
    virtualTimerImpl,
    /pub\s+struct\s+GuiVirtualTimerAdvance:[\s\S]*state\s+%GuiVirtualTimerState[\s\S]*event\s+%Option\s+GuiEvent/,
    "std/gui/virtual_timer must return Option GuiEvent, not a sentinel event",
);
assertNoMatch(
    virtualTimerImpl,
    /GuiEvent::None/,
    "std/gui/virtual_timer must not invent a GuiEvent sentinel",
);
assertMatch(
    virtualTimerImpl,
    /gui_virtual_timer_schedule[\s\S]*not\s+gui_virtual_timer_state_is_valid\s+&state[\s\S]*not\s+gui_virtual_timer_request_is_schedulable\s+&request/,
    "std/gui/virtual_timer schedule must revalidate incoming state and request",
);
assertMatch(
    virtualTimerImpl,
    /gui_virtual_timer_advance[\s\S]*not\s+gui_virtual_timer_state_is_valid\s+&state[\s\S]*lt\s+delta_ms\s+0[\s\S]*gt\s+delta_ms\s+max_delta[\s\S]*ge\s+tick\s+gui_virtual_timer_i32_max/,
    "std/gui/virtual_timer advance must reject malformed state, negative delta, elapsed overflow, and tick overflow",
);
assertMatch(
    virtualTimerImpl,
    /let\s+remainder\s+%i32\s+sub\s+next_elapsed\s+interval_ms[\s\S]*gui_virtual_timer_state_new\s+some\s+active\s+remainder\s+next_tick/,
    "std/gui/virtual_timer repeating catch-up must retain elapsed remainder instead of dropping it",
);
assertNoMatch(
    virtualTimerImpl,
    /\b(?:DOM|Canvas|KeyboardEvent|MouseEvent|PointerEvent|EventTarget|document\.|window\.|minifb|HWND|stdout|queue|SharedArrayBuffer|setTimeout|setInterval|video_memory|fallback|silent no-op)\b/i,
    "std/gui/virtual_timer must not depend on platform APIs, queues, video memory, or hidden fallback",
);
assertMatch(
    turnVirtualTimerImpl,
    /GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending:[\s\S]*pending\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerPending[\s\S]*timer_state\s+%GuiVirtualTimerState/,
    "std/gui turn virtual timer bridge must keep F5dw pending and virtual timer state together",
);
assertMatch(
    turnVirtualTimerImpl,
    /gui_virtual_timer_schedule\s+timer_state\s+request[\s\S]*gui_virtual_timer_advance\s+timer_state\s+delta_ms[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_timer_complete\s+timer_pending\s+timer_event/,
    "std/gui turn virtual timer bridge must connect F5dy schedule and advance to F5dw completion",
);
assertNoMatch(
    turnVirtualTimerImpl,
    /\b(?:DOM|Canvas|KeyboardEvent|MouseEvent|PointerEvent|EventTarget|document\.|window\.|minifb|HWND|stdout|queue|SharedArrayBuffer|setTimeout|setInterval|video_memory|fallback|silent no-op)\b/i,
    "std/gui turn virtual timer bridge must not depend on platform APIs, queues, video memory, or hidden fallback",
);

assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/offscreen"\s+as\s+\*/,
    "std/gui facade must re-export the offscreen snapshot contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/virtual_event"\s+as\s+\*/,
    "std/gui facade must re-export the virtual event contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/virtual_timer"\s+as\s+\*/,
    "std/gui facade must re-export the virtual timer contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer"\s+as\s+\*/,
    "std/gui facade must re-export the virtual timer turn bridge contract",
);
assertMatch(
    guiStdTests,
    /gui_offscreen_snapshot_requires_offscreen_present_command[\s\S]*headless unsupported[\s\S]*window unsupported[\s\S]*device unsupported[\s\S]*noop unsupported/,
    "std/gui tests must cover offscreen-only snapshot behavior across non-offscreen surface kinds",
);
assertMatch(
    guiStdTests,
    /gui_virtual_event_script_replays_typed_events_without_sentinel[\s\S]*empty poll none[\s\S]*malformed empty rejected[\s\S]*malformed one rejected[\s\S]*cursor overflow rejected/,
    "std/gui tests must cover Option-based virtual event replay and malformed public constructor states",
);
assertMatch(
    guiStdVirtualTimerTests,
    /gui_std_virtual_timer_repeating_remainder_drain_ok[\s\S]*repeating first remainder[\s\S]*repeating drain tick[\s\S]*tick overflow rejected/,
    "std/gui virtual timer focused doctest must cover repeating remainder drain and overflow validation",
);
assertMatch(
    guiStdVirtualTimerTests,
    /gui_std_virtual_timer_no_sentinel_no_queue_no_platform_no_fallback[\s\S]*malformed none state rejected[\s\S]*malformed active state rejected/,
    "std/gui virtual timer focused doctest must cover state invariant validation and no sentinel/queue/platform/fallback policy",
);
assertMatch(
    guiStdTurnVirtualTimerTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_schedule_owner_recovery_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_no_loop_no_backend_no_queue_no_fallback/,
    "std/gui turn virtual timer focused doctest must cover owner recovery and no backend/queue/fallback policy",
);

console.log("web GUI offscreen/headless contract passed");
