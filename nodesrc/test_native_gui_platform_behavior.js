#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8");
}

function withoutComments(text) {
    return text
        .split("\n")
        .filter((line) => !line.trimStart().startsWith("//"))
        .join("\n");
}

function textSliceBetween(source, startNeedle, endNeedle) {
    const start = source.indexOf(startNeedle);
    if (start < 0) {
        return "";
    }
    const end = source.indexOf(endNeedle, start + startNeedle.length);
    return end < 0 ? source.slice(start) : source.slice(start, end);
}

function runNativeGuiPlatformBehaviorRegression() {
    const mainSource = readRepoFile("nepl-gui-native", "src", "main.rs");
    const libSource = readRepoFile("nepl-gui-native", "src", "lib.rs");
    const platformDoc = readRepoFile("doc", "neplg2", "gui_native_platform_behavior.md");
    const implementationPlan = readRepoFile("doc", "neplg2", "gui_tui_implementation_plan.md");
    const standardSpec = readRepoFile("doc", "neplg2", "gui_standard_library_spec.md");
    const nativeFacade = readRepoFile("stdlib", "platforms", "gui", "native.nepl");
    const nativeClock = readRepoFile("stdlib", "platforms", "gui", "native", "clock.nepl");
    const nativeClockImpl = withoutComments(nativeClock);
    const nativeClockTest = readRepoFile("tests", "stdlib", "gui_platform_native_clock.n.md");
    const nativeClockHelper = textSliceBetween(
        libSource,
        "pub fn native_monotonic_clock_ms_from_elapsed_ms",
        "impl FromStr for GuiDemo",
    );
    const nativeSpanOperationHelper = textSliceBetween(
        libSource,
        "pub const GUI_NATIVE_SPAN_OPERATION_STATUS_OK",
        "impl FromStr for GuiDemo",
    );

    assert.match(mainSource, /WindowOptions\s*\{[\s\S]*resize:\s*true,[\s\S]*scale_mode:\s*ScaleMode::AspectRatioStretch/);
    assert.match(mainSource, /window\.set_target_fps\(60\)/);
    assert.match(mainSource, /window\.set_background_color\(9,\s*13,\s*18\)/);
    assert.match(mainSource, /let mut previous_size = window\.get_size\(\)/);
    assert.match(mainSource, /let current_size = window\.get_size\(\)/);
    assert.match(mainSource, /update_window_title\(&mut window,\s*options\.demo,\s*current_size\)/);
    assert.match(mainSource, /while window\.is_open\(\) && !window\.is_key_down\(Key::Escape\)/);
    assert.match(mainSource, /window\.get_unscaled_mouse_pos\(MouseMode::Discard\)/);
    assert.match(mainSource, /map_native_window_point_to_image\(/);
    assert.match(mainSource, /NativeWindowPresenterState/);
    assert.match(mainSource, /let mut presenter_state = NativeWindowPresenterState::new/);
    assert.match(mainSource, /presenter_state[\s\S]*\.present_buffer\(presenter_frame_id, &initial_buffer\)/);
    assert.match(mainSource, /presenter_state[\s\S]*\.resize_surface\(current_size\.0, current_size\.1\)/);
    assert.match(mainSource, /presenter_frame_id = presenter_frame_id[\s\S]*\.checked_add\(1\)/);
    assert.match(mainSource, /let present_frame = presenter_state[\s\S]*\.last_present_frame_required\(\)/);
    assert.match(mainSource, /update_with_buffer\(\s*present_frame\.pixels\(\),\s*present_frame\.width\(\),\s*present_frame\.height\(\),\s*\)/);
    assert.doesNotMatch(mainSource, /update_with_buffer\(&image\.pixels,\s*image\.width,\s*image\.height\)/);
    assert.doesNotMatch(mainSource, /let mut present_buffer|NativePresenterFrame::from_rgb0_present_buffer\(&present_buffer\)|wrapping_|saturating_|clamp|fallback|silent no-op/);
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
    assert.match(libSource, /pub const GUI_NATIVE_BACKEND_CLOCK_I32_MAX_MS: u128 = 2_147_483_647;/);
    assert.match(libSource, /pub const GUI_NATIVE_BACKEND_CLOCK_STATUS_UNSUPPORTED: i32 = -1;/);
    assert.match(libSource, /pub const GUI_NATIVE_BACKEND_CLOCK_STATUS_BACKEND_FAILURE: i32 = -2;/);
    assert.match(libSource, /pub fn native_monotonic_clock_ms_from_elapsed_ms\(elapsed_ms: u128\) -> i32/);
    assert.match(libSource, /pub fn native_monotonic_clock_ms_since\(start: &Instant\) -> i32/);
    assert.match(libSource, /if elapsed_ms > GUI_NATIVE_BACKEND_CLOCK_I32_MAX_MS/);
    assert.match(libSource, /native_monotonic_clock_elapsed_conversion_checks_i32_range/);
    assert.match(libSource, /native_monotonic_clock_since_uses_instant_source/);
    assert.doesNotMatch(nativeClockHelper, /saturating_|wrapping_|clamp|std::thread::sleep|SystemTime|UNIX_EPOCH|fallback|silent no-op/);
    assert.match(libSource, /pub const GUI_NATIVE_SPAN_OPERATION_STATUS_OK: i32 = 0;/);
    assert.match(libSource, /pub const GUI_NATIVE_SPAN_OPERATION_STATUS_UNSUPPORTED: i32 = -1;/);
    assert.match(libSource, /pub const GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT: i32 = -2;/);
    assert.match(libSource, /pub const GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED: i32 = -3;/);
    assert.match(libSource, /pub const GUI_NATIVE_SPAN_OPERATION_STATUS_NO_WRITABLE_SLOT: i32 = -4;/);
    assert.match(libSource, /pub const GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE: i32 = -5;/);
    assert.match(libSource, /pub const GUI_NATIVE_SPAN_OPERATION_STATUS_STALE_FRAME: i32 = -6;/);
    assert.match(libSource, /pub enum NativeSpanOperationTarget/);
    assert.match(libSource, /pub struct NativeSpanOperationDescriptor/);
    assert.match(libSource, /pub struct NativeSpanOperationRunSpan/);
    assert.match(libSource, /pub enum NativeSpanOperation/);
    assert.match(libSource, /pub trait NativeSpanOperationSink/);
    assert.match(libSource, /pub fn normalize_native_span_operation_status\(status: i32\) -> i32/);
    assert.match(libSource, /pub fn execute_native_span_operation_begin<S: NativeSpanOperationSink>/);
    assert.match(libSource, /pub fn execute_native_span_operation_run<S: NativeSpanOperationSink>/);
    assert.match(libSource, /pub fn execute_native_span_operation_end<S: NativeSpanOperationSink>/);
    assert.match(nativeSpanOperationHelper, /packet_frame_id != frame_id/);
    assert.match(nativeSpanOperationHelper, /stride_bytes != expected_stride/);
    assert.match(nativeSpanOperationHelper, /tile_count != expected_tile_count \|\| tile_index >= tile_count/);
    assert.match(nativeSpanOperationHelper, /sink\.execute_span_operation\(NativeSpanOperation::Begin/);
    assert.match(nativeSpanOperationHelper, /sink\.execute_span_operation\(NativeSpanOperation::RunSpan/);
    assert.match(nativeSpanOperationHelper, /sink\.execute_span_operation\(NativeSpanOperation::End/);
    assert.match(libSource, /pub const NATIVE_RGBA8888_PIXEL_TRANSPARENT: u32 = 0x00000000;/);
    assert.match(libSource, /pub enum NativeSpanFramebufferError/);
    assert.match(libSource, /pub struct NativeSpanFramebufferActiveSequence/);
    assert.match(libSource, /pub struct NativeRgba8888FrameBuffer\s*\{[\s\S]*width: i32,[\s\S]*height: i32,[\s\S]*stride_bytes: i32,[\s\S]*pixels: Vec<u32>,[\s\S]*active_sequence: Option<NativeSpanFramebufferActiveSequence>/);
    assert.match(nativeSpanOperationHelper, /semantic `0xRRGGBBAA` values/);
    assert.match(nativeSpanOperationHelper, /try_reserve_exact\(pixel_count\)/);
    assert.match(nativeSpanOperationHelper, /descriptor\.stride_bytes != self\.stride_bytes/);
    assert.match(nativeSpanOperationHelper, /seen_run_count: 0/);
    assert.match(nativeSpanOperationHelper, /run_span\.x < 0 \|\| run_span\.width <= 0 \|\| run_span\.height != 1/);
    assert.match(nativeSpanOperationHelper, /active\.seen_run_count >= descriptor\.total_run_count/);
    assert.match(nativeSpanOperationHelper, /active\.seen_run_count != descriptor\.total_run_count/);
    assert.match(nativeSpanOperationHelper, /active\.seen_run_count \+ 1/);
    assert.match(nativeSpanOperationHelper, /pub fn native_pack_rgba8888_pixel\(r: u8, g: u8, b: u8, a: u8\) -> u32/);
    assert.match(nativeSpanOperationHelper, /\(u32::from\(r\) << 24\) \| \(u32::from\(g\) << 16\) \| \(u32::from\(b\) << 8\) \| u32::from\(a\)/);
    assert.match(libSource, /pub struct NativeRgbColor/);
    assert.match(libSource, /pub struct NativeRgb0PresentBuffer\s*\{[\s\S]*width: i32,[\s\S]*height: i32,[\s\S]*pixels: Vec<u32>/);
    assert.match(libSource, /pub struct NativeRgb0PresenterSink\s*\{[\s\S]*frame_buffer: NativeRgba8888FrameBuffer,[\s\S]*background: NativeRgbColor,[\s\S]*last_present_buffer: Option<NativeRgb0PresentBuffer>,[\s\S]*last_presented_frame_id: Option<i32>/);
    assert.match(libSource, /pub enum NativeWindowPresenterSurfaceState\s*\{[\s\S]*Drawable\s*\{\s*width: usize,\s*height: usize\s*\},[\s\S]*Unavailable/);
    assert.match(libSource, /pub enum NativeWindowPresenterError\s*\{[\s\S]*InvalidSurfaceDimensions,[\s\S]*FrameMissing,[\s\S]*FrameIdMissing,[\s\S]*InvalidFrameId,[\s\S]*PresenterFrameValidationFailed\(NativePresenterFrameError\),[\s\S]*ResourceExhausted,[\s\S]*DimensionOverflow/);
    assert.match(libSource, /pub struct NativeWindowPresenterState\s*\{[\s\S]*surface_state: NativeWindowPresenterSurfaceState,[\s\S]*last_frame_id: Option<i32>,[\s\S]*last_frame_width: usize,[\s\S]*last_frame_height: usize,[\s\S]*last_pixels: Vec<u32>/);
    assert.match(libSource, /pub const NATIVE_RGB0_HIGH_BYTE_MASK: u32 = 0xff000000;/);
    assert.match(libSource, /pub enum NativePresenterFrameError/);
    assert.match(libSource, /pub struct NativePresenterFrame<'a>\s*\{[\s\S]*width: usize,[\s\S]*height: usize,[\s\S]*pixels: &'a \[u32\]/);
    assert.match(nativeSpanOperationHelper, /Converts a completed semantic RGBA8888 framebuffer into `0x00RRGGBB`/);
    assert.match(nativeSpanOperationHelper, /background: NativeRgbColor/);
    assert.match(nativeSpanOperationHelper, /frame_buffer\.active_sequence\(\)\.is_some\(\)/);
    assert.match(nativeSpanOperationHelper, /from_rgb0_pixels_for_smoke_demo/);
    assert.match(nativeSpanOperationHelper, /not the formal NEPL span presentation path/);
    assert.match(nativeSpanOperationHelper, /pixel & NATIVE_RGB0_HIGH_BYTE_MASK != 0/);
    assert.match(nativeSpanOperationHelper, /from_rgb0_present_buffer\(/);
    assert.match(nativeSpanOperationHelper, /PixelFormatMismatch/);
    assert.match(nativeSpanOperationHelper, /pub fn native_pack_rgb0_pixel\(r: u8, g: u8, b: u8\) -> u32/);
    assert.match(nativeSpanOperationHelper, /\(u32::from\(r\) << 16\) \| \(u32::from\(g\) << 8\) \| u32::from\(b\)/);
    assert.match(nativeSpanOperationHelper, /pub fn native_rgba8888_to_rgb0_over_background/);
    assert.match(nativeSpanOperationHelper, /fn native_rgb0_present_buffer_from_rgba8888_parts/);
    assert.match(nativeSpanOperationHelper, /fn end_sequence_to_rgb0_present_buffer/);
    assert.match(nativeSpanOperationHelper, /let present_buffer = native_rgb0_present_buffer_from_rgba8888_parts/);
    assert.match(nativeSpanOperationHelper, /self\.active_sequence = None;[\s\S]*Ok\(present_buffer\)/);
    assert.match(nativeSpanOperationHelper, /last_present_frame/);
    assert.match(nativeSpanOperationHelper, /last_presented_frame_id/);
    assert.match(nativeSpanOperationHelper, /fn native_presenter_frame_from_rgb0_parts/);
    assert.match(nativeSpanOperationHelper, /pub fn last_present_frame_required/);
    assert.match(nativeSpanOperationHelper, /pub fn present_buffer/);
    assert.match(nativeSpanOperationHelper, /pub fn present_frame/);
    assert.match(nativeSpanOperationHelper, /if frame_id <= 0[\s\S]*NativeWindowPresenterError::InvalidFrameId/);
    assert.match(nativeSpanOperationHelper, /pub fn present_sink_frame/);
    assert.match(nativeSpanOperationHelper, /ok_or\(NativeWindowPresenterError::FrameMissing\)/);
    assert.match(nativeSpanOperationHelper, /ok_or\(NativeWindowPresenterError::FrameIdMissing\)/);
    assert.match(nativeSpanOperationHelper, /self\.present_frame\(frame_id, source_frame\)\?/);
    assert.match(nativeSpanOperationHelper, /try_reserve_exact\(pixel_count\)[\s\S]*self\.last_pixels = next_pixels/);
    assert.match(nativeSpanOperationHelper, /NativeWindowPresenterSurfaceState::Unavailable/);
    assert.match(nativeSpanOperationHelper, /let source_r = \(\(rgba8888 >> 24\) & 0xff\) as u8/);
    assert.match(nativeSpanOperationHelper, /u32::from\(source\) \* alpha \+ u32::from\(background\) \* inverse_alpha \+ 127/);
    assert.match(libSource, /native_span_operation_records_valid_begin_run_end/);
    assert.match(libSource, /native_span_operation_rejects_invalid_descriptor_before_sink/);
    assert.match(libSource, /native_span_operation_requires_exact_tile_count_and_frame_id/);
    assert.match(libSource, /native_span_operation_rejects_invalid_run_span_before_sink/);
    assert.match(libSource, /native_span_operation_normalizes_sink_status/);
    assert.match(libSource, /native_span_framebuffer_constructor_checks_dimensions_and_layout/);
    assert.match(libSource, /native_span_framebuffer_writes_complete_sequence/);
    assert.match(libSource, /native_span_framebuffer_rejects_missing_and_nested_sequence/);
    assert.match(libSource, /native_span_framebuffer_rejects_invalid_run_without_partial_write/);
    assert.match(libSource, /native_framebuffer_run\(-1, 1, 1, 10\)/);
    assert.match(libSource, /height: 2,[\s\S]*native_framebuffer_run\(0, 0, 1, 10\)/);
    assert.match(libSource, /native_span_framebuffer_requires_exact_run_count_before_end/);
    assert.match(libSource, /native_span_framebuffer_rejects_end_descriptor_mismatch_and_keeps_active/);
    assert.match(libSource, /native_present_buffer_packs_rgb0_and_blends_alpha/);
    assert.match(libSource, /native_present_buffer_converts_completed_framebuffer/);
    assert.match(libSource, /native_present_buffer_rejects_active_framebuffer_sequence/);
    assert.match(libSource, /native_presenter_frame_imports_smoke_rgb0_pixels/);
    assert.match(libSource, /native_presenter_frame_rejects_invalid_rgb0_import/);
    assert.match(libSource, /native_presenter_frame_revalidates_buffer_contract/);
    assert.match(libSource, /native_rgb0_presenter_sink_updates_last_frame_on_complete_sequence/);
    assert.match(libSource, /native_rgb0_presenter_sink_keeps_previous_frame_on_invalid_sequence/);
    assert.match(libSource, /native_rgb0_presenter_private_helper_keeps_active_on_conversion_failure/);
    assert.match(libSource, /native_span_framebuffer_end_semantics_still_close_sequence/);
    assert.match(libSource, /native_window_presenter_state_requires_positive_initial_surface/);
    assert.match(libSource, /native_window_presenter_state_presents_sink_frame_after_complete_sequence/);
    assert.match(libSource, /native_window_presenter_state_presents_checked_buffer/);
    assert.match(libSource, /native_window_presenter_state_requires_valid_frame_id/);
    assert.match(libSource, /native_window_presenter_state_rejects_missing_completed_frame/);
    assert.match(libSource, /native_window_presenter_state_rejects_missing_frame_id/);
    assert.match(libSource, /native_window_presenter_state_rejects_invalid_sink_frame_id/);
    assert.match(libSource, /native_window_presenter_state_tracks_resize_without_stretching_last_frame/);
    assert.match(libSource, /native_window_presenter_state_failed_buffer_present_keeps_previous_frame/);
    assert.match(libSource, /native_window_presenter_state_failed_present_keeps_previous_frame/);
    assert.doesNotMatch(nativeSpanOperationHelper, /saturating_|wrapping_|clamp|std::thread::sleep|SystemTime|UNIX_EPOCH|setTimeout|setInterval|queue|stdout_protocol|Canvas|DOM|minifb|video_memory|fallback|silent no-op|from_raw_parts|transmute|to_ne_bytes|to_le_bytes|to_be_bytes|as_bytes|bytemuck/i);
    assert.doesNotMatch(mainSource, /NativeRgba8888FrameBuffer|NativeSpanFramebuffer|native_rgba8888_to_rgb0_over_background/);

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
    assert.match(platformDoc, /Native span operation host executor ABI checkpoint/);
    assert.match(platformDoc, /stride_bytes == width \* 4/);
    assert.match(platformDoc, /tile_count == ceil\(plan_row_count \/ tile_rows\)/);
    assert.match(platformDoc, /invalid scalar input returns -2 before the sink is called/);
    assert.match(platformDoc, /Native RGBA8888 framebuffer sink checkpoint/);
    assert.match(platformDoc, /0xRRGGBBAA/);
    assert.match(platformDoc, /seen_run_count == descriptor\.total_run_count/);
    assert.match(platformDoc, /silent partial frame/);
    assert.match(platformDoc, /Native RGB0 present buffer conversion checkpoint/);
    assert.match(platformDoc, /0x00RRGGBB/);
    assert.match(platformDoc, /explicit background color/);
    assert.match(platformDoc, /active sequence/);
    assert.match(platformDoc, /Native presenter frame adapter checkpoint/);
    assert.match(platformDoc, /high byte/);
    assert.match(platformDoc, /typed presenter frame/);
    assert.match(platformDoc, /Native RGB0 presenter sink checkpoint/);
    assert.match(platformDoc, /last completed/);
    assert.match(platformDoc, /conversion succeeds/);
    assert.match(platformDoc, /Native window presenter state checkpoint/);
    assert.match(platformDoc, /NativeWindowPresenterSurfaceState::Unavailable/);
    assert.match(platformDoc, /does not stretch/);
    assert.match(platformDoc, /previous frame/);
    assert.match(platformDoc, /https:\/\/developer\.apple\.com\/documentation\/appkit\/nsapplication\/run/);
    assert.match(platformDoc, /https:\/\/learn\.microsoft\.com\/en-us\/windows\/win32\/winmsg\/wm-close/);
    assert.match(platformDoc, /https:\/\/www\.x\.org\/releases\/X11R7\.7\/doc\/xorg-docs\/icccm\/icccm\.html/);
    assert.match(platformDoc, /https:\/\/docs\.rs\/minifb\/latest\/minifb\/enum\.ScaleMode\.html/);

    assert.match(implementationPlan, /native platform behavior checkpoint/);
    assert.match(implementationPlan, /macOS AppKit、Windows Win32、Linux Wayland \/ X11/);
    assert.match(standardSpec, /resizable minifb window smoke backend/);
    assert.match(standardSpec, /NativeSurfaceState::Unavailable/);
    assert.match(standardSpec, /F5er Native formal monotonic clock source checkpoint/);

    assert.match(nativeFacade, /pub #import "\.\/native\/clock" as @merge/);
    assert.match(nativeClock, /#extern "nepl_gui_native" "monotonic_clock_ms"/);
    assert.match(nativeClock, /GuiError::Unsupported/);
    assert.match(nativeClock, /GuiError::BackendFailure/);
    assert.match(nativeClock, /gui_rgba8888_row_tile_rle_present_host_span_operation_presenter_executor_session_turn_virtual_scheduler_backend_clock_sample raw/);
    assert.doesNotMatch(nativeClockImpl, /Date|performance|setTimeout|setInterval|sleep|queue|stdout_protocol|Canvas|DOM|minifb|video_memory|fallback|silent no-op|clamp|round/);
    assert.match(nativeClockTest, /native_runner_clock_instant_i32_guard_ok/);

    return {
        ok: true,
        checks: [
            "Native smoke runner uses OS-managed resize and close state",
            "Letterboxed framebuffer hit testing is modeled with explicit surface state",
            "Native monotonic clock source uses Instant with i32 range failure",
            "Native span-operation host ABI validates scalar packet input before injected sink execution",
            "Native RGBA8888 framebuffer sink requires complete span sequences without endian byte views",
            "Native RGB0 present buffer conversion uses explicit background alpha composition",
            "Native presenter frame adapter validates RGB0 pixels before minifb update",
            "Native RGB0 presenter sink converts complete span sequences into typed frames",
            "Native window presenter state keeps resize and frame ownership explicit",
            "Native smoke runner presents and hit-tests through NativeWindowPresenterState",
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
