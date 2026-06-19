use std::env;
use std::process::ExitCode;

use nepl_gui_native::{checksum_pixels, rasterize_frame, render_demo_frame, GuiDemo};
#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
use nepl_gui_native::{
    poll_minifb_window_event_pump, NativeWindowBackendLoop, NativeWindowHostAction,
    NativeWindowSize,
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
struct NativeGuiOptions {
    demo: GuiDemo,
    scale: usize,
    counter_value: i32,
    headless: bool,
}

impl NativeGuiOptions {
    fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut options = Self {
            demo: GuiDemo::Mandelbrot,
            scale: 4,
            counter_value: 0,
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
        "usage: nepl-gui-native [mandelbrot|life|counter] [--headless] [--scale N] [--counter N]"
    );
    eprintln!("window mode requires: cargo run -p nepl-gui-native --features window -- <demo>");
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn run_window(options: NativeGuiOptions) -> Result<(), String> {
    use minifb::{ScaleMode, Window, WindowOptions};

    let mut backend_loop =
        NativeWindowBackendLoop::new_for_scale(options.demo, options.counter_value, options.scale)
            .map_err(|error| {
                format!("native window backend loop initialization failed: {error:?}")
            })?;
    let initial_size = backend_loop.initial_size();
    let mut window = Window::new(
        "NEPLg2 GUI native preview",
        initial_size.width,
        initial_size.height,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::UpperLeft,
            ..WindowOptions::default()
        },
    )
    .map_err(|error| error.to_string())?;
    window.set_target_fps(60);
    window.set_background_color(9, 13, 18);
    update_window_title(&mut window, backend_loop.demo(), initial_size);

    loop {
        let event_snapshot =
            poll_minifb_window_event_pump(&window, backend_loop.event_pump_input())
                .map_err(|error| format!("native window event pump rejected input: {error:?}"))?;
        let action = backend_loop
            .step_host_action(event_snapshot)
            .map_err(|error| format!("native window host action step failed: {error:?}"))?;
        match action {
            NativeWindowHostAction::Terminate { .. } => break,
            NativeWindowHostAction::PumpEventsOnly {
                window_size,
                size_changed,
            } => {
                if size_changed {
                    update_window_title(&mut window, backend_loop.demo(), window_size);
                }
                window.update();
                continue;
            }
            NativeWindowHostAction::PresentFrame {
                window_size,
                size_changed,
                ..
            } => {
                if size_changed {
                    update_window_title(&mut window, backend_loop.demo(), window_size);
                }
            }
        }
        let present_frame = backend_loop
            .current_present_frame_for_window()
            .map_err(|error| {
                format!("native window backend loop has no drawable frame: {error:?}")
            })?;
        window
            .update_with_buffer(
                present_frame.pixels(),
                present_frame.width(),
                present_frame.height(),
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn update_window_title(window: &mut minifb::Window, demo: GuiDemo, size: NativeWindowSize) {
    let title = if size.width == 0 || size.height == 0 {
        format!(
            "NEPLg2 GUI native preview - {:?} - surface unavailable",
            demo
        )
    } else {
        format!(
            "NEPLg2 GUI native preview - {:?} - {}x{}",
            demo, size.width, size.height
        )
    };
    window.set_title(&title);
}

#[cfg(any(not(feature = "window"), target_arch = "wasm32"))]
fn run_window(_options: NativeGuiOptions) -> Result<(), String> {
    Err("native window mode requires the non-wasm window feature; use --headless or run with --features window".to_string())
}
