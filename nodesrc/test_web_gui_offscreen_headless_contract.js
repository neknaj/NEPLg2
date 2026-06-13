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
const stdGuiFacade = read("stdlib/std/gui.nepl");
const guiStdTests = read("tests/stdlib/gui_std.n.md");

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
    implementationPlan,
    /Phase 5\.1:[\s\S]*stdlib\/std\/gui\/offscreen\.nepl[\s\S]*stdlib\/std\/gui\/virtual_event\.nepl/,
    "implementation plan must track the offscreen and virtual event implementation slice",
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
    guiStdTests,
    /gui_offscreen_snapshot_requires_offscreen_present_command[\s\S]*headless unsupported[\s\S]*window unsupported[\s\S]*device unsupported[\s\S]*noop unsupported/,
    "std/gui tests must cover offscreen-only snapshot behavior across non-offscreen surface kinds",
);
assertMatch(
    guiStdTests,
    /gui_virtual_event_script_replays_typed_events_without_sentinel[\s\S]*empty poll none[\s\S]*malformed empty rejected[\s\S]*malformed one rejected[\s\S]*cursor overflow rejected/,
    "std/gui tests must cover Option-based virtual event replay and malformed public constructor states",
);

console.log("web GUI offscreen/headless contract passed");
