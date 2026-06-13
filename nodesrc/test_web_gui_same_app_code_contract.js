#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

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

const stdSurfaceSource = read("stdlib/std/gui/surface.nepl");
const stdSurface = withoutComments(stdSurfaceSource);
const stdGuiFacade = withoutComments(read("stdlib/std/gui.nepl"));
const allocApp = withoutComments(read("stdlib/alloc/gui/app/types.nepl"));
const stdRuntime = withoutComments(read("stdlib/std/gui/runtime.nepl"));
const webSurfaceSource = read("stdlib/platforms/gui/web/surface.nepl");
const webSurface = withoutComments(webSurfaceSource);
const webFacade = withoutComments(read("stdlib/platforms/gui/web.nepl"));
const webStdoutSource = read("stdlib/platforms/gui/web/stdout_protocol.nepl");
const implementationPlan = read("doc/neplg2/gui_redesign_implementation_plan.md");

assert.match(
    stdGuiFacade,
    /#import\s+"\.\/gui\/surface"\s+as\s+\*/,
    "std/gui facade must expose the platform-neutral surface contract",
);
assert.doesNotMatch(
    stdSurface,
    /^\s*#import\s+"platforms\//m,
    "std/gui/surface must not import concrete platform modules",
);
assert.doesNotMatch(
    stdSurface,
    /\b(?:DOM|Canvas|HTMLCanvasElement|SharedArrayBuffer|ImageData|stdout|Win32|AppKit|Wayland|minifb)\b/i,
    "std/gui/surface must not expose web/native transport details",
);
assert.match(
    stdSurface,
    /pub\s+struct\s+GuiPixelBufferDescriptor:[\s\S]*surface\s+%SurfaceId[\s\S]*width\s+%i32[\s\S]*height\s+%i32[\s\S]*stride_bytes\s+%i32[\s\S]*format\s+%ColorFormat/,
    "std/gui/surface must define a typed pixel buffer descriptor",
);
assert.match(
    stdSurface,
    /pub\s+enum\s+GuiSurfacePresentCommand:[\s\S]*PresentPixelFrame\s+%GuiSurfaceFrame/,
    "std/gui/surface must define a typed pixel frame present command",
);
assert.match(
    stdSurface,
    /ColorFormat::FormatRgba8888[\s\S]*gui_rgba8888_stride_is_valid[\s\S]*Result::Err\s+GuiError::InvalidGeometry/,
    "std/gui/surface must validate Rgba8888 stride instead of accepting arbitrary buffers",
);
assert.match(
    stdSurface,
    /gui_rgba8888_stride_is_valid[\s\S]*ge\s+stride_bytes\s+mul\s+width\s+4[\s\S]*gui_stride_is_word_aligned[\s\S]*rem_s\s+stride_bytes\s+4/,
    "std/gui/surface must require Rgba8888 stride to be 4-byte aligned",
);
assert.match(
    stdSurface,
    /_\s*:[\s\S]*Result::Err\s+GuiError::Unsupported/,
    "std/gui/surface must reject unsupported pixel formats with a typed error",
);
assert.doesNotMatch(
    allocApp,
    /^\s*#import\s+"std\/gui|GuiSurfacePresentCommand|GuiPixelBufferDescriptor|GuiSurfaceFrame/m,
    "alloc/gui/app must not depend on std/gui surface command types",
);
assert.match(
    allocApp,
    /pub\s+struct\s+PresentSurfaceEffect:[\s\S]*surface\s+%i32[\s\S]*frame\s+%i32[\s\S]*width\s+%i32[\s\S]*height\s+%i32[\s\S]*stride_bytes\s+%i32[\s\S]*format\s+%ColorFormat[\s\S]*dirty\s+%DirtyRegion/,
    "alloc/gui/app must keep present surface as app-facing request data",
);
assert.match(
    allocApp,
    /pub\s+enum\s+GuiEffect:[\s\S]*PresentSurface\s+%PresentSurfaceEffect/,
    "GuiEffect must expose a present surface request without platform calls",
);
assert.match(
    stdRuntime,
    /^\s*#import\s+"std\/gui\/surface"\s+as\s+\*/m,
    "std/gui/runtime must own conversion from request data to checked surface command",
);
assert.match(
    stdRuntime,
    /pub\s+enum\s+GuiRuntimeCommand:[\s\S]*PresentSurface\s+%GuiSurfacePresentCommand/,
    "runtime command must carry the checked surface present command",
);
assert.match(
    stdRuntime,
    /GuiEffect::PresentSurface\s+payload:[\s\S]*gui_runtime_can_present_surface\s+host[\s\S]*gui_runtime_present_command_from_effect\s+payload[\s\S]*GuiError::Unsupported/,
    "runtime must capability-gate PresentSurface and return typed unsupported errors",
);
assert.match(
    stdRuntime,
    /surface_id_result\s+surface_raw[\s\S]*frame_id_result\s+frame_raw[\s\S]*gui_pixel_buffer_descriptor\s+surface\s+width\s+height\s+stride_bytes\s+format[\s\S]*GuiRuntimeCommand::PresentSurface/,
    "runtime must validate request ids and pixel descriptor before producing PresentSurface",
);

assert.match(
    webFacade,
    /#import\s+"\.\/web\/surface"\s+as\s+@merge[\s\S]*#import\s+"\.\/web\/stdout_protocol"\s+as\s+@merge/,
    "web facade must expose the formal surface contract before the legacy stdout transport",
);
assert.match(
    webSurface,
    /#import\s+"std\/gui"\s+as\s+\*/,
    "web surface must build on the std/gui surface contract",
);
assert.doesNotMatch(
    webSurface,
    /#import\s+"std\/stdio"|gui_web_stdout_|NEPLG2_GUI_FRAME|print\b|println\b/,
    "web surface contract must not depend on stdout protocol helpers",
);
assert.match(
    webSurface,
    /pub\s+struct\s+GuiWebVideoMemorySurface:[\s\S]*descriptor\s+%GuiPixelBufferDescriptor[\s\S]*slot_count\s+%i32[\s\S]*resize_generation\s+%i32/,
    "web video memory surface must wrap the standard pixel buffer descriptor",
);
assert.match(
    webSurface,
    /if\s+lt\s+slot_count\s+2:[\s\S]*Result::Err\s+GuiError::InvalidCommand/,
    "web video memory surface must reject single-slot presentation explicitly",
);
assert.match(
    webSurface,
    /gui_web_video_memory_present_command[\s\S]*GuiSurfacePresentCommand[\s\S]*gui_surface_present_pixel_frame/,
    "web video memory surface must produce the standard present command",
);

assert.match(
    webStdoutSource,
    /legacy smoke \/ debug transport|legacy smoke \/ debug|legacy smoke/,
    "stdout protocol docs must identify it as a legacy smoke/debug transport",
);
assert.match(
    implementationPlan,
    /再開 target は Phase 3\.5 の same app code host surface gate/,
    "implementation plan must identify Phase 3.5 as the resumed implementation target",
);

process.stdout.write(JSON.stringify({
    ok: true,
    checks: [
        "std/gui exposes platform-neutral pixel surface present commands",
        "alloc/gui keeps present surface as request data and runtime builds checked commands",
        "web surface wraps the standard descriptor instead of stdout transport",
        "legacy stdout transport remains quarantined from the formal app-facing contract",
    ],
}, null, 2) + "\n");
