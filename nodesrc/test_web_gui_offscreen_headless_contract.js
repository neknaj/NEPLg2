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

function textSliceBetween(text, startMarker, endMarker) {
    const start = text.indexOf(startMarker);
    assert(start >= 0, `missing start marker: ${startMarker}`);
    const end = text.indexOf(endMarker, start + startMarker.length);
    assert(end >= 0, `missing end marker after ${startMarker}: ${endMarker}`);
    return text.slice(start, end);
}

function functionSlice(source, name) {
    const start = source.indexOf(`fn ${name} `);
    if (start < 0) {
        return "";
    }
    const candidates = [
        source.indexOf("\nfn ", start + 1),
        source.indexOf("\npub fn ", start + 1),
        source.indexOf("\nstruct ", start + 1),
        source.indexOf("\nenum ", start + 1),
        source.indexOf("\npub struct ", start + 1),
        source.indexOf("\npub enum ", start + 1),
        source.indexOf("\nimpl ", start + 1),
    ].filter((index) => index >= 0);
    const next = candidates.length === 0 ? -1 : Math.min(...candidates);
    return next < 0 ? source.slice(start) : source.slice(start, next);
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
const turnVirtualScheduler = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.nepl");
const turnVirtualSchedulerImpl = withoutComments(turnVirtualScheduler);
const turnVirtualSchedulerStep = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step.nepl");
const turnVirtualSchedulerStepImpl = withoutComments(turnVirtualSchedulerStep);
const turnVirtualSchedulerDrain = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain.nepl");
const turnVirtualSchedulerDrainImpl = withoutComments(turnVirtualSchedulerDrain);
const turnVirtualSchedulerTransition = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition.nepl");
const turnVirtualSchedulerTransitionImpl = withoutComments(turnVirtualSchedulerTransition);
const turnVirtualSchedulerSlice = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice.nepl");
const turnVirtualSchedulerSliceImpl = withoutComments(turnVirtualSchedulerSlice);
const turnVirtualSchedulerLoop = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop.nepl");
const turnVirtualSchedulerLoopImpl = withoutComments(turnVirtualSchedulerLoop);
const turnVirtualSchedulerLoopAction = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action.nepl");
const turnVirtualSchedulerLoopActionImpl = withoutComments(turnVirtualSchedulerLoopAction);
const turnVirtualSchedulerLoopTimerAdvance = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance.nepl");
const turnVirtualSchedulerLoopTimerAdvanceImpl = withoutComments(turnVirtualSchedulerLoopTimerAdvance);
const turnVirtualSchedulerLoopExecutorComplete = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete.nepl");
const turnVirtualSchedulerLoopExecutorCompleteImpl = withoutComments(turnVirtualSchedulerLoopExecutorComplete);
const turnVirtualSchedulerLoopYieldComplete = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete.nepl");
const turnVirtualSchedulerLoopYieldCompleteImpl = withoutComments(turnVirtualSchedulerLoopYieldComplete);
const turnVirtualSchedulerRealLoopStep = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step.nepl");
const turnVirtualSchedulerRealLoopStepImpl = withoutComments(turnVirtualSchedulerRealLoopStep);
const turnVirtualSchedulerRealLoopDriver = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver.nepl");
const turnVirtualSchedulerRealLoopDriverImpl = withoutComments(turnVirtualSchedulerRealLoopDriver);
const turnVirtualSchedulerHeadlessAppLoopStep = read("stdlib/std/gui/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step.nepl");
const turnVirtualSchedulerHeadlessAppLoopStepImpl = withoutComments(turnVirtualSchedulerHeadlessAppLoopStep);
const stdGuiFacade = read("stdlib/std/gui.nepl");
const guiStdTests = read("tests/stdlib/gui_std.n.md");
const guiStdVirtualTimerTests = read("tests/stdlib/gui_std_virtual_timer.n.md");
const guiStdTurnVirtualTimerTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md");
const guiStdTurnVirtualSchedulerTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.n.md");
const guiStdTurnVirtualSchedulerStepTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step.n.md");
const guiStdTurnVirtualSchedulerDrainTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain.n.md");
const guiStdTurnVirtualSchedulerTransitionTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition.n.md");
const guiStdTurnVirtualSchedulerSliceTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice.n.md");
const guiStdTurnVirtualSchedulerLoopTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop.n.md");
const guiStdTurnVirtualSchedulerLoopActionTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action.n.md");
const guiStdTurnVirtualSchedulerLoopTimerAdvanceTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance.n.md");
const guiStdTurnVirtualSchedulerLoopExecutorCompleteTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete.n.md");
const guiStdTurnVirtualSchedulerLoopYieldCompleteTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete.n.md");
const guiStdTurnVirtualSchedulerRealLoopStepTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step.n.md");
const guiStdTurnVirtualSchedulerRealLoopDriverTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver.n.md");
const guiStdTurnVirtualSchedulerHeadlessAppLoopStepTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step.n.md");

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
    implementationPlan,
    /Phase 5\.4:[\s\S]*tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler\.nepl[\s\S]*gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler\.n\.md/,
    "implementation plan must track the virtual scheduler state boundary implementation slice",
);
assertMatch(
    implementationPlan,
    /Phase 5\.5:[\s\S]*tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step\.nepl[\s\S]*gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step\.n\.md/,
    "implementation plan must track the virtual scheduler single-step implementation slice",
);
assertMatch(
    implementationPlan,
    /Phase 5\.6:[\s\S]*tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain\.nepl[\s\S]*GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainResult/,
    "implementation plan must track the virtual scheduler bounded-drain implementation slice",
);
assertMatch(
    implementationPlan,
    /Phase 5\.7:[\s\S]*tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition\.nepl[\s\S]*GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition/,
    "implementation plan must track the virtual scheduler transition implementation slice",
);
assertMatch(
    implementationPlan,
    /Phase 5\.8:[\s\S]*tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice\.nepl[\s\S]*GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceResult/,
    "implementation plan must track the virtual scheduler time-slice implementation slice",
);
assertMatch(
    implementationPlan,
    /Phase 5\.9:[\s\S]*tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop\.nepl[\s\S]*GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopResult/,
    "implementation plan must track the virtual scheduler loop result implementation slice",
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
    turnVirtualSchedulerImpl,
    /GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState:[\s\S]*Turn\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTurn[\s\S]*WaitingTimer\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending[\s\S]*Execute\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerExecute[\s\S]*Completed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerCompleted/,
    "std/gui turn virtual scheduler must expose phase-owned deterministic scheduler states",
);
assertMatch(
    turnVirtualSchedulerImpl,
    /GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTurn:[\s\S]*timer_state\s+%GuiVirtualTimerState[\s\S]*turn_state\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnState/,
    "std/gui turn virtual scheduler must keep dynamic timer state on Turn phase, not policy",
);
assertMatch(
    turnVirtualSchedulerImpl,
    /GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerReady::ContinueNow\s+turn_state:[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_turn\s+timer_state\s+turn_state/,
    "std/gui turn virtual scheduler must map ContinueNow to pollable Turn phase",
);
assertMatch(
    turnVirtualSchedulerImpl,
    /GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerReady::ScheduleTimer\s+pending:[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_schedule\s+pending\s+timer_state/,
    "std/gui turn virtual scheduler must call F5dz schedule only for ScheduleTimer",
);
assertMatch(
    turnVirtualSchedulerImpl,
    /gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_timer_advance\s+pending\s+delta_ms[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_decide\s+policy\s+gui_virtual_timer_empty\s+decision/,
    "std/gui turn virtual scheduler must advance F5dz once and re-enter decision with explicit empty one-shot timer state",
);
assertNoMatch(
    turnVirtualSchedulerImpl,
    /\b(?:while|loop|for|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op)\b/i,
    "std/gui turn virtual scheduler must not loop, drain, call backend timers, queue, platform APIs, raw render APIs, or fallback",
);
assertMatch(
    turnVirtualSchedulerStepImpl,
    /GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerStepPolicy:[\s\S]*scheduler_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnSchedulerPolicy[\s\S]*timer_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerPolicy/,
    "std/gui turn virtual scheduler step policy must hold only static scheduler/timer policy values",
);
assertNoMatch(
    textSliceBetween(
        turnVirtualSchedulerStepImpl,
        "GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerStepPolicy:",
        "GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerStepResult:",
    ),
    /GuiVirtualTimerState/,
    "std/gui turn virtual scheduler step policy must not hold dynamic virtual timer state",
);
assertMatch(
    turnVirtualSchedulerStepImpl,
    /GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerStepResult:[\s\S]*Advanced\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState[\s\S]*BlockedWaitingTimer\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending[\s\S]*BlockedExecute\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerExecute[\s\S]*Completed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerCompleted/,
    "std/gui turn virtual scheduler step must return explicit advanced/blocked/completed results",
);
assertMatch(
    turnVirtualSchedulerStepImpl,
    /StepTurnPollFailed:[\s\S]*category\s+%Option\s+GuiError[\s\S]*timer_state\s+%GuiVirtualTimerState[\s\S]*lower\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnDriverPollError[\s\S]*StepSchedulerDecisionFailed:[\s\S]*category\s+%Option\s+GuiError[\s\S]*timer_state\s+%GuiVirtualTimerState[\s\S]*lower\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnSchedulerDecisionError/,
    "std/gui turn virtual scheduler step errors must preserve timer state on poll and scheduler-decision failures",
);
assertMatch(
    turnVirtualSchedulerStepImpl,
    /GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState::Turn\s+turn:[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_poll\s+turn_state[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_scheduler_decide\s+scheduler_policy\s+driver_step[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_decide\s+timer_policy\s+timer_state\s+decision[\s\S]*StepResult::Advanced\s+next_state/,
    "std/gui turn virtual scheduler step must poll, schedule-decide, and timer-decide in that exact Turn path order",
);
assertMatch(
    turnVirtualSchedulerStepImpl,
    /VirtualSchedulerState::WaitingTimer\s+pending:[\s\S]*StepResult::BlockedWaitingTimer\s+pending[\s\S]*VirtualSchedulerState::Execute\s+execute:[\s\S]*StepResult::BlockedExecute\s+execute[\s\S]*VirtualSchedulerState::Completed\s+completed:[\s\S]*StepResult::Completed\s+completed/,
    "std/gui turn virtual scheduler step must preserve blocked phases explicitly",
);
assertNoMatch(
    turnVirtualSchedulerStepImpl,
    /_:/,
    "std/gui turn virtual scheduler step must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerStepImpl,
    /\b(?:while|loop|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op)\b/i,
    "std/gui turn virtual scheduler step must not loop, timeslice, call backend timers, queue, platform APIs, raw render APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerStepImpl,
    /[()]/,
    "std/gui turn virtual scheduler step implementation must preserve NEPL prefix style without parentheses",
);
assertNoMatch(
    textSliceBetween(
        turnVirtualSchedulerDrainImpl,
        "pub struct GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainPolicy:",
        "pub enum GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainPolicyErrorKind:",
    ),
    /GuiVirtualTimerState/,
    "std/gui turn virtual scheduler drain policy must not hold dynamic timer state",
);
assertMatch(
    turnVirtualSchedulerDrainImpl,
    /pub\s+enum\s+GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainResult:[\s\S]*BudgetExhausted[\s\S]*BlockedWaitingTimer[\s\S]*BlockedExecute[\s\S]*Completed/,
    "std/gui turn virtual scheduler drain must expose explicit budget and blocked terminal results",
);
const drainPublic = functionSlice(turnVirtualSchedulerDrainImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain");
const drainRemaining = functionSlice(turnVirtualSchedulerDrainImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain_remaining");
assertMatch(
    drainPublic,
    /drain_validate_max_advance_count\s+max_advance_count[\s\S]*PolicyInvalid[\s\S]*drain_remaining\s+policy\s+state\s+count/,
    "std/gui turn virtual scheduler drain must revalidate max_advance_count before calling the helper",
);
assertMatch(
    drainRemaining,
    /if\s+le\s+remaining_count\s+0:[\s\S]*BudgetExhausted[\s\S]*virtual_scheduler_step\s+step_policy\s+state[\s\S]*StepFailed[\s\S]*Advanced\s+next_state:[\s\S]*next_remaining_count\s+%i32\s+sub\s+remaining_count\s+1[\s\S]*BlockedWaitingTimer[\s\S]*BlockedExecute[\s\S]*Completed/,
    "std/gui turn virtual scheduler drain helper must return budget terminal before step and preserve blocked terminal order",
);
assertNoMatch(
    drainPublic,
    /\bvirtual_scheduler_step\b/,
    "std/gui turn virtual scheduler public drain must not call F5eb step before positive-budget helper",
);
assertNoMatch(
    turnVirtualSchedulerDrainImpl,
    /_:/,
    "std/gui turn virtual scheduler drain must not use wildcard matches",
);
assertNoMatch(
    turnVirtualSchedulerDrainImpl,
    /\b(?:while|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op)\b/i,
    "std/gui turn virtual scheduler drain must not loop, timeslice, call backend timers, queue, platform APIs, raw render APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerDrainImpl,
    /[()]/,
    "std/gui turn virtual scheduler drain implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    turnVirtualSchedulerTransitionImpl,
    /pub\s+enum\s+GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition:[\s\S]*YieldSlice\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransitionYieldSlice[\s\S]*AwaitTimer\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransitionAwaitTimer[\s\S]*ExecuteHostAction\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransitionExecuteHostAction[\s\S]*Done\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransitionDone/,
    "std/gui turn virtual scheduler transition must expose explicit yield, timer, host-action, and done actions",
);
assertMatch(
    turnVirtualSchedulerTransitionImpl,
    /TransitionYieldSlice:[\s\S]*state\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState[\s\S]*remaining_count\s+%i32[\s\S]*TransitionAwaitTimer:[\s\S]*pending\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending[\s\S]*remaining_count\s+%i32[\s\S]*TransitionExecuteHostAction:[\s\S]*execute\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerExecute[\s\S]*remaining_count\s+%i32[\s\S]*TransitionDone:[\s\S]*completed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerCompleted[\s\S]*remaining_count\s+%i32/,
    "std/gui turn virtual scheduler transition must rewrap drain terminals into transition-owned payload structs",
);
assertNoMatch(
    textSliceBetween(
        turnVirtualSchedulerTransitionImpl,
        "pub struct GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransitionYieldSlice:",
        "pub enum GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition:",
    ),
    /VirtualSchedulerDrain(?:BudgetExhausted|BlockedWaitingTimer|BlockedExecute|Completed)/,
    "std/gui turn virtual scheduler transition payloads must not expose F5ec drain payload structs",
);
const transitionFromDrain = functionSlice(turnVirtualSchedulerTransitionImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition_from_drain_result");
assertMatch(
    transitionFromDrain,
    /DrainResult::BudgetExhausted\s+exhausted:[\s\S]*drain_budget_exhausted_remaining_count\s+&exhausted[\s\S]*drain_budget_exhausted_state\s+exhausted[\s\S]*Transition::YieldSlice\s+payload[\s\S]*DrainResult::BlockedWaitingTimer\s+blocked:[\s\S]*drain_blocked_waiting_timer_remaining_count\s+&blocked[\s\S]*drain_blocked_waiting_timer_pending\s+blocked[\s\S]*Transition::AwaitTimer\s+payload[\s\S]*DrainResult::BlockedExecute\s+blocked:[\s\S]*drain_blocked_execute_remaining_count\s+&blocked[\s\S]*drain_blocked_execute_execute\s+blocked[\s\S]*Transition::ExecuteHostAction\s+payload[\s\S]*DrainResult::Completed\s+completed:[\s\S]*drain_completed_remaining_count\s+&completed[\s\S]*drain_completed_completed\s+completed[\s\S]*Transition::Done\s+payload/,
    "std/gui turn virtual scheduler transition must preserve F5ec terminal order and remaining_count before consuming owner payloads",
);
assertNoMatch(
    turnVirtualSchedulerTransitionImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition(?:YieldSlice|AwaitTimer|ExecuteHostAction|Done)?\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerTransition(?:YieldSlice|AwaitTimer|ExecuteHostAction|Done)?\s*:/,
    "std/gui turn virtual scheduler transition owner-bearing payloads must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerTransitionImpl,
    /_:/,
    "std/gui turn virtual scheduler transition must not use wildcard matches",
);
assertNoMatch(
    turnVirtualSchedulerTransitionImpl,
    /\b(?:while|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op|virtual_scheduler_step|virtual_scheduler_advance_timer|turn_driver_complete|executor_session_turn_driver_complete)\b/i,
    "std/gui turn virtual scheduler transition must not step, advance timer, complete host execution, queue, call platform APIs, raw render APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerTransitionImpl,
    /[()]/,
    "std/gui turn virtual scheduler transition implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    turnVirtualSchedulerSliceImpl,
    /SlicePolicy:[\s\S]*drain_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainPolicy[\s\S]*yield_delay_ms\s+%i32/,
    "std/gui turn virtual scheduler slice policy must hold F5ec drain policy and yield delay only",
);
assertMatch(
    turnVirtualSchedulerSliceImpl,
    /SliceResult:[\s\S]*YieldSlice\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceYieldSlice[\s\S]*AwaitTimer\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceAwaitTimer[\s\S]*ExecuteHostAction\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceExecuteHostAction[\s\S]*Done\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceDone/,
    "std/gui turn virtual scheduler slice result must expose explicit yield, timer, host-action, and done results",
);
assertMatch(
    turnVirtualSchedulerSliceImpl,
    /SliceYieldSlice:[\s\S]*state\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState[\s\S]*remaining_count\s+%i32[\s\S]*yield_delay_ms\s+%i32[\s\S]*SliceAwaitTimer:[\s\S]*pending\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending[\s\S]*remaining_count\s+%i32[\s\S]*SliceExecuteHostAction:[\s\S]*execute\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerExecute[\s\S]*remaining_count\s+%i32[\s\S]*SliceDone:[\s\S]*completed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerCompleted[\s\S]*remaining_count\s+%i32/,
    "std/gui turn virtual scheduler slice payloads must preserve authority payloads and remaining counts",
);
assertNoMatch(
    textSliceBetween(
        turnVirtualSchedulerSliceImpl,
        "pub struct GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceYieldSlice:",
        "pub enum GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceResult:",
    ),
    /VirtualScheduler(?:Drain|Transition)(?:BudgetExhausted|BlockedWaitingTimer|BlockedExecute|Completed|YieldSlice|AwaitTimer|ExecuteHostAction|Done)/,
    "std/gui turn virtual scheduler slice payloads must not expose F5ec or F5ed payload structs",
);
const slicePublic = functionSlice(turnVirtualSchedulerSliceImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice");
assertMatch(
    slicePublic,
    /slice_policy_yield_delay_ms\s+policy[\s\S]*slice_validate_yield_delay_ms\s+yield_delay_ms[\s\S]*PolicyInvalid[\s\S]*slice_policy_drain_policy_ref\s+policy[\s\S]*virtual_scheduler_drain\s+drain_policy\s+state[\s\S]*DrainFailed[\s\S]*transition_from_drain_result\s+drain_result[\s\S]*slice_result_from_transition\s+transition\s+delay_ms/,
    "std/gui turn virtual scheduler slice must validate policy, drain once, transition once, and rewrap result",
);
assert(
    (slicePublic.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain\b/g) || []).length === 1,
    "std/gui turn virtual scheduler slice public entry must call F5ec drain exactly once",
);
assert(
    (slicePublic.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition_from_drain_result\b/g) || []).length === 1,
    "std/gui turn virtual scheduler slice public entry must call F5ed transition exactly once",
);
assertMatch(
    turnVirtualSchedulerSliceImpl,
    /SliceDrainFailed:[\s\S]*lower\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerDrainError/,
    "std/gui turn virtual scheduler slice drain failure must wrap only the lower F5ec error",
);
assertNoMatch(
    turnVirtualSchedulerSliceImpl,
    /_:/,
    "std/gui turn virtual scheduler slice must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerSliceImpl,
    /\b(?:while|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op|virtual_scheduler_step|virtual_scheduler_advance_timer|turn_driver_complete|executor_session_turn_driver_complete)\b/i,
    "std/gui turn virtual scheduler slice must not step, advance timer, complete host execution, queue, call platform APIs, raw render APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerSliceImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSlice(?:Policy|YieldSlice|AwaitTimer|ExecuteHostAction|Done|Result|PolicyInvalid|DrainFailed|Error)\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSlice(?:Policy|YieldSlice|AwaitTimer|ExecuteHostAction|Done|Result|PolicyInvalid|DrainFailed|Error)\s*:/,
    "std/gui turn virtual scheduler slice owner-bearing payloads and policy must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerSliceImpl,
    /[()]/,
    "std/gui turn virtual scheduler slice implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    turnVirtualSchedulerLoopImpl,
    /LoopPolicy:[\s\S]*slice_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSlicePolicy/,
    "std/gui turn virtual scheduler loop policy must hold F5ee slice policy only",
);
assertNoMatch(
    textSliceBetween(
        turnVirtualSchedulerLoopImpl,
        "pub struct GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopPolicy:",
        "pub struct GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYield:",
    ),
    /\b(?:drain_policy|yield_delay_ms|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget)\b/i,
    "std/gui turn virtual scheduler loop policy must not own F5ec policy, timers, queue, backend, or raw render state",
);
assertMatch(
    turnVirtualSchedulerLoopImpl,
    /LoopResult:[\s\S]*Yield\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYield[\s\S]*AwaitTimer\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopAwaitTimer[\s\S]*ExecuteHostAction\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecuteHostAction[\s\S]*Done\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopDone/,
    "std/gui turn virtual scheduler loop result must expose explicit yield, timer, host-action, and done results",
);
assertMatch(
    turnVirtualSchedulerLoopImpl,
    /LoopYield:[\s\S]*state\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState[\s\S]*remaining_count\s+%i32[\s\S]*yield_delay_ms\s+%i32[\s\S]*LoopAwaitTimer:[\s\S]*pending\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending[\s\S]*remaining_count\s+%i32[\s\S]*LoopExecuteHostAction:[\s\S]*execute\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerExecute[\s\S]*remaining_count\s+%i32[\s\S]*LoopDone:[\s\S]*completed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerCompleted[\s\S]*remaining_count\s+%i32/,
    "std/gui turn virtual scheduler loop payloads must preserve authority payloads and remaining counts",
);
assertNoMatch(
    textSliceBetween(
        turnVirtualSchedulerLoopImpl,
        "pub struct GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYield:",
        "pub enum GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopResult:",
    ),
    /VirtualSchedulerSlice(?:YieldSlice|AwaitTimer|ExecuteHostAction|Done)/,
    "std/gui turn virtual scheduler loop payloads must not expose F5ee payload structs",
);
assertMatch(
    turnVirtualSchedulerLoopImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop must import F5ee slice boundary",
);
assertNoMatch(
    turnVirtualSchedulerLoopImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler(?:_step|_drain|_transition)?"\s+as\s+\*|#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop must not import F5ea/F5eb/F5ec/F5ed or timer modules directly",
);
const schedulerLoopResultFromSlice = functionSlice(turnVirtualSchedulerLoopImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_result_from_slice_result");
assertMatch(
    schedulerLoopResultFromSlice,
    /SliceResult::YieldSlice[\s\S]*slice_yield_slice_remaining_count\s+&slice_payload[\s\S]*slice_yield_slice_yield_delay_ms\s+&slice_payload[\s\S]*slice_yield_slice_state\s+slice_payload[\s\S]*LoopResult::Yield\s+payload[\s\S]*SliceResult::AwaitTimer[\s\S]*slice_await_timer_remaining_count\s+&slice_payload[\s\S]*slice_await_timer_pending\s+slice_payload[\s\S]*LoopResult::AwaitTimer\s+payload[\s\S]*SliceResult::ExecuteHostAction[\s\S]*slice_execute_host_action_remaining_count\s+&slice_payload[\s\S]*slice_execute_host_action_execute\s+slice_payload[\s\S]*LoopResult::ExecuteHostAction\s+payload[\s\S]*SliceResult::Done[\s\S]*slice_done_remaining_count\s+&slice_payload[\s\S]*slice_done_completed\s+slice_payload[\s\S]*LoopResult::Done\s+payload/,
    "std/gui turn virtual scheduler loop must explicitly rewrap every F5ee slice result variant",
);
const schedulerLoopStep = functionSlice(turnVirtualSchedulerLoopImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_step");
assertMatch(
    schedulerLoopStep,
    /loop_policy_slice_policy_ref\s+policy[\s\S]*virtual_scheduler_slice\s+slice_policy\s+state[\s\S]*SliceFailed\s+payload[\s\S]*loop_result_from_slice_result\s+slice_result/,
    "std/gui turn virtual scheduler loop step must call F5ee slice once and map result through loop-owned payloads",
);
assert(
    (schedulerLoopStep.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice\b/g) || []).length === 1,
    "std/gui turn virtual scheduler loop public step must call F5ee slice exactly once",
);
assertMatch(
    turnVirtualSchedulerLoopImpl,
    /LoopSliceFailed:[\s\S]*lower\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerSliceError/,
    "std/gui turn virtual scheduler loop failure must wrap only the lower F5ee slice error",
);
assertNoMatch(
    turnVirtualSchedulerLoopImpl,
    /_:/,
    "std/gui turn virtual scheduler loop must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerLoopImpl,
    /\b(?:while|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op|virtual_scheduler_step|virtual_scheduler_drain|transition_from_drain_result|virtual_scheduler_advance_timer|virtual_timer_advance|turn_driver_complete|executor_session_turn_driver_complete)\b/i,
    "std/gui turn virtual scheduler loop must not call lower scheduler phases, advance timers, complete host execution, queue, call platform APIs, raw render APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerLoopImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoop(?:Policy|Yield|AwaitTimer|ExecuteHostAction|Done|Result|SliceFailed|Error)\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoop(?:Policy|Yield|AwaitTimer|ExecuteHostAction|Done|Result|SliceFailed|Error)\s*:/,
    "std/gui turn virtual scheduler loop owner-bearing payloads and policy must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerLoopImpl,
    /[()]/,
    "std/gui turn virtual scheduler loop implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    turnVirtualSchedulerLoopActionImpl,
    /LoopAction:[\s\S]*YieldToClock\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionYieldToClock[\s\S]*AwaitTimerAdvance\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionAwaitTimerAdvance[\s\S]*ExecuteHostAction\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionExecuteHostAction[\s\S]*Complete\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionComplete/,
    "std/gui turn virtual scheduler loop action must expose explicit yield, timer, host-action, and complete actions",
);
assertMatch(
    turnVirtualSchedulerLoopActionImpl,
    /LoopActionYieldToClock:[\s\S]*state\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState[\s\S]*remaining_count\s+%i32[\s\S]*yield_delay_ms\s+%i32[\s\S]*LoopActionAwaitTimerAdvance:[\s\S]*pending\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualTimerPending[\s\S]*remaining_count\s+%i32[\s\S]*LoopActionExecuteHostAction:[\s\S]*execute\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerExecute[\s\S]*remaining_count\s+%i32[\s\S]*LoopActionComplete:[\s\S]*completed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerCompleted[\s\S]*remaining_count\s+%i32/,
    "std/gui turn virtual scheduler loop action payloads must preserve authority payloads and remaining counts",
);
assertMatch(
    turnVirtualSchedulerLoopActionImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop action must import F5ef loop boundary",
);
assertNoMatch(
    turnVirtualSchedulerLoopActionImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler(?:_step|_drain|_transition|_slice)?"\s+as\s+\*|#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer"\s+as\s+\*|#import\s+"std\/gui\/virtual_timer"\s+as\s+\*|#import\s+"std\/gui\/host"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop action must not import F5ea/F5eb/F5ec/F5ed/F5ee, virtual timer, or host modules directly",
);
assertNoMatch(
    textSliceBetween(
        turnVirtualSchedulerLoopActionImpl,
        "pub struct GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionYieldToClock:",
        "pub enum GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopAction:",
    ),
    /VirtualSchedulerLoop(?:Yield|AwaitTimer|ExecuteHostAction|Done)\b/,
    "std/gui turn virtual scheduler loop action payloads must not expose F5ef loop payload structs",
);
const schedulerLoopActionFromResult = functionSlice(turnVirtualSchedulerLoopActionImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_from_result");
assertMatch(
    schedulerLoopActionFromResult,
    /LoopResult::Yield[\s\S]*loop_yield_remaining_count\s+&loop_payload[\s\S]*loop_yield_yield_delay_ms\s+&loop_payload[\s\S]*loop_yield_state\s+loop_payload[\s\S]*LoopAction::YieldToClock\s+payload[\s\S]*LoopResult::AwaitTimer[\s\S]*loop_await_timer_remaining_count\s+&loop_payload[\s\S]*loop_await_timer_pending\s+loop_payload[\s\S]*LoopAction::AwaitTimerAdvance\s+payload[\s\S]*LoopResult::ExecuteHostAction[\s\S]*loop_execute_host_action_remaining_count\s+&loop_payload[\s\S]*loop_execute_host_action_execute\s+loop_payload[\s\S]*LoopAction::ExecuteHostAction\s+payload[\s\S]*LoopResult::Done[\s\S]*loop_done_remaining_count\s+&loop_payload[\s\S]*loop_done_completed\s+loop_payload[\s\S]*LoopAction::Complete\s+payload/,
    "std/gui turn virtual scheduler loop action must explicitly rewrap every F5ef loop result variant",
);
assertNoMatch(
    schedulerLoopActionFromResult,
    /\bResult::|\bResult\s+|LoopActionError/,
    "std/gui turn virtual scheduler loop action total mapping must not invent an error Result",
);
assertNoMatch(
    turnVirtualSchedulerLoopActionImpl,
    /_:/,
    "std/gui turn virtual scheduler loop action must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerLoopActionImpl,
    /\b(?:while|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op|virtual_scheduler_loop_step|virtual_scheduler_slice|virtual_scheduler_step|virtual_scheduler_drain|transition_from_drain_result|virtual_scheduler_advance_timer|virtual_timer_advance|turn_driver_complete|executor_session_turn_driver_complete)\b/i,
    "std/gui turn virtual scheduler loop action must not call loop step, lower scheduler phases, advance timers, complete host execution, queue, call platform APIs, raw render APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerLoopActionImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopAction(?:YieldToClock|AwaitTimerAdvance|ExecuteHostAction|Complete)?\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopAction(?:YieldToClock|AwaitTimerAdvance|ExecuteHostAction|Complete)?\s*:/,
    "std/gui turn virtual scheduler loop action owner-bearing payloads and action enum must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerLoopActionImpl,
    /[()]/,
    "std/gui turn virtual scheduler loop action implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    turnVirtualSchedulerLoopTimerAdvanceImpl,
    /LoopTimerAdvanceCompleted:[\s\S]*state\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState[\s\S]*remaining_count\s+%i32[\s\S]*LoopTimerAdvanceFailed:[\s\S]*lower\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerAdvanceError[\s\S]*remaining_count\s+%i32[\s\S]*LoopTimerAdvanceError:[\s\S]*AdvanceFailed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopTimerAdvanceFailed/,
    "std/gui turn virtual scheduler loop timer advance must expose typed success and lower-error result payloads",
);
assertMatch(
    turnVirtualSchedulerLoopTimerAdvanceImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler"\s+as\s+\*[\s\S]*#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop timer advance must import F5ea and F5eg boundaries",
);
assertNoMatch(
    turnVirtualSchedulerLoopTimerAdvanceImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_(?:step|drain|transition|slice|loop)"\s+as\s+\*|#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer"\s+as\s+\*|#import\s+"std\/gui\/virtual_timer"\s+as\s+\*|#import\s+"std\/gui\/host"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop timer advance must not import F5eb/F5ec/F5ed/F5ee/F5ef, direct virtual timer, or host modules",
);
const schedulerLoopTimerAdvance = functionSlice(turnVirtualSchedulerLoopTimerAdvanceImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance");
assertMatch(
    schedulerLoopTimerAdvance,
    /%fn\s+&GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerPolicy\s+fn\s+GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionAwaitTimerAdvance\s+fn\s+i32\s+Result/,
    "std/gui turn virtual scheduler loop timer advance must consume only AwaitTimerAdvance payload plus policy and delta",
);
assertMatch(
    schedulerLoopTimerAdvance,
    /loop_action_await_timer_advance_remaining_count\s+&action[\s\S]*loop_action_await_timer_advance_pending\s+action[\s\S]*virtual_scheduler_advance_timer\s+policy\s+pending\s+delta_ms[\s\S]*Result::Err\s+lower[\s\S]*AdvanceFailed\s+payload[\s\S]*Result::Ok\s+state[\s\S]*Result::Ok\s+payload/,
    "std/gui turn virtual scheduler loop timer advance must preserve remaining_count before owner consumption and call F5ea timer advance",
);
assert(
    (schedulerLoopTimerAdvance.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_advance_timer\b/g) || []).length === 1,
    "std/gui turn virtual scheduler loop timer advance public entry must call F5ea advance exactly once",
);
assertNoMatch(
    schedulerLoopTimerAdvance,
    /loop_action_from_result|LoopAction::|virtual_scheduler_loop_step|virtual_scheduler_slice|virtual_scheduler_drain|transition_from_drain_result|virtual_scheduler_step|virtual_timer_advance|turn_driver_complete|executor_session_turn_driver_complete/,
    "std/gui turn virtual scheduler loop timer advance must not rebuild actions, loop, lower-step, direct virtual-timer, or executor completion",
);
assertNoMatch(
    turnVirtualSchedulerLoopTimerAdvanceImpl,
    /_:/,
    "std/gui turn virtual scheduler loop timer advance must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerLoopTimerAdvanceImpl,
    /\b(?:while|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op)\b/i,
    "std/gui turn virtual scheduler loop timer advance must not queue, call platform/raw APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerLoopTimerAdvanceImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopTimerAdvance(?:Completed|Failed|Error)?\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopTimerAdvance(?:Completed|Failed|Error)?\s*:/,
    "std/gui turn virtual scheduler loop timer advance owner-bearing payloads and error enum must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerLoopTimerAdvanceImpl,
    /[()]/,
    "std/gui turn virtual scheduler loop timer advance implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    turnVirtualSchedulerLoopExecutorCompleteImpl,
    /LoopExecutorCompletePolicy:[\s\S]*scheduler_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnSchedulerPolicy[\s\S]*timer_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerPolicy/,
    "std/gui turn virtual scheduler loop executor complete must expose a policy with scheduler and timer policy only",
);
assertMatch(
    turnVirtualSchedulerLoopExecutorCompleteImpl,
    /LoopExecutorCompleteCompleted:[\s\S]*state\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerState[\s\S]*remaining_count\s+%i32[\s\S]*DriverCompleteFailed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompleteDriverCompleteFailed[\s\S]*SchedulerDecisionFailed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompleteSchedulerDecisionFailed[\s\S]*TimerDecisionFailed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompleteTimerDecisionFailed/,
    "std/gui turn virtual scheduler loop executor complete must expose typed success and lower-error result payloads",
);
assertMatch(
    turnVirtualSchedulerLoopExecutorCompleteImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_driver"\s+as\s+\*[\s\S]*#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_scheduler"\s+as\s+\*[\s\S]*#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler"\s+as\s+\*[\s\S]*#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop executor complete must import F5du, F5dv, F5ea, and F5eg boundaries",
);
assertNoMatch(
    turnVirtualSchedulerLoopExecutorCompleteImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn(?:_step|_virtual_scheduler_(?:step|drain|transition|slice|loop))"\s+as\s+\*|#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer"\s+as\s+\*|#import\s+"std\/gui\/virtual_timer"\s+as\s+\*|#import\s+"std\/gui\/host"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop executor complete must not import F5dt/F5eb/F5ec/F5ed/F5ee/F5ef, direct virtual timer, or host modules",
);
const schedulerLoopExecutorComplete = functionSlice(turnVirtualSchedulerLoopExecutorCompleteImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete");
assertMatch(
    schedulerLoopExecutorComplete,
    /%fn\s+&GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorCompletePolicy\s+fn\s+GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionExecuteHostAction\s+fn\s+Result\s+unit\s+GuiError\s+Result/,
    "std/gui turn virtual scheduler loop executor complete must consume only ExecuteHostAction payload plus policy and caller supplied outcome",
);
assertMatch(
    schedulerLoopExecutorComplete,
    /loop_action_execute_host_action_remaining_count\s+&action[\s\S]*loop_action_execute_host_action_execute\s+action[\s\S]*virtual_scheduler_execute_timer_state\s+&execute[\s\S]*virtual_scheduler_execute_pending\s+execute[\s\S]*turn_driver_complete\s+pending\s+outcome[\s\S]*Result::Err\s+lower[\s\S]*DriverCompleteFailed\s+payload[\s\S]*Result::Ok\s+driver_step[\s\S]*scheduler_decide\s+scheduler_policy\s+driver_step[\s\S]*SchedulerDecisionFailed\s+payload[\s\S]*Result::Ok\s+decision[\s\S]*virtual_scheduler_decide\s+timer_policy\s+timer_state\s+decision[\s\S]*TimerDecisionFailed\s+payload[\s\S]*Result::Ok\s+state[\s\S]*Result::Ok\s+payload/,
    "std/gui turn virtual scheduler loop executor complete must preserve remaining_count and timer_state before owner consumption and call F5du/F5dv/F5ea",
);
assert(
    (schedulerLoopExecutorComplete.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_driver_complete\b/g) || []).length === 1 &&
        (schedulerLoopExecutorComplete.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_scheduler_decide\b/g) || []).length === 1 &&
        (schedulerLoopExecutorComplete.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_decide\b/g) || []).length === 1,
    "std/gui turn virtual scheduler loop executor complete public entry must call F5du/F5dv/F5ea exactly once",
);
assertNoMatch(
    schedulerLoopExecutorComplete,
    /loop_action_from_result|LoopAction::|virtual_scheduler_loop_step|virtual_scheduler_slice|virtual_scheduler_drain|transition_from_drain_result|virtual_scheduler_step|virtual_scheduler_advance_timer|virtual_timer_advance|turn_step_complete|Result::Ok unit|Result::Err GuiError::/,
    "std/gui turn virtual scheduler loop executor complete must not rebuild actions, loop, lower-step, direct virtual-timer, or synthesize outcomes",
);
assertNoMatch(
    turnVirtualSchedulerLoopExecutorCompleteImpl,
    /_:/,
    "std/gui turn virtual scheduler loop executor complete must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerLoopExecutorCompleteImpl,
    /\b(?:while|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op)\b/i,
    "std/gui turn virtual scheduler loop executor complete must not queue, call platform/raw APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerLoopExecutorCompleteImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorComplete(?:Policy|Completed|DriverCompleteFailed|SchedulerDecisionFailed|TimerDecisionFailed|Error)?\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopExecutorComplete(?:Policy|Completed|DriverCompleteFailed|SchedulerDecisionFailed|TimerDecisionFailed|Error)?\s*:/,
    "std/gui turn virtual scheduler loop executor complete owner-bearing payloads and error enum must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerLoopExecutorCompleteImpl,
    /[()]/,
    "std/gui turn virtual scheduler loop executor complete implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    turnVirtualSchedulerLoopYieldCompleteImpl,
    /YieldReady\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldReady[\s\S]*YieldPending\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldPending[\s\S]*DeltaInvalid[\s\S]*YieldDelayInvalid[\s\S]*AdvanceFailed\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldAdvanceFailed/,
    "std/gui turn virtual scheduler loop yield complete must expose ready/pending result and typed delta/yield-delay failures",
);
assertMatch(
    turnVirtualSchedulerLoopYieldCompleteImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler"\s+as\s+\*[\s\S]*#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop yield complete must import F5ea and F5eg boundaries",
);
assertNoMatch(
    turnVirtualSchedulerLoopYieldCompleteImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn(?:_driver|_scheduler|_step|_virtual_timer|_virtual_scheduler_(?:step|drain|transition|slice|loop|loop_timer_advance|loop_executor_complete))"\s+as\s+\*|#import\s+"std\/gui\/virtual_timer"\s+as\s+\*|#import\s+"std\/gui\/host"\s+as\s+\*/,
    "std/gui turn virtual scheduler loop yield complete must not import scheduler internals beyond F5ea/F5eg or host modules",
);
const schedulerLoopYieldAdvance = functionSlice(turnVirtualSchedulerLoopYieldCompleteImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_yield_advance");
assertMatch(
    schedulerLoopYieldAdvance,
    /%fn\s+GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionYieldToClock\s+fn\s+i32\s+Result[\s\S]*loop_action_yield_to_clock_remaining_count\s+&action[\s\S]*loop_action_yield_to_clock_yield_delay_ms\s+&action[\s\S]*lt\s+delta_ms\s+0[\s\S]*DeltaInvalid\s+action\s+delta_ms[\s\S]*lt\s+yield_delay_ms\s+0[\s\S]*YieldDelayInvalid\s+action\s+delta_ms[\s\S]*loop_action_yield_to_clock_state\s+action[\s\S]*lt\s+delta_ms\s+yield_delay_ms[\s\S]*sub\s+yield_delay_ms\s+delta_ms[\s\S]*YieldPending\s+pending[\s\S]*YieldReady\s+ready/,
    "std/gui turn virtual scheduler loop yield complete must validate delta and yield delay before subtracting and returning pending/ready",
);
assert(
    schedulerLoopYieldAdvance.indexOf("sub yield_delay_ms delta_ms") > schedulerLoopYieldAdvance.indexOf("lt yield_delay_ms 0") &&
        schedulerLoopYieldAdvance.indexOf("sub yield_delay_ms delta_ms") > schedulerLoopYieldAdvance.indexOf("lt delta_ms yield_delay_ms"),
    "std/gui turn virtual scheduler loop yield complete must subtract only after validation and pending branch",
);
const schedulerLoopCompleteAck = functionSlice(turnVirtualSchedulerLoopYieldCompleteImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_complete_ack");
const schedulerLoopYieldCompleteRuntimeImpl = turnVirtualSchedulerLoopYieldCompleteImpl
    .replace(/impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldAdvanceErrorKind\s*:/g, "")
    .replace(/impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteYieldAdvanceErrorKind\s*:/g, "");
assertMatch(
    schedulerLoopCompleteAck,
    /%fn\s+GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopActionComplete\s+GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldCompleteCompleted[\s\S]*loop_action_complete_remaining_count\s+&action[\s\S]*loop_action_complete_completed\s+action[\s\S]*yield_complete_completed_new\s+completed\s+remaining_count/,
    "std/gui turn virtual scheduler loop yield complete must ack Complete by reading remaining_count before consuming completed owner",
);
assertNoMatch(
    turnVirtualSchedulerLoopYieldCompleteImpl,
    /_:/,
    "std/gui turn virtual scheduler loop yield complete must not use wildcard enum matches",
);
assertNoMatch(
    schedulerLoopYieldCompleteRuntimeImpl,
    /\b(?:while|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op)\b/i,
    "std/gui turn virtual scheduler loop yield complete must not queue, call platform/raw APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerLoopYieldCompleteImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldComplete(?!YieldAdvanceErrorKind\b)[A-Za-z0-9]*\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopYieldComplete(?!YieldAdvanceErrorKind\b)[A-Za-z0-9]*\s*:/,
    "std/gui turn virtual scheduler loop yield complete owner-bearing payloads and error enum must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerLoopYieldCompleteImpl,
    /[()]/,
    "std/gui turn virtual scheduler loop yield complete implementation must preserve NEPL prefix style without parentheses",
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
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler state contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler single-step contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler bounded drain contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler transition contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler slice contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler loop contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler loop action contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler loop timer advance contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler loop executor complete contract",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler loop yield complete contract",
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
assertMatch(
    guiStdTurnVirtualSchedulerTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_phase_state_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_ready_empty_timer_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_no_loop_no_backend_no_queue_no_fallback/,
    "std/gui turn virtual scheduler focused doctest must cover phase-owned state, empty one-shot timer state, and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerStepTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step_policy_no_dynamic_timer_state_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step_result_blocked_phase_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step_turn_exact_authority_order_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step_no_loop_timeslice_backend_queue_fallback/,
    "std/gui turn virtual scheduler step focused doctest must cover policy, blocked result, Turn path order, and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerDrainTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain_policy_max_advance_count_validation_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain_budget_exhausted_terminal_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain_zero_budget_no_step_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain_no_backend_queue_fallback/,
    "std/gui turn virtual scheduler drain focused doctest must cover budget validation, zero-budget terminal, and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerTransitionTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition_variants_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition_drain_terminal_mapping_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition_payload_rewrap_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition_remaining_count_preserved_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_transition_no_backend_queue_fallback/,
    "std/gui turn virtual scheduler transition focused doctest must cover variant mapping, payload rewrap, remaining count preservation, and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerSliceTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice_policy_validation_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice_result_variants_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice_one_drain_one_transition_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice_yield_slice_payload_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice_payload_rewrap_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice_drain_failure_lower_only_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice_no_backend_queue_fallback/,
    "std/gui turn virtual scheduler slice focused doctest must cover policy validation, one drain/transition, payload rewrap, lower-only failure, and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerLoopTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_facade_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_policy_owns_f5ee_only_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_result_variants_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_one_slice_call_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_payload_rewrap_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_lower_only_slice_error_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_no_wildcard_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_no_timer_executor_backend_queue_fallback/,
    "std/gui turn virtual scheduler loop focused doctest must cover F5ee-only policy, one slice call, payload rewrap, lower-only failure, and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerLoopActionTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_facade_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_variants_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_imports_f5ef_only_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_explicit_match_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_payload_rewrap_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_total_mapping_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_no_wildcard_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action_no_timer_executor_backend_queue_fallback/,
    "std/gui turn virtual scheduler loop action focused doctest must cover F5ef-only total mapping and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerLoopTimerAdvanceTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance_facade_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance_result_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance_imports_f5eg_f5ea_only_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance_consumes_await_timer_action_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance_one_f5ea_call_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance_remaining_count_preserved_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance_lower_error_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance_no_wildcard_backend_queue_fallback/,
    "std/gui turn virtual scheduler loop timer advance focused doctest must cover F5eg/F5ea timer authority and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerLoopExecutorCompleteTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_facade_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_policy_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_result_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_imports_f5eg_f5du_f5dv_f5ea_only_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_consumes_execute_host_action_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_caller_supplied_outcome_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_driver_scheduler_timer_order_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_remaining_count_preserved_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_lower_error_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_no_wildcard_backend_queue_fallback/,
    "std/gui turn virtual scheduler loop executor complete focused doctest must cover F5eg/F5du/F5dv/F5ea executor authority and no backend/queue/fallback policy",
);
assertMatch(
    guiStdTurnVirtualSchedulerLoopYieldCompleteTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_facade_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_result_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_error_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_imports_f5eg_f5ea_only_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_yield_advance_reads_before_consume_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_yield_advance_validates_delta_and_delay_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_yield_advance_pending_ready_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_complete_ack_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_no_wildcard_backend_queue_fallback/,
    "std/gui turn virtual scheduler loop yield complete focused doctest must cover F5eg/F5ea yield/complete authority and no backend/queue/fallback policy",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler real loop step contract",
);
assertMatch(
    turnVirtualSchedulerRealLoopStep,
    /RealLoopStepPolicy[\s\S]*scheduler_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnSchedulerPolicy[\s\S]*timer_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnTimerPolicy[\s\S]*ClockDelta\s+%i32[\s\S]*ExecutorOutcome\s+%Result\s+unit\s+GuiError[\s\S]*CompleteAck[\s\S]*YieldInputMismatch[\s\S]*TimerInputMismatch[\s\S]*ExecuteInputMismatch[\s\S]*CompleteInputMismatch/,
    "std/gui turn virtual scheduler real loop step must expose single timer-policy authority, explicit inputs, and mismatch errors",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopStep,
    /executor_policy\s+%|LoopExecutorCompletePolicy/,
    "std/gui turn virtual scheduler real loop step must not store executor policy and duplicate timer policy authority",
);
assertMatch(
    turnVirtualSchedulerRealLoopStepImpl,
    /#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_action"\s+as\s+\*[\s\S]*#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance"\s+as\s+\*[\s\S]*#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete"\s+as\s+\*[\s\S]*#import\s+"std\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete"\s+as\s+\*/,
    "std/gui turn virtual scheduler real loop step must import F5eg/F5eh/F5ei/F5ej authorities",
);
const schedulerRealLoopStep = functionSlice(turnVirtualSchedulerRealLoopStepImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step");
assertMatch(
    schedulerRealLoopStep,
    /LoopAction::YieldToClock\s+yield_action[\s\S]*RealLoopStepInput::ClockDelta\s+delta_ms[\s\S]*loop_yield_complete_yield_advance\s+yield_action\s+delta_ms[\s\S]*LoopAction::AwaitTimerAdvance\s+timer_action[\s\S]*RealLoopStepInput::ClockDelta\s+delta_ms[\s\S]*loop_timer_advance\s+timer_policy\s+timer_action\s+delta_ms[\s\S]*LoopAction::ExecuteHostAction\s+execute_action[\s\S]*RealLoopStepInput::ExecutorOutcome\s+outcome[\s\S]*loop_executor_complete_with_policy_refs\s+scheduler_policy\s+timer_policy\s+execute_action\s+outcome[\s\S]*LoopAction::Complete\s+complete_action[\s\S]*RealLoopStepInput::CompleteAck[\s\S]*loop_yield_complete_complete_ack\s+complete_action/,
    "std/gui turn virtual scheduler real loop step must dispatch exact action/input pairs to typed authorities",
);
assert(
    (schedulerRealLoopStep.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_yield_advance\b/g) || []).length === 1 &&
        (schedulerRealLoopStep.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_timer_advance\b/g) || []).length === 1 &&
        (schedulerRealLoopStep.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_executor_complete_with_policy_refs\b/g) || []).length === 1 &&
        (schedulerRealLoopStep.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_yield_complete_complete_ack\b/g) || []).length === 1,
    "std/gui turn virtual scheduler real loop step must call each typed authority exactly once",
);
assertNoMatch(
    schedulerRealLoopStep,
    /turn_driver_complete|scheduler_decide|virtual_scheduler_decide|virtual_scheduler_loop_step|virtual_scheduler_slice|virtual_scheduler_drain|transition_from_drain_result|Result::Ok unit|Result::Err GuiError::/,
    "std/gui turn virtual scheduler real loop step must not bypass typed authorities or synthesize outcomes",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopStepImpl,
    /_:/,
    "std/gui turn virtual scheduler real loop step must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopStepImpl,
    /\b(?:while|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op)\b/i,
    "std/gui turn virtual scheduler real loop step must not queue, call platform/raw APIs, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopStepImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStep[A-Za-z0-9]*\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStep[A-Za-z0-9]*\s*:/,
    "std/gui turn virtual scheduler real loop step owner-bearing payloads and error enum must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopStepImpl,
    /[()]/,
    "std/gui turn virtual scheduler real loop step implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    guiStdTurnVirtualSchedulerRealLoopStepTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_facade_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_policy_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_input_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_result_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_error_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_imports_f5eg_f5eh_f5ei_f5ej_only_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_dispatch_pairs_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_mismatch_owner_recovery_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_single_timer_policy_authority_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step_no_wildcard_backend_queue_fallback/,
    "std/gui turn virtual scheduler real loop step focused doctest must cover explicit input dispatch and no backend/queue/fallback policy",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler real loop driver contract",
);
assertMatch(
    turnVirtualSchedulerDrain,
    /RemainingCountInvalid[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain_validate_remaining_count[\s\S]*drain_remaining_count_invalid_result[\s\S]*PolicyInvalid[\s\S]*gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain_resume[\s\S]*Result::Ok\s+policy_count[\s\S]*Result::Ok\s+count[\s\S]*gt\s+count\s+policy_count[\s\S]*drain_remaining_count_invalid_result\s+state[\s\S]*drain_remaining\s+policy\s+state\s+count/,
    "std/gui turn virtual scheduler drain must expose checked resume budget boundary and reject over-budget resume",
);
assertMatch(
    turnVirtualSchedulerSlice,
    /gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_slice_resume[\s\S]*drain_resume\s+drain_policy\s+state\s+remaining_count/,
    "std/gui turn virtual scheduler slice must resume with caller supplied remaining count",
);
assertMatch(
    turnVirtualSchedulerLoop,
    /gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_resume[\s\S]*slice_resume\s+slice_policy\s+state\s+remaining_count/,
    "std/gui turn virtual scheduler loop must resume with caller supplied remaining count",
);
assertMatch(
    turnVirtualSchedulerRealLoopDriver,
    /RealLoopDriverPolicy[\s\S]*loop_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerLoopPolicy[\s\S]*NeedInput[\s\S]*Completed[\s\S]*LoopStepFailed[\s\S]*StepRemainingInvalid/,
    "std/gui turn virtual scheduler real loop driver must expose loop-only policy, request result, completion result, and typed errors",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopDriver,
    /RealLoopStepPolicy|scheduler_policy\s+%|timer_policy\s+%|ExecutorOutcome|ClockDelta|CompleteAck/,
    "std/gui turn virtual scheduler real loop driver must not own F5ek policy or backend input authority",
);
const schedulerRealLoopDriverNeedInputFromLoopResult = functionSlice(turnVirtualSchedulerRealLoopDriverImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_need_input_from_loop_result");
assertMatch(
    schedulerRealLoopDriverNeedInputFromLoopResult,
    /loop_action_from_result\s+loop_result[\s\S]*RealLoopDriverResult::NeedInput/,
    "std/gui turn virtual scheduler real loop driver must map loop result to action request through F5eg",
);
const schedulerRealLoopDriverStart = functionSlice(turnVirtualSchedulerRealLoopDriverImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_start");
assertMatch(
    schedulerRealLoopDriverStart,
    /loop_policy_ref\s+policy[\s\S]*virtual_scheduler_loop_step\s+loop_policy\s+state[\s\S]*loop_step_failed_result\s+lower[\s\S]*need_input_from_loop_result\s+loop_result/,
    "std/gui turn virtual scheduler real loop driver start must call F5ef loop step and F5eg action mapping",
);
assert(
    (schedulerRealLoopDriverStart.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_loop_step\b/g) || []).length === 1,
    "std/gui turn virtual scheduler real loop driver start must call F5ef loop step exactly once",
);
const schedulerRealLoopDriverResumeState = functionSlice(turnVirtualSchedulerRealLoopDriverImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_resume_state");
assertMatch(
    schedulerRealLoopDriverResumeState,
    /loop_policy_ref\s+policy[\s\S]*virtual_scheduler_loop_resume\s+loop_policy\s+state\s+remaining_count[\s\S]*loop_step_failed_result\s+lower[\s\S]*need_input_from_loop_result\s+loop_result/,
    "std/gui turn virtual scheduler real loop driver resume must call F5ef resume and F5eg action mapping",
);
const schedulerRealLoopDriverAfterStep = functionSlice(turnVirtualSchedulerRealLoopDriverImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_after_step");
assertMatch(
    schedulerRealLoopDriverAfterStep,
    /RealLoopStepResult::StateReady\s+ready[\s\S]*state_ready_remaining_count\s+&ready[\s\S]*state_ready_state\s+ready[\s\S]*lt\s+remaining_count\s+0[\s\S]*StepRemainingInvalid[\s\S]*real_loop_driver_resume_state\s+policy\s+state\s+remaining_count[\s\S]*RealLoopStepResult::YieldPending\s+pending[\s\S]*yield_pending_action\s+pending[\s\S]*LoopAction::YieldToClock\s+yield_action[\s\S]*RealLoopStepResult::Completed\s+completed_result[\s\S]*completed_remaining_count\s+&completed_result[\s\S]*completed_completed\s+completed_result/,
    "std/gui turn virtual scheduler real loop driver after-step must route StateReady/YieldPending/Completed explicitly",
);
assertNoMatch(
    schedulerRealLoopDriverAfterStep,
    /\b(?:le|eq)\s+remaining_count\s+0|CompleteAck|Result::Err\s+GuiError::|Result::Ok\s+unit/,
    "std/gui turn virtual scheduler real loop driver must not turn zero remaining budget into completion, CompleteAck, synthetic outcome, or fallback error",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopDriverImpl,
    /_:/,
    "std/gui turn virtual scheduler real loop driver must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopDriverImpl,
    /\b(?:while|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op|turn_driver_complete|scheduler_decide|virtual_scheduler_decide|loop_timer_advance|loop_executor_complete|loop_yield_complete)\b/i,
    "std/gui turn virtual scheduler real loop driver must not queue, call platform/raw APIs, bypass authorities, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopDriverImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopDriver[A-Za-z0-9]*\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopDriver[A-Za-z0-9]*\s*:/,
    "std/gui turn virtual scheduler real loop driver owner-bearing payloads and error enum must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerRealLoopDriverImpl,
    /[()]/,
    "std/gui turn virtual scheduler real loop driver implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    guiStdTurnVirtualSchedulerRealLoopDriverTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_facade_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_policy_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_result_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_error_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_start_dispatch_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_after_step_dispatch_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_resume_budget_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_negative_remaining_fail_closed_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_over_budget_fail_closed_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_zero_budget_yield_semantics_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_no_wildcard_backend_queue_fallback/,
    "std/gui turn virtual scheduler real loop driver focused doctest must cover policy, dispatch, resume budget, zero-budget yield, and no backend/queue/fallback policy",
);
assertMatch(
    stdGuiFacade,
    /#import\s+"\.\/gui\/tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step"\s+as\s+\*/,
    "std/gui facade must re-export the virtual scheduler headless app-loop step contract",
);
assertMatch(
    turnVirtualSchedulerHeadlessAppLoopStep,
    /HeadlessAppLoopStepPolicy[\s\S]*driver_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopDriverPolicy[\s\S]*step_policy\s+%GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerRealLoopStepPolicy[\s\S]*NeedInput[\s\S]*Completed[\s\S]*StartFailed[\s\S]*RealStepFailed[\s\S]*AfterStepFailed/,
    "std/gui turn virtual scheduler headless app-loop step must expose two-policy step result and typed lower errors",
);
assertNoMatch(
    turnVirtualSchedulerHeadlessAppLoopStep,
    /LoopPolicy|DrainPolicy|SlicePolicy|scheduler_policy\s+%|timer_policy\s+%|GuiHost|TimerRequest|TimerEvent/,
    "std/gui turn virtual scheduler headless app-loop step must not own lower scheduler/timer/backend authority",
);
const schedulerHeadlessAppLoopStepResultFromDriverResult = functionSlice(turnVirtualSchedulerHeadlessAppLoopStepImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_result_from_driver_result");
assertMatch(
    schedulerHeadlessAppLoopStepResultFromDriverResult,
    /RealLoopDriverResult::NeedInput\s+need[\s\S]*real_loop_driver_need_input_action\s+need[\s\S]*HeadlessAppLoopStepResult::NeedInput\s+payload[\s\S]*RealLoopDriverResult::Completed\s+completed_result[\s\S]*real_loop_driver_completed_remaining_count\s+&completed_result[\s\S]*real_loop_driver_completed_completed\s+completed_result[\s\S]*HeadlessAppLoopStepResult::Completed\s+payload/,
    "std/gui turn virtual scheduler headless app-loop step must map driver request/completion without synthesizing app-loop input",
);
const schedulerHeadlessAppLoopStepStart = functionSlice(turnVirtualSchedulerHeadlessAppLoopStepImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_start");
assertMatch(
    schedulerHeadlessAppLoopStepStart,
    /driver_policy_ref\s+policy[\s\S]*real_loop_driver_start\s+driver_policy\s+state[\s\S]*start_failed_result\s+lower[\s\S]*result_from_driver_result\s+driver_result/,
    "std/gui turn virtual scheduler headless app-loop step start must call only F5el start and map typed lower error",
);
assert(
    (schedulerHeadlessAppLoopStepStart.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_start\b/g) || []).length === 1,
    "std/gui turn virtual scheduler headless app-loop step start must call F5el start exactly once",
);
assertNoMatch(
    schedulerHeadlessAppLoopStepStart,
    /\b(?:real_loop_step|real_loop_driver_after_step|CompleteAck|ExecutorOutcome|ClockDelta)\b/,
    "std/gui turn virtual scheduler headless app-loop step start must not advance or synthesize backend input",
);
const schedulerHeadlessAppLoopStepAdvance = functionSlice(turnVirtualSchedulerHeadlessAppLoopStepImpl, "gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_advance");
assertMatch(
    schedulerHeadlessAppLoopStepAdvance,
    /step_policy_ref\s+policy[\s\S]*real_loop_step\s+step_policy\s+action\s+input[\s\S]*real_step_failed_result\s+lower[\s\S]*driver_policy_ref\s+policy[\s\S]*real_loop_driver_after_step\s+driver_policy\s+step_result[\s\S]*after_step_failed_result\s+lower[\s\S]*result_from_driver_result\s+driver_result/,
    "std/gui turn virtual scheduler headless app-loop step advance must call F5ek once, then F5el after-step only after success",
);
assert(
    (schedulerHeadlessAppLoopStepAdvance.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_step\b/g) || []).length === 1,
    "std/gui turn virtual scheduler headless app-loop step advance must call F5ek step exactly once",
);
assert(
    (schedulerHeadlessAppLoopStepAdvance.match(/\bgui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_real_loop_driver_after_step\b/g) || []).length === 1,
    "std/gui turn virtual scheduler headless app-loop step advance must call F5el after-step exactly once",
);
assertNoMatch(
    schedulerHeadlessAppLoopStepAdvance,
    /\b(?:remaining_count\s+0|CompleteAck|ExecutorOutcome|ClockDelta|real_loop_driver_start|Result::Ok\s+unit|Result::Err\s+GuiError::)\b/,
    "std/gui turn virtual scheduler headless app-loop step advance must not synthesize CompleteAck, collapse zero budget, call start, or fallback to generic success/error",
);
assertNoMatch(
    turnVirtualSchedulerHeadlessAppLoopStepImpl,
    /_:/,
    "std/gui turn virtual scheduler headless app-loop step must not use wildcard enum matches",
);
assertNoMatch(
    turnVirtualSchedulerHeadlessAppLoopStepImpl,
    /\b(?:while|for|timeslice|schedule_timer|setTimeout|setInterval|GuiHost|std\/gui\/host|queue|platforms\/gui|platform|Canvas|DOM|minifb|video_memory|RenderTarget|DrawTarget|#extern|#intrinsic|fallback|silent no-op|virtual_scheduler_loop_step|virtual_scheduler_loop_resume|virtual_scheduler_slice|virtual_scheduler_drain|loop_timer_advance|loop_executor_complete|loop_yield_complete|scheduler_decide|virtual_scheduler_decide|turn_driver_complete)\b/i,
    "std/gui turn virtual scheduler headless app-loop step must not queue, call platform/raw APIs, bypass authorities, or fallback",
);
assertNoMatch(
    turnVirtualSchedulerHeadlessAppLoopStepImpl,
    /impl Clone for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerHeadlessAppLoopStep[A-Za-z0-9]*\s*:|impl Copy for GuiRgba8888RowTileRlePresentHostSpanOperationPresenterExecutorSessionTurnVirtualSchedulerHeadlessAppLoopStep[A-Za-z0-9]*\s*:/,
    "std/gui turn virtual scheduler headless app-loop step owner-bearing payloads and error enum must be non-Copy and non-Clone",
);
assertNoMatch(
    turnVirtualSchedulerHeadlessAppLoopStepImpl,
    /[()]/,
    "std/gui turn virtual scheduler headless app-loop step implementation must preserve NEPL prefix style without parentheses",
);
assertMatch(
    guiStdTurnVirtualSchedulerHeadlessAppLoopStepTests,
    /std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_facade_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_policy_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_result_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_error_shape_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_start_dispatch_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_advance_dispatch_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_completed_terminal_only_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_no_synthetic_complete_ack_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_lower_error_order_ok[\s\S]*std_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_headless_app_loop_step_no_wildcard_backend_queue_fallback/,
    "std/gui turn virtual scheduler headless app-loop step focused doctest must cover policy, dispatch, terminal completion, CompleteAck non-synthesis, and no backend/queue/fallback policy",
);

console.log("web GUI offscreen/headless contract passed");
