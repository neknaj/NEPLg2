#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function runNativeGuiPlatformBehaviorRegression() {
    const mainSource = readRepoFile("nepl-gui-native", "src", "main.rs");
    const libSource = readRepoFile("nepl-gui-native", "src", "lib.rs");
    const platformDoc = readRepoFile("doc", "neplg2", "gui_native_platform_behavior.md");
    const implementationPlan = readRepoFile("doc", "neplg2", "gui_tui_implementation_plan.md");
    const standardSpec = readRepoFile("doc", "neplg2", "gui_standard_library_spec.md");

    assert.match(mainSource, /WindowOptions\s*\{[\s\S]*resize:\s*true,[\s\S]*scale_mode:\s*ScaleMode::AspectRatioStretch/);
    assert.match(mainSource, /window\.set_target_fps\(60\)/);
    assert.match(mainSource, /window\.set_background_color\(9,\s*13,\s*18\)/);
    assert.match(mainSource, /let mut previous_size = window\.get_size\(\)/);
    assert.match(mainSource, /let current_size = window\.get_size\(\)/);
    assert.match(mainSource, /update_window_title\(&mut window,\s*options\.demo,\s*current_size\)/);
    assert.match(mainSource, /while window\.is_open\(\) && !window\.is_key_down\(Key::Escape\)/);
    assert.match(mainSource, /window\.get_unscaled_mouse_pos\(MouseMode::Discard\)/);
    assert.match(mainSource, /map_native_window_point_to_image\(/);
    assert.match(mainSource, /update_with_buffer\(&image\.pixels,\s*image\.width,\s*image\.height\)/);
    assert.doesNotMatch(mainSource, /get_mouse_pos\(MouseMode::Clamp\)/);

    assert.match(libSource, /pub struct NativeSurfacePlacement/);
    assert.match(libSource, /pub enum NativeSurfaceState\s*\{[\s\S]*Drawable\(NativeSurfacePlacement\),[\s\S]*Unavailable/);
    assert.match(libSource, /pub fn native_aspect_ratio_placement\(/);
    assert.match(libSource, /pub fn map_native_window_point_to_image\(/);
    assert.match(libSource, /point_x\.is_finite\(\)/);
    assert.match(libSource, /NativeSurfaceState::Unavailable/);
    assert.match(libSource, /native_surface_placement_preserves_aspect_ratio_inside_window/);
    assert.match(libSource, /native_window_point_mapping_rejects_letterbox_and_maps_to_image/);
    assert.match(libSource, /native_window_point_mapping_handles_shrunken_window/);
    assert.match(libSource, /native_window_point_mapping_rejects_top_bottom_letterbox/);
    assert.match(libSource, /native_window_point_mapping_rejects_unavailable_and_invalid_points/);

    assert.match(platformDoc, /macOS AppKit/);
    assert.match(platformDoc, /Windows Win32/);
    assert.match(platformDoc, /Linux Wayland/);
    assert.match(platformDoc, /Linux X11/);
    assert.match(platformDoc, /NSApplication\.run/);
    assert.match(platformDoc, /NSWindowDelegate\.windowShouldClose/);
    assert.match(platformDoc, /WM_CLOSE/);
    assert.match(platformDoc, /WM_SIZE/);
    assert.match(platformDoc, /xdg_toplevel\.configure/);
    assert.match(platformDoc, /xdg_toplevel\.close/);
    assert.match(platformDoc, /WM_DELETE_WINDOW/);
    assert.match(platformDoc, /ConfigureNotify/);
    assert.match(platformDoc, /ScaleMode::AspectRatioStretch/);
    assert.match(platformDoc, /NativeSurfaceState::Unavailable/);
    assert.match(platformDoc, /https:\/\/developer\.apple\.com\/documentation\/appkit\/nsapplication\/run/);
    assert.match(platformDoc, /https:\/\/learn\.microsoft\.com\/en-us\/windows\/win32\/winmsg\/wm-close/);
    assert.match(platformDoc, /https:\/\/www\.x\.org\/releases\/X11R7\.7\/doc\/xorg-docs\/icccm\/icccm\.html/);
    assert.match(platformDoc, /https:\/\/docs\.rs\/minifb\/latest\/minifb\/enum\.ScaleMode\.html/);

    assert.match(implementationPlan, /native platform behavior checkpoint/);
    assert.match(implementationPlan, /macOS AppKit、Windows Win32、Linux Wayland \/ X11/);
    assert.match(standardSpec, /resizable minifb window smoke backend/);
    assert.match(standardSpec, /NativeSurfaceState::Unavailable/);

    return {
        ok: true,
        checks: [
            "Native smoke runner uses OS-managed resize and close state",
            "Letterboxed framebuffer hit testing is modeled with explicit surface state",
            "Native platform behavior notes cite macOS, Windows, Linux, and minifb contracts",
        ],
    };
}

if (require.main === module) {
    try {
        const result = runNativeGuiPlatformBehaviorRegression();
        process.stdout.write(JSON.stringify(result, null, 2) + "\n");
    } catch (error) {
        console.error(error && error.stack ? error.stack : String(error));
        process.exit(1);
    }
}

module.exports = {
    runNativeGuiPlatformBehaviorRegression,
};
