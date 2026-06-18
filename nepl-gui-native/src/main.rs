use std::env;
use std::process::ExitCode;

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
use nepl_gui_native::map_native_window_point_to_image;
use nepl_gui_native::{checksum_pixels, rasterize_frame, render_demo_frame, GuiDemo};
#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
use nepl_gui_native::{NativeRgb0PresentBuffer, NativeWindowPresenterState};

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

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn rasterize_demo_present_buffer(
    frame: &nepl_gui_native::GuiFrame,
    scale: usize,
) -> Result<NativeRgb0PresentBuffer, String> {
    let image = rasterize_frame(frame, scale);
    NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(
        image.width,
        image.height,
        image.pixels,
    )
    .map_err(|error| format!("native presenter RGB0 frame invalid: {error:?}"))
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn present_buffer_size(buffer: &NativeRgb0PresentBuffer) -> Result<(usize, usize), String> {
    if buffer.width() <= 0 || buffer.height() <= 0 {
        return Err("native presenter RGB0 frame has invalid dimensions".to_string());
    }
    let width = usize::try_from(buffer.width())
        .map_err(|_| "native presenter RGB0 width overflow".to_string())?;
    let height = usize::try_from(buffer.height())
        .map_err(|_| "native presenter RGB0 height overflow".to_string())?;
    Ok((width, height))
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn present_demo_frame_to_window_state(
    presenter_state: &mut NativeWindowPresenterState,
    frame_id: i32,
    frame: &nepl_gui_native::GuiFrame,
    scale: usize,
) -> Result<(), String> {
    let buffer = rasterize_demo_present_buffer(frame, scale)?;
    presenter_state
        .present_buffer(frame_id, &buffer)
        .map_err(|error| format!("native window presenter rejected frame: {error:?}"))
}

fn print_usage() {
    eprintln!(
        "usage: nepl-gui-native [mandelbrot|life|counter] [--headless] [--scale N] [--counter N]"
    );
    eprintln!("window mode requires: cargo run -p nepl-gui-native --features window -- <demo>");
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn run_window(mut options: NativeGuiOptions) -> Result<(), String> {
    use minifb::{Key, MouseButton, MouseMode, ScaleMode, Window, WindowOptions};

    let mut frame = render_demo_frame(options.demo, options.counter_value);
    let initial_buffer = rasterize_demo_present_buffer(&frame, options.scale)?;
    let initial_size = present_buffer_size(&initial_buffer)?;
    let mut presenter_state = NativeWindowPresenterState::new(initial_size.0, initial_size.1)
        .map_err(|error| format!("native window presenter rejected surface: {error:?}"))?;
    let mut presenter_frame_id = 1;
    presenter_state
        .present_buffer(presenter_frame_id, &initial_buffer)
        .map_err(|error| format!("native window presenter rejected frame: {error:?}"))?;
    let mut window = Window::new(
        "NEPLg2 GUI native preview",
        initial_size.0,
        initial_size.1,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .map_err(|error| error.to_string())?;
    window.set_target_fps(60);
    window.set_background_color(9, 13, 18);
    let mut previous_size = window.get_size();
    update_window_title(&mut window, options.demo, previous_size);
    let mut previous_mouse_down = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_size = window.get_size();
        if current_size != previous_size {
            previous_size = current_size;
            presenter_state
                .resize_surface(current_size.0, current_size.1)
                .map_err(|error| format!("native window presenter rejected resize: {error:?}"))?;
            update_window_title(&mut window, options.demo, current_size);
        }

        if options.demo == GuiDemo::Counter {
            let mouse_down = window.get_mouse_down(MouseButton::Left);
            if mouse_down && !previous_mouse_down {
                let counter_hit = if let Some((mouse_x, mouse_y)) =
                    window.get_unscaled_mouse_pos(MouseMode::Discard)
                {
                    let present_frame =
                        presenter_state
                            .last_present_frame_required()
                            .map_err(|error| {
                                format!("native window presenter has no frame: {error:?}")
                            })?;
                    match map_native_window_point_to_image(
                        current_size.0,
                        current_size.1,
                        present_frame.width(),
                        present_frame.height(),
                        mouse_x,
                        mouse_y,
                    ) {
                        Some((image_x, image_y)) => {
                            let scene_x = image_x / options.scale.max(1);
                            let scene_y = image_y / options.scale.max(1);
                            nepl_gui_native::counter_hit(&frame, scene_x, scene_y)
                        }
                        None => false,
                    }
                } else {
                    false
                };
                if counter_hit {
                    options.counter_value = options
                        .counter_value
                        .checked_add(1)
                        .ok_or_else(|| "counter value overflow".to_string())?;
                    presenter_frame_id = presenter_frame_id
                        .checked_add(1)
                        .ok_or_else(|| "native presenter frame id overflow".to_string())?;
                    frame = render_demo_frame(options.demo, options.counter_value);
                    present_demo_frame_to_window_state(
                        &mut presenter_state,
                        presenter_frame_id,
                        &frame,
                        options.scale,
                    )?;
                }
            }
            previous_mouse_down = mouse_down;
        }
        let present_frame = presenter_state
            .last_present_frame_required()
            .map_err(|error| format!("native window presenter has no frame: {error:?}"))?;
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
fn update_window_title(window: &mut minifb::Window, demo: GuiDemo, size: (usize, usize)) {
    let title = if size.0 == 0 || size.1 == 0 {
        format!(
            "NEPLg2 GUI native preview - {:?} - surface unavailable",
            demo
        )
    } else {
        format!(
            "NEPLg2 GUI native preview - {:?} - {}x{}",
            demo, size.0, size.1
        )
    };
    window.set_title(&title);
}

#[cfg(any(not(feature = "window"), target_arch = "wasm32"))]
fn run_window(_options: NativeGuiOptions) -> Result<(), String> {
    Err("native window mode requires the non-wasm window feature; use --headless or run with --features window".to_string())
}
