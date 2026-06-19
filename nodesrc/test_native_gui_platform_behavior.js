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
    const nativeSchedulerHostExecutor = readRepoFile("stdlib", "platforms", "gui", "native", "scheduler_host_executor.nepl");
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
    const nativeWindowEventPumpHelper = textSliceBetween(
        libSource,
        "pub struct NativeWindowSize",
        "impl NativeWindowPresenterState",
    );
    const nativeWindowBackendLoopHelper = textSliceBetween(
        libSource,
        "enum NativeWindowBackendLoopCounterIntent",
        "impl NativeWindowPresenterSession",
    );
    const nativeWindowRunLoopHelper = textSliceBetween(
        libSource,
        "impl NativeWindowRunLoopConfig",
        "pub fn render_demo_frame",
    );
    const nativeWindowHostLoopCore = textSliceBetween(
        libSource,
        "pub fn run_native_window_host_loop",
        "struct MinifbNativeWindowRunLoopHost",
    );
    const nativeWindowMinifbHostAdapter = textSliceBetween(
        libSource,
        "struct MinifbNativeWindowRunLoopHost",
        "pub fn run_minifb_window_loop",
    );
    const nativeWindowMinifbRunner = textSliceBetween(
        libSource,
        "pub fn run_minifb_window_loop",
        "pub fn render_demo_frame",
    );
    const nativeSpanOperationHelperWithoutEventPump = nativeSpanOperationHelper.replace(
        nativeWindowEventPumpHelper,
        "",
    );

    assert.match(mainSource, /NativeWindowRunLoopConfig::new\(options\.demo,\s*options\.counter_value,\s*options\.scale\)/);
    assert.match(mainSource, /run_minifb_window_loop\(config\)/);
    assert.doesNotMatch(mainSource, /WindowOptions|ScaleMode|NativeWindowBackendLoop|NativeWindowHostAction|NativeWindowBackendLoopStepOutcome|poll_minifb_window_event_pump|current_present_frame_for_window|update_with_buffer|window\.update\(|window\.set_target_fps|window\.set_background_color|use\s+minifb|minifb::|let mut previous_size|previous_mouse_down|NativeWindowEventPumpInput\s*\{|NativeWindowPresenterState|counter_hit\(|map_native_window_point_to_image\(|checked_add\(|rasterize_frame_to_surface\(|present_buffer\(|resize_surface\(|let mut present_buffer|NativePresenterFrame::from_rgb0_present_buffer\(&present_buffer\)|wrapping_|saturating_|clamp|fallback|silent no-op/);
    assert.doesNotMatch(mainSource, /get_mouse_pos\(MouseMode::Clamp\)/);
    assert.doesNotMatch(mainSource, /\bKey\b|\bMouseButton\b|\bMouseMode\b|window\.is_open\(\)|window\.is_key_down\(|window\.get_mouse_down\(|window\.get_unscaled_mouse_pos\(/);

    assert.match(libSource, /pub struct NativeWindowBackendLoopPresentation\s*\{[\s\S]*frame_id: i32,[\s\S]*width: usize,[\s\S]*height: usize/);
    assert.match(libSource, /pub enum NativeWindowBackendLoopPointerAction\s*\{[\s\S]*PressedUnavailable,[\s\S]*PressedOutside,[\s\S]*CounterIncremented/);
    assert.match(libSource, /pub enum NativeWindowBackendLoopStepOutcome\s*\{[\s\S]*CloseRequested[\s\S]*Unavailable[\s\S]*Drawable/);
    assert.match(libSource, /pub enum NativeWindowHostTerminalReason\s*\{[\s\S]*OsCloseRequested,[\s\S]*ExitShortcutRequested/);
    assert.match(libSource, /pub enum NativeWindowHostAction\s*\{[\s\S]*Terminate[\s\S]*PumpEventsOnly[\s\S]*PresentFrame/);
    assert.match(libSource, /pub enum NativeWindowHostActionError\s*\{[\s\S]*UnsupportedCloseState[\s\S]*StepFailed\(NativeWindowBackendLoopError\)/);
    assert.match(libSource, /pub struct NativeWindowRunLoopConfig\s*\{[\s\S]*pub demo: GuiDemo,[\s\S]*pub counter_value: i32,[\s\S]*pub scale: usize/);
    assert.match(libSource, /pub struct NativeWindowRunLoopExit\s*\{[\s\S]*pub reason: NativeWindowHostTerminalReason/);
    assert.match(libSource, /pub enum NativeWindowRunLoopError\s*\{[\s\S]*BackendLoopInitializationFailed\(NativeWindowBackendLoopError\)[\s\S]*WindowCreationFailed[\s\S]*EventPumpFailed\(NativeWindowEventPumpError\)[\s\S]*HostActionFailed\(NativeWindowHostActionError\)[\s\S]*PresenterFrameUnavailable\(NativeWindowBackendLoopError\)[\s\S]*WindowPresentFailed/);
    assert.match(libSource, /pub trait NativeWindowRunLoopHost\s*\{[\s\S]*type EventError;[\s\S]*type PresentError;[\s\S]*poll_event_snapshot[\s\S]*set_window_title[\s\S]*pump_events_only[\s\S]*present_frame/);
    assert.match(libSource, /pub enum NativeWindowHostLoopError<EventError, PresentError>\s*\{[\s\S]*HostEventPumpFailed\(EventError\)[\s\S]*HostActionFailed\(NativeWindowHostActionError\)[\s\S]*PresenterFrameUnavailable\(NativeWindowBackendLoopError\)[\s\S]*HostPresentFailed\(PresentError\)/);
    assert.match(libSource, /pub enum NativeWindowBackendLoopError\s*\{[\s\S]*FrameIdOverflow[\s\S]*CounterValueOverflow[\s\S]*RasterizeFailed[\s\S]*FrameWindowMismatch/);
    assert.match(libSource, /pub struct NativeWindowBackendLoop\s*\{[\s\S]*state: NativeWindowBackendLoopState,[\s\S]*presenter_state: NativeWindowPresenterState/);
    assert.match(libSource, /resize_redraw: Option<NativeWindowBackendLoopPresentation>/);
    assert.match(libSource, /CounterIncremented\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation/);
    assert.match(nativeWindowBackendLoopHelper, /pub fn new_for_scale/);
    assert.match(nativeWindowBackendLoopHelper, /pub fn event_pump_input\(&self\) -> NativeWindowEventPumpInput/);
    assert.match(nativeWindowBackendLoopHelper, /pub fn step\([\s\S]*NativeWindowEventPumpSnapshot[\s\S]*NativeWindowBackendLoopStepOutcome/);
    assert.match(nativeWindowBackendLoopHelper, /pub fn step_host_action\([\s\S]*NativeWindowEventPumpSnapshot[\s\S]*NativeWindowHostAction/);
    assert.match(libSource, /fn native_window_host_action_from_backend_loop_outcome/);
    assert.match(libSource, /NativeWindowHostAction::Terminate/);
    assert.match(libSource, /NativeWindowHostAction::PumpEventsOnly/);
    assert.match(libSource, /NativeWindowHostAction::PresentFrame/);
    assert.match(libSource, /NativeWindowHostActionError::UnsupportedCloseState/);
    assert.match(libSource, /NativeWindowHostActionError::StepFailed/);
    assert.match(nativeWindowRunLoopHelper, /pub fn native_window_title\(demo: GuiDemo, size: NativeWindowSize\) -> String/);
    assert.match(nativeWindowHostLoopCore, /pub fn run_native_window_host_loop<Host>\([\s\S]*backend_loop: &mut NativeWindowBackendLoop,[\s\S]*host: &mut Host/);
    assert.match(nativeWindowHostLoopCore, /Host: NativeWindowRunLoopHost/);
    assert.match(nativeWindowHostLoopCore, /host\.set_window_title\(&initial_title\)/);
    assert.match(nativeWindowHostLoopCore, /host[\s\S]*\.poll_event_snapshot\(backend_loop\.event_pump_input\(\)\)/);
    assert.match(nativeWindowHostLoopCore, /backend_loop[\s\S]*\.step_host_action\(event_snapshot\)/);
    assert.match(nativeWindowHostLoopCore, /NativeWindowHostAction::Terminate/);
    assert.match(nativeWindowHostLoopCore, /NativeWindowHostAction::PumpEventsOnly[\s\S]*host\.pump_events_only\(\)/);
    assert.match(nativeWindowHostLoopCore, /NativeWindowHostAction::PresentFrame[\s\S]*current_present_frame_for_window\(\)[\s\S]*host\.present_frame\(present_frame\)/);
    assert.doesNotMatch(nativeWindowHostLoopCore, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|set_background_color|\bKey\b|\bMouseButton\b|\bMouseMode\b|is_open\(|is_key_down\(|get_mouse_down\(|get_unscaled_mouse_pos\(|queue|timer|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowMinifbHostAdapter, /impl NativeWindowRunLoopHost for MinifbNativeWindowRunLoopHost/);
    assert.match(nativeWindowMinifbHostAdapter, /poll_minifb_window_event_pump\(self\.window,\s*input\)/);
    assert.match(nativeWindowMinifbHostAdapter, /self\.window\.set_title\(title\)/);
    assert.match(nativeWindowMinifbHostAdapter, /self\.window\.update\(\)/);
    assert.match(nativeWindowMinifbHostAdapter, /self\.window[\s\S]*\.update_with_buffer\(frame\.pixels\(\),\s*frame\.width\(\),\s*frame\.height\(\)\)/);
    assert.doesNotMatch(nativeWindowMinifbHostAdapter, /\bKey\b|\bMouseButton\b|\bMouseMode\b|window\.is_open\(\)|window\.is_key_down\(|window\.get_mouse_down\(|window\.get_unscaled_mouse_pos\(|queue|timer|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowRunLoopHelper, /pub fn run_minifb_window_loop\([\s\S]*NativeWindowRunLoopConfig[\s\S]*NativeWindowRunLoopExit/);
    assert.match(nativeWindowRunLoopHelper, /WindowOptions\s*\{[\s\S]*resize:\s*true,[\s\S]*scale_mode:\s*ScaleMode::UpperLeft/);
    assert.match(nativeWindowRunLoopHelper, /window\.set_target_fps\(60\)/);
    assert.match(nativeWindowRunLoopHelper, /window\.set_background_color\(9,\s*13,\s*18\)/);
    assert.match(nativeWindowRunLoopHelper, /let mut host = MinifbNativeWindowRunLoopHost/);
    assert.match(nativeWindowRunLoopHelper, /run_native_window_host_loop\(&mut backend_loop,\s*&mut host\)/);
    assert.match(nativeWindowRunLoopHelper, /NativeWindowRunLoopError::WindowPresentFailed/);
    assert.doesNotMatch(nativeWindowMinifbRunner, /poll_minifb_window_event_pump|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|update_with_buffer\(/);
    assert.match(nativeWindowBackendLoopHelper, /CloseRequested[\s\S]*return Ok\(NativeWindowBackendLoopStepOutcome::CloseRequested/);
    assert.match(nativeWindowBackendLoopHelper, /NativeWindowBackendLoopStepOutcome::Unavailable/);
    assert.match(nativeWindowBackendLoopHelper, /present_frame_to_surface_after_success/);
    assert.match(nativeWindowBackendLoopHelper, /native_window_backend_loop_next_frame_id/);
    assert.match(nativeWindowBackendLoopHelper, /CounterFrameIdMissing/);
    assert.match(libSource, /native_window_backend_loop_host_action_preserves_terminal_reason/);
    assert.match(libSource, /native_window_backend_loop_host_action_rejects_impossible_open_close/);
    assert.match(libSource, /native_window_backend_loop_host_action_unavailable_pumps_events_only/);
    assert.match(libSource, /native_window_backend_loop_host_action_drawable_presents_final_frame_evidence/);
    assert.match(libSource, /native_window_run_loop_config_preserves_demo_state/);
    assert.match(libSource, /native_window_title_reports_drawable_and_unavailable_surface/);
    assert.match(libSource, /native_window_host_loop_preserves_terminal_reason/);
    assert.match(libSource, /native_window_host_loop_pumps_unavailable_surface_without_presenting/);
    assert.match(libSource, /native_window_host_loop_presents_exact_current_frame/);
    assert.match(libSource, /native_window_host_loop_preserves_event_pump_error/);
    assert.match(libSource, /native_window_host_loop_preserves_present_error/);
    assert.match(libSource, /native_window_host_loop_preserves_host_action_error/);
    assert.match(libSource, /native_window_host_loop_preserves_presenter_frame_error/);
    assert.doesNotMatch(nativeWindowBackendLoopHelper, /minifb|DOM|Canvas|video_memory|stdout_protocol|window\.update\(|update_with_buffer|fallback|silent no-op/i);

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
    assert.match(libSource, /pub enum RasterizeSurfaceError/);
    assert.match(libSource, /pub fn rasterize_frame_to_surface/);
    assert.match(libSource, /try_reserve_exact\(pixel_count\)/);
    assert.match(libSource, /fn fill_surface_rect/);
    assert.match(libSource, /fn ceil_div_u128/);
    assert.match(libSource, /rasterize_frame_to_surface_matches_drawable_size/);
    assert.match(libSource, /rasterize_frame_to_surface_keeps_counter_hit_mapping/);
    assert.match(libSource, /rasterize_frame_to_surface_rejects_invalid_surface/);
    assert.match(libSource, /rasterize_frame_to_surface_rejects_out_of_bounds_command/);
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
    assert.match(libSource, /pub enum NativeSpanOperationStatus\s*\{[\s\S]*Ok,[\s\S]*Unsupported,[\s\S]*InvalidArgument,[\s\S]*ResourceExhausted,[\s\S]*NoWritableSlot,[\s\S]*BackendFailure,[\s\S]*StaleFrame/);
    assert.match(libSource, /pub enum NativeSpanOperationTarget/);
    assert.match(libSource, /pub struct NativeSpanOperationDescriptor/);
    assert.match(libSource, /pub struct NativeSpanOperationRunSpan/);
    assert.match(libSource, /pub enum NativeSpanOperation/);
    assert.match(libSource, /pub trait NativeSpanOperationSink/);
    assert.match(libSource, /pub fn normalize_native_span_operation_status\(status: i32\) -> i32/);
    assert.match(libSource, /pub fn execute_native_span_operation_begin<S: NativeSpanOperationSink>/);
    assert.match(libSource, /pub fn execute_native_span_operation_run<S: NativeSpanOperationSink>/);
    assert.match(libSource, /pub fn execute_native_span_operation_end<S: NativeSpanOperationSink>/);
    assert.match(libSource, /pub fn execute_native_window_presenter_session_operation\(/);
    assert.match(libSource, /pub fn execute_native_window_presenter_session_begin\(/);
    assert.match(libSource, /pub fn execute_native_window_presenter_session_run\(/);
    assert.match(libSource, /pub fn execute_native_window_presenter_session_end\(/);
    assert.match(nativeSpanOperationHelper, /packet_frame_id != frame_id/);
    assert.match(nativeSpanOperationHelper, /stride_bytes != expected_stride/);
    assert.match(nativeSpanOperationHelper, /tile_count != expected_tile_count \|\| tile_index >= tile_count/);
    assert.match(nativeSpanOperationHelper, /sink\.execute_span_operation\(NativeSpanOperation::Begin/);
    assert.match(nativeSpanOperationHelper, /sink\.execute_span_operation\(NativeSpanOperation::RunSpan/);
    assert.match(nativeSpanOperationHelper, /sink\.execute_span_operation\(NativeSpanOperation::End/);
    assert.match(nativeSpanOperationHelper, /NativeSpanOperationStatus::from_raw\(status\)/);
    assert.match(nativeSpanOperationHelper, /NativeWindowPresenterSessionHostError::ValidationFailed/);
    assert.match(nativeSpanOperationHelper, /NativeWindowPresenterSessionHostError::SessionFailed/);
    assert.match(nativeSpanOperationHelper, /validate_native_span_operation_descriptor\([\s\S]*\.map_err\(NativeWindowPresenterSessionHostError::from_validation_status\)/);
    assert.match(nativeSpanOperationHelper, /validate_native_span_operation_run_span\([\s\S]*\.map_err\(NativeWindowPresenterSessionHostError::from_validation_status\)/);
    assert.match(nativeSpanOperationHelper, /execute_native_window_presenter_session_operation\([\s\S]*NativeSpanOperation::Begin/);
    assert.match(nativeSpanOperationHelper, /execute_native_window_presenter_session_operation\([\s\S]*NativeSpanOperation::RunSpan/);
    assert.match(nativeSpanOperationHelper, /execute_native_window_presenter_session_operation\(session, NativeSpanOperation::End/);
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
    assert.match(libSource, /pub enum NativeRgb0PresenterSinkOutcome\s*\{[\s\S]*Accepted,[\s\S]*Completed\s*\{\s*frame_id: i32\s*\}/);
    assert.match(libSource, /pub struct NativeWindowPresenterSession\s*\{[\s\S]*sink: NativeRgb0PresenterSink,[\s\S]*presenter_state: NativeWindowPresenterState/);
    assert.match(libSource, /pub enum NativeWindowPresenterSessionOutcome\s*\{[\s\S]*NotPresented,[\s\S]*Presented\s*\{[\s\S]*frame_id: i32,[\s\S]*width: usize,[\s\S]*height: usize/);
    assert.match(libSource, /pub enum NativeWindowPresenterSessionError\s*\{[\s\S]*SinkFailed\(NativeSpanFramebufferError\),[\s\S]*PresenterFailed\(NativeWindowPresenterError\)/);
    assert.match(libSource, /pub enum NativeWindowPresenterSessionHostError\s*\{[\s\S]*ValidationFailed\(NativeSpanOperationStatus\),[\s\S]*SessionFailed\(NativeWindowPresenterSessionError\)/);
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
    assert.match(nativeSpanOperationHelper, /pub fn execute_span_operation_typed/);
    assert.match(nativeSpanOperationHelper, /NativeRgb0PresenterSinkOutcome::Accepted/);
    assert.match(nativeSpanOperationHelper, /NativeRgb0PresenterSinkOutcome::Completed\s*\{\s*frame_id\s*\}/);
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
    assert.match(nativeSpanOperationHelper, /impl NativeWindowPresenterSession/);
    assert.match(nativeSpanOperationHelper, /NativeRgb0PresenterSink::new\(framebuffer_width, framebuffer_height, background\)/);
    assert.match(nativeSpanOperationHelper, /NativeWindowPresenterState::new\(surface_width, surface_height\)/);
    assert.match(nativeSpanOperationHelper, /pub fn resize_surface\([\s\S]*NativeWindowPresenterSessionError/);
    assert.match(nativeSpanOperationHelper, /pub fn execute_span_operation\([\s\S]*NativeWindowPresenterSessionOutcome/);
    assert.match(nativeSpanOperationHelper, /\.execute_span_operation_typed\(operation\)[\s\S]*NativeWindowPresenterSessionError::SinkFailed/);
    assert.match(nativeSpanOperationHelper, /NativeRgb0PresenterSinkOutcome::Accepted[\s\S]*NativeWindowPresenterSessionOutcome::NotPresented/);
    assert.match(nativeSpanOperationHelper, /NativeRgb0PresenterSinkOutcome::Completed\s*\{\s*frame_id\s*\}[\s\S]*\.present_sink_frame\(&self\.sink\)[\s\S]*NativeWindowPresenterSessionOutcome::Presented/);
    assert.match(nativeSpanOperationHelper, /NativeWindowPresenterError::FrameMissing[\s\S]*NativeWindowPresenterError::FrameIdMissing[\s\S]*GUI_NATIVE_SPAN_OPERATION_STATUS_STALE_FRAME/);
    assert.match(nativeSpanOperationHelper, /NativeWindowPresenterSessionHostError::ValidationFailed\(status\) => status\.as_raw\(\)/);
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
    assert.match(libSource, /native_window_presenter_session_presents_only_after_end/);
    assert.match(libSource, /native_window_presenter_session_failed_sink_operation_keeps_previous_frame/);
    assert.match(libSource, /native_window_presenter_session_failed_present_keeps_previous_frame/);
    assert.match(libSource, /native_window_presenter_session_resize_keeps_frame_pixels_unscaled/);
    assert.match(libSource, /native_window_presenter_session_scalar_helper_presents_only_after_end/);
    assert.match(libSource, /native_window_presenter_session_scalar_validation_keeps_session_state/);
    assert.match(libSource, /native_window_presenter_session_scalar_sink_failure_keeps_previous_frame/);
    assert.match(libSource, /native_window_presenter_session_host_error_separates_presenter_failure/);
    assert.match(libSource, /pub struct NativeWindowSize/);
    assert.match(libSource, /pub enum NativeWindowEventPumpCloseState\s*\{[\s\S]*Open,[\s\S]*OsCloseRequested,[\s\S]*ExitShortcutRequested/);
    assert.match(libSource, /pub enum NativeWindowPointerButtonTransition\s*\{[\s\S]*Unchanged,[\s\S]*Pressed,[\s\S]*Released/);
    assert.match(libSource, /pub enum NativeWindowPointerSample\s*\{[\s\S]*Unavailable,[\s\S]*Available\s*\{\s*x: f32,\s*y: f32\s*\}/);
    assert.match(libSource, /pub enum NativeWindowEventPumpError\s*\{[\s\S]*InvalidPointerSample/);
    assert.match(libSource, /pub struct NativeWindowEventPumpInput/);
    assert.match(libSource, /pub struct NativeWindowEventPumpSnapshot/);
    assert.match(nativeWindowEventPumpHelper, /pub fn build_native_window_event_pump_snapshot_from_raw/);
    assert.match(nativeWindowEventPumpHelper, /pub fn poll_minifb_window_event_pump/);
    assert.match(nativeWindowEventPumpHelper, /!window\.is_open\(\)/);
    assert.match(nativeWindowEventPumpHelper, /window\.is_key_down\(minifb::Key::Escape\)/);
    assert.match(nativeWindowEventPumpHelper, /window\.get_mouse_down\(minifb::MouseButton::Left\)/);
    assert.match(nativeWindowEventPumpHelper, /window\.get_unscaled_mouse_pos\(minifb::MouseMode::Discard\)/);
    assert.match(nativeWindowEventPumpHelper, /NativeWindowEventPumpCloseState::OsCloseRequested/);
    assert.match(nativeWindowEventPumpHelper, /NativeWindowEventPumpCloseState::ExitShortcutRequested/);
    assert.match(nativeWindowEventPumpHelper, /NativeWindowEventPumpError::InvalidPointerSample/);
    assert.doesNotMatch(nativeWindowEventPumpHelper, /window\.update\(|update_with_buffer|queue|stdout_protocol|Canvas|DOM|video_memory|fallback|silent no-op/i);
    assert.match(libSource, /native_window_event_pump_tracks_positive_and_zero_resize/);
    assert.match(libSource, /native_window_event_pump_tracks_pointer_button_transitions/);
    assert.match(libSource, /native_window_event_pump_rejects_non_finite_pointer_sample/);
    assert.match(libSource, /native_window_event_pump_separates_os_close_and_exit_shortcut/);
    assert.doesNotMatch(nativeSpanOperationHelperWithoutEventPump, /saturating_|wrapping_|clamp|std::thread::sleep|SystemTime|UNIX_EPOCH|setTimeout|setInterval|queue|stdout_protocol|Canvas|DOM|minifb|video_memory|fallback|silent no-op|from_raw_parts|transmute|to_ne_bytes|to_le_bytes|to_be_bytes|as_bytes|bytemuck/i);
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
    assert.match(platformDoc, /ScaleMode::UpperLeft/);
    assert.match(platformDoc, /rasterize_frame_to_surface/);
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
    assert.match(platformDoc, /Native window resize redraw checkpoint/);
    assert.match(platformDoc, /same width and height as the current drawable surface/);
    assert.match(platformDoc, /Native presenter operation identity input checkpoint/);
    assert.match(platformDoc, /F5ev is the scheduler step input boundary/);
    assert.match(platformDoc, /gui_native_scheduler_executor_input/);
    assert.match(platformDoc, /WindowBegin/);
    assert.match(platformDoc, /DeviceEnd/);
    assert.match(platformDoc, /Native formal presenter session checkpoint/);
    assert.match(platformDoc, /NativeWindowPresenterSession/);
    assert.match(platformDoc, /NativeWindowPresenterSessionOutcome::NotPresented/);
    assert.match(platformDoc, /NativeWindowPresenterSessionOutcome::Presented/);
    assert.match(platformDoc, /NativeWindowPresenterSessionError::SinkFailed/);
    assert.match(platformDoc, /NativeWindowPresenterSessionError::PresenterFailed/);
    assert.match(platformDoc, /Native presenter session host helper checkpoint/);
    assert.match(platformDoc, /NativeWindowPresenterSessionHostError::ValidationFailed/);
    assert.match(platformDoc, /execute_native_window_presenter_session_begin/);
    assert.match(platformDoc, /execute_native_window_presenter_session_end/);
    assert.match(platformDoc, /Native presenter session host import checkpoint/);
    assert.match(platformDoc, /window_presenter_session_begin/);
    assert.match(platformDoc, /window_presenter_session_end/);
    assert.match(platformDoc, /Native window event pump boundary checkpoint/);
    assert.match(platformDoc, /NativeWindowEventPumpSnapshot/);
    assert.match(platformDoc, /OsCloseRequested/);
    assert.match(platformDoc, /ExitShortcutRequested/);
    assert.match(platformDoc, /NativeWindowPointerSample::Unavailable/);
    assert.match(platformDoc, /NativeWindowEventPumpError::InvalidPointerSample/);
    assert.match(platformDoc, /poll_minifb_window_event_pump/);
    assert.match(platformDoc, /window\.update` \/ `update_with_buffer/);
    assert.match(platformDoc, /Native backend loop step checkpoint/);
    assert.match(platformDoc, /NativeWindowBackendLoop/);
    assert.match(platformDoc, /new_for_scale/);
    assert.match(platformDoc, /positive resize は resize 先の RGB0 buffer を作り/);
    assert.match(platformDoc, /current_present_frame_for_window/);
    assert.match(platformDoc, /Native host action boundary checkpoint/);
    assert.match(platformDoc, /NativeWindowHostAction/);
    assert.match(platformDoc, /PumpEventsOnly/);
    assert.match(platformDoc, /PresentFrame/);
    assert.match(platformDoc, /NativeWindowHostTerminalReason/);
    assert.match(platformDoc, /Native minifb window run-loop adapter checkpoint/);
    assert.match(platformDoc, /run_minifb_window_loop/);
    assert.match(platformDoc, /NativeWindowRunLoopConfig/);
    assert.match(platformDoc, /NativeWindowRunLoopError/);
    assert.match(platformDoc, /WindowPresentFailed/);
    assert.match(platformDoc, /direct minifb input API を読まない/);
    assert.match(platformDoc, /set_target_fps\(60\).*busy spin 抑制/);
    assert.match(platformDoc, /Native window host-loop core checkpoint/);
    assert.match(platformDoc, /NativeWindowRunLoopHost/);
    assert.match(platformDoc, /run_native_window_host_loop/);
    assert.match(platformDoc, /NativeWindowHostLoopError/);
    assert.match(platformDoc, /backend state を失わない/);
    assert.match(platformDoc, /https:\/\/developer\.apple\.com\/documentation\/appkit\/nsapplication\/run/);
    assert.match(platformDoc, /https:\/\/learn\.microsoft\.com\/en-us\/windows\/win32\/winmsg\/wm-close/);
    assert.match(platformDoc, /https:\/\/www\.x\.org\/releases\/X11R7\.7\/doc\/xorg-docs\/icccm\/icccm\.html/);
    assert.match(platformDoc, /https:\/\/docs\.rs\/minifb\/latest\/minifb\/enum\.ScaleMode\.html/);

    assert.match(implementationPlan, /native platform behavior checkpoint/);
    assert.match(implementationPlan, /macOS AppKit、Windows Win32、Linux Wayland \/ X11/);
    assert.match(implementationPlan, /native presenter operation identity input boundary/);
    assert.match(implementationPlan, /native formal presenter session boundary/);
    assert.match(implementationPlan, /native presenter session host helper boundary/);
    assert.match(implementationPlan, /native presenter session host import boundary/);
    assert.match(implementationPlan, /Phase F5gd: Native window event pump boundary/);
    assert.match(implementationPlan, /minifb input API を `poll_minifb_window_event_pump` に閉じ/);
    assert.match(implementationPlan, /Phase F5ge: Native backend loop step boundary/);
    assert.match(implementationPlan, /NativeWindowBackendLoop::new_for_scale/);
    assert.match(implementationPlan, /Phase F5gf: Native host action boundary/);
    assert.match(implementationPlan, /step_host_action/);
    assert.match(implementationPlan, /fallback や silent no-op を作らない/);
    assert.match(implementationPlan, /Phase F5gg: Native minifb window run-loop adapter boundary/);
    assert.match(implementationPlan, /run_minifb_window_loop/);
    assert.match(implementationPlan, /NativeWindowRunLoopConfig/);
    assert.match(implementationPlan, /WindowPresentFailed/);
    assert.match(implementationPlan, /source policy は `run_minifb_window_loop` slice/);
    assert.match(implementationPlan, /Phase F5gh: Native window host-loop core boundary/);
    assert.match(implementationPlan, /NativeWindowRunLoopHost/);
    assert.match(implementationPlan, /run_native_window_host_loop/);
    assert.match(implementationPlan, /&mut NativeWindowBackendLoop/);
    assert.match(implementationPlan, /core loop slice と minifb host adapter slice/);
    assert.match(standardSpec, /resizable minifb window smoke backend/);
    assert.match(standardSpec, /NativeSurfaceState::Unavailable/);
    assert.match(standardSpec, /F5gd Native window event pump boundary/);
    assert.match(standardSpec, /NativeWindowEventPumpInput/);
    assert.match(standardSpec, /NativeWindowEventPumpSnapshot/);
    assert.match(standardSpec, /OsCloseRequested/);
    assert.match(standardSpec, /ExitShortcutRequested/);
    assert.match(standardSpec, /F5ge Native backend loop step boundary/);
    assert.match(standardSpec, /NativeWindowBackendLoop/);
    assert.match(standardSpec, /current_present_frame_for_window/);
    assert.match(standardSpec, /F5gf Native host action boundary/);
    assert.match(standardSpec, /NativeWindowHostAction/);
    assert.match(standardSpec, /F5gg Native minifb window run-loop adapter boundary/);
    assert.match(standardSpec, /NativeWindowRunLoopConfig/);
    assert.match(standardSpec, /WindowPresentFailed/);
    assert.match(standardSpec, /F5gh Native window host-loop core boundary/);
    assert.match(standardSpec, /NativeWindowRunLoopHost/);
    assert.match(standardSpec, /run_native_window_host_loop/);
    assert.match(standardSpec, /NativeWindowHostLoopError/);
    assert.match(standardSpec, /F5ff Native window resize redraw checkpoint/);
    assert.match(standardSpec, /F5fg Native presenter operation identity input boundary/);
    assert.match(standardSpec, /F5fh Native formal presenter session boundary/);
    assert.match(standardSpec, /F5fi Native presenter session host helper boundary/);
    assert.match(standardSpec, /F5fj Native presenter session host import boundary/);
    assert.match(standardSpec, /F5er Native formal monotonic clock source checkpoint/);

    assert.match(nativeFacade, /pub #import "\.\/native\/clock" as @merge/);
    assert.match(nativeSchedulerHostExecutor, /#extern "nepl_gui_native" "window_presenter_session_begin"/);
    assert.match(nativeSchedulerHostExecutor, /#extern "nepl_gui_native" "window_presenter_session_run"/);
    assert.match(nativeSchedulerHostExecutor, /#extern "nepl_gui_native" "window_presenter_session_end"/);
    assert.match(nativeSchedulerHostExecutor, /gui_native_window_presenter_session_begin_raw/);
    assert.match(nativeSchedulerHostExecutor, /gui_native_window_presenter_session_run_raw/);
    assert.match(nativeSchedulerHostExecutor, /gui_native_window_presenter_session_end_raw/);
    assert.doesNotMatch(nativeSchedulerHostExecutor, /#extern "nepl_gui_native" "execute_span_operation_(?:begin|run|end)"/);
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
            "Native smoke runner redraws exact-size buffers after resize",
            "Native host action boundary separates backend loop outcomes from host execution",
            "Native minifb run-loop adapter keeps main.rs out of window lifecycle execution",
            "Native host-loop core separates backend state from window host execution",
            "Native presenter input preserves typed operation identity before scheduler ready payload",
            "Native formal presenter session commits successful End operations to presenter state",
            "Native presenter session host helper validates scalar ABI before session execution",
            "Native presenter session host import exposes formal NEPL ABI names",
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
