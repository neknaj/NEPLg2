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

function walkNeplFiles(dir) {
    const files = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const child = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...walkNeplFiles(child));
        } else if (entry.isFile() && entry.name.endsWith(".nepl")) {
            files.push(child);
        }
    }
    return files;
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

const coreGuiRoot = path.join(repoRoot, "stdlib", "core", "gui");
for (const filePath of [
    path.join(repoRoot, "stdlib", "core", "gui.nepl"),
    ...walkNeplFiles(coreGuiRoot),
]) {
    const relPath = path.relative(repoRoot, filePath).split(path.sep).join("/");
    const text = fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
    const source = withoutComments(text);
    assertNoMatch(
        source,
        /^\s*#import\s+"(?:alloc|std|platforms)\//m,
        `${relPath} must not import alloc/std/platform modules`,
    );
    assertNoMatch(
        source,
        /\b(?:DOM|Canvas|ANSI|TTY|WASIX|Win32|UIKit|AndroidView|HtmlCanvas)\b/,
        `${relPath} must not expose concrete platform names in core/gui`,
    );
}

const allocGuiRoot = path.join(repoRoot, "stdlib", "alloc", "gui");
for (const filePath of [
    path.join(repoRoot, "stdlib", "alloc", "gui.nepl"),
    ...walkNeplFiles(allocGuiRoot),
]) {
    const relPath = path.relative(repoRoot, filePath).split(path.sep).join("/");
    const text = fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
    const source = withoutComments(text);
    assertNoMatch(
        source,
        /^\s*#import\s+"(?:std|platforms)\//m,
        `${relPath} must not import std/platform modules`,
    );
    assertNoMatch(
        source,
        /\b(?:DOM|Canvas|ANSI|TTY|WASIX|Win32|UIKit|AndroidView|HtmlCanvas)\b/,
        `${relPath} must not expose concrete platform names in alloc/gui`,
    );
}

const renderCommand = read("stdlib/core/gui/render_command.nepl");
const capabilitySource = read("stdlib/core/gui/capability.nepl");
const capabilityImpl = withoutComments(capabilitySource);
assertMatch(
    capabilityImpl,
    /pub\s+enum\s+SurfaceKind:[\s\S]*WindowPixel[\s\S]*OffscreenPixel[\s\S]*DevicePixel[\s\S]*TextGrid[\s\S]*Headless/,
    "SurfaceKind must distinguish visible, offscreen, device, text-grid, and headless surfaces",
);
assertNoMatch(
    capabilityImpl,
    /(?:^|\s)(?:Pixel|Command)(?:\s|$)/,
    "SurfaceKind must not keep old Pixel or Command surface variants",
);
assertMatch(
    capabilityImpl,
    /pub\s+fn\s+surface_kind_has_pixel_buffer[\s\S]*SurfaceKind::WindowPixel[\s\S]*SurfaceKind::OffscreenPixel[\s\S]*SurfaceKind::DevicePixel/,
    "core/gui capability must expose a platform-neutral pixel-buffer predicate",
);
assertMatch(renderCommand, /pub\s+struct\s+TextGridPoint:/, "core/gui must own TextGridPoint");
assertMatch(renderCommand, /pub\s+struct\s+TextCellStyle:/, "core/gui must own TextCellStyle");
assertMatch(renderCommand, /pub\s+struct\s+TextCellRun:/, "core/gui must own TextCellRun");
assertNoMatch(
    renderCommand,
    /pub\s+fn\s+text_cell_style_default\b/,
    "core/gui must not publish terminal-specific default TextCellStyle",
);

const stdGuiRoot = path.join(repoRoot, "stdlib", "std", "gui");
for (const filePath of [
    path.join(repoRoot, "stdlib", "std", "gui.nepl"),
    ...walkNeplFiles(stdGuiRoot),
]) {
    const relPath = path.relative(repoRoot, filePath).split(path.sep).join("/");
    const text = fs.readFileSync(filePath, "utf8").replace(/\r\n/g, "\n");
    const source = withoutComments(text);
    assertNoMatch(
        source,
        /^\s*#import\s+"platforms\//m,
        `${relPath} must not import concrete platform modules`,
    );
    assertNoMatch(
        source,
        /\b(?:DOM|Canvas|ANSI|TTY|WASIX|Win32|UIKit|AndroidView|HtmlCanvas)\b/,
        `${relPath} must not expose concrete platform names in std/gui`,
    );
}

const stdSurface = read("stdlib/std/gui/surface.nepl");
const stdSurfaceImpl = withoutComments(stdSurface);
assertMatch(
    stdSurfaceImpl,
    /pub\s+struct\s+GuiPixelBufferDescriptor:[\s\S]*surface\s+%SurfaceId[\s\S]*format\s+%ColorFormat/,
    "std/gui surface must define a platform-neutral pixel buffer descriptor",
);
assertMatch(
    stdSurfaceImpl,
    /pub\s+enum\s+GuiSurfacePresentCommand:[\s\S]*PresentPixelFrame\s+%GuiSurfaceFrame/,
    "std/gui surface must define a platform-neutral present command",
);
assertNoMatch(
    stdSurfaceImpl,
    /\b(?:DOM|Canvas|SharedArrayBuffer|ImageData|stdout|Win32|AppKit|Wayland|minifb)\b/i,
    "std/gui surface must not expose concrete platform transport details",
);

const terminalCapability = read("stdlib/platforms/gui/terminal/capability.nepl");
assertMatch(
    terminalCapability,
    /pub\s+struct\s+TerminalProfile:[\s\S]*capabilities\s+%GuiCapabilities[\s\S]*cols\s+%i32[\s\S]*rows\s+%i32/,
    "terminal profile must wrap common GuiCapabilities plus grid dimensions",
);
assertMatch(
    terminalCapability,
    /pub\s+fn\s+terminal_profile_full\s+%fn\s+i32\s+fn\s+i32\s+fn\s+GuiCapabilities\s+Result\s+TerminalProfile\s+GuiError/,
    "terminal_profile_full must validate custom capabilities through Result",
);
assertMatch(
    terminalCapability,
    /surface_kind_is_text_grid\s+gui_capabilities_surface_kind\s+&capabilities[\s\S]*Result::Err\s+GuiError::Unsupported/,
    "terminal_profile_full must reject non-TextGrid capabilities explicitly",
);
assertMatch(
    terminalCapability,
    /or\s+lt\s+cols\s+0\s+lt\s+rows\s+0[\s\S]*Result::Err\s+GuiError::InvalidGeometry/,
    "terminal_profile_full must reject negative grid sizes explicitly",
);

const terminalTextGrid = read("stdlib/platforms/gui/terminal/text_grid.nepl");
for (const typeName of ["TextGridPoint", "TextCellStyle", "TextCellRun"]) {
    assertNoMatch(
        terminalTextGrid,
        new RegExp(`pub\\s+struct\\s+${typeName}\\b`),
        `terminal/text_grid must not redefine core ${typeName}`,
    );
}
assertMatch(
    terminalTextGrid,
    /pub\s+fn\s+terminal_text_cell_style_default\b/,
    "terminal backend must own terminal default style",
);

const featuresGui = read("stdlib/features/gui.nepl");
assertMatch(featuresGui, /#import\s+"alloc\/gui"\s+as\s+@merge/, "features/gui must expose common alloc GUI substrate");
assertMatch(
    featuresGui,
    /#import\s+"platforms\/gui\/terminal"\s+as\s+@merge/,
    "features/gui must expose the explicit terminal backend profile",
);

console.log("stdlib gui layering policy passed");
