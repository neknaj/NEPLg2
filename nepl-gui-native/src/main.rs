use std::env;
use std::process::ExitCode;

#[cfg(all(feature = "window", target_os = "windows", not(target_arch = "wasm32")))]
use nepl_gui_native::run_windows_platform_wait_window_loop;
use nepl_gui_native::{checksum_pixels, rasterize_frame, render_demo_frame, GuiDemo};
#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
use nepl_gui_native::{
    native_window_host_loop_default_platform_wait_backend_selection, run_minifb_window_loop,
    validate_native_window_run_loop_platform_wait_runner_support, NativeWindowHostLoopRunPolicy,
    NativeWindowRunLoopConfig,
};
use nepl_gui_native::{
    NativeWindowTargetFps, NativeWindowTargetFpsError, NativeWindowTargetFpsInvalidReason,
};

fn main() -> ExitCode {
    let options = match NativeGuiOptions::parse(env::args().skip(1)) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            print_usage();
            return ExitCode::from(2);
        }
    };

    if options.headless {
        if let Err(error) = validate_headless_options(&options) {
            eprintln!("{error}");
            print_usage();
            return ExitCode::from(2);
        }
        print_headless_frame(options.demo, options.counter_value, options.scale);
        return ExitCode::SUCCESS;
    }

    match run_window(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum NativeGuiWindowWaitBackend {
    Minifb,
    Platform,
}

impl NativeGuiWindowWaitBackend {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw {
            "minifb" => Ok(Self::Minifb),
            "platform" => Ok(Self::Platform),
            other => Err(format!(
                "--wait-backend must be minifb or platform, got {other}"
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NativeGuiOptions {
    demo: GuiDemo,
    scale: usize,
    counter_value: i32,
    target_fps: NativeWindowTargetFps,
    wait_backend: Option<NativeGuiWindowWaitBackend>,
    headless: bool,
}

impl NativeGuiOptions {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self {
            demo: GuiDemo::Mandelbrot,
            scale: 4,
            counter_value: 0,
            target_fps: NativeWindowTargetFps::default(),
            wait_backend: None,
            headless: false,
        };

        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "--headless" => options.headless = true,
                "--scale" => {
                    let Some(raw) = iter.next() else {
                        return Err("--scale requires a value".to_string());
                    };
                    let scale = raw
                        .parse::<usize>()
                        .map_err(|_| "--scale must be a positive integer".to_string())?;
                    if scale == 0 {
                        return Err("--scale must be a positive integer".to_string());
                    }
                    options.scale = scale;
                }
                "--counter" => {
                    let Some(raw) = iter.next() else {
                        return Err("--counter requires a value".to_string());
                    };
                    options.counter_value = raw
                        .parse::<i32>()
                        .map_err(|_| "--counter must be an integer".to_string())?;
                }
                "--fps" => {
                    let Some(raw) = iter.next() else {
                        return Err("--fps requires a value".to_string());
                    };
                    let target_fps = raw
                        .parse::<usize>()
                        .map_err(|_| "--fps must be a positive integer".to_string())?;
                    options.target_fps = NativeWindowTargetFps::new(target_fps)
                        .map_err(native_window_target_fps_cli_error)?;
                }
                "--wait-backend" => {
                    if options.wait_backend.is_some() {
                        return Err("--wait-backend can be provided only once".to_string());
                    }
                    let Some(raw) = iter.next() else {
                        return Err("--wait-backend requires a value".to_string());
                    };
                    options.wait_backend = Some(NativeGuiWindowWaitBackend::parse(&raw)?);
                }
                "mandelbrot" | "life" | "counter" => {
                    options.demo = arg.parse::<GuiDemo>()?;
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }

        Ok(options)
    }

    fn window_wait_backend(&self) -> NativeGuiWindowWaitBackend {
        self.wait_backend
            .unwrap_or(NativeGuiWindowWaitBackend::Minifb)
    }
}

fn validate_headless_options(options: &NativeGuiOptions) -> Result<(), String> {
    if options.wait_backend.is_some() {
        return Err("--wait-backend requires window mode".to_string());
    }
    Ok(())
}

fn print_headless_frame(demo: GuiDemo, counter_value: i32, scale: usize) {
    let frame = render_demo_frame(demo, counter_value);
    let image = rasterize_frame(&frame, scale);
    println!("gui-native demo = {:?}", frame.demo);
    println!("commands = {}", frame.metrics.command_count);
    if let Some(inside) = frame.metrics.inside_count {
        println!("inside = {inside}");
    }
    if let Some(live_cells) = frame.metrics.live_cells {
        println!("live cells = {live_cells}");
    }
    if let Some(checksum) = frame.metrics.checksum {
        println!("life checksum = {checksum}");
    }
    if let Some(value) = frame.metrics.counter_value {
        println!("counter value = {value}");
    }
    if let Some(action) = frame.metrics.action_id {
        println!("counter action = {action}");
    }
    if let Some(target) = frame.metrics.redraw_target {
        println!("counter redraw target = {target}");
    }
    println!("pixels checksum = {}", checksum_pixels(&image.pixels));
}

fn print_usage() {
    eprintln!(
        "usage: nepl-gui-native [mandelbrot|life|counter] [--headless] [--scale N] [--counter N] [--fps N] [--wait-backend minifb|platform]"
    );
    eprintln!("--fps is used by window mode only and must be in the supported target FPS range");
    eprintln!("--wait-backend platform uses the native platform wait runner when supported");
    eprintln!("window mode requires: cargo run -p nepl-gui-native --features window -- <demo>");
}

fn native_window_target_fps_cli_error(error: NativeWindowTargetFpsError) -> String {
    match error.reason {
        NativeWindowTargetFpsInvalidReason::Zero => "--fps must be greater than zero".to_string(),
        NativeWindowTargetFpsInvalidReason::TooHigh { max } => {
            format!("--fps must be less than or equal to {max}")
        }
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn run_window(options: NativeGuiOptions) -> Result<(), String> {
    match options.window_wait_backend() {
        NativeGuiWindowWaitBackend::Minifb => run_minifb_wait_window(options),
        NativeGuiWindowWaitBackend::Platform => run_platform_wait_window(options),
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn run_minifb_wait_window(options: NativeGuiOptions) -> Result<(), String> {
    let config = NativeWindowRunLoopConfig::new_with_target_fps(
        options.demo,
        options.counter_value,
        options.scale,
        options.target_fps,
    );
    run_minifb_window_loop(config)
        .map(|_| ())
        .map_err(|error| format!("native window run loop failed: {error:?}"))
}

#[cfg(all(feature = "window", target_os = "windows", not(target_arch = "wasm32")))]
fn run_platform_wait_window(options: NativeGuiOptions) -> Result<(), String> {
    let config = platform_wait_window_run_loop_config(options)?;
    let config = validate_platform_wait_window_runner_support(config)?;
    run_windows_platform_wait_window_loop(config)
        .map(|_| ())
        .map_err(|error| format!("native platform wait window run loop failed: {error:?}"))
}

#[cfg(all(
    feature = "window",
    not(target_os = "windows"),
    not(target_arch = "wasm32")
))]
fn run_platform_wait_window(options: NativeGuiOptions) -> Result<(), String> {
    let config = platform_wait_window_run_loop_config(options)?;
    let _config = validate_platform_wait_window_runner_support(config)?;
    Err("native platform wait runner dispatch is unavailable for this target after support validation".to_string())
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn platform_wait_window_run_loop_config(
    options: NativeGuiOptions,
) -> Result<NativeWindowRunLoopConfig, String> {
    let selection = native_window_host_loop_default_platform_wait_backend_selection()
        .map_err(|error| format!("native platform wait backend selection failed: {error:?}"))?;
    Ok(
        NativeWindowRunLoopConfig::new_with_platform_wait_backend_selection(
            options.demo,
            options.counter_value,
            options.scale,
            options.target_fps,
            NativeWindowHostLoopRunPolicy::default(),
            selection,
        ),
    )
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn validate_platform_wait_window_runner_support(
    config: NativeWindowRunLoopConfig,
) -> Result<NativeWindowRunLoopConfig, String> {
    validate_native_window_run_loop_platform_wait_runner_support(config)
        .map(|_| config)
        .map_err(|error| format!("native platform wait runner unsupported: {error:?}"))
}

#[cfg(any(not(feature = "window"), target_arch = "wasm32"))]
fn run_window(_options: NativeGuiOptions) -> Result<(), String> {
    Err("native window mode requires the non-wasm window feature; use --headless or run with --features window".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_options(args: &[&str]) -> Result<NativeGuiOptions, String> {
        NativeGuiOptions::parse(args.iter().map(|arg| (*arg).to_string()))
    }

    #[test]
    fn window_wait_backend_defaults_to_minifb_for_window_mode() {
        let options = parse_options(&["life"]).unwrap();
        assert_eq!(
            options.window_wait_backend(),
            NativeGuiWindowWaitBackend::Minifb
        );
    }

    #[test]
    fn parse_wait_backend_platform_records_explicit_window_backend() {
        let options = parse_options(&["counter", "--wait-backend", "platform"]).unwrap();
        assert_eq!(
            options.wait_backend,
            Some(NativeGuiWindowWaitBackend::Platform)
        );
    }

    #[test]
    fn parse_rejects_duplicate_wait_backend() {
        let error =
            parse_options(&["--wait-backend", "minifb", "--wait-backend", "platform"]).unwrap_err();
        assert_eq!(error, "--wait-backend can be provided only once");
    }

    #[test]
    fn parse_rejects_unknown_wait_backend() {
        let error = parse_options(&["--wait-backend", "thread"]).unwrap_err();
        assert_eq!(
            error,
            "--wait-backend must be minifb or platform, got thread"
        );
    }

    #[test]
    fn headless_rejects_explicit_wait_backend() {
        let options = parse_options(&["--headless", "--wait-backend", "minifb"]).unwrap();
        assert_eq!(
            validate_headless_options(&options).unwrap_err(),
            "--wait-backend requires window mode"
        );
    }

    #[test]
    fn headless_allows_unspecified_wait_backend() {
        let options = parse_options(&["--headless"]).unwrap();
        validate_headless_options(&options).unwrap();
    }

    #[cfg(all(feature = "window", not(target_arch = "wasm32")))]
    #[test]
    fn platform_wait_config_builder_uses_platform_wait_backend() {
        let options = parse_options(&["counter", "--wait-backend", "platform"]).unwrap();
        let config = platform_wait_window_run_loop_config(options).unwrap();

        assert_eq!(config.demo, GuiDemo::Counter);
        assert_eq!(config.counter_value, 0);
        assert_eq!(config.scale, 4);
        assert!(matches!(
            config.wait_backend,
            nepl_gui_native::NativeWindowRunLoopWaitBackend::PlatformWait(_)
        ));
    }
}
