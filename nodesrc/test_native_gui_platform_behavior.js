#!/usr/bin/env node
"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

function readRepoFile(...parts) {
    return fs.readFileSync(path.resolve(__dirname, "..", ...parts), "utf8").replace(/\r\n/g, "\n");
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
    const nativeCargoToml = readRepoFile("nepl-gui-native", "Cargo.toml");
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
    const nativeWindowHostLoopInitializer = textSliceBetween(
        libSource,
        "pub fn initialize_native_window_host_loop",
        "pub fn native_window_host_loop_wait_decision",
    );
    const nativeWindowHostLoopWaitDecisionHelper = textSliceBetween(
        libSource,
        "pub fn native_window_host_loop_wait_decision",
        "pub fn run_native_window_host_loop_bounded",
    );
    const nativeWindowHostLoopWaitOutcome = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopWaitOutcome",
        "pub enum NativeWindowHostLoopThreadWaitError",
    );
    const nativeWindowThreadWaitBackend = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopThreadWaitError",
        "pub struct NativeWindowHostLoopTimerRegistrationId",
    );
    const nativeWindowTimerRegistrationBackend = textSliceBetween(
        libSource,
        "pub struct NativeWindowHostLoopTimerRegistrationId",
        "pub enum NativeWindowHostLoopTimerFireError",
    );
    const nativeWindowTimerRegistrationError = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopTimerRegistrationError",
        "pub enum NativeWindowHostLoopTimerRegistrationOutcome",
    );
    const nativeWindowTimerFireBackend = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopTimerFireError",
        "pub enum NativeWindowHostLoopTimerWakeError",
    );
    const nativeWindowTimerFireError = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopTimerFireError",
        "pub enum NativeWindowHostLoopTimerFireOutcome",
    );
    const nativeWindowTimerWakeupBackend = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopTimerWakeError",
        "pub struct NativeWindowHostLoopDeadlineTimerRecord",
    );
    const nativeWindowDeadlineTimerAdapter = textSliceBetween(
        libSource,
        "pub struct NativeWindowHostLoopDeadlineTimerRecord",
        "pub enum NativeWindowHostLoopInterruptibleDeadlineWake",
    );
    const nativeWindowInterruptibleDeadlineWaitAdapter = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopInterruptibleDeadlineWake",
        "pub enum NativeWindowHostLoopEventQueueWaitError",
    );
    const nativeWindowEventQueueWaitBackend = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopEventQueueWaitError",
        "pub enum NativeWindowHostLoopWaitOwnerError",
    );
    const nativeWindowHostLoopWaitOwner = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopWaitOwnerError",
        "pub struct NativeWindowHostOwnedDeadlineWaitRunLoopHost",
    );
    const nativeWindowHostOwnedDeadlineWaitRunLoopHost = textSliceBetween(
        libSource,
        "pub struct NativeWindowHostOwnedDeadlineWaitRunLoopHost",
        "pub struct NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost",
    );
    const nativeWindowInterruptibleDeadlineWaitRunLoopHost = textSliceBetween(
        libSource,
        "pub struct NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost",
        "pub enum NativeWindowHostLoopPlatformKind",
    );
    const nativeWindowHostEventSignalWaitGuardRunLoopHost = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostEventSignalWaitError",
        "pub enum NativeWindowHostLoopError",
    );
    const nativeWindowLinuxEventSourceCapabilityGate = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopLinuxEventSourceCapability",
        "pub enum NativeWindowHostLoopPlatformWaitBackendKind",
    );
    const nativeWindowLinuxWindowEventSourcePrepareConfig = textSliceBetween(
        libSource,
        "pub fn native_window_linux_window_event_source_prepare_platform_wait_backend_config",
        "impl NativeWindowRunLoopWaitBackend",
    );
    const nativeWindowLinuxWindowEventSourcePreparedConfigImpl = textSliceBetween(
        libSource,
        "impl<Provider> NativeWindowLinuxWindowEventSourcePreparedPlatformWaitConfig<Provider>",
        "pub fn native_window_linux_window_event_source_prepare_platform_wait_backend_config",
    );
    const nativeWindowLinuxWindowEventSourceRunLoopTypes = textSliceBetween(
        libSource,
        "pub struct NativeWindowLinuxWindowEventSourcePreparedRunLoopConfig",
        "pub enum NativeWindowRunLoopPlatformWaitBackendFromConfigError",
    );
    const nativeWindowLinuxWindowEventSourcePreparedRunLoopConfigImpl = textSliceBetween(
        libSource,
        "impl<Provider> NativeWindowLinuxWindowEventSourcePreparedRunLoopConfig<Provider>",
        "pub fn validate_minifb_window_run_loop_wait_backend",
    );
    const nativeWindowLinuxWindowEventSourceRunLoopHostImpl = textSliceBetween(
        libSource,
        "impl<Host, BackendApi, ProducerApi, Provider>\n    NativeWindowLinuxWindowEventSourceRunLoopHost<Host, BackendApi, ProducerApi, Provider>",
        "pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_prepared_window_event_source_with_apis",
    );
    const nativeWindowLinuxWindowEventSourceRunLoopHostHandoff = textSliceBetween(
        libSource,
        "pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_prepared_window_event_source_with_apis",
        "impl<Api> NativeWindowHostLoopDeadlineTimerClock",
    );
    const nativeWindowLinuxWindowEventSourceEventPumpHostImpl = textSliceBetween(
        libSource,
        "impl<Host, BackendApi, ProducerApi, Provider>\n    NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost",
        "impl<Host, BackendApi, ProducerApi, Provider> NativeWindowRunLoopHost\n    for NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost",
    );
    const nativeWindowLinuxWindowEventSourceEventPumpRunLoopHostImpl = textSliceBetween(
        libSource,
        "impl<Host, BackendApi, ProducerApi, Provider> NativeWindowRunLoopHost\n    for NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost",
        "pub fn native_window_linux_window_event_source_enable_provider_event_pump",
    );
    const nativeWindowLinuxWindowEventSourceEventPumpEnableHelper = textSliceBetween(
        libSource,
        "pub fn native_window_linux_window_event_source_enable_provider_event_pump",
        "impl NativeWindowLinuxWindowEventSourceObservation",
    );
    const nativeWindowLinuxWindowEventSourceObservationTypes = textSliceBetween(
        libSource,
        "pub struct NativeWindowLinuxWindowEventSourceObservation",
        "pub enum NativeWindowRunLoopPlatformWaitBackendFromConfigError",
    );
    const nativeWindowLinuxWindowEventSourceObservationImpl = textSliceBetween(
        libSource,
        "impl NativeWindowLinuxWindowEventSourceObservation",
        "impl<Api> NativeWindowHostLoopDeadlineTimerClock",
    );
    const nativeWindowLinuxWindowEventSourceObservationSurface = [
        nativeWindowLinuxWindowEventSourceObservationTypes,
        nativeWindowLinuxWindowEventSourceObservationImpl,
    ].join("\n");
    const nativeWindowLinuxWindowEventSourceEventPumpSurface = [
        nativeWindowLinuxWindowEventSourceRunLoopTypes,
        nativeWindowLinuxWindowEventSourceEventPumpHostImpl,
        nativeWindowLinuxWindowEventSourceEventPumpRunLoopHostImpl,
        nativeWindowLinuxWindowEventSourceEventPumpEnableHelper,
    ].join("\n");
    const nativeWindowLinuxWindowEventSourceRunLoopOwnershipSurface = [
        nativeWindowLinuxWindowEventSourceRunLoopTypes,
        nativeWindowLinuxWindowEventSourcePreparedRunLoopConfigImpl,
        nativeWindowLinuxWindowEventSourceRunLoopHostImpl,
        nativeWindowLinuxWindowEventSourceRunLoopHostHandoff,
    ].join("\n");
    const nativeWindowPlatformWaitRunnerSupportGate = textSliceBetween(
        libSource,
        "pub fn validate_native_window_run_loop_platform_wait_runner_support_for_platform",
        "pub struct NativeWindowLinuxPlatformWaitRunLoopInput",
    );
    const nativeWindowLinuxPlatformWaitRunLoopInputError = textSliceBetween(
        libSource,
        "pub enum NativeWindowLinuxPlatformWaitRunLoopInputBuildError",
        "pub enum NativeWindowLinuxPlatformWaitRunLoopHostBuildError",
    );
    const nativeWindowLinuxPlatformWaitRunLoopHostError = textSliceBetween(
        libSource,
        "pub enum NativeWindowLinuxPlatformWaitRunLoopHostBuildError",
        "pub enum NativeWindowLinuxPlatformWaitRunLoopHostFromConfigBuildError",
    );
    const nativeWindowLinuxPlatformWaitRunLoopHostFromConfigError = textSliceBetween(
        libSource,
        "pub enum NativeWindowLinuxPlatformWaitRunLoopHostFromConfigBuildError",
        "#[cfg(target_os = \"windows\")]",
    );
    const nativeWindowLinuxPlatformWaitRunLoopInput = textSliceBetween(
        libSource,
        "pub struct NativeWindowLinuxPlatformWaitRunLoopInput",
        "pub enum NativeWindowHostLoopPlatformWaitHostBuildError",
    );
    const nativeWindowPlatformWaitBackendKind = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopPlatformKind",
        "pub const NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY",
    );
    const nativeWindowPlatformWaitKindValidation = textSliceBetween(
        libSource,
        "pub fn validate_native_window_host_loop_platform_wait_backend_kind_for_platform",
        "pub fn native_window_host_loop_default_platform_wait_backend_kind_for_platform",
    );
    const nativeWindowPlatformWaitDefaultKind = textSliceBetween(
        libSource,
        "pub fn native_window_host_loop_default_platform_wait_backend_kind_for_platform",
        "pub fn native_window_host_loop_default_platform_wait_backend_kind()",
    );
    const nativeWindowMacosRunLoopTimerBackend = textSliceBetween(
        libSource,
        "pub struct NativeWindowHostLoopMacosRunLoopTimerHandle",
        "pub struct NativeWindowHostLoopLinuxSelectorFd",
    );
    const nativeWindowLinuxSelectorTimerFdBackend = textSliceBetween(
        libSource,
        "pub struct NativeWindowHostLoopLinuxSelectorFd",
        "#[cfg(target_os = \"linux\")]",
    );
    const nativeWindowLinuxExternallyWakeableEventSourceOwner = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwnerBuildError",
        "impl<Api> Drop for NativeWindowHostLoopLinuxHostEventSignalProducer",
    );
    const nativeWindowLinuxExternallyWakeableRunLoopHost = textSliceBetween(
        libSource,
        "pub struct NativeWindowHostLoopLinuxExternallyWakeableEventSourceWaitAdapter",
        "impl<Api> NativeWindowHostLoopDeadlineTimerClock",
    );
    const nativeWindowLinuxPlatformWaitRunLoopHostFromInput = textSliceBetween(
        libSource,
        "pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_input",
        "pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis",
    );
    const nativeWindowLinuxPlatformWaitRunLoopHostFromConfig = textSliceBetween(
        libSource,
        "pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis",
        "impl<Api> NativeWindowHostLoopDeadlineTimerClock",
    );
    const nativeWindowLinuxSelectorTimerFdSysApi = textSliceBetween(
        libSource,
        "pub struct NativeWindowHostLoopLinuxSelectorTimerFdSysApi",
        "pub enum NativeWindowHostLoopNeverWindowsWaitRawApi",
    );
    const nativeWindowLinuxPlatformWaitRunLoopHostSysWrapper = textSliceBetween(
        libSource,
        "pub type NativeWindowLinuxPlatformWaitRunLoopSysHost",
        "pub enum NativeWindowHostLoopNeverWindowsWaitRawApi",
    );
    const nativeWindowNeverWindowsWaitRawApi = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopNeverWindowsWaitRawApi",
        "pub enum NativeWindowHostLoopNeverMacosRunLoopTimerRawApi",
    );
    const nativeWindowNeverMacosRunLoopTimerRawApi = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopNeverMacosRunLoopTimerRawApi",
        "pub enum NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi",
    );
    const nativeWindowNeverLinuxSelectorTimerFdRawApi = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi",
        "pub struct NativeWindowHostLoopWindowsWaitHandle",
    );
    const nativeWindowEventQueueStatusAdapter = textSliceBetween(
        libSource,
        "pub const NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY",
        "pub enum NativeWindowHostLoopMessagePumpStatusAdapterError",
    );
    const nativeWindowMessagePumpStatusAdapter = textSliceBetween(
        libSource,
        "pub enum NativeWindowHostLoopMessagePumpStatusAdapterError",
        "pub enum NativeWindowHostLoopTurn",
    );
    const nativeWindowHostLoopBoundedRunner = textSliceBetween(
        libSource,
        "pub fn run_native_window_host_loop_bounded<Host>",
        "pub fn run_native_window_host_loop_scheduler_slice_with_policy",
    );
    const nativeWindowHostLoopSchedulerSlice = textSliceBetween(
        libSource,
        "pub fn run_native_window_host_loop_scheduler_slice_with_policy<Host>",
        "pub fn run_native_window_host_loop<Host>",
    );
    const nativeWindowHostLoopRunner = textSliceBetween(
        libSource,
        "pub fn run_native_window_host_loop<Host>",
        "pub fn step_native_window_host_loop",
    );
    const nativeWindowFrameIntervalWaitAuthorityMode = textSliceBetween(
        libSource,
        "pub enum NativeWindowFrameIntervalWaitAuthorityMode",
        "pub struct NativeWindowMinifbFramePacingAuthority",
    );
    const nativeWindowMinifbFramePacingAuthority = textSliceBetween(
        libSource,
        "pub struct NativeWindowMinifbFramePacingAuthority",
        "pub fn step_native_window_host_loop",
    );
    const minifbNativeWindowLinuxObservedInputSignalBridge = textSliceBetween(
        libSource,
        "pub struct MinifbNativeWindowLinuxHostEventSignalCallbackState",
        "fn wait_minifb_window_host_event_message_pump",
    );
    const nativeWindowHostLoopTurnCore = textSliceBetween(
        libSource,
        "pub fn step_native_window_host_loop",
        "struct MinifbNativeWindowHostLoopMessagePumpAdapter",
    );
    const nativeWindowMinifbMessagePumpAdapter = textSliceBetween(
        libSource,
        "struct MinifbNativeWindowHostLoopMessagePumpAdapter",
        "struct MinifbNativeWindowVisualRunLoopHost",
    ).replace(minifbNativeWindowLinuxObservedInputSignalBridge, "");
    const nativeWindowMinifbVisualHostAdapter = textSliceBetween(
        libSource,
        "struct MinifbNativeWindowVisualRunLoopHost",
        "struct MinifbNativeWindowRunLoopHost",
    );
    const nativeWindowMinifbHostAdapter = textSliceBetween(
        libSource,
        "struct MinifbNativeWindowRunLoopHost",
        "pub fn run_minifb_window_loop",
    );
    const nativeWindowMinifbWaitMethod = textSliceBetween(
        nativeWindowMinifbHostAdapter,
        "fn wait_after_budget_exhausted",
        "\n    }\n}",
    );
    const nativeWindowMinifbRunner = textSliceBetween(
        libSource,
        "pub fn run_minifb_window_loop",
        "pub fn render_demo_frame",
    );
    const nativeWindowWindowsPlatformWaitRunner = textSliceBetween(
        libSource,
        "pub fn run_windows_platform_wait_window_loop",
        "fn native_window_run_loop_error_from_host_loop",
    );
    const nativeSpanOperationHelperWithoutEventPump = nativeSpanOperationHelper
        .replace(nativeWindowEventPumpHelper, "")
        .replace(nativeWindowThreadWaitBackend, "")
        .replace(nativeWindowTimerRegistrationBackend, "")
        .replace(nativeWindowTimerFireBackend, "")
        .replace(nativeWindowTimerWakeupBackend, "")
        .replace(nativeWindowDeadlineTimerAdapter, "")
        .replace(nativeWindowInterruptibleDeadlineWaitAdapter, "")
        .replace(nativeWindowEventQueueWaitBackend, "")
        .replace(nativeWindowHostLoopWaitOwner, "")
        .replace(nativeWindowHostOwnedDeadlineWaitRunLoopHost, "")
        .replace(nativeWindowInterruptibleDeadlineWaitRunLoopHost, "")
        .replace(nativeWindowPlatformWaitBackendKind, "")
        .replace(nativeWindowEventQueueStatusAdapter, "")
        .replace(nativeWindowMessagePumpStatusAdapter, "")
        .replace(nativeWindowFrameIntervalWaitAuthorityMode, "")
        .replace(nativeWindowMinifbFramePacingAuthority, "")
        .replace(nativeWindowMinifbMessagePumpAdapter, "")
        .replace(nativeWindowMinifbVisualHostAdapter, "");
    const nativeClockHelperWithoutWaitBackends = nativeClockHelper
        .replace(nativeWindowThreadWaitBackend, "")
        .replace(nativeWindowTimerRegistrationBackend, "")
        .replace(nativeWindowTimerFireBackend, "")
        .replace(nativeWindowTimerWakeupBackend, "")
        .replace(nativeWindowDeadlineTimerAdapter, "")
        .replace(nativeWindowInterruptibleDeadlineWaitAdapter, "")
        .replace(nativeWindowEventQueueWaitBackend, "")
        .replace(nativeWindowHostLoopWaitOwner, "")
        .replace(nativeWindowHostOwnedDeadlineWaitRunLoopHost, "")
        .replace(nativeWindowInterruptibleDeadlineWaitRunLoopHost, "")
        .replace(nativeWindowPlatformWaitBackendKind, "")
        .replace(nativeWindowEventQueueStatusAdapter, "")
        .replace(nativeWindowMessagePumpStatusAdapter, "")
        .replace(nativeWindowFrameIntervalWaitAuthorityMode, "")
        .replace(nativeWindowMinifbFramePacingAuthority, "")
        .replace(nativeWindowMinifbMessagePumpAdapter, "")
        .replace(nativeWindowMinifbVisualHostAdapter, "");
    const nativeWindowEventPumpHelperWithoutWaitBackends = nativeWindowEventPumpHelper
        .replace(nativeWindowThreadWaitBackend, "")
        .replace(nativeWindowTimerRegistrationBackend, "")
        .replace(nativeWindowTimerFireBackend, "")
        .replace(nativeWindowTimerWakeupBackend, "")
        .replace(nativeWindowDeadlineTimerAdapter, "")
        .replace(nativeWindowInterruptibleDeadlineWaitAdapter, "")
        .replace(nativeWindowEventQueueWaitBackend, "")
        .replace(nativeWindowHostLoopWaitOwner, "")
        .replace(nativeWindowHostOwnedDeadlineWaitRunLoopHost, "")
        .replace(nativeWindowInterruptibleDeadlineWaitRunLoopHost, "")
        .replace(nativeWindowPlatformWaitBackendKind, "")
        .replace(nativeWindowEventQueueStatusAdapter, "")
        .replace(nativeWindowMessagePumpStatusAdapter, "")
        .replace(nativeWindowFrameIntervalWaitAuthorityMode, "")
        .replace(nativeWindowMinifbFramePacingAuthority, "")
        .replace(nativeWindowMinifbMessagePumpAdapter, "")
        .replace(nativeWindowMinifbVisualHostAdapter, "");

    assert.match(mainSource, /NativeWindowTargetFps::new\(target_fps\)/);
    assert.match(mainSource, /"--fps"/);
    assert.match(mainSource, /enum NativeGuiWindowWaitBackend\s*\{[\s\S]*Minifb,[\s\S]*Platform/);
    assert.match(mainSource, /"--wait-backend"[\s\S]*options\.wait_backend\.is_some\(\)[\s\S]*"--wait-backend can be provided only once"[\s\S]*"--wait-backend requires a value"[\s\S]*NativeGuiWindowWaitBackend::parse\(&raw\)/);
    assert.match(mainSource, /fn validate_headless_options[\s\S]*options\.wait_backend\.is_some\(\)[\s\S]*"--wait-backend requires window mode"/);
    assert.match(mainSource, /fn window_wait_backend\(&self\) -> NativeGuiWindowWaitBackend[\s\S]*unwrap_or\(NativeGuiWindowWaitBackend::Minifb\)/);
    assert.match(mainSource, /NativeWindowRunLoopConfig::new_with_target_fps\([\s\S]*options\.demo,[\s\S]*options\.counter_value,[\s\S]*options\.scale,[\s\S]*options\.target_fps/);
    assert.match(mainSource, /run_minifb_window_loop\(config\)/);
    assert.match(mainSource, /match options\.window_wait_backend\(\)[\s\S]*NativeGuiWindowWaitBackend::Minifb => run_minifb_wait_window\(options\),[\s\S]*NativeGuiWindowWaitBackend::Platform => run_platform_wait_window\(options\)/);
    assert.match(mainSource, /#\[cfg\(all\(feature = "window", target_os = "windows", not\(target_arch = "wasm32"\)\)\)\][\s\S]*run_windows_platform_wait_window_loop/);
    assert.match(mainSource, /fn platform_wait_window_run_loop_config\([\s\S]*options: NativeGuiOptions[\s\S]*native_window_host_loop_default_platform_wait_backend_selection\(\)[\s\S]*NativeWindowRunLoopConfig::new_with_platform_wait_backend_selection\([\s\S]*NativeWindowHostLoopRunPolicy::default\(\)[\s\S]*selection/);
    assert.match(mainSource, /fn validate_platform_wait_window_runner_support\([\s\S]*config: NativeWindowRunLoopConfig[\s\S]*validate_native_window_run_loop_platform_wait_runner_support\(config\)[\s\S]*native platform wait runner unsupported/);
    assert.match(mainSource, /#\[cfg\(all\(feature = "window", target_os = "windows", not\(target_arch = "wasm32"\)\)\)\][\s\S]*fn run_platform_wait_window[\s\S]*platform_wait_window_run_loop_config\(options\)\?[\s\S]*validate_platform_wait_window_runner_support\(config\)\?[\s\S]*run_windows_platform_wait_window_loop\(config\)/);
    assert.match(mainSource, /#\[cfg\(all\([\s\S]*feature = "window"[\s\S]*not\(target_os = "windows"\)[\s\S]*not\(target_arch = "wasm32"\)[\s\S]*fn run_platform_wait_window[\s\S]*platform_wait_window_run_loop_config\(options\)\?[\s\S]*validate_platform_wait_window_runner_support\(config\)\?[\s\S]*native platform wait runner dispatch is unavailable/);
    assert.match(mainSource, /parse_rejects_duplicate_wait_backend/);
    assert.match(mainSource, /headless_rejects_explicit_wait_backend/);
    assert.match(mainSource, /platform_wait_config_builder_uses_platform_wait_backend/);
    assert.doesNotMatch(mainSource, /WindowOptions|ScaleMode|NativeWindowBackendLoop|NativeWindowHostAction|NativeWindowBackendLoopStepOutcome|poll_minifb_window_event_pump|current_present_frame_for_window|update_with_buffer|window\.update\(|window\.set_target_fps|window\.set_background_color|use\s+minifb|minifb::|let mut previous_size|previous_mouse_down|NativeWindowEventPumpInput\s*\{|NativeWindowPresenterState|counter_hit\(|map_native_window_point_to_image\(|checked_add\(|rasterize_frame_to_surface\(|present_buffer\(|resize_surface\(|let mut present_buffer|NativePresenterFrame::from_rgb0_present_buffer\(&present_buffer\)|wrapping_|saturating_|clamp|fallback|silent no-op/);
    assert.doesNotMatch(mainSource, /get_mouse_pos\(MouseMode::Clamp\)/);
    assert.doesNotMatch(mainSource, /\bKey\b|\bMouseButton\b|\bMouseMode\b|window\.is_open\(\)|window\.is_key_down\(|window\.get_mouse_down\(|window\.get_unscaled_mouse_pos\(/);

    assert.match(libSource, /pub struct NativeWindowBackendLoopPresentation\s*\{[\s\S]*frame_id: i32,[\s\S]*width: usize,[\s\S]*height: usize/);
    assert.match(libSource, /pub enum NativeWindowBackendLoopPointerAction\s*\{[\s\S]*PressedUnavailable,[\s\S]*PressedOutside,[\s\S]*CounterIncremented/);
    assert.match(libSource, /pub enum NativeWindowBackendLoopStepOutcome\s*\{[\s\S]*CloseRequested[\s\S]*Unavailable[\s\S]*Drawable/);
    assert.match(libSource, /pub enum NativeWindowHostTerminalReason\s*\{[\s\S]*OsCloseRequested,[\s\S]*ExitShortcutRequested/);
    assert.match(libSource, /pub enum NativeWindowHostAction\s*\{[\s\S]*Terminate[\s\S]*PumpEventsOnly[\s\S]*PresentFrame/);
    assert.match(libSource, /pub enum NativeWindowHostActionError\s*\{[\s\S]*UnsupportedCloseState[\s\S]*StepFailed\(NativeWindowBackendLoopError\)/);
    assert.match(libSource, /pub struct NativeWindowTargetFps\s*\{[\s\S]*value: u16/);
    assert.match(libSource, /pub enum NativeWindowTargetFpsInvalidReason\s*\{[\s\S]*Zero,[\s\S]*TooHigh\s*\{\s*max: u16\s*\}/);
    assert.match(libSource, /pub const NATIVE_WINDOW_RUN_LOOP_MIN_TARGET_FPS: u16 = 1/);
    assert.match(libSource, /pub const NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS: u16 = 240/);
    assert.match(libSource, /pub const NATIVE_WINDOW_RUN_LOOP_DEFAULT_TARGET_FPS: u16 = 60/);
    assert.match(libSource, /pub struct NativeWindowHostLoopTurnSlice\s*\{[\s\S]*value: u16/);
    assert.match(libSource, /pub enum NativeWindowHostLoopTurnSliceInvalidReason\s*\{[\s\S]*Zero,[\s\S]*TooHigh\s*\{\s*max: u16\s*\}/);
    assert.match(libSource, /pub struct NativeWindowHostLoopRunPolicy\s*\{[\s\S]*pub turn_slice: NativeWindowHostLoopTurnSlice/);
    assert.match(libSource, /pub const NATIVE_WINDOW_HOST_LOOP_MIN_TURN_SLICE: u16 = 1/);
    assert.match(libSource, /pub const NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE: u16 = 4096/);
    assert.match(libSource, /pub const NATIVE_WINDOW_HOST_LOOP_DEFAULT_TURN_SLICE: u16 = 1/);
    assert.match(libSource, /pub enum NativeWindowRunLoopFrameIntervalWaitBackend\s*\{[\s\S]*MinifbInternalTargetFps,[\s\S]*HostOwnedDeadlineTimer/);
    assert.match(libSource, /pub struct NativeWindowRunLoopPlatformWaitBackendConfig\s*\{[\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection,[\s\S]*linux_event_source_capability: Option<NativeWindowHostLoopLinuxEventSourceCapability>,[\s\S]*linux_window_event_source_raw_fd: Option<i32>/);
    assert.doesNotMatch(libSource, /pub struct NativeWindowRunLoopPlatformWaitBackendConfig\s*\{[\s\S]*pub selection:|pub struct NativeWindowRunLoopPlatformWaitBackendConfig\s*\{[\s\S]*pub linux_event_source_capability:|pub struct NativeWindowRunLoopPlatformWaitBackendConfig\s*\{[\s\S]*pub linux_window_event_source_raw_fd:/);
    assert.match(libSource, /pub enum NativeWindowRunLoopWaitBackend\s*\{[\s\S]*MinifbInternalTargetFps,[\s\S]*HostOwnedDeadlineTimer,[\s\S]*PlatformWait\(NativeWindowRunLoopPlatformWaitBackendConfig\)/);
    assert.match(libSource, /pub enum NativeWindowRunLoopFrameIntervalWaitBackendRunner\s*\{[\s\S]*Minifb/);
    assert.match(libSource, /pub enum NativeWindowRunLoopFrameIntervalWaitBackendError\s*\{[\s\S]*Unsupported\s*\{[\s\S]*runner: NativeWindowRunLoopFrameIntervalWaitBackendRunner,[\s\S]*requested: NativeWindowRunLoopWaitBackend,[\s\S]*reason: NativeWindowFrameIntervalWaitAuthorityModeError/);
    assert.match(libSource, /pub enum NativeWindowRunLoopPlatformWaitBackendConfigError\s*\{[\s\S]*NotPlatformWaitBackend\s*\{[\s\S]*requested: NativeWindowRunLoopWaitBackend[\s\S]*MissingLinuxEventSourceCapability\s*\{[\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection[\s\S]*MissingLinuxWindowEventSourceRawFd\s*\{[\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection/);
    assert.match(libSource, /pub enum NativeWindowRunLoopPlatformWaitBackendFromConfigError\s*\{[\s\S]*Config\(NativeWindowRunLoopPlatformWaitBackendConfigError\),[\s\S]*Build\(NativeWindowHostLoopPlatformWaitHostBuildError\)/);
    assert.match(libSource, /pub enum NativeWindowRunLoopPlatformWaitRunnerMissingIntegration\s*\{[\s\S]*LinuxWindowEventSourceFdMissing\s*\{[\s\S]*capability: NativeWindowHostLoopLinuxEventSourceCapability[\s\S]*LinuxWindowEventSourceEventParsingMissing\s*\{[\s\S]*capability: NativeWindowHostLoopLinuxEventSourceCapability[\s\S]*MacosActualSysShimMissing/);
    assert.match(libSource, /pub enum NativeWindowRunLoopPlatformWaitRunnerSupportError\s*\{[\s\S]*Config\(NativeWindowRunLoopPlatformWaitBackendConfigError\),[\s\S]*BackendSupportFailed\(NativeWindowHostLoopPlatformWaitBackendSupportError\),[\s\S]*LinuxEventSourceSupportFailed\(NativeWindowHostLoopLinuxPlatformWaitEventSourceSupportError\),[\s\S]*PlatformRunnerIntegrationMissing\s*\{[\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection,[\s\S]*missing: NativeWindowRunLoopPlatformWaitRunnerMissingIntegration[\s\S]*PlatformRunnerUnavailable/);
    assert.doesNotMatch(libSource, /LinuxExternallyWakeableEventSourceOwnerMissing|LinuxExternallyWakeableEventSourceIntegrationMissing/);
    assert.match(libSource, /pub enum NativeWindowLinuxWindowEventSourceKind\s*\{[\s\S]*X11Connection,[\s\S]*WaylandDisplay,[\s\S]*ToolkitExternal/);
    assert.match(libSource, /pub struct NativeWindowLinuxWindowEventSourceDescriptor\s*\{[\s\S]*source_kind: NativeWindowLinuxWindowEventSourceKind,[\s\S]*raw_fd: i32/);
    assert.match(libSource, /pub trait NativeWindowLinuxWindowEventSourceProvider\s*\{[\s\S]*type Error;[\s\S]*window_event_source_descriptor\([\s\S]*&mut self,[\s\S]*Result<NativeWindowLinuxWindowEventSourceDescriptor, Self::Error>/);
    assert.match(libSource, /pub struct NativeWindowLinuxWindowEventSourcePreparedPlatformWaitConfig<Provider>\s*\{[\s\S]*platform_wait_config: NativeWindowRunLoopPlatformWaitBackendConfig,[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*provider: Provider/);
    assert.doesNotMatch(libSource, /#\[derive\([^\]]*(Clone|Copy)[^\]]*\)\]\s*pub struct NativeWindowLinuxWindowEventSourcePreparedPlatformWaitConfig/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourcePreparedConfigImpl, /pub fn new\s*\(/);
    assert.match(libSource, /pub enum NativeWindowLinuxWindowEventSourcePreparePlatformWaitConfigError<Provider, ProviderError>\s*\{[\s\S]*BackendSupportFailed\s*\{[\s\S]*provider: Provider,[\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection,[\s\S]*error: NativeWindowHostLoopPlatformWaitBackendSupportError[\s\S]*ProviderFailed\s*\{[\s\S]*provider: Provider,[\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection,[\s\S]*error: ProviderError[\s\S]*InvalidDescriptor\s*\{[\s\S]*provider: Provider,[\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection,[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*error: NativeWindowHostLoopLinuxSelectorTimerFdBackendError/);
    assert.match(nativeWindowLinuxWindowEventSourcePrepareConfig, /validate_native_window_host_loop_platform_wait_backend_kind_for_platform\([\s\S]*NativeWindowHostLoopPlatformKind::Linux,[\s\S]*selection\.backend\(\)[\s\S]*provider\.window_event_source_descriptor\(\)[\s\S]*native_window_host_loop_linux_window_event_source_fd_from_raw\(descriptor\.raw_fd\(\)\)[\s\S]*NativeWindowRunLoopPlatformWaitBackendConfig::new_with_linux_window_event_source_raw_fd\([\s\S]*selection,[\s\S]*descriptor\.raw_fd\(\)[\s\S]*NativeWindowLinuxWindowEventSourcePreparedPlatformWaitConfig::new/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourcePrepareConfig, /NativeWindowRunLoopConfig::|new_with_platform_wait_backend_config|run_linux_platform_wait_window_loop|run_windows_platform_wait_window_loop|run_minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd_create|eventfd\(|read\(|drain|close\(|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopTypes, /pub struct NativeWindowLinuxWindowEventSourcePreparedRunLoopConfig<Provider>\s*\{[\s\S]*config: NativeWindowRunLoopConfig,[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*provider: Provider/);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopTypes, /pub struct NativeWindowLinuxWindowEventSourceRunLoopHost<[\s\S]*Host,[\s\S]*BackendApi,[\s\S]*ProducerApi,[\s\S]*Provider[\s\S]*>\s*where[\s\S]*host:\s*NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost<[\s\S]*Host,[\s\S]*BackendApi,[\s\S]*ProducerApi[\s\S]*>[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*provider: Provider/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceRunLoopTypes, /#\[derive\([^\]]*(Clone|Copy)[^\]]*\)\]\s*pub struct NativeWindowLinuxWindowEventSourcePreparedRunLoopConfig|#\[derive\([^\]]*(Clone|Copy)[^\]]*\)\]\s*pub struct NativeWindowLinuxWindowEventSourceRunLoopHost/);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopTypes, /pub enum NativeWindowLinuxWindowEventSourceRunLoopHostFromPreparedConfigError<[\s\S]*HostBuildFailed\s*\{[\s\S]*provider: Provider,[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*error: NativeWindowLinuxPlatformWaitRunLoopHostFromConfigBuildError<[\s\S]*Host,[\s\S]*BackendApi,[\s\S]*ProducerApi/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourcePreparedRunLoopConfigImpl, /pub fn new\s*\(|into_config|pub fn config\s*\(\s*self|NativeWindowLinuxWindowEventSourcePreparedRunLoopConfig::new\([\s\S]*selection/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceRunLoopOwnershipSurface, /pub fn into_config|pub fn into_host|pub fn config\s*\(\s*self\s*\)|pub fn host\s*\(\s*self\s*\)/);
    assert.match(nativeWindowLinuxWindowEventSourcePreparedRunLoopConfigImpl, /pub fn config\(&self\)[\s\S]*&NativeWindowRunLoopConfig[\s\S]*pub fn into_parts\([\s\S]*NativeWindowRunLoopConfig,[\s\S]*NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*Provider/);
    assert.match(nativeWindowLinuxWindowEventSourcePreparedRunLoopConfigImpl, /pub fn native_window_linux_window_event_source_prepare_run_loop_config<Provider>[\s\S]*prepared_platform_wait_config: NativeWindowLinuxWindowEventSourcePreparedPlatformWaitConfig<[\s\S]*Provider[\s\S]*demo: GuiDemo,[\s\S]*counter_value: i32,[\s\S]*scale: usize,[\s\S]*target_fps: NativeWindowTargetFps,[\s\S]*host_loop_policy: NativeWindowHostLoopRunPolicy[\s\S]*prepared_platform_wait_config\.into_parts\(\)[\s\S]*NativeWindowRunLoopConfig::new_with_platform_wait_backend_config\([\s\S]*demo,[\s\S]*counter_value,[\s\S]*scale,[\s\S]*target_fps,[\s\S]*host_loop_policy,[\s\S]*platform_wait_config[\s\S]*NativeWindowLinuxWindowEventSourcePreparedRunLoopConfig::new/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourcePreparedRunLoopConfigImpl, /validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing|run_linux_platform_wait_window_loop|run_windows_platform_wait_window_loop|run_minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd_create|eventfd\(|read\(|drain|close\(|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopHostImpl, /pub fn host\([\s\S]*&self[\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost[\s\S]*pub fn into_parts\([\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost<[\s\S]*NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*Provider/);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopHostImpl, /impl<Host, BackendApi, ProducerApi, Provider> NativeWindowRunLoopHost[\s\S]*NativeWindowLinuxWindowEventSourceRunLoopHost<Host, BackendApi, ProducerApi, Provider>[\s\S]*fn poll_event_snapshot[\s\S]*self\.host\.poll_event_snapshot\(input\)[\s\S]*fn wait_after_budget_exhausted[\s\S]*self\.host\.wait_after_budget_exhausted\(instruction\)/);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopHostHandoff, /pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_prepared_window_event_source_with_apis<[\s\S]*host: Host,[\s\S]*prepared_run_loop_config: NativeWindowLinuxWindowEventSourcePreparedRunLoopConfig<Provider>,[\s\S]*backend_api: BackendApi,[\s\S]*producer_api: ProducerApi[\s\S]*prepared_run_loop_config\.into_parts\(\)[\s\S]*native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis\([\s\S]*host,[\s\S]*config,[\s\S]*backend_api,[\s\S]*producer_api[\s\S]*NativeWindowLinuxWindowEventSourceRunLoopHost::new\([\s\S]*host,[\s\S]*descriptor,[\s\S]*provider[\s\S]*HostBuildFailed\s*\{[\s\S]*provider,[\s\S]*descriptor,[\s\S]*error/);
    assert.strictEqual((nativeWindowLinuxWindowEventSourceRunLoopHostHandoff.match(/native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis/g) || []).length, 1);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceRunLoopHostHandoff, /validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing|run_linux_platform_wait_window_loop|run_windows_platform_wait_window_loop|run_minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd_create|eventfd\(|read\(|drain|close\(|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopTypes, /pub trait NativeWindowLinuxWindowEventSourceEventPumpProvider\s*\{[\s\S]*type Error;[\s\S]*poll_window_event_source_snapshot\([\s\S]*&mut self,[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*input: NativeWindowEventPumpInput,[\s\S]*Result<NativeWindowEventPumpSnapshot, Self::Error>/);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopTypes, /pub struct NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost<[\s\S]*Host,[\s\S]*BackendApi,[\s\S]*ProducerApi,[\s\S]*Provider[\s\S]*>\s*where[\s\S]*host:\s*NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost<[\s\S]*Host,[\s\S]*BackendApi,[\s\S]*ProducerApi[\s\S]*>[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*provider: Provider/);
    assert.match(nativeWindowLinuxWindowEventSourceRunLoopTypes, /pub enum NativeWindowLinuxWindowEventSourceEventPumpRunLoopHostError<ProviderError>\s*\{[\s\S]*ProviderPollFailed\s*\{[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*error: ProviderError/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceEventPumpSurface, /pub fn into_host|pub fn into_provider|pub fn host\s*\(\s*self\s*\)|pub fn provider\s*\(\s*self\s*\)/);
    assert.match(nativeWindowLinuxWindowEventSourceEventPumpHostImpl, /pub fn host\([\s\S]*&self[\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost[\s\S]*pub fn provider\([\s\S]*&self[\s\S]*&Provider[\s\S]*pub fn into_parts\([\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost<[\s\S]*NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*Provider/);
    assert.match(nativeWindowLinuxWindowEventSourceEventPumpRunLoopHostImpl, /fn poll_event_snapshot\([\s\S]*input: NativeWindowEventPumpInput[\s\S]*self\.provider[\s\S]*poll_window_event_source_snapshot\(self\.descriptor, input\)[\s\S]*ProviderPollFailed\s*\{[\s\S]*descriptor: self\.descriptor,[\s\S]*error/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceEventPumpRunLoopHostImpl, /self\.host\.poll_event_snapshot/);
    assert.match(nativeWindowLinuxWindowEventSourceEventPumpRunLoopHostImpl, /fn set_window_title[\s\S]*self\.host\.set_window_title\(title\)[\s\S]*fn pump_events_only[\s\S]*self\.host\.pump_events_only\(\)[\s\S]*fn present_frame[\s\S]*self\.host\.present_frame\(frame\)[\s\S]*fn wait_after_budget_exhausted[\s\S]*self\.host\.wait_after_budget_exhausted\(instruction\)/);
    assert.match(nativeWindowLinuxWindowEventSourceEventPumpEnableHelper, /pub fn native_window_linux_window_event_source_enable_provider_event_pump<[\s\S]*host: NativeWindowLinuxWindowEventSourceRunLoopHost<Host, BackendApi, ProducerApi, Provider>[\s\S]*\) -> NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost<[\s\S]*host\.into_parts\(\)[\s\S]*NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost::new\(host, descriptor, provider\)/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceEventPumpEnableHelper, /Result<|Err\(|ProviderPollFailed|validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceEventPumpSurface, /validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing|run_linux_platform_wait_window_loop|run_windows_platform_wait_window_loop|run_minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd_create|eventfd\(|read\(|drain|close\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(nativeWindowLinuxWindowEventSourceObservationTypes, /pub struct NativeWindowLinuxWindowEventSourceObservation\s*\{[\s\S]*os_close_requested: bool,[\s\S]*exit_shortcut_requested: bool,[\s\S]*current_size: NativeWindowSize,[\s\S]*mouse_down: bool,[\s\S]*pointer_raw: Option<\(f32, f32\)>/);
    assert.match(nativeWindowLinuxWindowEventSourceObservationTypes, /pub enum NativeWindowLinuxWindowEventSourceObservationSnapshotError\s*\{[\s\S]*SnapshotConstructionFailed\s*\{[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*error: NativeWindowEventPumpError/);
    assert.match(nativeWindowLinuxWindowEventSourceObservationTypes, /pub trait NativeWindowLinuxWindowEventSourceObservationProvider\s*\{[\s\S]*type Error;[\s\S]*poll_window_event_source_observation\([\s\S]*&mut self,[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*input: NativeWindowEventPumpInput,[\s\S]*Result<NativeWindowLinuxWindowEventSourceObservation, Self::Error>/);
    assert.match(nativeWindowLinuxWindowEventSourceObservationTypes, /pub struct NativeWindowLinuxWindowEventSourceObservationEventPumpProvider<Provider>\s*\{[\s\S]*provider: Provider/);
    assert.match(nativeWindowLinuxWindowEventSourceObservationTypes, /pub enum NativeWindowLinuxWindowEventSourceObservationEventPumpProviderError<ProviderError>\s*\{[\s\S]*ProviderObservationFailed\s*\{[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*error: ProviderError[\s\S]*SnapshotConstructionFailed\s*\{[\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*error: NativeWindowLinuxWindowEventSourceObservationSnapshotError/);
    assert.match(nativeWindowLinuxWindowEventSourceObservationImpl, /pub fn native_window_linux_window_event_source_snapshot_from_observation\([\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*input: NativeWindowEventPumpInput,[\s\S]*observation: NativeWindowLinuxWindowEventSourceObservation[\s\S]*build_native_window_event_pump_snapshot_from_raw\([\s\S]*input,[\s\S]*observation\.os_close_requested\(\),[\s\S]*observation\.exit_shortcut_requested\(\),[\s\S]*observation\.current_size\(\),[\s\S]*observation\.mouse_down\(\),[\s\S]*observation\.pointer_raw\(\)[\s\S]*SnapshotConstructionFailed\s*\{[\s\S]*descriptor,[\s\S]*error/);
    assert.match(nativeWindowLinuxWindowEventSourceObservationImpl, /pub fn native_window_linux_window_event_source_enable_observation_provider_event_pump<[\s\S]*host: NativeWindowLinuxWindowEventSourceRunLoopHost<Host, BackendApi, ProducerApi, Provider>[\s\S]*\) -> NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost<[\s\S]*NativeWindowLinuxWindowEventSourceObservationEventPumpProvider<Provider>[\s\S]*Provider: NativeWindowLinuxWindowEventSourceObservationProvider[\s\S]*host\.into_parts\(\)[\s\S]*native_window_linux_window_event_source_observation_event_pump_provider\(provider\)[\s\S]*NativeWindowLinuxWindowEventSourceEventPumpRunLoopHost::new\(host, descriptor, provider\)/);
    assert.match(nativeWindowLinuxWindowEventSourceObservationImpl, /impl<Provider> NativeWindowLinuxWindowEventSourceEventPumpProvider[\s\S]*NativeWindowLinuxWindowEventSourceObservationEventPumpProvider<Provider>[\s\S]*Provider: NativeWindowLinuxWindowEventSourceObservationProvider[\s\S]*poll_window_event_source_snapshot\([\s\S]*descriptor: NativeWindowLinuxWindowEventSourceDescriptor,[\s\S]*input: NativeWindowEventPumpInput[\s\S]*self[\s\S]*\.provider[\s\S]*\.poll_window_event_source_observation\(descriptor, input\)[\s\S]*ProviderObservationFailed\s*\{[\s\S]*descriptor,[\s\S]*error[\s\S]*native_window_linux_window_event_source_snapshot_from_observation\([\s\S]*descriptor,[\s\S]*input,[\s\S]*observation[\s\S]*SnapshotConstructionFailed\s*\{[\s\S]*descriptor,[\s\S]*error/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceObservationImpl, /NativeWindowPointerSample::Unavailable[\s\S]*InvalidPointerSample|unwrap_or|unwrap_or_else|ok\(\)|Err\(.*InvalidPointerSample.*\)\.ok|self\.provider\s*=|drop\(|std::mem::take|std::mem::replace/);
    assert.doesNotMatch(nativeWindowLinuxWindowEventSourceObservationSurface, /validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing|run_linux_platform_wait_window_loop|run_windows_platform_wait_window_loop|run_minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd_create|eventfd\(|read\(|drain|close\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(libSource, /native_window_linux_window_event_source_observation_run_loop_adapter_polls_observation_provider/);
    assert.match(libSource, /native_window_linux_window_event_source_observation_run_loop_adapter_keeps_typed_failures/);
    assert.match(libSource, /native_window_linux_window_event_source_observation_run_loop_adapter_delegates_visual_and_wait/);
    assert.match(libSource, /pub type NativeWindowWindowsPlatformWaitHostLoopError[\s\S]*NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError[\s\S]*NativeWindowHostLoopPlatformWaitBackendError<[\s\S]*NativeWindowHostLoopWindowsWaitBackendError,[\s\S]*NativeWindowHostLoopMacosRunLoopTimerBackendError,[\s\S]*NativeWindowHostLoopLinuxSelectorTimerFdBackendError/);
    assert.match(libSource, /impl NativeWindowRunLoopFrameIntervalWaitBackend[\s\S]*pub fn authority_mode[\s\S]*MinifbInternalTargetFps[\s\S]*native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps[\s\S]*HostOwnedDeadlineTimer[\s\S]*native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer/);
    assert.match(libSource, /impl Default for NativeWindowRunLoopFrameIntervalWaitBackend[\s\S]*Self::MinifbInternalTargetFps/);
    assert.match(libSource, /impl From<NativeWindowRunLoopFrameIntervalWaitBackend> for NativeWindowRunLoopWaitBackend[\s\S]*HostOwnedDeadlineTimer[\s\S]*Self::HostOwnedDeadlineTimer/);
    assert.match(libSource, /impl NativeWindowRunLoopPlatformWaitBackendConfig[\s\S]*pub fn new\([\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection[\s\S]*linux_event_source_capability: None[\s\S]*linux_window_event_source_raw_fd: None[\s\S]*pub fn new_with_linux_event_source_capability\([\s\S]*linux_event_source_capability: NativeWindowHostLoopLinuxEventSourceCapability[\s\S]*Some\(linux_event_source_capability\)[\s\S]*linux_window_event_source_raw_fd: None[\s\S]*pub fn new_with_linux_window_event_source_raw_fd\([\s\S]*raw_fd: i32[\s\S]*ExternallyWakeableEventSource[\s\S]*linux_window_event_source_raw_fd: Some\(raw_fd\)[\s\S]*pub fn selection\(self\)[\s\S]*pub fn linux_event_source_capability[\s\S]*pub fn linux_window_event_source_raw_fd/);
    assert.match(libSource, /impl NativeWindowRunLoopWaitBackend[\s\S]*pub fn authority_mode[\s\S]*MinifbInternalTargetFps[\s\S]*native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps[\s\S]*HostOwnedDeadlineTimer[\s\S]*PlatformWait\(_\)[\s\S]*native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer/);
    assert.match(libSource, /impl Default for NativeWindowRunLoopWaitBackend[\s\S]*Self::MinifbInternalTargetFps/);
    assert.match(nativeWindowPlatformWaitRunnerSupportGate, /validate_native_window_run_loop_platform_wait_runner_support_for_platform[\s\S]*native_window_run_loop_platform_wait_backend_config\(config\)[\s\S]*validate_native_window_host_loop_platform_wait_backend_kind_for_platform[\s\S]*selection\.platform\(\) != current[\s\S]*NativeWindowRunLoopPlatformWaitRunnerSupportError::BackendSupportFailed/);
    assert.match(nativeWindowPlatformWaitRunnerSupportGate, /NativeWindowHostLoopPlatformKind::Windows => Ok\(selection\)/);
    assert.match(nativeWindowPlatformWaitRunnerSupportGate, /NativeWindowHostLoopPlatformKind::Linux[\s\S]*native_window_run_loop_linux_event_source_capability_from_platform_wait_config[\s\S]*validate_native_window_host_loop_linux_blocking_wait_event_source_capability[\s\S]*native_window_run_loop_linux_window_event_source_raw_fd_from_platform_wait_config[\s\S]*PlatformRunnerIntegrationMissing[\s\S]*LinuxWindowEventSourceEventParsingMissing/);
    assert.match(nativeWindowPlatformWaitRunnerSupportGate, /NativeWindowHostLoopPlatformKind::Macos[\s\S]*PlatformRunnerIntegrationMissing[\s\S]*MacosActualSysShimMissing/);
    assert.match(nativeWindowPlatformWaitRunnerSupportGate, /NativeWindowHostLoopPlatformKind::Unsupported[\s\S]*PlatformRunnerUnavailable/);
    assert.match(nativeWindowPlatformWaitRunnerSupportGate, /validate_native_window_run_loop_platform_wait_runner_support\([\s\S]*native_window_host_loop_current_platform_kind\(\)/);
    assert.doesNotMatch(nativeWindowPlatformWaitRunnerSupportGate, /native_window_host_loop_platform_wait_backend_from_selection|build_native_window_host_loop_platform_wait_backend_from_selection|native_window_run_loop_platform_wait_backend_from_config|NativeWindowHostLoopLinuxSelectorTimerFdSysApi|NativeWindowHostLoopWindowsWaitSysApi|NativeWindowHostLoopMacosRunLoopTimerBackend|WindowOptions|ScaleMode|run_minifb|run_windows_platform_wait_window_loop|run_linux_platform_wait_window_loop|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd|eventfd|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopInputError, /pub enum NativeWindowLinuxPlatformWaitRunLoopInputBuildError<BackendApi, ProducerApi>[\s\S]*Config\s*\{[\s\S]*config: NativeWindowRunLoopConfig,[\s\S]*owner: NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner<BackendApi, ProducerApi>,[\s\S]*error: NativeWindowRunLoopPlatformWaitBackendConfigError[\s\S]*PlatformUnavailable\s*\{[\s\S]*WrongCurrentPlatform\s*\{[\s\S]*BackendSupportFailed\s*\{[\s\S]*LinuxEventSourceSupportFailed\s*\{[\s\S]*OwnerClosed\s*\{/);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopHostError, /pub enum NativeWindowLinuxPlatformWaitRunLoopHostBuildError<Host, BackendApi, ProducerApi>[\s\S]*Config\s*\{[\s\S]*host: Host,[\s\S]*input: NativeWindowLinuxPlatformWaitRunLoopInput<BackendApi, ProducerApi>,[\s\S]*error: NativeWindowRunLoopPlatformWaitBackendConfigError[\s\S]*BackendSupportFailed\s*\{[\s\S]*LinuxEventSourceSupportFailed\s*\{[\s\S]*OwnerClosed\s*\{/);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopHostFromConfigError, /pub enum NativeWindowLinuxPlatformWaitRunLoopHostFromConfigBuildError<Host, BackendApi, ProducerApi>[\s\S]*Config\s*\{[\s\S]*host: Host,[\s\S]*config: NativeWindowRunLoopConfig,[\s\S]*error: NativeWindowRunLoopPlatformWaitBackendConfigError[\s\S]*BackendSupportFailed\s*\{[\s\S]*LinuxEventSourceSupportFailed\s*\{[\s\S]*BackendBuildFailed\s*\{[\s\S]*WindowEventSourceRegisterFailed\s*\{[\s\S]*backend: NativeWindowHostLoopLinuxSelectorTimerFdBackend<BackendApi>[\s\S]*error: NativeWindowHostLoopLinuxSelectorTimerFdBackendError[\s\S]*OwnerBuildFailed\s*\{[\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwnerBuildError<BackendApi>[\s\S]*InputBuildFailed\s*\{[\s\S]*NativeWindowLinuxPlatformWaitRunLoopInputBuildError<BackendApi, ProducerApi>[\s\S]*HostBuildFailed\s*\{[\s\S]*NativeWindowLinuxPlatformWaitRunLoopHostBuildError<Host, BackendApi, ProducerApi>/);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopInput, /pub struct NativeWindowLinuxPlatformWaitRunLoopInput<BackendApi, ProducerApi>[\s\S]*config: NativeWindowRunLoopConfig,[\s\S]*owner: NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner<BackendApi, ProducerApi>/);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopInput, /pub fn into_parts\([\s\S]*self[\s\S]*NativeWindowRunLoopConfig,[\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner<BackendApi, ProducerApi>[\s\S]*self\.config,[\s\S]*self\.owner/);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopInput, /pub fn native_window_linux_platform_wait_run_loop_input_for_platform<[\s\S]*current: NativeWindowHostLoopPlatformKind,[\s\S]*config: NativeWindowRunLoopConfig,[\s\S]*owner: NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner<BackendApi, ProducerApi>[\s\S]*native_window_run_loop_platform_wait_backend_config\(config\)[\s\S]*current == NativeWindowHostLoopPlatformKind::Unsupported[\s\S]*current != NativeWindowHostLoopPlatformKind::Linux[\s\S]*validate_native_window_host_loop_platform_wait_backend_kind_for_platform\([\s\S]*NativeWindowHostLoopPlatformKind::Linux,[\s\S]*selection\.backend\(\)[\s\S]*selection\.platform\(\) != NativeWindowHostLoopPlatformKind::Linux[\s\S]*native_window_run_loop_linux_event_source_capability_from_platform_wait_config[\s\S]*validate_native_window_host_loop_linux_blocking_wait_event_source_capability[\s\S]*!owner\.are_handles_open\(\)[\s\S]*NativeWindowLinuxPlatformWaitRunLoopInput\s*\{[\s\S]*config,[\s\S]*owner/);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopInput, /pub fn native_window_linux_platform_wait_run_loop_input<[\s\S]*native_window_host_loop_current_platform_kind\(\),[\s\S]*config,[\s\S]*owner/);
    assert.doesNotMatch(nativeWindowLinuxPlatformWaitRunLoopInput, /validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing|native_window_host_loop_platform_wait_backend_from_selection|build_native_window_host_loop_platform_wait_backend_from_selection|native_window_run_loop_platform_wait_backend_from_config|NativeWindowHostLoopLinuxSelectorTimerFdSysApi|NativeWindowHostLoopWindowsWaitSysApi|NativeWindowHostLoopMacosRunLoopTimerBackend|WindowOptions|ScaleMode|run_minifb|run_windows_platform_wait_window_loop|run_linux_platform_wait_window_loop|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd_create|timerfd_settime|timerfd_gettime|eventfd\(|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.doesNotMatch(nativeWindowLinuxPlatformWaitRunLoopInput, /backend: NativeWindowHostLoopLinuxSelectorTimerFdBackend<BackendApi>|producer: NativeWindowHostLoopLinuxHostEventSignalProducer<ProducerApi>|into_backend|self\.owner\.backend_mut\(\)\.signal_host_event/);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopHostFromInput, /pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_input<[\s\S]*host: Host,[\s\S]*input: NativeWindowLinuxPlatformWaitRunLoopInput<BackendApi, ProducerApi>[\s\S]*native_window_run_loop_platform_wait_backend_config\(config\)[\s\S]*validate_native_window_host_loop_platform_wait_backend_kind_for_platform\([\s\S]*NativeWindowHostLoopPlatformKind::Linux,[\s\S]*selection\.backend\(\)[\s\S]*selection\.platform\(\) != NativeWindowHostLoopPlatformKind::Linux[\s\S]*native_window_run_loop_linux_event_source_capability_from_platform_wait_config[\s\S]*validate_native_window_host_loop_linux_blocking_wait_event_source_capability[\s\S]*!input\.owner\(\)\.are_handles_open\(\)[\s\S]*input\.into_parts\(\)[\s\S]*native_window_host_loop_linux_externally_wakeable_event_source_run_loop_host_from_owner\([\s\S]*host,[\s\S]*owner/);
    assert.doesNotMatch(nativeWindowLinuxPlatformWaitRunLoopHostFromInput, /validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing|native_window_host_loop_platform_wait_backend_from_selection|build_native_window_host_loop_platform_wait_backend_from_selection|native_window_run_loop_platform_wait_backend_from_config|NativeWindowHostLoopLinuxSelectorTimerFdSysApi|NativeWindowHostLoopWindowsWaitSysApi|NativeWindowHostLoopMacosRunLoopTimerBackend|WindowOptions|ScaleMode|run_minifb|run_windows_platform_wait_window_loop|run_linux_platform_wait_window_loop|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd_create|timerfd_settime|timerfd_gettime|eventfd\(|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopHostFromConfig, /pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis<[\s\S]*host: Host,[\s\S]*config: NativeWindowRunLoopConfig,[\s\S]*backend_api: BackendApi,[\s\S]*producer_api: ProducerApi[\s\S]*native_window_run_loop_platform_wait_backend_config\(config\)[\s\S]*validate_native_window_host_loop_platform_wait_backend_kind_for_platform\([\s\S]*NativeWindowHostLoopPlatformKind::Linux,[\s\S]*selection\.backend\(\)[\s\S]*selection\.platform\(\) != NativeWindowHostLoopPlatformKind::Linux[\s\S]*native_window_run_loop_linux_event_source_capability_from_platform_wait_config[\s\S]*validate_native_window_host_loop_linux_blocking_wait_event_source_capability[\s\S]*native_window_run_loop_linux_window_event_source_raw_fd_from_platform_wait_config[\s\S]*build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection\([\s\S]*selection,[\s\S]*backend_api[\s\S]*register_window_event_source_fd_from_raw\(window_event_source_raw_fd\)[\s\S]*native_window_host_loop_linux_externally_wakeable_event_source_owner_from_backend\([\s\S]*backend,[\s\S]*producer_api[\s\S]*native_window_linux_platform_wait_run_loop_input_for_platform\([\s\S]*NativeWindowHostLoopPlatformKind::Linux,[\s\S]*config,[\s\S]*owner[\s\S]*native_window_host_loop_linux_platform_wait_run_loop_host_from_input\(host, input\)/);
    assert.doesNotMatch(nativeWindowLinuxPlatformWaitRunLoopHostFromConfig, /validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing|native_window_host_loop_platform_wait_backend_from_selection\(|build_native_window_host_loop_platform_wait_backend_from_selection|native_window_run_loop_platform_wait_backend_from_config|NativeWindowHostLoopLinuxSelectorTimerFdSysApi|NativeWindowHostLoopWindowsWaitSysApi|NativeWindowHostLoopMacosRunLoopTimerBackend|WindowOptions|ScaleMode|run_minifb|run_windows_platform_wait_window_loop|run_linux_platform_wait_window_loop|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|timerfd_create|timerfd_settime|timerfd_gettime|eventfd\(|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(nativeWindowLinuxPlatformWaitRunLoopHostSysWrapper, /pub type NativeWindowLinuxPlatformWaitRunLoopSysHost<Host>[\s\S]*NativeWindowHostLoopLinuxSelectorTimerFdSysApi[\s\S]*#\[cfg\(target_os = "linux"\)\][\s\S]*pub type NativeWindowLinuxPlatformWaitRunLoopHostFromConfigSysBuildError<Host>[\s\S]*#\[cfg\(target_os = "linux"\)\][\s\S]*pub fn native_window_host_loop_linux_platform_wait_run_loop_host_from_config<Host>[\s\S]*native_window_host_loop_linux_platform_wait_run_loop_host_from_config_with_apis\([\s\S]*host,[\s\S]*config,[\s\S]*NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new\(\),[\s\S]*NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new\(\)/);
    assert.doesNotMatch(nativeWindowLinuxPlatformWaitRunLoopHostSysWrapper, /validate_native_window_run_loop_platform_wait_runner_support_for_platform|PlatformRunnerIntegrationMissing|run_minifb|run_windows_platform_wait_window_loop|run_linux_platform_wait_window_loop|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|fallback|silent no-op|synthetic/i);
    assert.doesNotMatch(nativeWindowLinuxPlatformWaitRunLoopHostFromInput, /backend: NativeWindowHostLoopLinuxSelectorTimerFdBackend<BackendApi>|producer: NativeWindowHostLoopLinuxHostEventSignalProducer<ProducerApi>|into_backend|native_window_host_loop_platform_wait_run_loop_host_from_backend|NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd|self\.owner\.backend_mut\(\)\.signal_host_event/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_input_accepts_open_externally_wakeable_owner/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_input_rejects_missing_linux_capability_with_owner/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_input_rejects_observed_input_with_owner/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_input_rejects_wrong_current_platform_with_owner/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_input_rejects_closed_owner/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_input_delegates_visual_and_waits_through_owner/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_input_rejects_closed_owner_with_host_and_input/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_config_with_apis_builds_owner_host/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_config_rejects_config_before_raw_api/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_config_rejects_missing_window_fd_before_raw_api/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_config_preserves_backend_failure/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_config_preserves_window_fd_raw_registration_failure/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_config_preserves_window_fd_registration_failure/);
    assert.match(libSource, /native_window_linux_platform_wait_run_loop_host_from_config_preserves_owner_failure/);
    assert.match(libSource, /pub struct NativeWindowRunLoopConfig\s*\{[\s\S]*pub demo: GuiDemo,[\s\S]*pub counter_value: i32,[\s\S]*pub scale: usize,[\s\S]*pub target_fps: NativeWindowTargetFps,[\s\S]*pub host_loop_policy: NativeWindowHostLoopRunPolicy,[\s\S]*pub wait_backend: NativeWindowRunLoopWaitBackend/);
    assert.doesNotMatch(libSource, /pub frame_interval_wait_backend: NativeWindowRunLoopFrameIntervalWaitBackend/);
    assert.match(libSource, /pub struct NativeWindowRunLoopExit\s*\{[\s\S]*pub reason: NativeWindowHostTerminalReason/);
    assert.match(libSource, /pub enum NativeWindowRunLoopError\s*\{[\s\S]*TargetFpsInvalid\s*\{[\s\S]*value: usize,[\s\S]*reason: NativeWindowTargetFpsInvalidReason,[\s\S]*BackendLoopInitializationFailed\(NativeWindowBackendLoopError\)[\s\S]*WindowCreationFailed[\s\S]*EventPumpFailed\(NativeWindowEventPumpError\)[\s\S]*HostActionFailed\(NativeWindowHostActionError\)[\s\S]*PresenterFrameUnavailable\(NativeWindowBackendLoopError\)[\s\S]*WindowPresentFailed[\s\S]*HostWaitFailed[\s\S]*PlatformWaitRunnerUnsupported\(NativeWindowRunLoopPlatformWaitRunnerSupportError\)[\s\S]*PlatformWaitBackendFromConfigFailed\(NativeWindowRunLoopPlatformWaitBackendFromConfigError\)[\s\S]*WindowsPlatformWaitHostLoopFailed\(NativeWindowWindowsPlatformWaitHostLoopError\)[\s\S]*FrameIntervalWaitBackendUnsupported\(NativeWindowRunLoopFrameIntervalWaitBackendError\)[\s\S]*TimerFireResumeRequired/);
    assert.match(libSource, /pub trait NativeWindowRunLoopHost\s*\{[\s\S]*type EventError;[\s\S]*type PresentError;[\s\S]*type WaitError;[\s\S]*poll_event_snapshot[\s\S]*set_window_title[\s\S]*pump_events_only[\s\S]*present_frame[\s\S]*wait_after_budget_exhausted/);
    assert.match(libSource, /pub enum NativeWindowHostLoopError<EventError, PresentError, WaitError>\s*\{[\s\S]*HostEventPumpFailed\(EventError\)[\s\S]*HostActionFailed\(NativeWindowHostActionError\)[\s\S]*PresenterFrameUnavailable\(NativeWindowBackendLoopError\)[\s\S]*HostPresentFailed\(PresentError\)[\s\S]*HostWaitFailed\(WaitError\)[\s\S]*TimerFireResumeRequired[\s\S]*WaitDecisionMissing/);
    assert.match(libSource, /pub enum NativeWindowHostLoopContinueEvidence\s*\{[\s\S]*PumpedEventsOnly\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*PresentedFrame\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(libSource, /pub enum NativeWindowHostLoopWaitDecision\s*\{[\s\S]*WaitForHostEvent\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*WaitForFrameInterval\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(libSource, /pub const NATIVE_WINDOW_NANOS_PER_SECOND: u32 = 1_000_000_000/);
    assert.match(libSource, /pub struct NativeWindowFrameIntervalRequest\s*\{[\s\S]*target_fps: NativeWindowTargetFps,[\s\S]*nanos_per_frame: u32,[\s\S]*remainder_nanos_per_second: u32/);
    assert.match(libSource, /pub enum NativeWindowHostLoopWaitRequest\s*\{[\s\S]*WaitForHostEvent\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*WaitForFrameInterval\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*frame_interval: NativeWindowFrameIntervalRequest/);
    assert.match(libSource, /pub enum NativeWindowHostLoopWaitInstruction\s*\{[\s\S]*WaitForHostEvent\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*WaitForFrameInterval\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*frame_interval: NativeWindowFrameIntervalRequest,[\s\S]*wait_nanos: u32/);
    assert.match(nativeWindowHostLoopWaitOutcome, /pub enum NativeWindowHostLoopWaitOutcome/);
    assert.match(nativeWindowHostLoopWaitOutcome, /HostEventPumpAlreadyPaced\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(nativeWindowHostLoopWaitOutcome, /FramePresentAlreadyPaced\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(nativeWindowHostLoopWaitOutcome, /FrameIntervalTimerRegistered\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*wait_nanos: u32,[\s\S]*timer_registration_id: NativeWindowHostLoopTimerRegistrationId/);
    assert.match(nativeWindowHostLoopWaitOutcome, /FrameIntervalTimerFired\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*wait_nanos: u32,[\s\S]*timer_registration_id: NativeWindowHostLoopTimerRegistrationId/);
    assert.match(libSource, /pub enum NativeWindowHostLoopThreadWaitError<SleeperError>\s*\{[\s\S]*HostEventWaitUnsupported\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*FrameIntervalWaitNanosMismatch\s*\{[\s\S]*wait_nanos: u32,[\s\S]*nanos_per_frame: u32,[\s\S]*SleeperFailed\(SleeperError\)/);
    assert.match(libSource, /pub enum NativeWindowHostLoopThreadWaitOutcome\s*\{[\s\S]*FrameIntervalSlept\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*wait_nanos: u32/);
    assert.match(libSource, /pub trait NativeWindowHostLoopThreadSleeper\s*\{[\s\S]*type Error;[\s\S]*sleep_for_nanos\(&mut self,\s*wait_nanos: u32\) -> Result<\(\), Self::Error>/);
    assert.match(libSource, /pub struct NativeWindowHostLoopTimerRegistrationId\s*\{[\s\S]*raw_id: u32/);
    assert.match(nativeWindowTimerRegistrationError, /pub enum NativeWindowHostLoopTimerRegistrationError<RegistrarError>/);
    assert.match(nativeWindowTimerRegistrationError, /HostEventTimerRegistrationUnsupported\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(nativeWindowTimerRegistrationError, /FrameIntervalWaitNanosMismatch\s*\{[\s\S]*wait_nanos: u32,[\s\S]*nanos_per_frame: u32/);
    assert.match(nativeWindowTimerRegistrationError, /InvalidTimerRegistrationId\s*\{[\s\S]*raw_id: u32/);
    assert.match(nativeWindowTimerRegistrationError, /RegistrarFailed\(RegistrarError\)/);
    assert.match(libSource, /pub enum NativeWindowHostLoopTimerRegistrationOutcome\s*\{[\s\S]*FrameIntervalTimerRegistered\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*wait_nanos: u32,[\s\S]*timer_registration_id: NativeWindowHostLoopTimerRegistrationId/);
    assert.match(libSource, /pub trait NativeWindowHostLoopTimerRegistrar\s*\{[\s\S]*type Error;[\s\S]*register_timer_nanos\(&mut self,\s*wait_nanos: u32\) -> Result<u32, Self::Error>/);
    assert.match(nativeWindowTimerFireError, /pub enum NativeWindowHostLoopTimerFireError<WaiterError>/);
    assert.match(nativeWindowTimerFireError, /HostEventPumpOutcomeUnsupported\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(nativeWindowTimerFireError, /FramePresentOutcomeUnsupported\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(nativeWindowTimerFireError, /InvalidFiredTimerRegistrationId\s*\{[\s\S]*raw_id: u32/);
    assert.match(nativeWindowTimerFireError, /FiredTimerRegistrationMismatch\s*\{[\s\S]*expected_raw_id: u32,[\s\S]*actual_raw_id: u32/);
    assert.match(nativeWindowTimerFireError, /WaiterFailed\(WaiterError\)/);
    assert.match(libSource, /pub enum NativeWindowHostLoopTimerFireOutcome\s*\{[\s\S]*FrameIntervalTimerFired\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*wait_nanos: u32,[\s\S]*timer_registration_id: NativeWindowHostLoopTimerRegistrationId/);
    assert.match(libSource, /pub trait NativeWindowHostLoopTimerFireWaiter\s*\{[\s\S]*type Error;[\s\S]*wait_for_timer_fire\(\s*&mut self,[\s\S]*timer_registration_id: NativeWindowHostLoopTimerRegistrationId,[\s\S]*\) -> Result<u32, Self::Error>/);
    assert.match(libSource, /pub enum NativeWindowHostLoopTimerWakeError<RegistrarError,\s*FireWaiterError>\s*\{[\s\S]*RegistrationFailed\(NativeWindowHostLoopTimerRegistrationError<RegistrarError>\),[\s\S]*FireFailed\(NativeWindowHostLoopTimerFireError<FireWaiterError>\)/);
    assert.match(libSource, /pub fn execute_native_window_host_loop_timer_wakeup_with_backend<Registrar,\s*Waiter>/);
    assert.match(libSource, /pub enum NativeWindowHostLoopEventQueueWaitError<WaiterError>\s*\{[\s\S]*FrameIntervalEventQueueWaitUnsupported\s*\{[\s\S]*presentation: NativeWindowBackendLoopPresentation,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*frame_interval: NativeWindowFrameIntervalRequest,[\s\S]*wait_nanos: u32,[\s\S]*WaiterFailed\(WaiterError\)/);
    assert.match(libSource, /pub enum NativeWindowHostLoopEventQueueWaitOutcome\s*\{[\s\S]*HostEventReady\s*\{[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(libSource, /pub trait NativeWindowHostLoopEventQueueWaiter\s*\{[\s\S]*type Error;[\s\S]*wait_for_host_event\(\s*&mut self,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*\) -> Result<\(\), Self::Error>/);
    assert.match(libSource, /pub enum NativeWindowHostLoopWaitOwnerError<EventQueueError,\s*TimerClockError,\s*TimerSleeperError>\s*\{[\s\S]*EventQueueWaitFailed\(NativeWindowHostLoopEventQueueWaitError<EventQueueError>\),[\s\S]*FrameIntervalTimerWakeFailed\([\s\S]*NativeWindowHostLoopDeadlineTimerWakeError<TimerClockError,\s*TimerSleeperError>/);
    assert.match(libSource, /pub struct NativeWindowHostLoopWaitOwner<EventQueueWaiter,\s*TimerClock,\s*TimerSleeper>\s*\{[\s\S]*event_queue_waiter: EventQueueWaiter,[\s\S]*frame_interval_timer: NativeWindowHostLoopDeadlineTimerAdapter<TimerClock,\s*TimerSleeper>/);
    assert.match(libSource, /pub const NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY: u32 = 1/);
    assert.match(libSource, /pub enum NativeWindowHostLoopEventQueueStatusAdapterError<AdapterError>\s*\{[\s\S]*InvalidRawStatus\s*\{\s*raw_status: u32\s*\},[\s\S]*AdapterFailed\(AdapterError\)/);
    assert.match(libSource, /pub trait NativeWindowHostLoopEventQueueStatusAdapter\s*\{[\s\S]*type Error;[\s\S]*wait_for_host_event_raw_status\(\s*&mut self,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*\) -> Result<u32, Self::Error>/);
    assert.match(libSource, /pub struct NativeWindowHostLoopEventQueueStatusWaiter<Adapter>\s*\{[\s\S]*adapter: Adapter/);
    assert.match(libSource, /pub enum NativeWindowHostLoopMessagePumpStatusAdapterError<PumpError>\s*\{[\s\S]*PumpFailed\(PumpError\)/);
    assert.match(libSource, /pub trait NativeWindowHostLoopMessagePumpAdapter\s*\{[\s\S]*type Error;[\s\S]*pump_host_messages\(\s*&mut self,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool,[\s\S]*\) -> Result<\(\), Self::Error>/);
    assert.match(libSource, /pub struct NativeWindowHostLoopMessagePumpStatusAdapter<Adapter>\s*\{[\s\S]*adapter: Adapter/);
    assert.match(libSource, /pub enum NativeWindowHostLoopTurn\s*\{[\s\S]*Continue\(NativeWindowHostLoopContinueEvidence\),[\s\S]*Exit\(NativeWindowRunLoopExit\)/);
    assert.match(libSource, /pub struct NativeWindowHostLoopRunnerState\s*\{[\s\S]*title_initialized: bool/);
    assert.match(libSource, /pub enum NativeWindowHostLoopInitialization\s*\{[\s\S]*Initialized,[\s\S]*AlreadyInitialized/);
    assert.match(libSource, /pub enum NativeWindowHostLoopBoundedRunResult\s*\{[\s\S]*Exited\s*\{[\s\S]*exit: NativeWindowRunLoopExit,[\s\S]*completed_turns: usize,[\s\S]*BudgetExhausted\s*\{[\s\S]*completed_turns: usize,[\s\S]*last_wait_decision: Option<NativeWindowHostLoopWaitDecision>/);
    assert.match(libSource, /pub struct NativeWindowHostLoopSchedulerState\s*\{[\s\S]*runner_state: NativeWindowHostLoopRunnerState,[\s\S]*wait_strategy_state: NativeWindowHostLoopWaitStrategyState/);
    assert.match(libSource, /pub struct NativeWindowHostLoopWaitStrategyState\s*\{[\s\S]*frame_pacing_target_fps: Option<NativeWindowTargetFps>,[\s\S]*frame_pacing_remainder_nanos: u32/);
    assert.match(libSource, /pub struct NativeWindowHostLoopWaitInstructionPlan\s*\{[\s\S]*pub next_strategy_state: NativeWindowHostLoopWaitStrategyState,[\s\S]*pub instruction: NativeWindowHostLoopWaitInstruction/);
    assert.match(libSource, /pub enum NativeWindowHostLoopSchedulerSliceResult\s*\{[\s\S]*Exited\s*\{[\s\S]*exit: NativeWindowRunLoopExit,[\s\S]*completed_turns: usize,[\s\S]*Waited\s*\{[\s\S]*completed_turns: usize,[\s\S]*decision: NativeWindowHostLoopWaitDecision,[\s\S]*request: NativeWindowHostLoopWaitRequest,[\s\S]*instruction: NativeWindowHostLoopWaitInstruction,[\s\S]*outcome: NativeWindowHostLoopWaitOutcome/);
    assert.match(libSource, /pub enum NativeWindowHostLoopSchedulerResumeReady\s*\{[\s\S]*HostEventPumped[\s\S]*FramePresentPaced[\s\S]*FrameIntervalTimerFired/);
    assert.match(libSource, /pub enum NativeWindowHostLoopSchedulerResumeState\s*\{[\s\S]*Ready\(NativeWindowHostLoopSchedulerResumeReady\),[\s\S]*WaitingForFrameIntervalTimer/);
    assert.match(libSource, /pub enum NativeWindowFrameIntervalWaitAuthorityMode\s*\{[\s\S]*MinifbInternalTargetFps\s*\{\s*target_fps: NativeWindowTargetFps\s*\},[\s\S]*HostOwnedDeadlineTimer/);
    assert.match(libSource, /pub enum NativeWindowFrameIntervalWaitAuthorityModeError\s*\{[\s\S]*ConflictingFrameIntervalAuthorities\s*\{[\s\S]*active: NativeWindowFrameIntervalWaitAuthorityMode,[\s\S]*requested: NativeWindowFrameIntervalWaitAuthorityMode,[\s\S]*TargetFpsMismatch\s*\{[\s\S]*authority_target_fps: NativeWindowTargetFps,[\s\S]*instruction_target_fps: NativeWindowTargetFps/);
    assert.match(libSource, /pub struct NativeWindowMinifbFramePacingAuthority\s*\{[\s\S]*target_fps: NativeWindowTargetFps/);
    assert.match(libSource, /pub enum NativeWindowMinifbFramePacingAuthorityError\s*\{[\s\S]*FrameIntervalAuthorityConflict\s*\{[\s\S]*active: NativeWindowFrameIntervalWaitAuthorityMode,[\s\S]*requested: NativeWindowFrameIntervalWaitAuthorityMode,[\s\S]*FrameIntervalTargetFpsMismatch\s*\{[\s\S]*authority_target_fps: NativeWindowTargetFps,[\s\S]*instruction_target_fps: NativeWindowTargetFps,[\s\S]*FrameIntervalWaitNanosMismatch\s*\{[\s\S]*wait_nanos: u32,[\s\S]*nanos_per_frame: u32/);
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
    assert.match(nativeWindowHostLoopInitializer, /pub fn initialize_native_window_host_loop<Host>\([\s\S]*runner_state: &mut NativeWindowHostLoopRunnerState,[\s\S]*backend_loop: &NativeWindowBackendLoop,[\s\S]*host: &mut Host/);
    assert.match(nativeWindowHostLoopInitializer, /NativeWindowHostLoopInitialization/);
    assert.match(nativeWindowHostLoopInitializer, /AlreadyInitialized/);
    assert.match(nativeWindowHostLoopInitializer, /host\.set_window_title\(&initial_title\)/);
    assert.doesNotMatch(nativeWindowHostLoopInitializer, /poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|queue|timer|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowHostLoopWaitDecisionHelper, /NativeWindowHostLoopContinueEvidence::PumpedEventsOnly[\s\S]*NativeWindowHostLoopWaitDecision::WaitForHostEvent/);
    assert.match(nativeWindowHostLoopWaitDecisionHelper, /NativeWindowHostLoopContinueEvidence::PresentedFrame[\s\S]*NativeWindowHostLoopWaitDecision::WaitForFrameInterval/);
    assert.match(nativeWindowHostLoopWaitDecisionHelper, /pub fn native_window_frame_interval_request[\s\S]*NATIVE_WINDOW_NANOS_PER_SECOND \/ target_fps_value[\s\S]*NATIVE_WINDOW_NANOS_PER_SECOND % target_fps_value/);
    assert.match(nativeWindowHostLoopWaitDecisionHelper, /pub fn native_window_host_loop_wait_request[\s\S]*NativeWindowHostLoopWaitRequest::WaitForHostEvent[\s\S]*NativeWindowHostLoopWaitRequest::WaitForFrameInterval[\s\S]*native_window_frame_interval_request\(target_fps\)/);
    assert.match(nativeWindowHostLoopWaitDecisionHelper, /pub fn native_window_host_loop_wait_instruction_plan[\s\S]*NativeWindowHostLoopWaitRequest::WaitForHostEvent[\s\S]*NativeWindowHostLoopWaitInstruction::WaitForHostEvent/);
    assert.match(nativeWindowHostLoopWaitDecisionHelper, /NativeWindowHostLoopWaitRequest::WaitForFrameInterval[\s\S]*let carried_remainder = if strategy_state\.frame_pacing_target_fps\(\) == Some\(target_fps\)[\s\S]*strategy_state\.frame_pacing_remainder_nanos\(\)[\s\S]*else\s*\{[\s\S]*0[\s\S]*combined_remainder >= target_fps_value[\s\S]*frame_interval\.nanos_per_frame\(\) \+ 1[\s\S]*combined_remainder - target_fps_value[\s\S]*frame_interval\.nanos_per_frame\(\)[\s\S]*frame_pacing_remainder_nanos: next_remainder[\s\S]*NativeWindowHostLoopWaitInstruction::WaitForFrameInterval/);
    assert.match(nativeWindowHostLoopWaitDecisionHelper, /pub fn native_window_host_loop_scheduler_resume_state_from_wait_outcome[\s\S]*HostEventPumpAlreadyPaced[\s\S]*NativeWindowHostLoopSchedulerResumeReady::HostEventPumped[\s\S]*FramePresentAlreadyPaced[\s\S]*NativeWindowHostLoopSchedulerResumeReady::FramePresentPaced[\s\S]*FrameIntervalTimerRegistered[\s\S]*NativeWindowHostLoopSchedulerResumeState::WaitingForFrameIntervalTimer[\s\S]*FrameIntervalTimerFired[\s\S]*NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired/);
    assert.match(nativeWindowHostLoopWaitDecisionHelper, /pub fn native_window_host_loop_scheduler_resume_ready_from_timer_fire[\s\S]*NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired[\s\S]*NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired/);
    assert.doesNotMatch(nativeWindowHostLoopWaitDecisionHelper, /Result<|Err\(|panic!|pixels\(\)|frame\.pixels|&\s*\[[^\]]*\]|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|queue|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowThreadWaitBackend, /WaitForHostEvent\s*\{[\s\S]*NativeWindowHostLoopThreadWaitError::HostEventWaitUnsupported/);
    assert.match(nativeWindowThreadWaitBackend, /WaitForFrameInterval\s*\{[\s\S]*let nanos_per_frame = frame_interval\.nanos_per_frame\(\)[\s\S]*wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame \+ 1[\s\S]*NativeWindowHostLoopThreadWaitError::FrameIntervalWaitNanosMismatch/);
    assert.match(nativeWindowThreadWaitBackend, /sleeper[\s\S]*\.sleep_for_nanos\(wait_nanos\)[\s\S]*NativeWindowHostLoopThreadWaitError::SleeperFailed/);
    assert.match(nativeWindowThreadWaitBackend, /NativeWindowHostLoopThreadWaitOutcome::FrameIntervalSlept/);
    assert.match(nativeWindowThreadWaitBackend, /std::thread::sleep\(std::time::Duration::from_nanos\(u64::from\(wait_nanos\)\)\)/);
    assert.doesNotMatch(nativeWindowThreadWaitBackend, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|queue|timer|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowTimerRegistrationBackend, /WaitForHostEvent\s*\{[\s\S]*NativeWindowHostLoopTimerRegistrationError::HostEventTimerRegistrationUnsupported/);
    assert.match(nativeWindowTimerRegistrationBackend, /WaitForFrameInterval\s*\{[\s\S]*let nanos_per_frame = frame_interval\.nanos_per_frame\(\)[\s\S]*wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame \+ 1[\s\S]*NativeWindowHostLoopTimerRegistrationError::FrameIntervalWaitNanosMismatch/);
    assert.match(nativeWindowTimerRegistrationBackend, /registrar[\s\S]*\.register_timer_nanos\(wait_nanos\)[\s\S]*NativeWindowHostLoopTimerRegistrationError::RegistrarFailed/);
    assert.match(nativeWindowTimerRegistrationBackend, /if raw_id == 0[\s\S]*NativeWindowHostLoopTimerRegistrationError::InvalidTimerRegistrationId/);
    assert.match(nativeWindowTimerRegistrationBackend, /NativeWindowHostLoopTimerRegistrationOutcome::FrameIntervalTimerRegistered/);
    assert.match(nativeWindowTimerRegistrationBackend, /pub fn execute_native_window_host_loop_timer_registration_wait_with_registrar/);
    assert.match(nativeWindowTimerRegistrationBackend, /execute_native_window_host_loop_timer_registration_with_registrar\(instruction,\s*registrar\)\?/);
    assert.match(nativeWindowTimerRegistrationBackend, /NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered/);
    assert.doesNotMatch(nativeWindowTimerRegistrationBackend, /FramePresentAlreadyPaced/);
    assert.doesNotMatch(nativeWindowTimerRegistrationBackend, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|queue|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowTimerFireBackend, /HostEventPumpAlreadyPaced\s*\{[\s\S]*NativeWindowHostLoopTimerFireError::HostEventPumpOutcomeUnsupported/);
    assert.match(nativeWindowTimerFireBackend, /FramePresentAlreadyPaced\s*\{[\s\S]*NativeWindowHostLoopTimerFireError::FramePresentOutcomeUnsupported/);
    assert.match(nativeWindowTimerFireBackend, /FrameIntervalTimerRegistered\s*\{[\s\S]*waiter[\s\S]*\.wait_for_timer_fire\(timer_registration_id\)[\s\S]*NativeWindowHostLoopTimerFireError::WaiterFailed/);
    assert.match(nativeWindowTimerFireBackend, /if actual_raw_id == 0[\s\S]*NativeWindowHostLoopTimerFireError::InvalidFiredTimerRegistrationId/);
    assert.match(nativeWindowTimerFireBackend, /let expected_raw_id = timer_registration_id\.raw_id\(\)[\s\S]*if actual_raw_id != expected_raw_id[\s\S]*NativeWindowHostLoopTimerFireError::FiredTimerRegistrationMismatch/);
    assert.match(nativeWindowTimerFireBackend, /FrameIntervalTimerFired\s*\{[\s\S]*NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired/);
    assert.match(nativeWindowTimerFireBackend, /NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired/);
    assert.doesNotMatch(nativeWindowTimerFireBackend, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|queue|register_timer_nanos|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowTimerWakeupBackend, /RegistrationFailed\(NativeWindowHostLoopTimerRegistrationError<RegistrarError>\)/);
    assert.match(nativeWindowTimerWakeupBackend, /FireFailed\(NativeWindowHostLoopTimerFireError<FireWaiterError>\)/);
    assert.match(nativeWindowTimerWakeupBackend, /execute_native_window_host_loop_timer_registration_wait_with_registrar\(\s*instruction,\s*registrar,\s*\)[\s\S]*NativeWindowHostLoopTimerWakeError::RegistrationFailed/);
    assert.match(nativeWindowTimerWakeupBackend, /execute_native_window_host_loop_timer_fire_wait_with_waiter\(registration_outcome,\s*waiter\)[\s\S]*NativeWindowHostLoopTimerWakeError::FireFailed/);
    assert.match(nativeWindowTimerWakeupBackend, /pub fn native_window_host_loop_wait_outcome_from_timer_fire[\s\S]*NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired[\s\S]*NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired/);
    assert.match(nativeWindowTimerWakeupBackend, /pub fn execute_native_window_host_loop_timer_wakeup_wait_with_backend<Registrar,\s*Waiter>[\s\S]*execute_native_window_host_loop_timer_wakeup_with_backend\(instruction,\s*registrar,\s*waiter\)[\s\S]*\.map\(native_window_host_loop_wait_outcome_from_timer_fire\)/);
    assert.doesNotMatch(nativeWindowTimerWakeupBackend, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|queue|register_timer_nanos|wait_for_timer_fire|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowDeadlineTimerAdapter, /pub struct NativeWindowHostLoopDeadlineTimerRecord\s*\{[\s\S]*timer_registration_id: NativeWindowHostLoopTimerRegistrationId,[\s\S]*deadline_nanos: u64/);
    assert.match(nativeWindowDeadlineTimerAdapter, /pub enum NativeWindowHostLoopDeadlineTimerAdapterError<ClockError,\s*SleeperError>\s*\{[\s\S]*ActiveTimerAlreadyRegistered[\s\S]*NoActiveTimer[\s\S]*TimerRegistrationIdOverflow[\s\S]*DeadlineNanosOverflow[\s\S]*ClockFailed\(ClockError\)[\s\S]*SleeperFailed\(SleeperError\)[\s\S]*FiredTimerRegistrationMismatch/);
    assert.match(nativeWindowDeadlineTimerAdapter, /pub trait NativeWindowHostLoopDeadlineTimerClock\s*\{[\s\S]*now_nanos\(&mut self\) -> Result<u64,\s*Self::Error>/);
    assert.match(nativeWindowDeadlineTimerAdapter, /pub trait NativeWindowHostLoopDeadlineTimerSleeper\s*\{[\s\S]*sleep_until_nanos\(&mut self,\s*deadline_nanos: u64\) -> Result<\(\),\s*Self::Error>/);
    assert.match(nativeWindowDeadlineTimerAdapter, /impl<Clock,\s*Sleeper> NativeWindowHostLoopTimerRegistrar[\s\S]*for NativeWindowHostLoopDeadlineTimerAdapter<Clock,\s*Sleeper>/);
    assert.match(nativeWindowDeadlineTimerAdapter, /active_timer[\s\S]*ActiveTimerAlreadyRegistered[\s\S]*checked_add\(1\)[\s\S]*TimerRegistrationIdOverflow[\s\S]*clock[\s\S]*\.now_nanos\(\)[\s\S]*ClockFailed[\s\S]*checked_add\(u64::from\(wait_nanos\)\)[\s\S]*DeadlineNanosOverflow/);
    assert.match(nativeWindowDeadlineTimerAdapter, /impl<Clock,\s*Sleeper> NativeWindowHostLoopTimerFireWaiter[\s\S]*for NativeWindowHostLoopDeadlineTimerAdapter<Clock,\s*Sleeper>/);
    assert.match(nativeWindowDeadlineTimerAdapter, /NoActiveTimer[\s\S]*FiredTimerRegistrationMismatch[\s\S]*sleeper[\s\S]*\.sleep_until_nanos\(active_timer\.deadline_nanos\)[\s\S]*SleeperFailed[\s\S]*self\.active_timer = None/);
    assert.match(nativeWindowDeadlineTimerAdapter, /execute_native_window_host_loop_deadline_timer_wakeup_with_adapter[\s\S]*execute_native_window_host_loop_timer_registration_wait_with_registrar[\s\S]*execute_native_window_host_loop_timer_fire_wait_with_waiter/);
    assert.match(nativeWindowDeadlineTimerAdapter, /execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter[\s\S]*execute_native_window_host_loop_deadline_timer_wakeup_with_adapter\(instruction,\s*adapter\)[\s\S]*\.map\(native_window_host_loop_wait_outcome_from_timer_fire\)/);
    assert.match(nativeWindowDeadlineTimerAdapter, /StdNativeWindowHostLoopDeadlineTimerClock[\s\S]*StdNativeWindowHostLoopDeadlineTimerSleeper[\s\S]*std::time::Instant[\s\S]*std::thread::sleep\(std::time::Duration::from_nanos/);
    assert.doesNotMatch(nativeWindowDeadlineTimerAdapter, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|queue|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(libSource, /native_window_deadline_timer_adapter_registers_and_fires_frame_interval/);
    assert.match(libSource, /native_window_deadline_timer_adapter_rejects_active_overlap/);
    assert.match(libSource, /native_window_deadline_timer_adapter_rejects_missing_active_timer/);
    assert.match(libSource, /native_window_deadline_timer_adapter_rejects_mismatched_fire_id/);
    assert.match(libSource, /native_window_deadline_timer_adapter_rejects_registration_id_overflow/);
    assert.match(libSource, /native_window_deadline_timer_adapter_rejects_deadline_overflow/);
    assert.match(libSource, /native_window_deadline_timer_adapter_preserves_clock_error/);
    assert.match(libSource, /native_window_deadline_timer_adapter_preserves_active_timer_on_sleep_error/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /NativeWindowHostLoopInterruptibleDeadlineWake[\s\S]*HostEventReady[\s\S]*DeadlineReached/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError<ClockError,\s*WaiterError>[\s\S]*HostEventWaitFailed\(WaiterError\)[\s\S]*FrameIntervalWaitNanosMismatch[\s\S]*TimerRegistrationIdOverflow[\s\S]*DeadlineNanosOverflow[\s\S]*ClockFailed\(ClockError\)[\s\S]*FrameIntervalWaitFailed\(WaiterError\)/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /pub trait NativeWindowHostLoopInterruptibleDeadlineWaiter[\s\S]*wait_for_host_event\([\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool[\s\S]*wait_until_deadline_or_host_event\([\s\S]*deadline_nanos: u64,[\s\S]*window_size: NativeWindowSize,[\s\S]*size_changed: bool/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /pub struct NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock,\s*Waiter>[\s\S]*next_raw_id: u32,[\s\S]*clock: Clock,[\s\S]*waiter: Waiter/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /execute_native_window_host_loop_interruptible_deadline_wait_with_adapter[\s\S]*WaitForHostEvent[\s\S]*wait_for_host_event\(window_size,\s*size_changed\)[\s\S]*HostEventWaitFailed[\s\S]*HostEventPumpAlreadyPaced/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /WaitForFrameInterval[\s\S]*let nanos_per_frame = frame_interval\.nanos_per_frame\(\)[\s\S]*wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame \+ 1[\s\S]*FrameIntervalWaitNanosMismatch/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /let raw_id = adapter\.next_raw_id[\s\S]*checked_add\(1\)[\s\S]*TimerRegistrationIdOverflow[\s\S]*clock[\s\S]*\.now_nanos\(\)[\s\S]*ClockFailed[\s\S]*checked_add\(u64::from\(wait_nanos\)\)[\s\S]*DeadlineNanosOverflow/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /adapter\.next_raw_id = next_raw_id[\s\S]*wait_until_deadline_or_host_event\(deadline_nanos,\s*window_size,\s*size_changed\)[\s\S]*FrameIntervalWaitFailed/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady[\s\S]*NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached[\s\S]*NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError<BackendError>[\s\S]*HostEventWaitFailed\(BackendError\)[\s\S]*FrameIntervalWaitNanosMismatch[\s\S]*TimerRegistrationIdOverflow[\s\S]*DeadlineNanosOverflow[\s\S]*ClockFailed\(BackendError\)[\s\S]*FrameIntervalWaitFailed\(BackendError\)/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /pub struct NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend>[\s\S]*next_raw_id: u32,[\s\S]*backend: Backend/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /Backend: NativeWindowHostLoopDeadlineTimerClock[\s\S]*NativeWindowHostLoopInterruptibleDeadlineWaiter<[\s\S]*Error = <Backend as NativeWindowHostLoopDeadlineTimerClock>::Error/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter[\s\S]*WaitForHostEvent[\s\S]*adapter[\s\S]*\.backend[\s\S]*wait_for_host_event\(window_size,\s*size_changed\)[\s\S]*HostEventWaitFailed[\s\S]*HostEventPumpAlreadyPaced/);
    assert.match(nativeWindowInterruptibleDeadlineWaitAdapter, /execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter[\s\S]*let raw_id = adapter\.next_raw_id[\s\S]*checked_add\(1\)[\s\S]*TimerRegistrationIdOverflow[\s\S]*adapter\.backend\.now_nanos\(\)[\s\S]*ClockFailed[\s\S]*adapter\.next_raw_id = next_raw_id[\s\S]*adapter[\s\S]*\.backend[\s\S]*wait_until_deadline_or_host_event/);
    assert.doesNotMatch(nativeWindowInterruptibleDeadlineWaitAdapter, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(libSource, /native_window_interruptible_deadline_wait_waits_for_host_event_only/);
    assert.match(libSource, /native_window_interruptible_deadline_wait_returns_timer_fired_on_deadline/);
    assert.match(libSource, /native_window_interruptible_deadline_wait_returns_host_ready_without_timer_fire/);
    assert.match(libSource, /native_window_interruptible_deadline_wait_rejects_invalid_wait_before_side_effects/);
    assert.match(libSource, /native_window_interruptible_deadline_wait_preserves_host_event_wait_error/);
    assert.match(libSource, /native_window_interruptible_deadline_wait_preserves_clock_error/);
    assert.match(libSource, /native_window_interruptible_deadline_wait_rejects_deadline_overflow/);
    assert.match(libSource, /native_window_interruptible_deadline_wait_rejects_timer_id_overflow/);
    assert.match(libSource, /native_window_interruptible_deadline_wait_preserves_frame_wait_error/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_waits_for_host_event_only/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_returns_timer_fired_on_deadline/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_returns_host_ready_without_timer_fire/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_rejects_invalid_wait_before_side_effects/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_rejects_timer_id_overflow_before_side_effects/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_preserves_frame_wait_error/);
    assert.match(nativeWindowEventQueueWaitBackend, /WaitForHostEvent\s*\{[\s\S]*waiter[\s\S]*\.wait_for_host_event\(window_size,\s*size_changed\)[\s\S]*NativeWindowHostLoopEventQueueWaitError::WaiterFailed/);
    assert.match(nativeWindowEventQueueWaitBackend, /NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady/);
    assert.match(nativeWindowEventQueueWaitBackend, /WaitForFrameInterval\s*\{[\s\S]*NativeWindowHostLoopEventQueueWaitError::FrameIntervalEventQueueWaitUnsupported/);
    assert.doesNotMatch(nativeWindowEventQueueWaitBackend, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|register_timer_nanos|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowHostLoopWaitOwner, /EventQueueWaitFailed\(NativeWindowHostLoopEventQueueWaitError<EventQueueError>\)/);
    assert.match(nativeWindowHostLoopWaitOwner, /FrameIntervalAuthorityFailed\(NativeWindowFrameIntervalWaitAuthorityModeError\)/);
    assert.match(nativeWindowHostLoopWaitOwner, /FrameIntervalTimerWakeFailed\([\s\S]*NativeWindowHostLoopDeadlineTimerWakeError<TimerClockError,\s*TimerSleeperError>/);
    assert.match(nativeWindowHostLoopWaitOwner, /event_queue_waiter: EventQueueWaiter/);
    assert.match(nativeWindowHostLoopWaitOwner, /frame_interval_timer: NativeWindowHostLoopDeadlineTimerAdapter<TimerClock,\s*TimerSleeper>/);
    assert.match(nativeWindowHostLoopWaitOwner, /pub fn frame_interval_wait_authority_mode\(&self\) -> NativeWindowFrameIntervalWaitAuthorityMode[\s\S]*native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer\(\)/);
    assert.match(nativeWindowHostLoopWaitOwner, /execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode[\s\S]*requested_authority_mode: NativeWindowFrameIntervalWaitAuthorityMode/);
    assert.match(nativeWindowHostLoopWaitOwner, /WaitForHostEvent\s*\{[\s\S]*execute_native_window_host_loop_event_queue_wait_with_waiter\([\s\S]*owner\.event_queue_waiter_mut\(\)[\s\S]*NativeWindowHostLoopWaitOwnerError::EventQueueWaitFailed/);
    assert.match(nativeWindowHostLoopWaitOwner, /NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady[\s\S]*NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced/);
    assert.match(nativeWindowHostLoopWaitOwner, /WaitForFrameInterval\s*\{[\s\S]*let active_authority_mode = owner\.frame_interval_wait_authority_mode\(\)[\s\S]*combine_native_window_frame_interval_wait_authority_mode\([\s\S]*active_authority_mode,[\s\S]*requested_authority_mode,[\s\S]*FrameIntervalAuthorityFailed[\s\S]*validate_native_window_frame_interval_wait_authority_mode\([\s\S]*authority_mode,[\s\S]*frame_interval,[\s\S]*FrameIntervalAuthorityFailed[\s\S]*execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter/);
    assert.match(nativeWindowHostLoopWaitOwner, /execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode\([\s\S]*instruction,[\s\S]*owner,[\s\S]*authority_mode,[\s\S]*\)/);
    assert.match(nativeWindowHostLoopWaitOwner, /execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter\([\s\S]*owner\.frame_interval_timer_mut\(\)[\s\S]*NativeWindowHostLoopWaitOwnerError::FrameIntervalTimerWakeFailed/);
    assert.doesNotMatch(nativeWindowHostLoopWaitOwner, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(libSource, /native_window_wait_owner_dispatches_host_event_to_event_queue_only/);
    assert.match(libSource, /native_window_wait_owner_dispatches_frame_interval_to_timer_only/);
    assert.match(libSource, /native_window_wait_owner_ignores_frame_authority_for_host_event_wait/);
    assert.match(libSource, /native_window_wait_owner_rejects_minifb_frame_authority_before_timer_mutation/);
    assert.match(libSource, /native_window_wait_owner_preserves_event_queue_error_stage/);
    assert.match(libSource, /native_window_wait_owner_preserves_frame_interval_timer_error_stage/);
    assert.match(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /pub struct NativeWindowHostOwnedDeadlineWaitRunLoopHost<[\s\S]*host: Host,[\s\S]*wait_owner: NativeWindowHostLoopWaitOwner<EventQueueWaiter,\s*TimerClock,\s*TimerSleeper>/);
    assert.match(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /impl<Host,\s*EventQueueWaiter,\s*TimerClock,\s*TimerSleeper> NativeWindowRunLoopHost[\s\S]*for NativeWindowHostOwnedDeadlineWaitRunLoopHost/);
    assert.match(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /type EventError = Host::EventError;[\s\S]*type PresentError = Host::PresentError;[\s\S]*type WaitError = NativeWindowHostLoopWaitOwnerError<[\s\S]*EventQueueWaiter::Error,[\s\S]*TimerClock::Error,[\s\S]*TimerSleeper::Error/);
    assert.match(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /poll_event_snapshot[\s\S]*self\.host\.poll_event_snapshot\(input\)/);
    assert.match(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /set_window_title[\s\S]*self\.host\.set_window_title\(title\)/);
    assert.match(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /pump_events_only[\s\S]*self\.host\.pump_events_only\(\)/);
    assert.match(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /present_frame[\s\S]*self\.host\.present_frame\(frame\)/);
    assert.match(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /wait_after_budget_exhausted[\s\S]*execute_native_window_host_loop_wait_with_owner\(instruction,\s*&mut self\.wait_owner\)/);
    assert.doesNotMatch(nativeWindowHostOwnedDeadlineWaitRunLoopHost, /self\.host\.wait_after_budget_exhausted|stringify|to_string|format!|minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /pub struct NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost<Host,\s*Clock,\s*Waiter>\s*\{[\s\S]*host: Host,[\s\S]*wait_adapter: NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock,\s*Waiter>/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /impl<Host,\s*Clock,\s*Waiter> NativeWindowRunLoopHost[\s\S]*for NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost<Host,\s*Clock,\s*Waiter>/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /type EventError = Host::EventError;[\s\S]*type PresentError = Host::PresentError;[\s\S]*type WaitError =[\s\S]*NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError<Clock::Error,\s*Waiter::Error>/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /poll_event_snapshot[\s\S]*self\.host\.poll_event_snapshot\(input\)/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /set_window_title[\s\S]*self\.host\.set_window_title\(title\)/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /pump_events_only[\s\S]*self\.host\.pump_events_only\(\)/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /present_frame[\s\S]*self\.host\.present_frame\(frame\)/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /wait_after_budget_exhausted[\s\S]*execute_native_window_host_loop_interruptible_deadline_wait_with_adapter\([\s\S]*instruction,[\s\S]*&mut self\.wait_adapter/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /pub struct NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost<Host,\s*Backend>\s*\{[\s\S]*host: Host,[\s\S]*wait_adapter: NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend>/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /impl<Host,\s*Backend> NativeWindowRunLoopHost[\s\S]*for NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost<Host,\s*Backend>/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /Backend: NativeWindowHostLoopDeadlineTimerClock[\s\S]*NativeWindowHostLoopInterruptibleDeadlineWaiter<[\s\S]*Error = <Backend as NativeWindowHostLoopDeadlineTimerClock>::Error/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /type WaitError = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError<[\s\S]*<Backend as NativeWindowHostLoopDeadlineTimerClock>::Error/);
    assert.match(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /wait_after_budget_exhausted[\s\S]*execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter\([\s\S]*instruction,[\s\S]*&mut self\.wait_adapter/);
    assert.doesNotMatch(nativeWindowInterruptibleDeadlineWaitRunLoopHost, /self\.host\.wait_after_budget_exhausted|execute_native_window_host_loop_wait_with_owner|NativeWindowHostLoopWaitOwner|stringify|to_string|format!|minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowHostEventSignalWaitGuardRunLoopHost, /pub enum NativeWindowHostEventSignalWaitError<WaitError>\s*\{[\s\S]*HostEventSignalFailed\(NativeWindowHostLoopLinuxHostEventSignalProducerError\),[\s\S]*DelegateWaitFailed\(WaitError\)/);
    assert.match(nativeWindowHostEventSignalWaitGuardRunLoopHost, /pub trait NativeWindowHostEventSignalErrorState\s*\{[\s\S]*take_host_event_signal_error[\s\S]*Option<NativeWindowHostLoopLinuxHostEventSignalProducerError>/);
    assert.match(nativeWindowHostEventSignalWaitGuardRunLoopHost, /pub struct NativeWindowHostEventSignalWaitGuardRunLoopHost<Host,\s*SignalState>\s*\{[\s\S]*host: Host,[\s\S]*signal_state: SignalState/);
    assert.match(nativeWindowHostEventSignalWaitGuardRunLoopHost, /impl<Host,\s*SignalState> NativeWindowRunLoopHost[\s\S]*for NativeWindowHostEventSignalWaitGuardRunLoopHost<Host,\s*SignalState>[\s\S]*type WaitError = NativeWindowHostEventSignalWaitError<Host::WaitError>/);
    assert.match(nativeWindowHostEventSignalWaitGuardRunLoopHost, /wait_after_budget_exhausted\([\s\S]*if let Some\(error\) = self\.signal_state\.take_host_event_signal_error\(\)[\s\S]*HostEventSignalFailed[\s\S]*self\.host[\s\S]*\.wait_after_budget_exhausted\(instruction\)[\s\S]*DelegateWaitFailed/);
    assert.match(libSource, /native_window_host_event_signal_wait_guard_returns_signal_error_before_delegate_wait/);
    assert.match(libSource, /native_window_host_event_signal_wait_guard_delegates_without_synthetic_outcome/);
    assert.match(nativeWindowLinuxEventSourceCapabilityGate, /pub enum NativeWindowHostLoopLinuxEventSourceCapability\s*\{[\s\S]*ObservedInputOnly,[\s\S]*ExternallyWakeableEventSource/);
    assert.match(nativeWindowLinuxEventSourceCapabilityGate, /pub enum NativeWindowHostLoopLinuxPlatformWaitEventSourceSupportError\s*\{[\s\S]*ObservedInputOnlyUnsupportedForBlockingWait\s*\{[\s\S]*requested: NativeWindowHostLoopLinuxEventSourceCapability/);
    assert.match(nativeWindowLinuxEventSourceCapabilityGate, /validate_native_window_host_loop_linux_blocking_wait_event_source_capability[\s\S]*NativeWindowHostLoopLinuxEventSourceCapability::ObservedInputOnly[\s\S]*ObservedInputOnlyUnsupportedForBlockingWait[\s\S]*NativeWindowHostLoopLinuxEventSourceCapability::ExternallyWakeableEventSource[\s\S]*Ok\(requested\)/);
    assert.doesNotMatch(nativeWindowLinuxEventSourceCapabilityGate, /NativeWindowHostLoopWaitOutcome|HostEventReady|FrameIntervalTimerFired|TimerFired|register_|create_|signal_|selector_wait|run_linux_platform_wait_window_loop|run_windows_platform_wait_window_loop|run_minifb_window_loop|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(libSource, /native_window_linux_blocking_wait_event_source_rejects_observed_input_only/);
    assert.match(libSource, /native_window_linux_blocking_wait_event_source_accepts_externally_wakeable_classification/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopPlatformKind\s*\{[\s\S]*Macos,[\s\S]*Windows,[\s\S]*Linux,[\s\S]*Unsupported/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopPlatformWaitBackendKind\s*\{[\s\S]*MacosRunLoopTimer,[\s\S]*WindowsWaitableTimerMessageWait,[\s\S]*LinuxSelectorTimerFd,[\s\S]*HeadlessScripted/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopPlatformWaitBackendSupportError\s*\{[\s\S]*DefaultBackendUnsupportedPlatform[\s\S]*RequestedBackendUnsupportedPlatform[\s\S]*BackendPlatformMismatch/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub fn native_window_host_loop_current_platform_kind\(\) -> NativeWindowHostLoopPlatformKind[\s\S]*#\[cfg\(target_os = "macos"\)\][\s\S]*Macos[\s\S]*#\[cfg\(target_os = "windows"\)\][\s\S]*Windows[\s\S]*#\[cfg\(target_os = "linux"\)\][\s\S]*Linux[\s\S]*#\[cfg\(not\(any\(target_os = "macos", target_os = "windows", target_os = "linux"\)\)\)\][\s\S]*Unsupported/);
    assert.match(nativeWindowPlatformWaitKindValidation, /NativeWindowHostLoopPlatformKind::Unsupported/);
    assert.match(nativeWindowPlatformWaitKindValidation, /RequestedBackendUnsupportedPlatform/);
    assert.match(nativeWindowPlatformWaitKindValidation, /NativeWindowHostLoopPlatformKind::Macos[\s\S]*NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer/);
    assert.match(nativeWindowPlatformWaitKindValidation, /NativeWindowHostLoopPlatformKind::Windows[\s\S]*NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait/);
    assert.match(nativeWindowPlatformWaitKindValidation, /NativeWindowHostLoopPlatformKind::Linux[\s\S]*NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd/);
    assert.match(nativeWindowPlatformWaitKindValidation, /BackendPlatformMismatch/);
    assert.match(nativeWindowPlatformWaitDefaultKind, /NativeWindowHostLoopPlatformKind::Macos[\s\S]*NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer/);
    assert.match(nativeWindowPlatformWaitDefaultKind, /NativeWindowHostLoopPlatformKind::Windows[\s\S]*NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait/);
    assert.match(nativeWindowPlatformWaitDefaultKind, /NativeWindowHostLoopPlatformKind::Linux[\s\S]*NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd/);
    assert.match(nativeWindowPlatformWaitDefaultKind, /NativeWindowHostLoopPlatformKind::Unsupported/);
    assert.match(nativeWindowPlatformWaitDefaultKind, /DefaultBackendUnsupportedPlatform/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_host_loop_default_platform_wait_backend_kind\(\)[\s\S]*native_window_host_loop_current_platform_kind\(\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub struct NativeWindowHostLoopPlatformWaitBackendSelection\s*\{[\s\S]*platform: NativeWindowHostLoopPlatformKind,[\s\S]*backend: NativeWindowHostLoopPlatformWaitBackendKind/);
    assert.doesNotMatch(nativeWindowPlatformWaitBackendKind, /pub platform:|pub backend:/);
    assert.match(nativeWindowPlatformWaitBackendKind, /validate_native_window_host_loop_platform_wait_backend_selection_for_platform[\s\S]*validate_native_window_host_loop_platform_wait_backend_kind_for_platform\([\s\S]*platform,\s*requested,[\s\S]*\)\?[\s\S]*NativeWindowHostLoopPlatformWaitBackendSelection\s*\{\s*platform,\s*backend\s*\}/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_host_loop_default_platform_wait_backend_selection_for_platform[\s\S]*native_window_host_loop_default_platform_wait_backend_kind_for_platform\(platform\)\?[\s\S]*validate_native_window_host_loop_platform_wait_backend_selection_for_platform\([\s\S]*platform,\s*requested,[\s\S]*\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_run_loop_platform_wait_backend_config[\s\S]*match config\.wait_backend[\s\S]*NativeWindowRunLoopWaitBackend::PlatformWait\(platform_wait_config\)[\s\S]*Ok\(platform_wait_config\)[\s\S]*NotPlatformWaitBackend\s*\{\s*requested\s*\}/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_run_loop_platform_wait_backend_selection[\s\S]*native_window_run_loop_platform_wait_backend_config\(config\)\.map\(\|config\| config\.selection\(\)\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_run_loop_linux_event_source_capability_from_platform_wait_config[\s\S]*let selection = config\.selection\(\)[\s\S]*linux_event_source_capability\(\)\.ok_or\([\s\S]*MissingLinuxEventSourceCapability\s*\{[\s\S]*selection,[\s\S]*\}/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopPlatformWaitHostBuildError\s*\{[\s\S]*BackendSupportFailed\(NativeWindowHostLoopPlatformWaitBackendSupportError\),[\s\S]*BackendImplementationUnavailable\s*\{[\s\S]*platform: NativeWindowHostLoopPlatformKind,[\s\S]*backend: NativeWindowHostLoopPlatformWaitBackendKind[\s\S]*WindowsWaitBackendFailed\(NativeWindowHostLoopWindowsWaitBackendError\)[\s\S]*MacosRunLoopTimerBackendFailed\(NativeWindowHostLoopMacosRunLoopTimerBackendError\)[\s\S]*LinuxSelectorTimerFdBackendFailed\(NativeWindowHostLoopLinuxSelectorTimerFdBackendError\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /build_native_window_host_loop_platform_wait_backend_from_selection[\s\S]*BackendImplementationUnavailable\s*\{[\s\S]*platform: selection\.platform\(\),[\s\S]*backend: selection\.backend\(\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /build_native_window_host_loop_platform_wait_backend_for_platform[\s\S]*validate_native_window_host_loop_platform_wait_backend_selection_for_platform\([\s\S]*platform,\s*requested,[\s\S]*BackendSupportFailed[\s\S]*build_native_window_host_loop_platform_wait_backend_from_selection\(selection\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopPlatformWaitBackend<WindowsApi,\s*MacosApi,\s*LinuxApi>[\s\S]*WindowsWaitableTimerMessageWait\(NativeWindowHostLoopWindowsWaitBackend<WindowsApi>\)[\s\S]*MacosRunLoopTimer\(NativeWindowHostLoopMacosRunLoopTimerBackend<MacosApi>\)[\s\S]*LinuxSelectorTimerFd\(NativeWindowHostLoopLinuxSelectorTimerFdBackend<LinuxApi>\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub type NativeWindowHostLoopWindowsOnlyPlatformWaitBackend<WindowsApi>[\s\S]*NativeWindowHostLoopNeverMacosRunLoopTimerRawApi[\s\S]*NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub type NativeWindowHostLoopLinuxOnlyPlatformWaitBackend<LinuxApi>[\s\S]*NativeWindowHostLoopNeverWindowsWaitRawApi[\s\S]*NativeWindowHostLoopNeverMacosRunLoopTimerRawApi[\s\S]*LinuxApi/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopPlatformWaitBackendError<WindowsError,\s*MacosError,\s*LinuxError>\s*\{[\s\S]*WindowsWaitableTimerMessageWait\(WindowsError\)[\s\S]*MacosRunLoopTimer\(MacosError\)[\s\S]*LinuxSelectorTimerFd\(LinuxError\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /impl<WindowsApi,\s*MacosApi,\s*LinuxApi> NativeWindowHostLoopDeadlineTimerClock[\s\S]*for NativeWindowHostLoopPlatformWaitBackend<WindowsApi,\s*MacosApi,\s*LinuxApi>[\s\S]*WindowsWaitableTimerMessageWait[\s\S]*MacosRunLoopTimer[\s\S]*LinuxSelectorTimerFd/);
    assert.match(nativeWindowPlatformWaitBackendKind, /impl<WindowsApi,\s*MacosApi,\s*LinuxApi> NativeWindowHostLoopInterruptibleDeadlineWaiter[\s\S]*for NativeWindowHostLoopPlatformWaitBackend<WindowsApi,\s*MacosApi,\s*LinuxApi>[\s\S]*wait_for_host_event[\s\S]*WindowsWaitableTimerMessageWait[\s\S]*MacosRunLoopTimer[\s\S]*LinuxSelectorTimerFd[\s\S]*wait_until_deadline_or_host_event[\s\S]*WindowsWaitableTimerMessageWait[\s\S]*MacosRunLoopTimer[\s\S]*LinuxSelectorTimerFd/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopNeverWindowsWaitRawApi\s*\{\s*\}[\s\S]*impl NativeWindowHostLoopWindowsWaitRawApi[\s\S]*match \*self \{\}/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopNeverMacosRunLoopTimerRawApi\s*\{\s*\}[\s\S]*impl NativeWindowHostLoopMacosRunLoopTimerRawApi[\s\S]*match \*self \{\}/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi\s*\{\s*\}[\s\S]*impl NativeWindowHostLoopLinuxSelectorTimerFdRawApi[\s\S]*match \*self \{\}/);
    assert.match(nativeWindowNeverWindowsWaitRawApi, /create_waitable_timer_raw[\s\S]*match \*self \{\}[\s\S]*set_waitable_timer_relative_100ns[\s\S]*match \*self \{\}[\s\S]*msg_wait_for_timer_or_message_raw[\s\S]*match \*self \{\}[\s\S]*msg_wait_for_message_raw[\s\S]*match \*self \{\}[\s\S]*close_handle_raw[\s\S]*match \*self \{\}[\s\S]*last_error_code[\s\S]*match \*self \{\}/);
    assert.match(nativeWindowNeverMacosRunLoopTimerRawApi, /create_run_loop_timer_raw[\s\S]*match \*self \{\}[\s\S]*schedule_run_loop_timer_relative_nanos[\s\S]*match \*self \{\}[\s\S]*run_loop_wait_for_timer_or_event_raw[\s\S]*match \*self \{\}[\s\S]*run_loop_wait_for_event_raw[\s\S]*match \*self \{\}[\s\S]*invalidate_run_loop_timer_raw[\s\S]*match \*self \{\}[\s\S]*last_error_code[\s\S]*match \*self \{\}/);
    assert.match(nativeWindowNeverLinuxSelectorTimerFdRawApi, /create_selector_raw[\s\S]*match \*self \{\}[\s\S]*create_timer_fd_raw[\s\S]*match \*self \{\}[\s\S]*create_host_event_fd_raw[\s\S]*match \*self \{\}[\s\S]*register_timer_fd_raw[\s\S]*match \*self \{\}[\s\S]*register_host_event_fd_raw[\s\S]*match \*self \{\}[\s\S]*signal_host_event_fd_raw[\s\S]*match \*self \{\}[\s\S]*arm_timer_fd_relative_timespec[\s\S]*match \*self \{\}[\s\S]*selector_wait_for_timer_or_event_raw[\s\S]*match \*self \{\}[\s\S]*selector_wait_for_event_raw[\s\S]*match \*self \{\}[\s\S]*close_selector_raw[\s\S]*match \*self \{\}[\s\S]*close_timer_fd_raw[\s\S]*match \*self \{\}[\s\S]*close_host_event_fd_raw[\s\S]*match \*self \{\}[\s\S]*last_error_code[\s\S]*match \*self \{\}/);
    assert.doesNotMatch(nativeWindowNeverWindowsWaitRawApi, /panic!|unreachable!|todo!|Ok\(|return true|return false|STATUS_|fallback|silent no-op/i);
    assert.doesNotMatch(nativeWindowNeverMacosRunLoopTimerRawApi, /panic!|unreachable!|todo!|Ok\(|return true|return false|STATUS_|fallback|silent no-op/i);
    assert.doesNotMatch(nativeWindowNeverLinuxSelectorTimerFdRawApi, /panic!|unreachable!|todo!|Ok\(|return true|return false|STATUS_|fallback|silent no-op/i);
    assert.match(nativeWindowPlatformWaitBackendKind, /LinuxEventSourceSupportFailed\([\s\S]*NativeWindowHostLoopLinuxPlatformWaitEventSourceSupportError[\s\S]*\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis[\s\S]*linux_event_source_capability: NativeWindowHostLoopLinuxEventSourceCapability[\s\S]*validate_native_window_host_loop_platform_wait_backend_selection_for_platform\([\s\S]*selection\.platform\(\),[\s\S]*selection\.backend\(\)[\s\S]*WindowsWaitableTimerMessageWait[\s\S]*windows_api[\s\S]*MacosRunLoopTimer[\s\S]*macos_api[\s\S]*LinuxSelectorTimerFd[\s\S]*validate_native_window_host_loop_linux_blocking_wait_event_source_capability\([\s\S]*linux_event_source_capability[\s\S]*LinuxEventSourceSupportFailed[\s\S]*linux_api/);
    assert.match(nativeWindowPlatformWaitBackendKind, /build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api[\s\S]*validate_native_window_host_loop_platform_wait_backend_selection_for_platform\([\s\S]*selection\.platform\(\),[\s\S]*selection\.backend\(\)[\s\S]*BackendSupportFailed[\s\S]*NativeWindowHostLoopPlatformKind::Windows[\s\S]*WindowsWaitableTimerMessageWait[\s\S]*build_native_window_host_loop_windows_wait_backend_from_selection\([\s\S]*checked_selection,[\s\S]*api/);
    assert.match(nativeWindowPlatformWaitBackendKind, /build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api[\s\S]*event_source_capability: NativeWindowHostLoopLinuxEventSourceCapability[\s\S]*validate_native_window_host_loop_platform_wait_backend_selection_for_platform\([\s\S]*selection\.platform\(\),[\s\S]*selection\.backend\(\)[\s\S]*BackendSupportFailed[\s\S]*NativeWindowHostLoopPlatformKind::Linux[\s\S]*LinuxSelectorTimerFd[\s\S]*validate_native_window_host_loop_linux_blocking_wait_event_source_capability\([\s\S]*event_source_capability[\s\S]*LinuxEventSourceSupportFailed[\s\S]*build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection\([\s\S]*checked_selection,[\s\S]*api/);
    assert.match(nativeWindowPlatformWaitBackendKind, /WindowsWaitBackendFailed\(NativeWindowHostLoopWindowsWaitBackendError\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /LinuxSelectorTimerFdBackendFailed\(NativeWindowHostLoopLinuxSelectorTimerFdBackendError\)/);
    assert.doesNotMatch(libSource, /build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api\(\s*selection,\s*(?:api|NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new\(\)),\s*\)/);
    assert.match(libSource, /native_window_platform_wait_backend_with_linux_api_rejects_observed_input_source_before_raw_calls/);
    assert.match(libSource, /native_window_platform_wait_backend_with_raw_apis_rejects_observed_input_source_before_linux_raw_calls/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub type NativeWindowHostLoopPlatformWaitRunLoopHost<Host,\s*WindowsApi,\s*MacosApi,\s*LinuxApi>\s*=[\s\S]*NativeWindowHostLoopPlatformWaitBackend<WindowsApi,\s*MacosApi,\s*LinuxApi>/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub fn native_window_host_loop_platform_wait_run_loop_host_from_backend<[\s\S]*host: Host,[\s\S]*backend: NativeWindowHostLoopPlatformWaitBackend<WindowsApi,\s*MacosApi,\s*LinuxApi>[\s\S]*NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new\(backend\)[\s\S]*NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost::new\(host,\s*wait_adapter\)/);
    assert.doesNotMatch(nativeWindowPlatformWaitBackendKind, /build_native_window_host_loop_platform_wait_run_loop_host_from_selection/);
    assert.doesNotMatch(nativeWindowPlatformWaitBackendKind, /pub fn [A-Za-z0-9_]*from_selection[A-Za-z0-9_]*<Host[\s\S]*host: Host/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub struct NativeWindowHostLoopWindowsWaitHandle\s*\{[\s\S]*raw_handle: isize/);
    assert.doesNotMatch(nativeWindowPlatformWaitBackendKind, /pub raw_handle:|pub fn raw_handle|pub fn handle\(/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub fn is_handle_open\(&self\) -> bool[\s\S]*self\.handle\.is_some\(\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub trait NativeWindowHostLoopWindowsWaitRawApi\s*\{[\s\S]*create_waitable_timer_raw[\s\S]*set_waitable_timer_relative_100ns[\s\S]*msg_wait_for_timer_or_message_raw[\s\S]*msg_wait_for_message_raw[\s\S]*close_handle_raw[\s\S]*last_error_code/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_host_loop_windows_wait_handle_from_raw[\s\S]*raw_handle == 0 \|\| raw_handle == -1[\s\S]*InvalidRawHandle/);
    assert.match(nativeWindowPlatformWaitBackendKind, /pub enum NativeWindowHostLoopWindowsDeadlinePlan\s*\{[\s\S]*AlreadyReached,[\s\S]*Relative100ns\(i64\)/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_host_loop_windows_deadline_plan[\s\S]*deadline_nanos <= now_nanos[\s\S]*AlreadyReached[\s\S]*checked_add\(99\)[\s\S]*Relative100ns\([\s\S]*-relative_100ns_i64/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_host_loop_windows_wait_wake_from_timer_or_message_status[\s\S]*TIMER_SIGNALED[\s\S]*DeadlineReached[\s\S]*MESSAGE_READY_ONE_HANDLE[\s\S]*HostEventReady[\s\S]*WAIT_STATUS_FAILED[\s\S]*WaitFailed[\s\S]*UnexpectedWaitStatus/);
    assert.match(nativeWindowPlatformWaitBackendKind, /native_window_host_loop_windows_host_event_from_message_status[\s\S]*MESSAGE_READY_ZERO_HANDLES[\s\S]*Ok\(\(\)\)[\s\S]*WAIT_STATUS_FAILED[\s\S]*WaitFailed[\s\S]*UnexpectedWaitStatus/);
    assert.match(nativeWindowPlatformWaitBackendKind, /impl<Api> NativeWindowHostLoopInterruptibleDeadlineWaiter[\s\S]*for NativeWindowHostLoopWindowsWaitBackend<Api>[\s\S]*fn wait_for_host_event[\s\S]*msg_wait_for_message_raw\(\)[\s\S]*native_window_host_loop_windows_host_event_from_message_status/);
    assert.match(nativeWindowPlatformWaitBackendKind, /fn wait_until_deadline_or_host_event[\s\S]*native_window_host_loop_windows_deadline_plan[\s\S]*set_waitable_timer_relative_100ns[\s\S]*msg_wait_for_timer_or_message_raw[\s\S]*native_window_host_loop_windows_wait_wake_from_timer_or_message_status/);
    assert.match(nativeWindowPlatformWaitBackendKind, /build_native_window_host_loop_windows_wait_backend_from_selection[\s\S]*NativeWindowHostLoopPlatformKind::Windows[\s\S]*WindowsWaitableTimerMessageWait[\s\S]*WaitBackendFailed/);
    assert.match(nativeWindowPlatformWaitBackendKind, /#\[cfg\(target_os = "windows"\)\][\s\S]*pub struct NativeWindowHostLoopWindowsWaitSysApi/);
    assert.match(nativeWindowPlatformWaitBackendKind, /#\[cfg\(target_os = "windows"\)\][\s\S]*CreateWaitableTimerW[\s\S]*SetWaitableTimer[\s\S]*MsgWaitForMultipleObjects[\s\S]*CloseHandle[\s\S]*GetLastError/);
    assert.match(nativeWindowPlatformWaitBackendKind, /#\[cfg\(target_os = "windows"\)\][\s\S]*pub fn native_window_run_loop_platform_wait_backend_from_config\([\s\S]*config: NativeWindowRunLoopConfig[\s\S]*native_window_run_loop_platform_wait_backend_config\(config\)[\s\S]*native_window_host_loop_platform_wait_backend_from_selection\(platform_wait_config\.selection\(\)\)/);
    assert.doesNotMatch(nativeWindowPlatformWaitBackendKind, /pub fn native_window_run_loop_platform_wait_backend_from_config<Host[\s\S]*host: Host/);
    assert.doesNotMatch(nativeWindowPlatformWaitBackendKind, /std::env|env::var|env::consts|from_str|parse::<|stringify|to_string|format!|HeadlessScripted\s*\)|=>\s*Ok\(\s*NativeWindowHostLoopPlatformWaitBackendKind::HeadlessScripted|Minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|execute_native_window_host_loop_interruptible_deadline_wait_with_adapter|execute_native_window_host_loop_wait_with_owner|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub struct NativeWindowHostLoopMacosRunLoopTimerHandle\s*\{[\s\S]*raw_handle: isize/);
    assert.doesNotMatch(nativeWindowMacosRunLoopTimerBackend, /pub raw_handle:|pub fn raw_handle|pub fn handle\(/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub const NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED: u32 = 1/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub const NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_HOST_EVENT_READY: u32 = 2/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub enum NativeWindowHostLoopMacosRunLoopWake\s*\{[\s\S]*TimerFired,[\s\S]*HostEventReady/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub trait NativeWindowHostLoopMacosRunLoopTimerRawApi\s*\{[\s\S]*create_run_loop_timer_raw[\s\S]*schedule_run_loop_timer_relative_nanos[\s\S]*run_loop_wait_for_timer_or_event_raw[\s\S]*run_loop_wait_for_event_raw[\s\S]*invalidate_run_loop_timer_raw[\s\S]*last_error_code/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /native_window_host_loop_macos_run_loop_deadline_plan[\s\S]*deadline_nanos <= now_nanos[\s\S]*AlreadyReached[\s\S]*checked_sub\(now_nanos\)[\s\S]*RelativeNanos/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /native_window_host_loop_macos_run_loop_wake_from_timer_or_event_status[\s\S]*STATUS_TIMER_FIRED[\s\S]*TimerFired[\s\S]*STATUS_HOST_EVENT_READY[\s\S]*HostEventReady[\s\S]*STATUS_FAILED[\s\S]*RunLoopWaitFailed[\s\S]*UnexpectedRunLoopStatus/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /native_window_host_loop_macos_run_loop_host_event_from_status[\s\S]*STATUS_HOST_EVENT_READY[\s\S]*Ok\(\(\)\)[\s\S]*STATUS_FAILED[\s\S]*RunLoopWaitFailed[\s\S]*UnexpectedRunLoopStatus/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub struct NativeWindowHostLoopMacosRunLoopTimerBackend<[\s\S]*origin: std::time::Instant,[\s\S]*api: Api,[\s\S]*handle: Option<NativeWindowHostLoopMacosRunLoopTimerHandle>/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub fn invalidate_handle_if_open\(&mut self\) -> bool[\s\S]*self\.handle\.take\(\)[\s\S]*invalidate_run_loop_timer_raw/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub fn wait_for_host_event[\s\S]*run_loop_wait_for_event_raw\(\)[\s\S]*native_window_host_loop_macos_run_loop_host_event_from_status/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /pub fn wait_until_deadline_or_host_event[\s\S]*native_window_host_loop_macos_run_loop_deadline_plan[\s\S]*schedule_run_loop_timer_relative_nanos[\s\S]*run_loop_wait_for_timer_or_event_raw[\s\S]*native_window_host_loop_macos_run_loop_wake_from_timer_or_event_status/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /impl<Api> Drop for NativeWindowHostLoopMacosRunLoopTimerBackend<Api>[\s\S]*self\.invalidate_handle_if_open\(\)/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /build_native_window_host_loop_macos_run_loop_timer_backend_from_selection[\s\S]*validate_native_window_host_loop_platform_wait_backend_selection_for_platform[\s\S]*NativeWindowHostLoopPlatformKind::Macos[\s\S]*NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer[\s\S]*RunLoopTimerBackendFailed/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /impl<Api> NativeWindowHostLoopDeadlineTimerClock[\s\S]*for NativeWindowHostLoopMacosRunLoopTimerBackend<Api>[\s\S]*type Error = NativeWindowHostLoopMacosRunLoopTimerBackendError[\s\S]*self\.elapsed_nanos\(\)/);
    assert.match(nativeWindowMacosRunLoopTimerBackend, /impl<Api> NativeWindowHostLoopInterruptibleDeadlineWaiter[\s\S]*for NativeWindowHostLoopMacosRunLoopTimerBackend<Api>[\s\S]*wait_for_host_event[\s\S]*NativeWindowHostLoopMacosRunLoopTimerBackend::wait_for_host_event[\s\S]*wait_until_deadline_or_host_event[\s\S]*TimerFired[\s\S]*NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached[\s\S]*HostEventReady[\s\S]*NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady/);
    assert.doesNotMatch(nativeWindowMacosRunLoopTimerBackend, /NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer|build_native_window_host_loop_platform_wait_backend_from_selection_with_macos_api|#\[cfg\(target_os = "macos"\)\]|CoreFoundation|CFRunLoop|AppKit|minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|saturating|clamp/i);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub struct NativeWindowHostLoopLinuxSelectorFd\s*\{[\s\S]*raw_fd: i32/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub struct NativeWindowHostLoopLinuxTimerFd\s*\{[\s\S]*raw_fd: i32/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub struct NativeWindowHostLoopLinuxHostEventFd\s*\{[\s\S]*raw_fd: i32/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub struct NativeWindowHostLoopLinuxWindowEventSourceFd\s*\{[\s\S]*raw_fd: i32/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub struct NativeWindowHostLoopLinuxHostEventSignalFd\s*\{[\s\S]*raw_fd: i32/);
    assert.doesNotMatch(nativeWindowLinuxSelectorTimerFdBackend, /pub raw_fd:|pub fn raw_fd|pub fn fd\(/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub const NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED: u32 = 1/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub const NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY: u32 = 2/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub const NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_WINDOW_EVENT_SOURCE_READY: u32 = 3/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub struct NativeWindowHostLoopLinuxTimerFdTimespec\s*\{[\s\S]*seconds: i64,[\s\S]*nanoseconds: i64/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub enum NativeWindowHostLoopLinuxSelectorTimerFdWake\s*\{[\s\S]*TimerFired,[\s\S]*HostEventReady,[\s\S]*WindowEventSourceReady/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub trait NativeWindowHostLoopLinuxSelectorTimerFdRawApi\s*\{[\s\S]*create_selector_raw[\s\S]*create_timer_fd_raw[\s\S]*create_host_event_fd_raw[\s\S]*register_timer_fd_raw[\s\S]*register_host_event_fd_raw[\s\S]*register_window_event_source_fd_raw[\s\S]*window_event_source: &NativeWindowHostLoopLinuxWindowEventSourceFd[\s\S]*unregister_window_event_source_fd_raw[\s\S]*window_event_source: &NativeWindowHostLoopLinuxWindowEventSourceFd[\s\S]*signal_host_event_fd_raw[\s\S]*host_event: &NativeWindowHostLoopLinuxHostEventFd[\s\S]*arm_timer_fd_relative_timespec[\s\S]*selector_wait_for_timer_or_event_raw[\s\S]*host_event: &NativeWindowHostLoopLinuxHostEventFd[\s\S]*selector_wait_for_timer_host_or_window_event_raw[\s\S]*window_event_source: &NativeWindowHostLoopLinuxWindowEventSourceFd[\s\S]*selector_wait_for_event_raw[\s\S]*host_event: &NativeWindowHostLoopLinuxHostEventFd[\s\S]*close_selector_raw[\s\S]*close_timer_fd_raw[\s\S]*close_host_event_fd_raw[\s\S]*last_error_code/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub trait NativeWindowHostLoopLinuxHostEventSignalRawApi\s*\{[\s\S]*clone_host_event_signal_fd_raw[\s\S]*host_event: &NativeWindowHostLoopLinuxHostEventFd[\s\S]*signal_host_event_signal_fd_raw[\s\S]*signal: &NativeWindowHostLoopLinuxHostEventSignalFd[\s\S]*close_host_event_signal_fd_raw[\s\S]*signal: &NativeWindowHostLoopLinuxHostEventSignalFd[\s\S]*last_error_code/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_selector_fd_from_raw[\s\S]*raw_fd < 0[\s\S]*InvalidSelectorRawFd/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_timer_fd_from_raw[\s\S]*raw_fd < 0[\s\S]*InvalidTimerRawFd/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_host_event_fd_from_raw[\s\S]*raw_fd < 0[\s\S]*InvalidHostEventRawFd/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_window_event_source_fd_from_raw[\s\S]*raw_fd < 0[\s\S]*InvalidWindowEventSourceRawFd/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_host_event_signal_fd_from_raw[\s\S]*raw_fd < 0[\s\S]*InvalidHostEventSignalRawFd/);
    assert.doesNotMatch(nativeWindowLinuxSelectorTimerFdBackend, /raw_fd == 0/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_timer_fd_timespec_from_nanos[\s\S]*delta_nanos \/ 1_000_000_000[\s\S]*delta_nanos % 1_000_000_000[\s\S]*i64::try_from/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_selector_timer_fd_deadline_plan[\s\S]*deadline_nanos <= now_nanos[\s\S]*AlreadyReached[\s\S]*checked_sub\(now_nanos\)[\s\S]*RelativeTimespec/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_selector_timer_fd_wake_from_status[\s\S]*STATUS_TIMER_FIRED[\s\S]*TimerFired[\s\S]*STATUS_HOST_EVENT_READY[\s\S]*HostEventReady[\s\S]*STATUS_WINDOW_EVENT_SOURCE_READY[\s\S]*WindowEventSourceReady[\s\S]*STATUS_FAILED[\s\S]*SelectorWaitFailed[\s\S]*UnexpectedSelectorStatus/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /native_window_host_loop_linux_selector_timer_fd_host_event_from_status[\s\S]*STATUS_HOST_EVENT_READY[\s\S]*Ok\(\(\)\)[\s\S]*STATUS_FAILED[\s\S]*SelectorWaitFailed[\s\S]*UnexpectedSelectorStatus/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub struct NativeWindowHostLoopLinuxSelectorTimerFdBackend<[\s\S]*origin: std::time::Instant,[\s\S]*api: Api,[\s\S]*selector: Option<NativeWindowHostLoopLinuxSelectorFd>,[\s\S]*timer: Option<NativeWindowHostLoopLinuxTimerFd>,[\s\S]*host_event: Option<NativeWindowHostLoopLinuxHostEventFd>,[\s\S]*window_event_source: Option<NativeWindowHostLoopLinuxWindowEventSourceFd>/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn new\(mut api: Api\)[\s\S]*create_selector_raw\(\)[\s\S]*create_timer_fd_raw\(\)[\s\S]*register_timer_fd_raw\(&selector,\s*&timer\)[\s\S]*create_host_event_fd_raw\(\)[\s\S]*register_host_event_fd_raw\(&selector,\s*&host_event\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /CreateTimerFdFailed[\s\S]*close_selector_raw\(&selector\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /RegisterTimerFdFailed[\s\S]*close_timer_fd_raw\(&timer\)[\s\S]*close_selector_raw\(&selector\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /CreateHostEventFdFailed[\s\S]*close_timer_fd_raw\(&timer\)[\s\S]*close_selector_raw\(&selector\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /RegisterHostEventFdFailed[\s\S]*close_host_event_fd_raw\(&host_event\)[\s\S]*close_timer_fd_raw\(&timer\)[\s\S]*close_selector_raw\(&selector\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn register_window_event_source_fd_from_raw[\s\S]*native_window_host_loop_linux_window_event_source_fd_from_raw\(raw_fd\)\?[\s\S]*WindowEventSourceFdAlreadyRegistered[\s\S]*register_window_event_source_fd_raw\(selector,\s*&window_event_source\)[\s\S]*RegisterWindowEventSourceFdFailed[\s\S]*self\.window_event_source = Some\(window_event_source\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn unregister_window_event_source_fd_if_registered[\s\S]*self\.window_event_source\.take\(\)[\s\S]*unregister_window_event_source_fd_raw\(selector,\s*&window_event_source\)[\s\S]*self\.window_event_source = Some\(window_event_source\)[\s\S]*UnregisterWindowEventSourceFdFailed/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn try_close_handles_if_open[\s\S]*unregister_window_event_source_fd_if_registered\(\)\?[\s\S]*self\.host_event\.take\(\)[\s\S]*self\.timer\.take\(\)[\s\S]*self\.selector\.take\(\)[\s\S]*close_host_event_fd_raw[\s\S]*close_timer_fd_raw[\s\S]*close_selector_raw[\s\S]*Ok\(closed\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn close_handles_if_open\(&mut self\) -> bool[\s\S]*self\.try_close_handles_if_open\(\)\.unwrap_or\(false\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn wait_until_deadline_or_host_event[\s\S]*self\.window_event_source\.as_ref\(\)[\s\S]*selector_wait_for_timer_host_or_window_event_raw\([\s\S]*window_event_source[\s\S]*selector_wait_for_timer_or_event_raw\(selector,\s*timer,\s*host_event\)/);
    assert.doesNotMatch(nativeWindowLinuxSelectorTimerFdBackend, /close_window_event_source|close_window_event_source_fd_raw|drain_window_event_source|read_window_event_source|signal_window_event_source/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /SignalHostEventFdFailed\s*\{\s*code: u32\s*\}/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub enum NativeWindowHostLoopLinuxHostEventSignalProducerError\s*\{[\s\S]*InvalidHostEventRawFd\s*\{[\s\S]*raw_fd: i32[\s\S]*InvalidHostEventSignalRawFd\s*\{[\s\S]*raw_fd: i32[\s\S]*CreateHostEventSignalFdFailed\s*\{[\s\S]*code: u32[\s\S]*SignalHostEventSignalFdFailed\s*\{[\s\S]*code: u32/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn create_host_event_signal_producer<[\s\S]*ProducerApi: NativeWindowHostLoopLinuxHostEventSignalRawApi[\s\S]*host_event = self\.host_event\.as_ref\(\)[\s\S]*InvalidHostEventRawFd\s*\{\s*raw_fd: -1,?\s*\}[\s\S]*clone_host_event_signal_fd_raw\(host_event\)[\s\S]*CreateHostEventSignalFdFailed\s*\{[\s\S]*code: producer_api\.last_error_code\(\)[\s\S]*NativeWindowHostLoopLinuxHostEventSignalProducer::new/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn signal_host_event[\s\S]*host_event = self\.host_event\.as_ref\(\)[\s\S]*InvalidHostEventRawFd\s*\{\s*raw_fd: -1,?\s*\}[\s\S]*signal_host_event_fd_raw\(host_event\)[\s\S]*SignalHostEventFdFailed\s*\{[\s\S]*code: self\.api\.last_error_code\(\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub struct NativeWindowHostLoopLinuxHostEventSignalProducer<[\s\S]*api: Api,[\s\S]*signal: Option<NativeWindowHostLoopLinuxHostEventSignalFd>/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /impl<Api> NativeWindowHostLoopLinuxHostEventSignalProducer<Api>[\s\S]*pub fn close_signal_handle_if_open\(&mut self\) -> bool[\s\S]*self\.signal\.take\(\)[\s\S]*close_host_event_signal_fd_raw\(&signal\)[\s\S]*pub fn signal_host_event[\s\S]*signal = self\.signal\.as_ref\(\)[\s\S]*InvalidHostEventSignalRawFd\s*\{\s*raw_fd: -1,?\s*\}[\s\S]*signal_host_event_signal_fd_raw\(signal\)[\s\S]*SignalHostEventSignalFdFailed\s*\{[\s\S]*code: self\.api\.last_error_code\(\)/);
    assert.match(nativeWindowLinuxExternallyWakeableEventSourceOwner, /pub enum NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwnerBuildError<BackendApi>[\s\S]*BackendClosed\s*\{[\s\S]*backend: NativeWindowHostLoopLinuxSelectorTimerFdBackend<BackendApi>[\s\S]*HostEventSignalProducerFailed\s*\{[\s\S]*backend: NativeWindowHostLoopLinuxSelectorTimerFdBackend<BackendApi>,[\s\S]*error: NativeWindowHostLoopLinuxHostEventSignalProducerError/);
    assert.match(nativeWindowLinuxExternallyWakeableEventSourceOwner, /pub struct NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner<BackendApi,\s*ProducerApi>[\s\S]*backend: NativeWindowHostLoopLinuxSelectorTimerFdBackend<BackendApi>,[\s\S]*producer: NativeWindowHostLoopLinuxHostEventSignalProducer<ProducerApi>/);
    assert.match(nativeWindowLinuxExternallyWakeableEventSourceOwner, /pub fn native_window_host_loop_linux_externally_wakeable_event_source_owner_from_backend<[\s\S]*backend: NativeWindowHostLoopLinuxSelectorTimerFdBackend<BackendApi>,[\s\S]*producer_api: ProducerApi[\s\S]*if !backend\.are_handles_open\(\)[\s\S]*BackendClosed\s*\{[\s\S]*backend,[\s\S]*create_host_event_signal_producer\(producer_api\)[\s\S]*HostEventSignalProducerFailed\s*\{[\s\S]*backend,[\s\S]*error/);
    assert.match(nativeWindowLinuxExternallyWakeableEventSourceOwner, /pub fn signal_host_event\([\s\S]*&mut self,[\s\S]*Result<\(\), NativeWindowHostLoopLinuxHostEventSignalProducerError>[\s\S]*self\.producer\.signal_host_event\(\)/);
    assert.doesNotMatch(nativeWindowLinuxExternallyWakeableEventSourceOwner, /pub fn new\(/);
    assert.doesNotMatch(nativeWindowLinuxExternallyWakeableEventSourceOwner, /pub fn into_parts/);
    assert.doesNotMatch(nativeWindowLinuxExternallyWakeableEventSourceOwner, /self\.backend\.signal_host_event|NativeWindowHostLoopWaitOutcome|HostEventReady|FrameIntervalTimerFired|TimerFired|SchedulerResume|run_linux_platform_wait_window_loop|run_windows_platform_wait_window_loop|run_minifb_window_loop|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(nativeWindowLinuxExternallyWakeableRunLoopHost, /pub struct NativeWindowHostLoopLinuxExternallyWakeableEventSourceWaitAdapter<[\s\S]*next_raw_id: u32,[\s\S]*owner: NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner<BackendApi, ProducerApi>/);
    assert.match(nativeWindowLinuxExternallyWakeableRunLoopHost, /pub fn execute_native_window_host_loop_linux_externally_wakeable_event_source_wait_with_adapter<[\s\S]*adapter: &mut NativeWindowHostLoopLinuxExternallyWakeableEventSourceWaitAdapter[\s\S]*owner[\s\S]*backend_mut\(\)[\s\S]*wait_for_host_event[\s\S]*owner[\s\S]*backend_mut\(\)[\s\S]*wait_until_deadline_or_host_event/);
    assert.match(nativeWindowLinuxExternallyWakeableRunLoopHost, /pub struct NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost<[\s\S]*host: Host,[\s\S]*wait_adapter:[\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceWaitAdapter<BackendApi, ProducerApi>/);
    assert.match(nativeWindowLinuxExternallyWakeableRunLoopHost, /pub fn native_window_host_loop_linux_externally_wakeable_event_source_run_loop_host_from_owner<[\s\S]*host: Host,[\s\S]*owner: NativeWindowHostLoopLinuxExternallyWakeableEventSourceOwner<BackendApi, ProducerApi>[\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost::new\(host, owner\)/);
    assert.match(nativeWindowLinuxExternallyWakeableRunLoopHost, /impl<Host, BackendApi, ProducerApi> NativeWindowRunLoopHost[\s\S]*NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost[\s\S]*poll_event_snapshot[\s\S]*self\.host\.poll_event_snapshot\(input\)[\s\S]*present_frame[\s\S]*self\.host\.present_frame\(frame\)[\s\S]*wait_after_budget_exhausted[\s\S]*execute_native_window_host_loop_linux_externally_wakeable_event_source_wait_with_adapter/);
    assert.match(nativeWindowLinuxExternallyWakeableRunLoopHost, /pub fn signal_host_event\([\s\S]*self\.owner\.signal_host_event\(\)[\s\S]*pub struct NativeWindowHostLoopLinuxExternallyWakeableEventSourceRunLoopHost[\s\S]*pub fn signal_host_event\([\s\S]*self\.wait_adapter\.signal_host_event\(\)/);
    assert.doesNotMatch(nativeWindowLinuxExternallyWakeableRunLoopHost, /native_window_host_loop_platform_wait_run_loop_host_from_backend|NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd|into_backend|backend: NativeWindowHostLoopLinuxSelectorTimerFdBackend<BackendApi>|self\.wait_adapter\.owner\.backend_mut\(\)\.signal_host_event|run_linux_platform_wait_window_loop|run_windows_platform_wait_window_loop|run_minifb_window_loop|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|libc|epoll|poll\(|select\(|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(libSource, /native_window_linux_externally_wakeable_run_loop_host_waits_without_splitting_owner/);
    assert.match(libSource, /native_window_linux_externally_wakeable_run_loop_host_keeps_producer_after_timer_wait/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_registers_window_event_source_without_owning_fd/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_rejects_window_event_source_before_raw_calls/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_preserves_window_event_source_register_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_rejects_duplicate_window_event_source_before_raw_call/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_unregister_failure_keeps_handles_open/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_wait_until_deadline_maps_window_event_source_ready/);
    assert.match(libSource, /native_window_linux_externally_wakeable_run_loop_host_maps_window_event_source_to_event_pump/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /impl<Api> Drop for NativeWindowHostLoopLinuxHostEventSignalProducer<Api>[\s\S]*self\.close_signal_handle_if_open\(\)/);
    assert.match(minifbNativeWindowLinuxObservedInputSignalBridge, /pub struct MinifbNativeWindowLinuxHostEventSignalCallbackState<[\s\S]*producer: NativeWindowHostLoopLinuxHostEventSignalProducer<Api>,[\s\S]*first_error: Option<NativeWindowHostLoopLinuxHostEventSignalProducerError>/);
    assert.match(minifbNativeWindowLinuxObservedInputSignalBridge, /signal_observed_input[\s\S]*if self\.first_error\.is_some\(\)[\s\S]*return;[\s\S]*self\.producer\.signal_host_event\(\)[\s\S]*self\.first_error = Some\(error\)/);
    assert.match(minifbNativeWindowLinuxObservedInputSignalBridge, /impl<Api> NativeWindowHostEventSignalErrorState[\s\S]*Rc<std::cell::RefCell<MinifbNativeWindowLinuxHostEventSignalCallbackState<Api>>>[\s\S]*borrow_mut\(\)\.take_first_error\(\)/);
    assert.match(minifbNativeWindowLinuxObservedInputSignalBridge, /pub struct MinifbNativeWindowLinuxHostEventSignalInputCallback<[\s\S]*state: std::rc::Rc/);
    assert.match(minifbNativeWindowLinuxObservedInputSignalBridge, /impl<Api> minifb::InputCallback[\s\S]*for MinifbNativeWindowLinuxHostEventSignalInputCallback<Api>[\s\S]*fn add_char[\s\S]*signal_observed_input\(\)[\s\S]*fn set_key_state[\s\S]*signal_observed_input\(\)/);
    assert.doesNotMatch(minifbNativeWindowLinuxObservedInputSignalBridge, /NativeWindowHostLoopWaitOutcome|HostEventReady|FrameIntervalTimerFired|TimerFired|run_linux_platform_wait_window_loop|set_target_fps|window\.update\(|update_with_buffer|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic/i);
    assert.match(libSource, /native_window_linux_minifb_input_callback_signals_observed_input/);
    assert.match(libSource, /native_window_linux_minifb_input_callback_records_first_signal_error/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn wait_for_host_event[\s\S]*host_event = self\.host_event\.as_ref\(\)[\s\S]*selector_wait_for_event_raw\(selector,\s*host_event\)[\s\S]*native_window_host_loop_linux_selector_timer_fd_host_event_from_status/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /pub fn wait_until_deadline_or_host_event[\s\S]*native_window_host_loop_linux_selector_timer_fd_deadline_plan[\s\S]*host_event = self\.host_event\.as_ref\(\)[\s\S]*arm_timer_fd_relative_timespec\(timer,\s*timespec\)[\s\S]*selector_wait_for_timer_or_event_raw\(selector,\s*timer,\s*host_event\)[\s\S]*native_window_host_loop_linux_selector_timer_fd_wake_from_status/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /impl<Api> Drop for NativeWindowHostLoopLinuxSelectorTimerFdBackend<Api>[\s\S]*self\.close_handles_if_open\(\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection[\s\S]*validate_native_window_host_loop_platform_wait_backend_selection_for_platform[\s\S]*NativeWindowHostLoopPlatformKind::Linux[\s\S]*NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd[\s\S]*SelectorTimerFdBackendFailed/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /impl<Api> NativeWindowHostLoopDeadlineTimerClock[\s\S]*for NativeWindowHostLoopLinuxSelectorTimerFdBackend<Api>[\s\S]*type Error = NativeWindowHostLoopLinuxSelectorTimerFdBackendError[\s\S]*self\.elapsed_nanos\(\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdBackend, /impl<Api> NativeWindowHostLoopInterruptibleDeadlineWaiter[\s\S]*for NativeWindowHostLoopLinuxSelectorTimerFdBackend<Api>[\s\S]*wait_for_host_event[\s\S]*NativeWindowHostLoopLinuxSelectorTimerFdBackend::wait_for_host_event[\s\S]*wait_until_deadline_or_host_event[\s\S]*TimerFired[\s\S]*NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached[\s\S]*HostEventReady[\s\S]*NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady/);
    assert.doesNotMatch(nativeWindowLinuxSelectorTimerFdBackend, /NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd|build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api|#\[cfg\(target_os = "linux"\)\]|libc|nix::|epoll|poll\(|select\(|timerfd_|minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic|saturating|clamp/i);
    assert.match(nativeCargoToml, /\[target\.'cfg\(target_os = "linux"\)'\.dependencies\][\s\S]*libc = "0\.2"/);
    assert.match(libSource, /#\[cfg\(target_os = "linux"\)\]\s*#\[derive\(Debug, Default\)\]\s*pub struct NativeWindowHostLoopLinuxSelectorTimerFdSysApi/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /last_error_code: u32/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /impl NativeWindowHostLoopLinuxSelectorTimerFdRawApi[\s\S]*for NativeWindowHostLoopLinuxSelectorTimerFdSysApi/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /epoll_create1\(libc::EPOLL_CLOEXEC\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /timerfd_create\([\s\S]*libc::CLOCK_MONOTONIC,[\s\S]*libc::TFD_CLOEXEC \| libc::TFD_NONBLOCK/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /eventfd\(0,\s*libc::EFD_CLOEXEC \| libc::EFD_NONBLOCK\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /epoll_ctl\([\s\S]*libc::EPOLL_CTL_ADD,[\s\S]*native_window_host_loop_linux_timer_fd_raw\(timer\),[\s\S]*&mut event/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /register_host_event_fd_raw[\s\S]*epoll_ctl\([\s\S]*libc::EPOLL_CTL_ADD,[\s\S]*native_window_host_loop_linux_host_event_fd_raw\(host_event\),[\s\S]*&mut event/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /write_eventfd_counter_raw[\s\S]*let counter = 1_u64[\s\S]*libc::write\([\s\S]*raw_fd,[\s\S]*std::mem::size_of::<u64>\(\)[\s\S]*write_result != std::mem::size_of::<u64>\(\) as libc::ssize_t[\s\S]*libc::EIO/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /signal_host_event_fd_raw[\s\S]*write_eventfd_counter_raw\(native_window_host_loop_linux_host_event_fd_raw\(host_event\)\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /impl NativeWindowHostLoopLinuxHostEventSignalRawApi[\s\S]*for NativeWindowHostLoopLinuxSelectorTimerFdSysApi[\s\S]*clone_host_event_signal_fd_raw[\s\S]*libc::fcntl\([\s\S]*native_window_host_loop_linux_host_event_fd_raw\(host_event\),[\s\S]*libc::F_DUPFD_CLOEXEC,[\s\S]*0,[\s\S]*signal_host_event_signal_fd_raw[\s\S]*write_eventfd_counter_raw[\s\S]*native_window_host_loop_linux_host_event_signal_fd_raw[\s\S]*close_host_event_signal_fd_raw[\s\S]*libc::close[\s\S]*native_window_host_loop_linux_host_event_signal_fd_raw[\s\S]*signal/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /timerfd_settime\([\s\S]*native_window_host_loop_linux_timer_fd_raw\(timer\),[\s\S]*0,[\s\S]*&timer_spec,[\s\S]*std::ptr::null_mut\(\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /epoll_wait\([\s\S]*native_window_host_loop_linux_selector_fd_raw\(selector\),[\s\S]*&mut event,[\s\S]*1,[\s\S]*-1/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /drain_timer_fd[\s\S]*native_window_host_loop_linux_timer_fd_raw\(timer\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /drain_host_event_fd[\s\S]*native_window_host_loop_linux_host_event_fd_raw\(host_event\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /read_result != std::mem::size_of::<u64>\(\) as libc::ssize_t[\s\S]*libc::EIO/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /counter == 0[\s\S]*libc::EIO/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /if event\.u64 == native_window_host_loop_linux_timer_fd_raw\(timer\) as u64[\s\S]*NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED[\s\S]*if event\.u64 == native_window_host_loop_linux_host_event_fd_raw\(host_event\) as u64[\s\S]*NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY[\s\S]*libc::EINVAL/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /selector_wait_for_timer_host_or_window_event_raw[\s\S]*window_event_source: &NativeWindowHostLoopLinuxWindowEventSourceFd[\s\S]*if event\.u64[\s\S]*native_window_host_loop_linux_window_event_source_fd_raw\(window_event_source\) as u64[\s\S]*NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_WINDOW_EVENT_SOURCE_READY/);
    assert.doesNotMatch(textSliceBetween(nativeWindowLinuxSelectorTimerFdSysApi, "fn selector_wait_for_timer_host_or_window_event_raw", "fn selector_wait_for_event_raw"), /drain_window_event_source|read_window_event_source|signal_window_event_source|close_window_event_source|libc::read\([\s\S]*window_event_source|libc::close\([\s\S]*window_event_source/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /selector_wait_for_event_raw[\s\S]*if event\.u64 != native_window_host_loop_linux_host_event_fd_raw\(host_event\) as u64[\s\S]*libc::EINVAL[\s\S]*drain_host_event_fd\(host_event\)[\s\S]*NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /libc::close\(native_window_host_loop_linux_selector_fd_raw\(selector\)\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /libc::close\(native_window_host_loop_linux_timer_fd_raw\(timer\)\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /libc::close\(native_window_host_loop_linux_host_event_fd_raw\(host_event\)\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /libc::close[\s\S]*native_window_host_loop_linux_host_event_signal_fd_raw[\s\S]*signal/);
    assert.doesNotMatch(textSliceBetween(nativeWindowLinuxSelectorTimerFdSysApi, "fn close_selector_raw", "fn close_timer_fd_raw"), /self\.clear_error\(\)/);
    assert.doesNotMatch(textSliceBetween(nativeWindowLinuxSelectorTimerFdSysApi, "fn close_timer_fd_raw", "fn close_host_event_fd_raw"), /self\.clear_error\(\)/);
    assert.doesNotMatch(textSliceBetween(nativeWindowLinuxSelectorTimerFdSysApi, "fn close_host_event_fd_raw", "fn last_error_code"), /self\.clear_error\(\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /native_window_host_loop_linux_selector_timer_fd_backend_from_selection[\s\S]*NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new\(\)/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /#\[cfg\(target_os = "linux"\)\][\s\S]*pub fn native_window_host_loop_platform_wait_backend_from_selection\([\s\S]*selection: NativeWindowHostLoopPlatformWaitBackendSelection,[\s\S]*event_source_capability: NativeWindowHostLoopLinuxEventSourceCapability,[\s\S]*NativeWindowHostLoopLinuxOnlyPlatformWaitBackend<[\s\S]*NativeWindowHostLoopLinuxSelectorTimerFdSysApi[\s\S]*>[\s\S]*build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api\([\s\S]*selection,[\s\S]*NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new\(\),[\s\S]*event_source_capability/);
    assert.match(nativeWindowLinuxSelectorTimerFdSysApi, /#\[cfg\(target_os = "linux"\)\][\s\S]*pub fn native_window_run_loop_platform_wait_backend_from_config\([\s\S]*config: NativeWindowRunLoopConfig[\s\S]*native_window_run_loop_platform_wait_backend_config\(config\)[\s\S]*native_window_run_loop_linux_event_source_capability_from_platform_wait_config\([\s\S]*platform_wait_config[\s\S]*native_window_host_loop_platform_wait_backend_from_selection\([\s\S]*platform_wait_config\.selection\(\),[\s\S]*event_source_capability/);
    assert.doesNotMatch(nativeWindowLinuxSelectorTimerFdSysApi, /native_window_run_loop_platform_wait_backend_from_config[\s\S]*NativeWindowHostLoopLinuxEventSourceCapability::ExternallyWakeableEventSource/);
    assert.doesNotMatch(nativeWindowLinuxSelectorTimerFdSysApi, /run_minifb|run_linux_platform_wait_window_loop|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|synthetic|saturating|clamp/i);
    assert.doesNotMatch(libSource, /run_linux_platform_wait_window_loop|LinuxPlatformWaitHostLoopFailed|set_target_fps\(0\)/);
    assert.doesNotMatch(mainSource, /run_linux_platform_wait_window_loop|LinuxPlatformWaitHostLoopFailed|target_os = "linux"|ExternallyWakeableEventSource|ObservedInputOnly|set_target_fps\(0\)/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_handles_accept_zero_and_reject_negative_raw_fds/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_timespec_uses_checked_seconds_and_nanoseconds/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_deadline_plan_uses_already_reached_or_timespec/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_status_maps_timer_host_event_and_failures/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_host_event_status_rejects_timer_fired/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_rejects_selector_creation_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_rejects_timer_fd_creation_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_rejects_register_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_wait_for_host_event_uses_event_only_wait/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_wait_until_deadline_arms_timespec_and_maps_timer/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_wait_until_deadline_maps_host_ready/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_wait_until_deadline_rejects_arm_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_wait_until_deadline_already_reached_avoids_raw_wait/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_rejects_host_event_creation_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_rejects_host_event_register_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_signal_host_event_writes_event_fd/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_signal_host_event_preserves_raw_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_signal_host_event_rejects_closed_backend/);
    assert.match(libSource, /native_window_linux_host_event_signal_producer_duplicates_and_signals_handle/);
    assert.match(libSource, /native_window_linux_host_event_signal_producer_preserves_clone_failure/);
    assert.match(libSource, /native_window_linux_host_event_signal_producer_rejects_closed_backend/);
    assert.match(libSource, /native_window_linux_host_event_signal_producer_preserves_signal_failure/);
    assert.match(libSource, /native_window_linux_host_event_signal_producer_closes_signal_handle_once/);
    assert.match(libSource, /native_window_linux_externally_wakeable_owner_keeps_backend_and_producer/);
    assert.match(libSource, /native_window_linux_externally_wakeable_owner_signals_through_producer_only/);
    assert.match(libSource, /native_window_linux_externally_wakeable_owner_preserves_backend_on_producer_failure/);
    assert.match(libSource, /native_window_linux_externally_wakeable_owner_rejects_closed_backend/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_closes_selector_timer_and_host_event_once/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_builder_requires_validated_linux_selection/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_backend_builder_preserves_raw_api_failure/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_wait_trait_maps_timer_to_deadline_reached/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_wait_trait_keeps_host_ready_non_timer/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_wait_trait_rejects_timer_status_for_event_wait/);
    assert.match(libSource, /native_window_linux_selector_timer_fd_wait_trait_preserves_arm_error/);
    assert.match(libSource, /native_window_platform_wait_backend_validation_accepts_matching_backend/);
    assert.match(libSource, /native_window_platform_wait_backend_validation_rejects_all_real_platform_mismatches/);
    assert.match(libSource, /native_window_platform_wait_backend_validation_rejects_unsupported_platform/);
    assert.match(libSource, /native_window_platform_wait_backend_default_maps_real_platforms_without_headless_fallback/);
    assert.match(libSource, /native_window_platform_wait_backend_default_rejects_unsupported_platform/);
    assert.match(libSource, /native_window_current_platform_wait_backend_default_matches_cfg_platform/);
    assert.match(libSource, /native_window_platform_wait_backend_selection_carries_validated_platform_and_backend/);
    assert.match(libSource, /native_window_platform_wait_backend_selection_rejects_headless_scripted_for_native/);
    assert.match(libSource, /native_window_platform_wait_backend_selection_rejects_unsupported_platform/);
    assert.match(libSource, /native_window_platform_wait_backend_default_selection_matches_supported_platforms/);
    assert.match(libSource, /native_window_platform_wait_backend_builder_preserves_selection_as_unavailable/);
    assert.match(libSource, /native_window_platform_wait_backend_builder_returns_support_failure_before_unavailable/);
    assert.match(libSource, /native_window_platform_wait_backend_with_windows_api_builds_windows_backend/);
    assert.match(libSource, /native_window_platform_wait_backend_with_windows_api_preserves_unavailable_real_backends/);
    assert.match(libSource, /native_window_platform_wait_backend_with_windows_api_preserves_support_failure/);
    assert.match(libSource, /native_window_platform_wait_backend_with_windows_api_preserves_windows_failure/);
    assert.match(libSource, /native_window_platform_wait_backend_with_linux_api_builds_linux_backend/);
    assert.match(libSource, /native_window_platform_wait_backend_with_linux_api_preserves_unavailable_real_backends/);
    assert.match(libSource, /native_window_platform_wait_backend_with_linux_api_preserves_support_failure_before_raw_calls/);
    assert.match(libSource, /native_window_platform_wait_backend_with_linux_api_preserves_linux_failure/);
    assert.match(libSource, /native_window_platform_wait_backend_with_raw_apis_builds_selected_macos_backend/);
    assert.match(libSource, /native_window_platform_wait_backend_with_raw_apis_builds_selected_linux_backend/);
    assert.match(libSource, /native_window_platform_wait_backend_with_raw_apis_preserves_macos_failure/);
    assert.match(libSource, /native_window_platform_wait_backend_with_raw_apis_preserves_linux_failure/);
    assert.match(libSource, /native_window_platform_wait_backend_with_raw_apis_support_failure_precedes_raw_create/);
    assert.match(libSource, /native_window_platform_wait_run_loop_host_wraps_existing_backend_infallibly/);
    assert.match(libSource, /native_window_platform_wait_run_loop_host_keeps_host_ready_outcome_non_timer/);
    assert.match(libSource, /native_window_platform_wait_run_loop_host_wraps_macos_backend/);
    assert.match(libSource, /native_window_platform_wait_run_loop_host_wraps_linux_backend/);
    assert.match(libSource, /native_window_run_loop_platform_wait_config_extracts_typed_config_and_selection/);
    assert.match(libSource, /native_window_run_loop_platform_wait_config_requires_explicit_linux_event_source/);
    assert.match(libSource, /native_window_platform_wait_runner_support_accepts_windows_only/);
    assert.match(libSource, /native_window_platform_wait_runner_support_rejects_non_platform_config/);
    assert.match(libSource, /native_window_platform_wait_runner_support_rejects_macos_until_runner_exists/);
    assert.match(libSource, /native_window_platform_wait_runner_support_rejects_unsupported_as_unavailable/);
    assert.match(libSource, /native_window_platform_wait_runner_support_rejects_linux_missing_event_source/);
    assert.match(libSource, /native_window_platform_wait_runner_support_rejects_linux_observed_input_only/);
    assert.match(libSource, /native_window_platform_wait_runner_support_rejects_linux_externally_wakeable_without_raw_fd/);
    assert.match(libSource, /native_window_platform_wait_runner_support_rejects_linux_externally_wakeable_with_raw_fd_until_event_parsing_exists/);
    assert.match(libSource, /native_window_platform_wait_runner_support_rejects_cross_platform_selection/);
    assert.match(libSource, /native_window_windows_platform_wait_runner_rejects_non_platform_config_before_backend/);
    assert.match(libSource, /native_window_windows_platform_wait_runner_rejects_cross_platform_config_before_backend/);
    assert.match(libSource, /native_window_minifb_run_loop_backend_validation_rejects_platform_wait_backend/);
    assert.match(libSource, /native_window_macos_run_loop_timer_handle_rejects_null_and_invalid_raw_handles/);
    assert.match(libSource, /native_window_macos_run_loop_deadline_plan_uses_checked_relative_nanos/);
    assert.match(libSource, /native_window_macos_run_loop_status_maps_timer_event_and_failures/);
    assert.match(libSource, /native_window_macos_run_loop_backend_wait_for_host_event_uses_event_only_wait/);
    assert.match(libSource, /native_window_macos_run_loop_backend_wait_until_deadline_schedules_relative_timer/);
    assert.match(libSource, /native_window_macos_run_loop_backend_invalidates_handle_once/);
    assert.match(libSource, /native_window_macos_run_loop_backend_builder_requires_validated_macos_selection/);
    assert.match(libSource, /native_window_macos_run_loop_backend_builder_preserves_raw_api_failure/);
    assert.match(libSource, /native_window_macos_run_loop_wait_trait_maps_timer_to_deadline_reached/);
    assert.match(libSource, /native_window_macos_run_loop_wait_trait_keeps_host_ready_non_timer/);
    assert.match(libSource, /native_window_macos_run_loop_wait_trait_rejects_timer_status_for_event_wait/);
    assert.match(libSource, /native_window_macos_run_loop_wait_trait_preserves_schedule_error/);
    assert.match(libSource, /native_window_windows_wait_handle_rejects_null_and_invalid_raw_handles/);
    assert.match(libSource, /native_window_windows_deadline_plan_uses_already_reached_or_rounded_relative_100ns/);
    assert.match(libSource, /native_window_windows_wait_status_maps_timer_message_and_failures/);
    assert.match(libSource, /native_window_windows_backend_wait_for_host_event_uses_message_only_wait/);
    assert.match(libSource, /native_window_windows_backend_wait_until_deadline_sets_timer_and_maps_deadline/);
    assert.match(libSource, /native_window_windows_backend_close_handle_once/);
    assert.match(libSource, /native_window_windows_backend_builder_requires_validated_windows_selection/);
    assert.match(libSource, /native_window_host_owned_deadline_wait_host_delegates_non_wait_operations/);
    assert.match(libSource, /native_window_host_owned_deadline_wait_host_uses_owner_for_host_event_wait/);
    assert.match(libSource, /native_window_host_owned_deadline_wait_host_uses_owner_for_frame_interval_wait/);
    assert.match(libSource, /native_window_host_owned_deadline_wait_host_preserves_owner_wait_error/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_host_delegates_non_wait_operations/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_host_uses_adapter_for_host_event_wait/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_host_returns_timer_fired_on_deadline/);
    assert.match(libSource, /native_window_single_owner_interruptible_deadline_wait_host_preserves_adapter_wait_error/);
    assert.match(nativeWindowEventQueueStatusAdapter, /wait_for_host_event_raw_status\(window_size,\s*size_changed\)[\s\S]*NativeWindowHostLoopEventQueueStatusAdapterError::AdapterFailed/);
    assert.match(nativeWindowEventQueueStatusAdapter, /raw_status != NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY[\s\S]*NativeWindowHostLoopEventQueueStatusAdapterError::InvalidRawStatus/);
    assert.match(nativeWindowEventQueueStatusAdapter, /impl<Adapter> NativeWindowHostLoopEventQueueWaiter[\s\S]*NativeWindowHostLoopEventQueueStatusWaiter<Adapter>/);
    assert.match(nativeWindowEventQueueStatusAdapter, /wait_native_window_host_loop_event_queue_raw_status_with_adapter\([\s\S]*&mut self\.adapter,[\s\S]*window_size,[\s\S]*size_changed/);
    assert.doesNotMatch(nativeWindowEventQueueStatusAdapter, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|register_timer_nanos|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowMessagePumpStatusAdapter, /impl<Adapter> NativeWindowHostLoopEventQueueStatusAdapter[\s\S]*NativeWindowHostLoopMessagePumpStatusAdapter<Adapter>/);
    assert.match(nativeWindowMessagePumpStatusAdapter, /pump_host_messages\(window_size,\s*size_changed\)[\s\S]*NativeWindowHostLoopMessagePumpStatusAdapterError::PumpFailed/);
    assert.match(nativeWindowMessagePumpStatusAdapter, /Ok\(NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY\)/);
    assert.doesNotMatch(nativeWindowMessagePumpStatusAdapter, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|register_timer_nanos|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowHostLoopBoundedRunner, /pub fn run_native_window_host_loop_bounded<Host>\([\s\S]*runner_state: &mut NativeWindowHostLoopRunnerState,[\s\S]*max_turn_count: usize/);
    assert.match(nativeWindowHostLoopBoundedRunner, /initialize_native_window_host_loop\(runner_state,\s*backend_loop,\s*host\)/);
    assert.match(nativeWindowHostLoopBoundedRunner, /while completed_turns < max_turn_count/);
    assert.match(nativeWindowHostLoopBoundedRunner, /step_native_window_host_loop\(backend_loop,\s*host\)\?/);
    assert.match(nativeWindowHostLoopBoundedRunner, /NativeWindowHostLoopTurn::Continue\(evidence\)/);
    assert.match(nativeWindowHostLoopBoundedRunner, /last_wait_decision = Some\(native_window_host_loop_wait_decision\(evidence\)\)/);
    assert.match(nativeWindowHostLoopBoundedRunner, /NativeWindowHostLoopBoundedRunResult::Exited/);
    assert.match(nativeWindowHostLoopBoundedRunner, /NativeWindowHostLoopBoundedRunResult::BudgetExhausted/);
    assert.doesNotMatch(nativeWindowHostLoopBoundedRunner, /usize::MAX|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|queue|timer|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowHostLoopSchedulerSlice, /pub fn run_native_window_host_loop_scheduler_slice_with_policy<Host>\([\s\S]*scheduler_state: &mut NativeWindowHostLoopSchedulerState,[\s\S]*policy: NativeWindowHostLoopRunPolicy/);
    assert.match(nativeWindowHostLoopSchedulerSlice, /pub fn run_native_window_host_loop_scheduler_slice_with_policy_and_target_fps<Host>\([\s\S]*scheduler_state: &mut NativeWindowHostLoopSchedulerState,[\s\S]*policy: NativeWindowHostLoopRunPolicy,[\s\S]*target_fps: NativeWindowTargetFps/);
    assert.match(nativeWindowHostLoopSchedulerSlice, /let max_turn_count = policy\.turn_slice\.as_usize\(\)/);
    assert.match(nativeWindowHostLoopSchedulerSlice, /run_native_window_host_loop_bounded\([\s\S]*&mut scheduler_state\.runner_state,[\s\S]*backend_loop,[\s\S]*host,[\s\S]*max_turn_count/);
    assert.match(nativeWindowHostLoopSchedulerSlice, /NativeWindowHostLoopBoundedRunResult::Exited[\s\S]*NativeWindowHostLoopSchedulerSliceResult::Exited/);
    assert.match(nativeWindowHostLoopSchedulerSlice, /last_wait_decision: Some\(decision\)[\s\S]*let request = native_window_host_loop_wait_request\(decision\.clone\(\), target_fps\)[\s\S]*let instruction_plan = native_window_host_loop_wait_instruction_plan\([\s\S]*scheduler_state\.wait_strategy_state,[\s\S]*request\.clone\(\)[\s\S]*host[\s\S]*\.wait_after_budget_exhausted\(instruction_plan\.instruction\.clone\(\)\)[\s\S]*NativeWindowHostLoopError::HostWaitFailed[\s\S]*scheduler_state\.wait_strategy_state = instruction_plan\.next_strategy_state[\s\S]*NativeWindowHostLoopSchedulerSliceResult::Waited[\s\S]*request,[\s\S]*instruction: instruction_plan\.instruction/);
    assert.match(nativeWindowHostLoopSchedulerSlice, /last_wait_decision: None[\s\S]*NativeWindowHostLoopError::WaitDecisionMissing/);
    assert.doesNotMatch(nativeWindowHostLoopSchedulerSlice, /last_wait_decision: _/);
    assert.doesNotMatch(nativeWindowHostLoopSchedulerSlice, /usize::MAX|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|queue|timer|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowHostLoopRunner, /pub fn run_native_window_host_loop<Host>\([\s\S]*backend_loop: &mut NativeWindowBackendLoop,[\s\S]*host: &mut Host/);
    assert.match(nativeWindowHostLoopRunner, /Host: NativeWindowRunLoopHost/);
    assert.match(nativeWindowHostLoopRunner, /NativeWindowHostLoopRunPolicy::default\(\)/);
    assert.match(nativeWindowHostLoopRunner, /pub fn run_native_window_host_loop_with_policy<Host>/);
    assert.match(nativeWindowHostLoopRunner, /let mut scheduler_state = NativeWindowHostLoopSchedulerState::new\(\)/);
    assert.match(nativeWindowHostLoopRunner, /pub fn run_native_window_host_loop_with_policy_and_target_fps<Host>[\s\S]*target_fps: NativeWindowTargetFps/);
    assert.match(nativeWindowHostLoopRunner, /run_native_window_host_loop_scheduler_slice_with_policy_and_target_fps\([\s\S]*&mut scheduler_state,[\s\S]*backend_loop,[\s\S]*host,[\s\S]*policy,[\s\S]*target_fps/);
    assert.match(nativeWindowHostLoopRunner, /NativeWindowHostLoopSchedulerSliceResult::Exited\s*\{[\s\S]*exit/);
    assert.match(nativeWindowHostLoopRunner, /NativeWindowHostLoopSchedulerSliceResult::Waited\s*\{[\s\S]*outcome,[\s\S]*\} => \{/);
    assert.match(nativeWindowHostLoopRunner, /native_window_host_loop_scheduler_resume_state_from_wait_outcome\(outcome\)/);
    assert.match(nativeWindowHostLoopRunner, /NativeWindowHostLoopSchedulerResumeState::Ready\(_\) => \{\}/);
    assert.match(nativeWindowHostLoopRunner, /NativeWindowHostLoopSchedulerResumeState::WaitingForFrameIntervalTimer[\s\S]*NativeWindowHostLoopError::TimerFireResumeRequired/);
    assert.doesNotMatch(nativeWindowHostLoopRunner, /run_native_window_host_loop_bounded|wait_after_budget_exhausted|last_wait_decision|WaitDecisionMissing|HostWaitFailed/);
    assert.doesNotMatch(nativeWindowHostLoopRunner, /usize::MAX|poll_event_snapshot|step_host_action|NativeWindowHostAction::|current_present_frame_for_window|host\.present_frame|host\.pump_events_only|window\.update\(|update_with_buffer|queue|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowFrameIntervalWaitAuthorityMode, /native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps[\s\S]*NativeWindowFrameIntervalWaitAuthorityMode::MinifbInternalTargetFps/);
    assert.match(nativeWindowFrameIntervalWaitAuthorityMode, /native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer[\s\S]*NativeWindowFrameIntervalWaitAuthorityMode::HostOwnedDeadlineTimer/);
    assert.match(nativeWindowFrameIntervalWaitAuthorityMode, /combine_native_window_frame_interval_wait_authority_mode[\s\S]*MinifbInternalTargetFps[\s\S]*active_target_fps == requested_target_fps[\s\S]*HostOwnedDeadlineTimer[\s\S]*HostOwnedDeadlineTimer[\s\S]*ConflictingFrameIntervalAuthorities/);
    assert.match(nativeWindowFrameIntervalWaitAuthorityMode, /validate_native_window_frame_interval_wait_authority_mode[\s\S]*MinifbInternalTargetFps[\s\S]*instruction_target_fps != target_fps[\s\S]*TargetFpsMismatch[\s\S]*HostOwnedDeadlineTimer => Ok\(\(\)\)/);
    assert.doesNotMatch(nativeWindowFrameIntervalWaitAuthorityMode, /FramePresentAlreadyPaced|FrameIntervalTimerRegistered|FrameIntervalTimerFired|set_target_fps|window\.update\(|update_with_buffer|execute_native_window_host_loop_wait_with_owner|NativeWindowHostLoopDeadlineTimerAdapter|native_window_host_loop_std_deadline_timer_adapter|register_timer_nanos|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|fallback|silent no-op/i);
    assert.match(nativeWindowMinifbFramePacingAuthority, /impl NativeWindowMinifbFramePacingAuthority[\s\S]*pub fn new\(target_fps: NativeWindowTargetFps\)/);
    assert.match(nativeWindowMinifbFramePacingAuthority, /pub fn target_fps\(self\) -> NativeWindowTargetFps/);
    assert.match(nativeWindowMinifbFramePacingAuthority, /pub fn target_fps_usize\(self\) -> usize[\s\S]*self\.target_fps\.as_usize\(\)/);
    assert.match(nativeWindowMinifbFramePacingAuthority, /pub fn frame_interval_wait_authority_mode[\s\S]*native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps\(self\.target_fps\)/);
    assert.match(nativeWindowMinifbFramePacingAuthority, /validate_native_window_frame_interval_wait_authority_mode\([\s\S]*self\.frame_interval_wait_authority_mode\(\),[\s\S]*frame_interval,[\s\S]*\)[\s\S]*TargetFpsMismatch[\s\S]*FrameIntervalTargetFpsMismatch/);
    assert.match(nativeWindowMinifbFramePacingAuthority, /let nanos_per_frame = frame_interval\.nanos_per_frame\(\)[\s\S]*wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame \+ 1[\s\S]*FrameIntervalWaitNanosMismatch/);
    assert.match(nativeWindowMinifbFramePacingAuthority, /NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced/);
    assert.doesNotMatch(nativeWindowMinifbFramePacingAuthority, /set_target_fps|window\.update\(|update_with_buffer|execute_native_window_host_loop_wait_with_owner|NativeWindowHostLoopDeadlineTimerAdapter|native_window_host_loop_std_deadline_timer_adapter|register_timer_nanos|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|fallback|silent no-op/i);
    assert.match(nativeWindowHostLoopTurnCore, /pub fn step_native_window_host_loop<Host>\([\s\S]*backend_loop: &mut NativeWindowBackendLoop,[\s\S]*host: &mut Host/);
    assert.match(nativeWindowHostLoopTurnCore, /Host: NativeWindowRunLoopHost/);
    assert.doesNotMatch(nativeWindowHostLoopTurnCore, /host\.set_window_title\(&initial_title\)/);
    assert.match(nativeWindowHostLoopTurnCore, /host[\s\S]*\.poll_event_snapshot\(backend_loop\.event_pump_input\(\)\)/);
    assert.match(nativeWindowHostLoopTurnCore, /backend_loop[\s\S]*\.step_host_action\(event_snapshot\)/);
    assert.match(nativeWindowHostLoopTurnCore, /NativeWindowHostAction::Terminate[\s\S]*NativeWindowHostLoopTurn::Exit/);
    assert.match(nativeWindowHostLoopTurnCore, /NativeWindowHostAction::PumpEventsOnly[\s\S]*host\.pump_events_only\(\)[\s\S]*NativeWindowHostLoopContinueEvidence::PumpedEventsOnly/);
    assert.match(nativeWindowHostLoopTurnCore, /NativeWindowHostAction::PresentFrame[\s\S]*current_present_frame_for_window\(\)[\s\S]*host\.present_frame\(present_frame\)[\s\S]*NativeWindowHostLoopContinueEvidence::PresentedFrame/);
    assert.doesNotMatch(nativeWindowHostLoopTurnCore, /minifb|WindowOptions|ScaleMode|window\.update\(|update_with_buffer|set_target_fps|set_background_color|\bKey\b|\bMouseButton\b|\bMouseMode\b|is_open\(|is_key_down\(|get_mouse_down\(|get_unscaled_mouse_pos\(|queue|timer|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op|pixels\(\)|frame\.pixels|&\s*\[[^\]]*\]/i);
    assert.match(nativeWindowMinifbMessagePumpAdapter, /impl NativeWindowHostLoopMessagePumpAdapter for MinifbNativeWindowHostLoopMessagePumpAdapter/);
    assert.match(nativeWindowMinifbMessagePumpAdapter, /self\.window\.update\(\)/);
    assert.match(nativeWindowMinifbMessagePumpAdapter, /NativeWindowHostLoopMessagePumpStatusAdapter::new/);
    assert.match(nativeWindowMinifbMessagePumpAdapter, /NativeWindowHostLoopEventQueueStatusWaiter::new/);
    assert.match(nativeWindowMinifbMessagePumpAdapter, /execute_native_window_host_loop_event_queue_wait_with_waiter/);
    assert.doesNotMatch(nativeWindowMinifbMessagePumpAdapter, /update_with_buffer|\bKey\b|\bMouseButton\b|\bMouseMode\b|window\.is_open\(\)|window\.is_key_down\(|window\.get_mouse_down\(|window\.get_unscaled_mouse_pos\(|register_timer_nanos|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowMinifbVisualHostAdapter, /struct MinifbNativeWindowVisualRunLoopHost/);
    assert.match(nativeWindowMinifbVisualHostAdapter, /impl NativeWindowRunLoopHost for MinifbNativeWindowVisualRunLoopHost/);
    assert.match(nativeWindowMinifbVisualHostAdapter, /type WaitError = MinifbNativeWindowVisualHostWaitError/);
    assert.match(nativeWindowMinifbVisualHostAdapter, /poll_minifb_window_event_pump\(self\.window,\s*input\)/);
    assert.match(nativeWindowMinifbVisualHostAdapter, /self\.window\.set_title\(title\)/);
    assert.match(nativeWindowMinifbVisualHostAdapter, /self\.window\.update\(\)/);
    assert.match(nativeWindowMinifbVisualHostAdapter, /self\.window[\s\S]*\.update_with_buffer\(frame\.pixels\(\),\s*frame\.width\(\),\s*frame\.height\(\)\)/);
    assert.match(nativeWindowMinifbVisualHostAdapter, /VisualHostWaitUnsupported\s*\{\s*instruction\s*\}/);
    assert.doesNotMatch(nativeWindowMinifbVisualHostAdapter, /frame_pacing_authority|NativeWindowMinifbFramePacingAuthority|FramePresentAlreadyPaced|wait_minifb_window_host_event_message_pump|configure_minifb_window_frame_pacing|set_target_fps|NativeWindowHostOwnedDeadlineWaitRunLoopHost|NativeWindowHostLoopInterruptibleDeadline|execute_native_window_host_loop_wait_with_owner|execute_native_window_host_loop_interruptible_deadline_wait_with_adapter|std::thread::sleep|Duration|setTimeout|setInterval|fallback|silent no-op/i);
    assert.match(nativeWindowMinifbHostAdapter, /impl NativeWindowRunLoopHost for MinifbNativeWindowRunLoopHost/);
    assert.match(nativeWindowMinifbHostAdapter, /frame_pacing_authority: NativeWindowMinifbFramePacingAuthority/);
    assert.match(nativeWindowMinifbHostAdapter, /type WaitError = MinifbNativeWindowHostLoopWaitError/);
    assert.match(nativeWindowMinifbHostAdapter, /poll_minifb_window_event_pump\(self\.window,\s*input\)/);
    assert.match(nativeWindowMinifbHostAdapter, /self\.window\.set_title\(title\)/);
    assert.match(nativeWindowMinifbHostAdapter, /self\.window\.update\(\)/);
    assert.match(nativeWindowMinifbHostAdapter, /self\.window[\s\S]*\.update_with_buffer\(frame\.pixels\(\),\s*frame\.width\(\),\s*frame\.height\(\)\)/);
    assert.match(nativeWindowMinifbWaitMethod, /NativeWindowHostLoopWaitInstruction::WaitForHostEvent[\s\S]*wait_minifb_window_host_event_message_pump\(self\.window,\s*window_size,\s*size_changed\)[\s\S]*MinifbNativeWindowHostLoopWaitError::EventQueueWaitFailed/);
    assert.match(nativeWindowMinifbWaitMethod, /NativeWindowHostLoopWaitInstruction::WaitForFrameInterval[\s\S]*frame_interval,[\s\S]*wait_nanos,[\s\S]*self[\s\S]*\.frame_pacing_authority[\s\S]*\.frame_interval_wait_outcome\([\s\S]*presentation,[\s\S]*window_size,[\s\S]*size_changed,[\s\S]*frame_interval,[\s\S]*wait_nanos/);
    assert.match(nativeWindowMinifbWaitMethod, /MinifbNativeWindowHostLoopWaitError::FramePacingAuthorityFailed/);
    assert.doesNotMatch(nativeWindowMinifbWaitMethod, /FramePresentAlreadyPaced|window\.update\(|update_with_buffer|limit_update_rate|NativeWindowHostOwnedDeadlineWaitRunLoopHost|NativeWindowHostLoopInterruptibleDeadline|execute_native_window_host_loop_wait_with_owner|execute_native_window_host_loop_interruptible_deadline_wait_with_adapter|NativeWindowHostLoopDeadlineTimerAdapter|native_window_host_loop_std_deadline_timer_adapter|EventQueueFull|VecDeque|push_back|pop_front|timer|std::thread::sleep|Duration|setTimeout|setInterval|fallback|silent no-op/i);
    assert.doesNotMatch(nativeWindowMinifbHostAdapter, /\bKey\b|\bMouseButton\b|\bMouseMode\b|window\.is_open\(\)|window\.is_key_down\(|window\.get_mouse_down\(|window\.get_unscaled_mouse_pos\(|NativeWindowHostOwnedDeadlineWaitRunLoopHost|NativeWindowHostLoopInterruptibleDeadline|execute_native_window_host_loop_wait_with_owner|execute_native_window_host_loop_interruptible_deadline_wait_with_adapter|NativeWindowHostLoopDeadlineTimerAdapter|native_window_host_loop_std_deadline_timer_adapter|EventQueueFull|VecDeque|push_back|pop_front|timer|std::thread::sleep|Duration|setTimeout|setInterval|DOM|Canvas|video_memory|stdout_protocol|fallback|silent no-op/i);
    assert.match(nativeWindowRunLoopHelper, /pub fn run_minifb_window_loop\([\s\S]*NativeWindowRunLoopConfig[\s\S]*NativeWindowRunLoopExit/);
    assert.match(nativeWindowRunLoopHelper, /pub fn validate_minifb_window_run_loop_wait_backend[\s\S]*NativeWindowRunLoopWaitBackend::MinifbInternalTargetFps\.authority_mode\(target_fps\)[\s\S]*requested\.authority_mode\(target_fps\)[\s\S]*combine_native_window_frame_interval_wait_authority_mode[\s\S]*NativeWindowRunLoopFrameIntervalWaitBackendError::Unsupported/);
    assert.match(nativeWindowRunLoopHelper, /pub fn validate_minifb_window_run_loop_frame_interval_wait_backend[\s\S]*NativeWindowRunLoopWaitBackend::from\(requested\)[\s\S]*target_fps/);
    assert.match(nativeWindowRunLoopHelper, /run_minifb_window_loop[\s\S]*validate_minifb_window_run_loop_wait_backend\([\s\S]*config\.wait_backend,[\s\S]*config\.target_fps[\s\S]*\)[\s\S]*NativeWindowRunLoopError::FrameIntervalWaitBackendUnsupported[\s\S]*let frame_pacing_authority = minifb_native_window_frame_pacing_authority\(config\.target_fps\)[\s\S]*NativeWindowBackendLoop::new_for_scale[\s\S]*Window::new[\s\S]*configure_minifb_window_frame_pacing/);
    assert.match(nativeWindowRunLoopHelper, /WindowOptions\s*\{[\s\S]*resize:\s*true,[\s\S]*scale_mode:\s*ScaleMode::UpperLeft/);
    assert.match(nativeWindowRunLoopHelper, /let frame_pacing_authority = minifb_native_window_frame_pacing_authority\(config\.target_fps\)/);
    assert.match(nativeWindowRunLoopHelper, /configure_minifb_window_frame_pacing\(&mut window,\s*frame_pacing_authority\)/);
    assert.match(nativeWindowRunLoopHelper, /fn configure_minifb_window_frame_pacing[\s\S]*let target_fps = authority\.target_fps_usize\(\)[\s\S]*window\.set_target_fps\(target_fps\)/);
    assert.doesNotMatch(nativeWindowRunLoopHelper, /set_target_fps\(0\)|set_target_fps\(60\)|set_target_fps\(config\.target_fps/);
    assert.match(nativeWindowRunLoopHelper, /window\.set_background_color\(9,\s*13,\s*18\)/);
    assert.match(nativeWindowRunLoopHelper, /let mut host = MinifbNativeWindowRunLoopHost[\s\S]*frame_pacing_authority/);
    assert.match(nativeWindowRunLoopHelper, /run_native_window_host_loop_with_policy_and_target_fps\([\s\S]*&mut backend_loop,[\s\S]*&mut host,[\s\S]*config\.host_loop_policy,[\s\S]*config\.target_fps/);
    assert.match(nativeWindowRunLoopHelper, /NativeWindowRunLoopError::WindowPresentFailed/);
    assert.match(nativeWindowRunLoopHelper, /NativeWindowRunLoopError::WaitDecisionMissing/);
    assert.match(nativeWindowRunLoopHelper, /minifb_native_window_host_loop_wait_error_message/);
    assert.doesNotMatch(nativeWindowRunLoopHelper, /NativeWindowHostOwnedDeadlineWaitRunLoopHost|NativeWindowHostLoopInterruptibleDeadline|execute_native_window_host_loop_wait_with_owner|execute_native_window_host_loop_interruptible_deadline_wait_with_adapter|NativeWindowHostLoopDeadlineTimerAdapter|native_window_host_loop_std_deadline_timer_adapter/);
    assert.match(nativeWindowRunLoopHelper, /#\[cfg\(all\(feature = "window", target_os = "windows", not\(target_arch = "wasm32"\)\)\)\][\s\S]*pub fn run_windows_platform_wait_window_loop/);
    assert.match(nativeWindowWindowsPlatformWaitRunner, /pub fn run_windows_platform_wait_window_loop\([\s\S]*config: NativeWindowRunLoopConfig[\s\S]*NativeWindowRunLoopExit/);
    assert.match(nativeWindowWindowsPlatformWaitRunner, /validate_native_window_run_loop_platform_wait_runner_support_for_platform\([\s\S]*NativeWindowHostLoopPlatformKind::Windows,[\s\S]*config,[\s\S]*\)[\s\S]*PlatformWaitRunnerUnsupported[\s\S]*native_window_run_loop_platform_wait_backend_from_config\(config\)[\s\S]*PlatformWaitBackendFromConfigFailed[\s\S]*NativeWindowBackendLoop::new_for_scale[\s\S]*Window::new/);
    assert.match(nativeWindowWindowsPlatformWaitRunner, /WindowOptions\s*\{[\s\S]*resize:\s*true,[\s\S]*scale_mode:\s*ScaleMode::UpperLeft/);
    assert.match(nativeWindowWindowsPlatformWaitRunner, /let visual_host = MinifbNativeWindowVisualRunLoopHost[\s\S]*native_window_host_loop_platform_wait_run_loop_host_from_backend\([\s\S]*visual_host,[\s\S]*platform_wait_backend/);
    assert.match(nativeWindowWindowsPlatformWaitRunner, /run_native_window_host_loop_with_policy_and_target_fps\([\s\S]*&mut backend_loop,[\s\S]*&mut host,[\s\S]*config\.host_loop_policy,[\s\S]*config\.target_fps/);
    assert.match(nativeWindowWindowsPlatformWaitRunner, /NativeWindowRunLoopError::WindowsPlatformWaitHostLoopFailed/);
    assert.doesNotMatch(nativeWindowWindowsPlatformWaitRunner, /validate_minifb_window_run_loop_wait_backend|minifb_native_window_frame_pacing_authority|MinifbNativeWindowRunLoopHost|configure_minifb_window_frame_pacing|set_target_fps|FramePresentAlreadyPaced|HeadlessScripted|std::thread::sleep|Duration|setTimeout|setInterval|fallback|silent no-op/i);
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
    assert.match(libSource, /native_window_target_fps_accepts_default_and_custom_values/);
    assert.match(libSource, /native_window_target_fps_rejects_zero_and_too_high_values/);
    assert.match(libSource, /native_window_frame_interval_wait_authority_combines_same_minifb_target/);
    assert.match(libSource, /native_window_frame_interval_wait_authority_rejects_minifb_and_deadline_conflict/);
    assert.match(libSource, /native_window_frame_interval_wait_authority_rejects_minifb_target_mismatch/);
    assert.match(libSource, /native_window_frame_interval_wait_authority_validates_minifb_instruction_target_fps/);
    assert.match(libSource, /native_window_frame_interval_wait_authority_validates_host_owned_deadline_timer/);
    assert.match(libSource, /native_window_minifb_frame_pacing_authority_accepts_matching_frame_interval/);
    assert.match(libSource, /native_window_minifb_frame_pacing_authority_accepts_remainder_carry_wait_nanos/);
    assert.match(libSource, /native_window_minifb_frame_pacing_authority_rejects_target_fps_mismatch/);
    assert.match(libSource, /native_window_minifb_frame_pacing_authority_rejects_invalid_wait_nanos/);
    assert.match(libSource, /native_window_host_loop_turn_slice_accepts_default_and_custom_values/);
    assert.match(libSource, /native_window_host_loop_turn_slice_rejects_zero_and_too_high_values/);
    assert.match(libSource, /native_window_run_loop_config_preserves_demo_state/);
    assert.match(libSource, /native_window_run_loop_frame_interval_backend_maps_to_authority_mode/);
    assert.match(libSource, /native_window_minifb_run_loop_backend_validation_rejects_host_owned_deadline_timer/);
    assert.match(libSource, /native_window_minifb_run_loop_backend_validation_accepts_minifb_internal_pacing/);
    assert.match(libSource, /native_window_title_reports_drawable_and_unavailable_surface/);
    assert.match(libSource, /initialize_native_window_host_loop_reports_idempotent_title_state/);
    assert.match(libSource, /native_window_host_loop_wait_decision_maps_continue_evidence/);
    assert.match(libSource, /native_window_host_loop_wait_request_builds_typed_backend_plan/);
    assert.match(libSource, /native_window_host_loop_scheduler_resume_state_accepts_already_paced_waits/);
    assert.match(libSource, /native_window_host_loop_scheduler_resume_state_requires_timer_fire/);
    assert.match(libSource, /native_window_host_loop_scheduler_resume_ready_accepts_timer_fire_evidence/);
    assert.match(libSource, /run_native_window_host_loop_bounded_zero_budget_initializes_without_polling/);
    assert.match(libSource, /run_native_window_host_loop_bounded_counts_exit_turn/);
    assert.match(libSource, /run_native_window_host_loop_bounded_yields_after_continue_budget/);
    assert.match(libSource, /run_native_window_host_loop_bounded_reports_last_wait_decision/);
    assert.match(libSource, /run_native_window_host_loop_bounded_keeps_initial_title_across_slices/);
    assert.match(libSource, /run_native_window_host_loop_bounded_preserves_event_pump_error/);
    assert.match(libSource, /run_native_window_host_loop_bounded_preserves_present_error/);
    assert.match(libSource, /run_native_window_host_loop_bounded_preserves_host_action_error/);
    assert.match(libSource, /run_native_window_host_loop_bounded_preserves_presenter_frame_error/);
    assert.match(libSource, /native_window_host_loop_scheduler_slice_waits_after_budget_exhaustion/);
    assert.match(libSource, /native_window_host_loop_scheduler_slice_keeps_initial_title_across_calls/);
    assert.match(libSource, /native_window_host_loop_scheduler_slice_preserves_wait_error_without_next_poll/);
    assert.match(libSource, /native_window_host_loop_scheduler_slice_exits_without_wait/);
    assert.match(libSource, /native_window_host_loop_with_policy_exits_across_single_turn_slices/);
    assert.match(libSource, /native_window_host_loop_with_policy_dispatches_frame_interval_wait/);
    assert.match(libSource, /native_window_host_loop_with_policy_requires_timer_fire_before_resume/);
    assert.match(libSource, /native_window_host_loop_with_policy_uses_explicit_target_fps_for_wait_request/);
    assert.match(libSource, /native_window_host_loop_with_policy_preserves_event_pump_error/);
    assert.match(libSource, /native_window_host_loop_with_policy_preserves_wait_error_without_next_poll/);
    assert.match(libSource, /step_native_window_host_loop_close_turn_has_no_initial_title_or_present/);
    assert.match(libSource, /step_native_window_host_loop_pump_only_resize_updates_title/);
    assert.match(libSource, /step_native_window_host_loop_drawable_resize_presents_exact_frame/);
    assert.match(libSource, /step_native_window_host_loop_drawable_without_resize_keeps_title_empty/);
    assert.match(libSource, /step_native_window_host_loop_preserves_event_pump_error/);
    assert.match(libSource, /step_native_window_host_loop_preserves_present_error/);
    assert.match(libSource, /step_native_window_host_loop_preserves_host_action_error/);
    assert.match(libSource, /step_native_window_host_loop_preserves_presenter_frame_error/);
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
    assert.doesNotMatch(nativeClockHelperWithoutWaitBackends, /saturating_|wrapping_|clamp|std::thread::sleep|SystemTime|UNIX_EPOCH|fallback|silent no-op/);
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
    assert.doesNotMatch(nativeWindowEventPumpHelperWithoutWaitBackends, /window\.update\(|update_with_buffer|queue|stdout_protocol|Canvas|DOM|video_memory|fallback|silent no-op/i);
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
    assert.match(platformDoc, /Linux selector timerfd raw backend checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopLinuxSelectorTimerFdRawApi/);
    assert.match(platformDoc, /fd `0` は有効/);
    assert.match(platformDoc, /TimerFired` と `HostEventReady` を分け/);
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
    assert.match(platformDoc, /Native window frame pacing config checkpoint/);
    assert.match(platformDoc, /NativeWindowTargetFps/);
    assert.match(platformDoc, /NativeWindowRunLoopConfig\.target_fps/);
    assert.match(platformDoc, /1\.\.=240/);
    assert.match(platformDoc, /Window::set_target_fps/);
    assert.match(platformDoc, /Native window host-loop core checkpoint/);
    assert.match(platformDoc, /NativeWindowRunLoopHost/);
    assert.match(platformDoc, /run_native_window_host_loop/);
    assert.match(platformDoc, /NativeWindowHostLoopError/);
    assert.match(platformDoc, /backend state を失わない/);
    assert.match(platformDoc, /Native window host-loop turn checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopTurn/);
    assert.match(platformDoc, /step_native_window_host_loop/);
    assert.match(platformDoc, /initial title を設定しない/);
    assert.match(platformDoc, /Native window host-loop bounded runner checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopRunnerState/);
    assert.match(platformDoc, /initialize_native_window_host_loop/);
    assert.match(platformDoc, /run_native_window_host_loop_bounded/);
    assert.match(platformDoc, /BudgetExhausted/);
    assert.match(platformDoc, /Native window host-loop scheduler slice checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopSchedulerState/);
    assert.match(platformDoc, /NativeWindowHostLoopSchedulerSliceResult/);
    assert.match(platformDoc, /run_native_window_host_loop_scheduler_slice_with_policy/);
    assert.match(platformDoc, /Native window host-loop wait request plan checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopWaitRequest/);
    assert.match(platformDoc, /NativeWindowFrameIntervalRequest/);
    assert.match(platformDoc, /Native window host-loop wait strategy instruction checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopWaitInstruction/);
    assert.match(platformDoc, /NativeWindowHostLoopWaitStrategyState/);
    assert.match(platformDoc, /Native window host-loop thread wait backend checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopThreadSleeper/);
    assert.match(platformDoc, /HostEventWaitUnsupported/);
    assert.match(platformDoc, /Native window host-loop timer registration backend checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopTimerRegistrar/);
    assert.match(platformDoc, /InvalidTimerRegistrationId/);
    assert.match(platformDoc, /Native window host-loop timer fire\/wakeup backend checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopTimerFireWaiter/);
    assert.match(platformDoc, /FrameIntervalTimerFired/);
    assert.match(platformDoc, /Native window host-loop event queue wait backend checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopEventQueueWaiter/);
    assert.match(platformDoc, /FrameIntervalEventQueueWaitUnsupported/);
    assert.match(platformDoc, /Native window host-loop event queue normalized status adapter checkpoint/);
    assert.match(platformDoc, /NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY/);
    assert.match(platformDoc, /NativeWindowHostLoopEventQueueStatusAdapter/);
    assert.match(platformDoc, /InvalidRawStatus/);
    assert.match(platformDoc, /Native window host-loop message pump adapter checkpoint/);
    assert.match(platformDoc, /NativeWindowHostLoopMessagePumpAdapter/);
    assert.match(platformDoc, /MinifbNativeWindowHostLoopMessagePumpAdapter/);
    assert.match(platformDoc, /wait_minifb_window_host_event_message_pump/);
    assert.match(platformDoc, /Native window host-loop frame interval timer registration outcome checkpoint/);
    assert.match(platformDoc, /FrameIntervalTimerRegistered/);
    assert.match(platformDoc, /timer registration 成功の wait completion 偽装/);
    assert.match(platformDoc, /F5he/);
    assert.match(platformDoc, /NativeWindowMinifbFramePacingAuthority/);
    assert.match(platformDoc, /set_target_fps 0/);
    assert.match(platformDoc, /tight loop/);
    assert.match(platformDoc, /F5hs/);
    assert.match(platformDoc, /--wait-backend minifb\|platform/);
    assert.match(platformDoc, /F5hy/);
    assert.match(platformDoc, /NativeWindowHostLoopLinuxSelectorTimerFdSysApi/);
    assert.match(platformDoc, /epoll_create1/);
    assert.match(platformDoc, /timerfd_create/);
    assert.match(platformDoc, /F5hz/);
    assert.match(platformDoc, /NativeWindowHostLoopLinuxHostEventFd/);
    assert.match(platformDoc, /eventfd 0 EFD_CLOEXEC \| EFD_NONBLOCK/);
    assert.match(platformDoc, /host-event-only wait は host event fd readiness だけを成功/);
    assert.match(platformDoc, /headless で明示指定された wait backend、重複指定、不明な値は error/);
    assert.match(platformDoc, /non-Windows platform selection は typed unsupported error/);
    assert.match(platformDoc, /F5hx/);
    assert.match(platformDoc, /NativeWindowHostLoopPlatformWaitBackend WindowsApi MacosApi LinuxApi/);
    assert.match(platformDoc, /build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis/);
    assert.match(platformDoc, /selected されていない raw API は呼ばず/);
    assert.match(platformDoc, /NativeWindowHostLoopWindowsOnlyPlatformWaitBackend WindowsApi/);
    assert.match(platformDoc, /method body は `match \*self \{\}` のみ/);
    assert.match(platformDoc, /F5ia/);
    assert.match(platformDoc, /NativeWindowHostLoopLinuxOnlyPlatformWaitBackend LinuxApi/);
    assert.match(platformDoc, /build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api/);
    assert.match(platformDoc, /raw API method 呼び出し前に `BackendSupportFailed`/);
    assert.match(platformDoc, /eventfd producer、runner \/ CLI、minifb wait path へはまだ接続しない/);
    assert.match(platformDoc, /F5ib/);
    assert.match(platformDoc, /NativeWindowHostLoopLinuxSelectorTimerFdBackend::signal_host_event/);
    assert.match(platformDoc, /SignalHostEventFdFailed/);
    assert.match(platformDoc, /`u64` 値 `1` を exactly 8 bytes/);
    assert.match(platformDoc, /runner \/ CLI \/ minifb wait path への接続はまだ行わない/);
    assert.match(platformDoc, /F5ic/);
    assert.match(platformDoc, /NativeWindowHostLoopLinuxHostEventSignalFd/);
    assert.match(platformDoc, /create_host_event_signal_producer/);
    assert.match(platformDoc, /F_DUPFD_CLOEXEC/);
    assert.match(platformDoc, /synthetic readiness、pre-signal busy loop/);
    assert.match(platformDoc, /F5id/);
    assert.match(platformDoc, /MinifbNativeWindowLinuxHostEventSignalInputCallback/);
    assert.match(platformDoc, /keyboard \/ text input callback/);
    assert.match(platformDoc, /blocking wait の実 event source ではなく/);
    assert.match(platformDoc, /HostEventSignalFailed/);
    assert.match(platformDoc, /run_linux_platform_wait_window_loop/);
    assert.match(platformDoc, /F5ie/);
    assert.match(platformDoc, /NativeWindowHostLoopLinuxEventSourceCapability::ObservedInputOnly/);
    assert.match(platformDoc, /ObservedInputOnlyUnsupportedForBlockingWait/);
    assert.match(platformDoc, /ExternallyWakeableEventSource/);
    assert.match(platformDoc, /fd owner、selector registration、`HostEventReady` outcome/);
    assert.match(platformDoc, /F5iq/);
    assert.match(platformDoc, /LinuxWindowEventSourceFdMissing/);
    assert.match(platformDoc, /actual X11 \/ Wayland window event source fd integration/);
    assert.match(platformDoc, /F5if/);
    assert.match(platformDoc, /LinuxEventSourceSupportFailed/);
    assert.match(platformDoc, /暗黙に `ExternallyWakeableEventSource` を渡さない/);
    assert.match(platformDoc, /F5ig/);
    assert.match(platformDoc, /NativeWindowRunLoopPlatformWaitBackendConfig/);
    assert.match(platformDoc, /MissingLinuxEventSourceCapability/);
    assert.match(platformDoc, /F5hf/);
    assert.match(platformDoc, /NativeWindowFrameIntervalWaitAuthorityMode/);
    assert.match(platformDoc, /HostOwnedDeadlineTimer/);
    assert.match(platformDoc, /ConflictingFrameIntervalAuthorities/);
    assert.match(platformDoc, /wait evidence を作らない/);
    assert.match(platformDoc, /F5hg/);
    assert.match(platformDoc, /execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode/);
    assert.match(platformDoc, /FrameIntervalAuthorityFailed/);
    assert.match(platformDoc, /timer registration、clock read、sleeper call、active timer mutation は起こさない/);
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
    assert.match(implementationPlan, /Phase F5gi: Native window host-loop turn boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopTurn/);
    assert.match(implementationPlan, /step_native_window_host_loop/);
    assert.match(implementationPlan, /long loop runner slice/);
    assert.match(implementationPlan, /Phase F5gj: Native window host-loop bounded runner boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopRunnerState/);
    assert.match(implementationPlan, /NativeWindowHostLoopInitialization/);
    assert.match(implementationPlan, /run_native_window_host_loop_bounded/);
    assert.match(implementationPlan, /BudgetExhausted/);
    assert.match(implementationPlan, /Phase F5gk: Native window frame pacing config boundary/);
    assert.match(implementationPlan, /NativeWindowTargetFps/);
    assert.match(implementationPlan, /NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS = 240/);
    assert.match(implementationPlan, /set_target_fps 60/);
    assert.match(implementationPlan, /CLI に `--fps N`/);
    assert.match(implementationPlan, /Phase F5go: Native window host-loop wait dispatch boundary/);
    assert.match(implementationPlan, /wait_after_budget_exhausted/);
    assert.match(implementationPlan, /HostWaitFailed/);
    assert.match(implementationPlan, /WaitDecisionMissing/);
    assert.match(implementationPlan, /Phase F5gp: Native window host-loop scheduler slice boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopSchedulerState/);
    assert.match(implementationPlan, /NativeWindowHostLoopSchedulerSliceResult/);
    assert.match(implementationPlan, /run_native_window_host_loop_scheduler_slice_with_policy/);
    assert.match(implementationPlan, /Phase F5gq: Native window host-loop wait request plan boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopWaitRequest/);
    assert.match(implementationPlan, /NativeWindowFrameIntervalRequest/);
    assert.match(implementationPlan, /Phase F5gr: Native window host-loop wait strategy instruction boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopWaitInstruction/);
    assert.match(implementationPlan, /NativeWindowHostLoopWaitStrategyState/);
    assert.match(implementationPlan, /Phase F5gs: Native window host-loop thread wait backend boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopThreadSleeper/);
    assert.match(implementationPlan, /HostEventWaitUnsupported/);
    assert.match(implementationPlan, /Phase F5gt: Native window host-loop timer registration backend boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopTimerRegistrar/);
    assert.match(implementationPlan, /InvalidTimerRegistrationId/);
    assert.match(implementationPlan, /Phase F5gu: Native window host-loop event queue wait backend boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopEventQueueWaiter/);
    assert.match(implementationPlan, /FrameIntervalEventQueueWaitUnsupported/);
    assert.match(implementationPlan, /Phase F5gv: Native window host-loop event queue normalized status adapter boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopEventQueueStatusAdapter/);
    assert.match(implementationPlan, /InvalidRawStatus/);
    assert.match(implementationPlan, /Phase F5gw: Native window host-loop message pump adapter boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopMessagePumpAdapter/);
    assert.match(implementationPlan, /wait_minifb_window_host_event_message_pump/);
    assert.match(implementationPlan, /Phase F5gx: Native window host-loop frame interval timer registration outcome boundary/);
    assert.match(implementationPlan, /FrameIntervalTimerRegistered/);
    assert.match(implementationPlan, /wait completion の偽装は禁止/);
    assert.match(implementationPlan, /Phase F5gy: Native window host-loop timer fire\/wakeup backend boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopTimerFireWaiter/);
    assert.match(implementationPlan, /FiredTimerRegistrationMismatch/);
    assert.match(implementationPlan, /Phase F5gz: Native window host-loop timer wakeup executor boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopTimerWakeError/);
    assert.match(implementationPlan, /RegistrationFailed/);
    assert.match(implementationPlan, /FireFailed/);
    assert.match(implementationPlan, /Phase F5ha: Native window host-loop scheduler timer resume gate boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopSchedulerResumeState/);
    assert.match(implementationPlan, /TimerFireResumeRequired/);
    assert.match(implementationPlan, /PLAN_APPROVED/);
    assert.match(implementationPlan, /Phase F5he: Native minifb frame pacing authority boundary/);
    assert.match(implementationPlan, /NativeWindowMinifbFramePacingAuthority/);
    assert.match(implementationPlan, /FramePresentAlreadyPaced/);
    assert.match(implementationPlan, /set_target_fps 0/);
    assert.match(implementationPlan, /deadline timer owner 未接続/);
    assert.match(implementationPlan, /Phase F5hf: Native frame interval wait authority mode boundary/);
    assert.match(implementationPlan, /NativeWindowFrameIntervalWaitAuthorityMode/);
    assert.match(implementationPlan, /HostOwnedDeadlineTimer/);
    assert.match(implementationPlan, /ConflictingFrameIntervalAuthorities/);
    assert.match(implementationPlan, /selector \/ message-loop timer ownership の実装ではなく/);
    assert.match(implementationPlan, /wait evidence を生成しない/);
    assert.match(implementationPlan, /Phase F5hg: Native wait owner frame interval authority connection boundary/);
    assert.match(implementationPlan, /execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode/);
    assert.match(implementationPlan, /FrameIntervalAuthorityFailed/);
    assert.match(implementationPlan, /deadline timer registration \/ clock read \/ sleeper call \/ active timer mutation より前に authority を検査/);
    assert.match(implementationPlan, /macOS run loop timer、Windows waitable timer \/ message wait、Linux selector \/ timerfd の実装ではない/);
    assert.match(implementationPlan, /Phase F5hm: Native platform wait backend construction gate boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopPlatformWaitBackendSelection/);
    assert.match(implementationPlan, /BackendImplementationUnavailable/);
    assert.match(implementationPlan, /selection token の field は private/);
    assert.match(implementationPlan, /実 OS backend の代替として headless scripted、minifb、thread sleep、busy loop、synthetic timer fire を返さない/);
    assert.match(implementationPlan, /Phase F5hn: Native Windows waitable timer message wait raw backend boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopWindowsWaitRawApi/);
    assert.match(implementationPlan, /NativeWindowHostLoopWindowsDeadlinePlan/);
    assert.match(implementationPlan, /message-only wait/);
    assert.match(implementationPlan, /generic `build_native_window_host_loop_platform_wait_backend_from_selection` は F5hm の fail-closed behavior を維持/);
    assert.match(implementationPlan, /Phase F5ho: Native single-owner interruptible wait adapter boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter/);
    assert.match(implementationPlan, /両 trait の `Error` type が同一/);
    assert.match(implementationPlan, /deadline wait 直前に進めた timer id/);
    assert.match(implementationPlan, /Phase F5hp: Native Windows platform wait support gate boundary/);
    assert.match(implementationPlan, /build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api/);
    assert.match(implementationPlan, /host owner を消費しない/);
    assert.match(implementationPlan, /no-owner fail-closed probe/);
    assert.match(implementationPlan, /Phase F5hs: Native Windows platform wait CLI selection boundary/);
    assert.match(implementationPlan, /--wait-backend minifb\|platform/);
    assert.match(implementationPlan, /未指定時は従来通り `run_minifb_window_loop`/);
    assert.match(implementationPlan, /non-Windows platform selection は unsupported error/);
    assert.match(implementationPlan, /Phase F5hu: Native Linux selector timerfd raw backend boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopLinuxSelectorTimerFdRawApi/);
    assert.match(implementationPlan, /fd `0` が有効/);
    assert.match(implementationPlan, /generic `NativeWindowHostLoopPlatformWaitBackend` へ `LinuxSelectorTimerFd\(\.\.\.\)` owner variant を追加しない/);
    assert.match(implementationPlan, /Phase F5hv: Native Linux selector timerfd single-owner wait trait boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopInterruptibleDeadlineWaiter/);
    assert.match(implementationPlan, /TimerFired` を `NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached/);
    assert.match(implementationPlan, /actual Linux syscall shim や generic platform wait enum 統合へは進まない/);
    assert.match(implementationPlan, /Phase F5hw: Native macOS run loop timer single-owner wait trait boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopMacosRunLoopTimerBackend` は monotonic origin からの checked elapsed nanoseconds/);
    assert.match(implementationPlan, /macOS raw wake の `TimerFired` を `NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached/);
    assert.match(implementationPlan, /generic `NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer/);
    assert.match(implementationPlan, /Phase F5hy: Native Linux selector timerfd actual sys shim boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopLinuxSelectorTimerFdSysApi/);
    assert.match(implementationPlan, /epoll_create1/);
    assert.match(implementationPlan, /timerfd_create/);
    assert.match(implementationPlan, /host event fd registration が未設計/);
    assert.match(implementationPlan, /EINTR` retry loop/);
    assert.match(implementationPlan, /Phase F5hz: Native Linux host event fd integration boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopLinuxHostEventFd/);
    assert.match(implementationPlan, /eventfd 0 EFD_CLOEXEC \| EFD_NONBLOCK/);
    assert.match(implementationPlan, /host-event-only wait は eventfd readiness だけを成功/);
    assert.match(implementationPlan, /eventfd write \/ signal producer/);
    assert.match(implementationPlan, /Phase F5ia: Native Linux platform wait support helper boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopLinuxOnlyPlatformWaitBackend LinuxApi/);
    assert.match(implementationPlan, /build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api/);
    assert.match(implementationPlan, /support failure before raw method calls/);
    assert.match(implementationPlan, /Phase F5ib: Native Linux host event fd producer boundary/);
    assert.match(implementationPlan, /signal_host_event_fd_raw/);
    assert.match(implementationPlan, /SignalHostEventFdFailed/);
    assert.match(implementationPlan, /exactly 8 bytes/);
    assert.match(implementationPlan, /eventfd full \/ `EAGAIN` を成功扱いする fallback/);
    assert.match(implementationPlan, /Phase F5ic: Native Linux host event signal producer handle boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopLinuxHostEventSignalRawApi/);
    assert.match(implementationPlan, /NativeWindowHostLoopLinuxHostEventSignalProducerError/);
    assert.match(implementationPlan, /fcntl F_DUPFD_CLOEXEC/);
    assert.match(implementationPlan, /pre-signal busy loop/);
    assert.match(implementationPlan, /Phase F5id: Native Linux minifb observed-input signal bridge boundary/);
    assert.match(implementationPlan, /NativeWindowHostEventSignalWaitError WaitError/);
    assert.match(implementationPlan, /MinifbNativeWindowLinuxHostEventSignalCallbackState/);
    assert.match(implementationPlan, /HostEventReady` outcome や timer fired evidence を生成しない/);
    assert.match(implementationPlan, /Linux platform wait runner、CLI dispatch、`run_linux_platform_wait_window_loop`/);
    assert.match(implementationPlan, /Phase F5ie: Native Linux blocking wait event source capability gate/);
    assert.match(implementationPlan, /NativeWindowHostLoopLinuxEventSourceCapability/);
    assert.match(implementationPlan, /ObservedInputOnlyUnsupportedForBlockingWait/);
    assert.match(implementationPlan, /fd owner、selector registration、wait outcome、runner dispatch を意味しない/);
    assert.match(implementationPlan, /ExternallyWakeableEventSource` を受理しても/);
    assert.match(implementationPlan, /Phase F5iq: Native Linux window event source fd missing reason boundary/);
    assert.match(implementationPlan, /LinuxWindowEventSourceFdMissing/);
    assert.match(implementationPlan, /old variant rejection|旧 `LinuxExternallyWakeableEventSourceOwnerMissing` 名を拒否/);
    assert.match(implementationPlan, /F5it 以降の current contract/);
    assert.match(implementationPlan, /MissingLinuxWindowEventSourceRawFd/);
    assert.match(implementationPlan, /LinuxWindowEventSourceEventParsingMissing/);
    assert.match(implementationPlan, /fd-present config は `PlatformRunnerIntegrationMissing LinuxWindowEventSourceEventParsingMissing`/);
    assert.match(implementationPlan, /Phase F5if: Native Linux platform wait backend event source config gate/);
    assert.match(implementationPlan, /LinuxEventSourceSupportFailed/);
    assert.match(implementationPlan, /旧 two-argument Linux helper call/);
    assert.match(implementationPlan, /cfg Linux `native_window_host_loop_platform_wait_backend_from_selection` は `event_source_capability` を explicit input/);
    assert.match(implementationPlan, /Phase F5ig: Native run-loop platform wait config event source gate/);
    assert.match(implementationPlan, /NativeWindowRunLoopPlatformWaitBackendConfig/);
    assert.match(implementationPlan, /MissingLinuxEventSourceCapability/);
    assert.match(implementationPlan, /`new_with_platform_wait_backend_selection` を rename \/ remove しない/);
    assert.match(implementationPlan, /Phase F5hx: Native platform wait multi-backend owner boundary/);
    assert.match(implementationPlan, /NativeWindowHostLoopPlatformWaitBackend WindowsApi MacosApi LinuxApi/);
    assert.match(implementationPlan, /build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis/);
    assert.match(implementationPlan, /selected されていない raw API は fallback \/ dummy \/ no-op として呼ばない/);
    assert.match(implementationPlan, /NativeWindowHostLoopWindowsOnlyPlatformWaitBackend WindowsApi/);
    assert.match(implementationPlan, /NativeWindowHostLoopNeverMacosRunLoopTimerRawApi/);
    assert.match(implementationPlan, /NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi/);
    assert.match(implementationPlan, /method body は `match \*self \{\}` だけ/);
    assert.match(implementationPlan, /`#\[cfg\(target_os = "linux"\)\]` actual Linux sys shim、`#\[cfg\(target_os = "macos"\)\]` actual macOS sys shim/);
    assert.match(implementationPlan, /PLAN_APPROVED/);
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
    assert.match(standardSpec, /F5gi Native window host-loop turn boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopTurn/);
    assert.match(standardSpec, /step_native_window_host_loop/);
    assert.match(standardSpec, /F5gj Native window host-loop bounded runner boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopRunnerState/);
    assert.match(standardSpec, /initialize_native_window_host_loop/);
    assert.match(standardSpec, /run_native_window_host_loop_bounded/);
    assert.match(standardSpec, /F5gk Native window frame pacing config boundary/);
    assert.match(standardSpec, /NativeWindowTargetFps/);
    assert.match(standardSpec, /NativeWindowRunLoopConfig\.target_fps/);
    assert.match(standardSpec, /NativeWindowRunLoopError::TargetFpsInvalid/);
    assert.match(standardSpec, /F5go Native window host-loop wait dispatch boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopWaitOutcome/);
    assert.match(standardSpec, /NativeWindowRunLoopError::WaitDecisionMissing/);
    assert.match(standardSpec, /F5gp Native window host-loop scheduler slice boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopSchedulerState/);
    assert.match(standardSpec, /NativeWindowHostLoopSchedulerSliceResult/);
    assert.match(standardSpec, /run_native_window_host_loop_scheduler_slice_with_policy/);
    assert.match(standardSpec, /F5gq Native window host-loop wait request plan boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopWaitRequest/);
    assert.match(standardSpec, /NativeWindowFrameIntervalRequest/);
    assert.match(standardSpec, /F5gr Native window host-loop wait strategy instruction boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopWaitInstruction/);
    assert.match(standardSpec, /NativeWindowHostLoopWaitStrategyState/);
    assert.match(standardSpec, /F5gs Native window host-loop thread wait backend boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopThreadSleeper/);
    assert.match(standardSpec, /HostEventWaitUnsupported/);
    assert.match(standardSpec, /F5gt Native window host-loop timer registration backend boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopTimerRegistrar/);
    assert.match(standardSpec, /InvalidTimerRegistrationId/);
    assert.match(standardSpec, /F5gu Native window host-loop event queue wait backend boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopEventQueueWaiter/);
    assert.match(standardSpec, /FrameIntervalEventQueueWaitUnsupported/);
    assert.match(standardSpec, /F5gv Native window host-loop event queue normalized status adapter boundary/);
    assert.match(standardSpec, /NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY/);
    assert.match(standardSpec, /NativeWindowHostLoopEventQueueStatusWaiter/);
    assert.match(standardSpec, /F5gw Native window host-loop message pump adapter boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopMessagePumpStatusAdapter/);
    assert.match(standardSpec, /wait_minifb_window_host_event_message_pump/);
    assert.match(standardSpec, /F5gx Native window host-loop frame interval timer registration outcome boundary/);
    assert.match(standardSpec, /FrameIntervalTimerRegistered/);
    assert.match(standardSpec, /already-paced outcome ではない/);
    assert.match(standardSpec, /F5gy Native window host-loop timer fire\/wakeup backend boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopTimerFireWaiter/);
    assert.match(standardSpec, /FiredTimerRegistrationMismatch/);
    assert.match(standardSpec, /F5gz Native window host-loop timer wakeup executor boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopTimerWakeError/);
    assert.match(standardSpec, /RegistrationFailed/);
    assert.match(standardSpec, /FireFailed/);
    assert.match(standardSpec, /F5ha Native window host-loop scheduler timer resume gate boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopSchedulerResumeState/);
    assert.match(standardSpec, /TimerFireResumeRequired/);
    assert.match(standardSpec, /F5he Native minifb frame pacing authority boundary/);
    assert.match(standardSpec, /NativeWindowMinifbFramePacingAuthority/);
    assert.match(standardSpec, /minifb internal `Window::set_target_fps` pacing が active authority/);
    assert.match(standardSpec, /set_target_fps 0/);
    assert.match(standardSpec, /F5hf Native frame interval wait authority mode boundary/);
    assert.match(standardSpec, /NativeWindowFrameIntervalWaitAuthorityMode/);
    assert.match(standardSpec, /HostOwnedDeadlineTimer/);
    assert.match(standardSpec, /ConflictingFrameIntervalAuthorities/);
    assert.match(standardSpec, /FrameIntervalTimerRegistered/);
    assert.match(standardSpec, /FrameIntervalTimerFired/);
    assert.match(standardSpec, /F5hg Native wait owner frame interval authority connection boundary/);
    assert.match(standardSpec, /execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode/);
    assert.match(standardSpec, /FrameIntervalAuthorityFailed/);
    assert.match(standardSpec, /deadline timer registration、clock read、sleeper call、active timer mutation は起こさない/);
    assert.match(standardSpec, /F5hm Native platform wait backend construction gate boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopPlatformWaitBackendSelection/);
    assert.match(standardSpec, /BackendSupportFailed/);
    assert.match(standardSpec, /BackendImplementationUnavailable/);
    assert.match(standardSpec, /actual backend を作ったことにしない/);
    assert.match(standardSpec, /F5hn Native Windows waitable timer message wait raw backend boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopWindowsWaitHandle/);
    assert.match(standardSpec, /NativeWindowHostLoopWindowsWaitRawApi/);
    assert.match(standardSpec, /msg_wait_for_message_raw/);
    assert.match(standardSpec, /MsgWaitForMultipleObjects/);
    assert.match(standardSpec, /F5ho Native single-owner interruptible wait adapter boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter/);
    assert.match(standardSpec, /両者の `Error` type は同一/);
    assert.match(standardSpec, /HostEventReady` から timer-fired evidence は生成しない/);
    assert.match(standardSpec, /F5hp Native Windows platform wait support gate boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopPlatformWaitBackend Api/);
    assert.match(standardSpec, /WindowsWaitBackendFailed/);
    assert.match(standardSpec, /support failure や Windows backend construction failure では host owner を消費しない/);
    assert.match(standardSpec, /F5hs Native Windows platform wait CLI selection boundary/);
    assert.match(standardSpec, /--wait-backend minifb\|platform/);
    assert.match(standardSpec, /headless mode では明示指定を error/);
    assert.match(standardSpec, /non-Windows platform selection は unsupported error/);
    assert.match(standardSpec, /F5hu Native Linux selector timerfd raw backend boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopLinuxSelectorFd/);
    assert.match(standardSpec, /fd `0` は有効/);
    assert.match(standardSpec, /SelectorWaitFailed/);
    assert.match(standardSpec, /F5hv Native Linux selector timerfd single-owner wait trait boundary/);
    assert.match(standardSpec, /single-owner interruptible deadline wait contract/);
    assert.match(standardSpec, /`TimerFired` を `DeadlineReached`/);
    assert.match(standardSpec, /generic `NativeWindowHostLoopPlatformWaitBackend` の Linux owner variant/);
    assert.match(standardSpec, /F5hw Native macOS run loop timer single-owner wait trait boundary/);
    assert.match(standardSpec, /macOS raw wake の `TimerFired` は `DeadlineReached`/);
    assert.match(standardSpec, /host-event-only wait では timer-fired status を host event として受け入れない/);
    assert.match(standardSpec, /generic `NativeWindowHostLoopPlatformWaitBackend` の macOS owner variant/);
    assert.match(standardSpec, /F5hx Native platform wait multi-backend owner boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopPlatformWaitBackend WindowsApi MacosApi LinuxApi/);
    assert.match(standardSpec, /NativeWindowHostLoopPlatformWaitBackendError WindowsError MacosError LinuxError/);
    assert.match(standardSpec, /build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis/);
    assert.match(standardSpec, /selected されていない raw API を触って fallback backend/);
    assert.match(standardSpec, /NativeWindowHostLoopWindowsOnlyPlatformWaitBackend WindowsApi/);
    assert.match(standardSpec, /NativeWindowHostLoopNeverMacosRunLoopTimerRawApi/);
    assert.match(standardSpec, /NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi/);
    assert.match(standardSpec, /trait method body は `match \*self \{\}` のみ/);
    assert.match(standardSpec, /actual sys shim、CoreFoundation \/ AppKit binding/);
    assert.match(standardSpec, /F5hy Native Linux selector timerfd actual sys shim boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopLinuxSelectorTimerFdSysApi/);
    assert.match(standardSpec, /epoll_create1 EPOLL_CLOEXEC/);
    assert.match(standardSpec, /timerfd_create CLOCK_MONOTONIC/);
    assert.match(standardSpec, /read` で `u64` expiration count を exactly drain/);
    assert.match(standardSpec, /F5hz Native Linux host event fd integration boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopLinuxHostEventFd/);
    assert.match(standardSpec, /eventfd 0 EFD_CLOEXEC \| EFD_NONBLOCK/);
    assert.match(standardSpec, /host-event-only wait は host event fd readiness だけを成功/);
    assert.match(standardSpec, /F5ia Native Linux platform wait support helper boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopLinuxOnlyPlatformWaitBackend LinuxApi/);
    assert.match(standardSpec, /build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api/);
    assert.match(standardSpec, /raw API method を呼ぶ前に `BackendSupportFailed`/);
    assert.match(standardSpec, /F5ib Native Linux host event fd producer boundary/);
    assert.match(standardSpec, /signal_host_event_fd_raw/);
    assert.match(standardSpec, /SignalHostEventFdFailed/);
    assert.match(standardSpec, /`u64` 値 `1` を `libc::write` で exactly/);
    assert.match(standardSpec, /F5ic Native Linux host event signal producer handle boundary/);
    assert.match(standardSpec, /NativeWindowHostLoopLinuxHostEventSignalFd/);
    assert.match(standardSpec, /NativeWindowHostLoopLinuxHostEventSignalRawApi/);
    assert.match(standardSpec, /fcntl F_DUPFD_CLOEXEC/);
    assert.match(standardSpec, /pre-signal busy loop/);
    assert.match(standardSpec, /F5id Native Linux minifb observed-input signal bridge boundary/);
    assert.match(standardSpec, /observed keyboard \/ text input/);
    assert.match(standardSpec, /HostEventSignalFailed/);
    assert.match(standardSpec, /DelegateWaitFailed/);
    assert.match(standardSpec, /Linux platform wait runner、CLI dispatch、`run_linux_platform_wait_window_loop`/);
    assert.match(standardSpec, /F5ie Native Linux blocking wait event source capability gate/);
    assert.match(standardSpec, /NativeWindowHostLoopLinuxEventSourceCapability/);
    assert.match(standardSpec, /ObservedInputOnlyUnsupportedForBlockingWait/);
    assert.match(standardSpec, /fd owner、selector registration、wait outcome、runner dispatch/);
    assert.match(standardSpec, /actual X11 \/ Wayland fd integration/);
    assert.match(standardSpec, /F5iq/);
    assert.match(standardSpec, /LinuxWindowEventSourceFdMissing/);
    assert.match(standardSpec, /window event source fd integration/);
    assert.match(standardSpec, /F5if Native Linux platform wait backend event source config gate/);
    assert.match(standardSpec, /LinuxEventSourceSupportFailed/);
    assert.match(standardSpec, /cfg Linux `native_window_host_loop_platform_wait_backend_from_selection` も explicit event source capability を要求/);
    assert.match(standardSpec, /F5ig Native run-loop platform wait config event source gate/);
    assert.match(standardSpec, /NativeWindowRunLoopPlatformWaitBackendConfig/);
    assert.match(standardSpec, /MissingLinuxEventSourceCapability/);
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
            "Native host-loop Continue turns carry typed pump-only or presented-frame evidence",
            "Native bounded host-loop runners count Continue evidence without consuming scheduler policy",
            "Native host-loop wait decisions classify Continue evidence without implementing sleep or queues",
            "Native bounded host-loop budget exhaustion reports the last wait decision",
            "Native host-loop wait dispatch calls the host wait hook and fails closed on missing decisions",
            "Native host-loop scheduler slice exposes bounded run and wait dispatch to external schedulers",
            "Native host-loop wait request plans carry validated frame interval timing without sleep or queues",
            "Native host-loop wait strategy instructions distribute frame interval remainder without sleep or queues",
            "Native host-loop thread wait backend sleeps only frame intervals and rejects host-event waits",
            "Native host-loop timer registration backend registers only frame intervals and validates raw timer ids",
            "Native host-loop timer fire backend validates fired timer ids before wakeup evidence",
            "Native host-loop timer wakeup executor preserves registration and fire failure stages",
            "Native host-loop std deadline timer adapter owns timer state without minifb pacing changes",
            "Native host-loop wait owner dispatches event queue and frame timer paths without minifb pacing changes",
            "Native host-loop scheduler resume gate requires timer fire before resuming registered timers",
            "Native host-loop message pump adapter maps pump success through normalized event status",
            "Native platform wait backend construction gate keeps actual OS backend unavailable fail-closed",
            "Native platform wait backend owns selected Windows, macOS, or Linux raw backend without fallback",
            "Native Windows wait raw backend maps waitable timer and message statuses through typed errors",
            "Native macOS run loop timer raw backend keeps handle and wake semantics typed before integration",
            "Native Linux selector timerfd raw backend keeps fd ownership and wake semantics typed before integration",
            "Native Linux platform wait helper builds the cfg Linux sys backend without runner fallback",
            "Native Linux host event fd producer writes explicit eventfd signals without runner fallback",
            "Native single-owner interruptible wait adapter keeps clock and waiter in one backend owner",
            "Native Linux window event source normalized observation converts through typed snapshot helper",
            "Native Linux window event source observation provider plugs into run-loop event pump",
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
