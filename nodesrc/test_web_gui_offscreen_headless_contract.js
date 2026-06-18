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
const stdGuiFacade = read("stdlib/std/gui.nepl");
const guiStdTests = read("tests/stdlib/gui_std.n.md");
const guiStdVirtualTimerTests = read("tests/stdlib/gui_std_virtual_timer.n.md");
const guiStdTurnVirtualTimerTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_timer.n.md");
const guiStdTurnVirtualSchedulerTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler.n.md");
const guiStdTurnVirtualSchedulerStepTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_step.n.md");
const guiStdTurnVirtualSchedulerDrainTests = read("tests/stdlib/gui_std_tile_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_drain.n.md");

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

console.log("web GUI offscreen/headless contract passed");
