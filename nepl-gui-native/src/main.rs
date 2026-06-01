use std::env;
use std::process::ExitCode;

use nepl_gui_native::{checksum_pixels, rasterize_frame, render_demo_frame, GuiDemo};

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
                    options.scale = raw
                        .parse::<usize>()
                        .map_err(|_| "--scale must be a positive integer".to_string())?
                        .max(1);
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
fn run_window(mut options: NativeGuiOptions) -> Result<(), String> {
    use minifb::{Key, MouseButton, MouseMode, Window, WindowOptions};

    let mut frame = render_demo_frame(options.demo, options.counter_value);
    let mut image = rasterize_frame(&frame, options.scale);
    let mut window = Window::new(
        "NEPLg2 GUI native preview",
        image.width,
        image.height,
        WindowOptions::default(),
    )
    .map_err(|error| error.to_string())?;
    let mut previous_mouse_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if options.demo == GuiDemo::Counter {
            let mouse_down = window.get_mouse_down(MouseButton::Left);
            if mouse_down && !previous_mouse_down {
                if let Some((mouse_x, mouse_y)) = window.get_mouse_pos(MouseMode::Clamp) {
                    let scene_x = (mouse_x.max(0.0) as usize) / options.scale.max(1);
                    let scene_y = (mouse_y.max(0.0) as usize) / options.scale.max(1);
                    if nepl_gui_native::counter_hit(&frame, scene_x, scene_y) {
                        options.counter_value += 1;
                        frame = render_demo_frame(options.demo, options.counter_value);
                        image = rasterize_frame(&frame, options.scale);
                    }
                }
            }
            previous_mouse_down = mouse_down;
        }
        window
            .update_with_buffer(&image.pixels, image.width, image.height)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

#[cfg(any(not(feature = "window"), target_arch = "wasm32"))]
fn run_window(_options: NativeGuiOptions) -> Result<(), String> {
    Err("native window mode requires the non-wasm window feature; use --headless or run with --features window".to_string())
}
