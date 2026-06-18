use std::str::FromStr;
use std::time::Instant;

pub const GUI_NATIVE_BACKEND_CLOCK_I32_MAX_MS: u128 = 2_147_483_647;
pub const GUI_NATIVE_BACKEND_CLOCK_STATUS_UNSUPPORTED: i32 = -1;
pub const GUI_NATIVE_BACKEND_CLOCK_STATUS_BACKEND_FAILURE: i32 = -2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuiDemo {
    Mandelbrot,
    Life,
    Counter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RectCommand {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub color: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuiMetrics {
    pub command_count: usize,
    pub inside_count: Option<usize>,
    pub live_cells: Option<usize>,
    pub checksum: Option<usize>,
    pub counter_value: Option<i32>,
    pub action_id: Option<i32>,
    pub redraw_target: Option<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuiFrame {
    pub demo: GuiDemo,
    pub width: usize,
    pub height: usize,
    pub rects: Vec<RectCommand>,
    pub metrics: GuiMetrics,
    pub counter_hit_target: Option<RectCommand>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RasterImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSurfacePlacement {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSurfaceState {
    Drawable(NativeSurfacePlacement),
    Unavailable,
}

pub fn native_monotonic_clock_ms_from_elapsed_ms(elapsed_ms: u128) -> i32 {
    if elapsed_ms > GUI_NATIVE_BACKEND_CLOCK_I32_MAX_MS {
        GUI_NATIVE_BACKEND_CLOCK_STATUS_BACKEND_FAILURE
    } else {
        elapsed_ms as i32
    }
}

pub fn native_monotonic_clock_ms_since(start: &Instant) -> i32 {
    native_monotonic_clock_ms_from_elapsed_ms(start.elapsed().as_millis())
}

impl FromStr for GuiDemo {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "mandelbrot" => Ok(Self::Mandelbrot),
            "life" => Ok(Self::Life),
            "counter" => Ok(Self::Counter),
            other => Err(format!("unknown GUI demo: {other}")),
        }
    }
}

pub fn render_demo_frame(demo: GuiDemo, counter_value: i32) -> GuiFrame {
    match demo {
        GuiDemo::Mandelbrot => render_mandelbrot_frame(),
        GuiDemo::Life => render_life_frame(),
        GuiDemo::Counter => render_counter_frame(counter_value),
    }
}

pub fn rasterize_frame(frame: &GuiFrame, scale: usize) -> RasterImage {
    let scale = scale.max(1);
    let width = frame.width * scale;
    let height = frame.height * scale;
    let mut pixels = vec![0x0d1117; width * height];

    for rect in &frame.rects {
        fill_rect(&mut pixels, width, height, scale, rect);
    }

    RasterImage {
        width,
        height,
        pixels,
    }
}

pub fn checksum_pixels(pixels: &[u32]) -> u64 {
    pixels.iter().fold(0xcbf29ce484222325, |hash, pixel| {
        let mixed = hash ^ u64::from(*pixel);
        mixed.wrapping_mul(0x100000001b3)
    })
}

pub fn counter_hit(frame: &GuiFrame, scene_x: usize, scene_y: usize) -> bool {
    let Some(target) = frame.counter_hit_target else {
        return false;
    };
    scene_x >= target.x
        && scene_x < target.x + target.width
        && scene_y >= target.y
        && scene_y < target.y + target.height
}

pub fn native_aspect_ratio_placement(
    window_width: usize,
    window_height: usize,
    image_width: usize,
    image_height: usize,
) -> NativeSurfaceState {
    if window_width == 0 || window_height == 0 || image_width == 0 || image_height == 0 {
        return NativeSurfaceState::Unavailable;
    }

    let window_wide = (window_width as u128) * (image_height as u128)
        > (window_height as u128) * (image_width as u128);
    let (width, height) = if window_wide {
        let height = window_height;
        let width = ((height as u128) * (image_width as u128) / (image_height as u128))
            .max(1)
            .min(window_width as u128) as usize;
        (width, height)
    } else {
        let width = window_width;
        let height = ((width as u128) * (image_height as u128) / (image_width as u128))
            .max(1)
            .min(window_height as u128) as usize;
        (width, height)
    };

    NativeSurfaceState::Drawable(NativeSurfacePlacement {
        x: (window_width - width) / 2,
        y: (window_height - height) / 2,
        width,
        height,
    })
}

pub fn map_native_window_point_to_image(
    window_width: usize,
    window_height: usize,
    image_width: usize,
    image_height: usize,
    point_x: f32,
    point_y: f32,
) -> Option<(usize, usize)> {
    if !point_x.is_finite() || !point_y.is_finite() || point_x < 0.0 || point_y < 0.0 {
        return None;
    }
    let NativeSurfaceState::Drawable(placement) =
        native_aspect_ratio_placement(window_width, window_height, image_width, image_height)
    else {
        return None;
    };

    let x = point_x.floor() as usize;
    let y = point_y.floor() as usize;
    if x < placement.x
        || y < placement.y
        || x >= placement.x + placement.width
        || y >= placement.y + placement.height
    {
        return None;
    }

    let image_x = ((x - placement.x) as u128) * (image_width as u128) / (placement.width as u128);
    let image_y = ((y - placement.y) as u128) * (image_height as u128) / (placement.height as u128);
    Some((
        (image_x as usize).min(image_width - 1),
        (image_y as usize).min(image_height - 1),
    ))
}

fn render_mandelbrot_frame() -> GuiFrame {
    let width = 8;
    let height = 8;
    let cell_size = 18;
    let mut rects = Vec::with_capacity(width * height);
    let mut inside_count = 0;

    for y in 0..height {
        for x in 0..width {
            let iter = mandelbrot_cell_iter(x as i32, y as i32);
            if iter == mandelbrot_limit() {
                inside_count += 1;
            }
            rects.push(RectCommand {
                x: x * cell_size,
                y: y * cell_size,
                width: cell_size,
                height: cell_size,
                color: mandelbrot_color(iter),
            });
        }
    }

    GuiFrame {
        demo: GuiDemo::Mandelbrot,
        width: width * cell_size,
        height: height * cell_size,
        metrics: GuiMetrics {
            command_count: rects.len(),
            inside_count: Some(inside_count),
            live_cells: None,
            checksum: None,
            counter_value: None,
            action_id: None,
            redraw_target: None,
        },
        rects,
        counter_hit_target: None,
    }
}

fn mandelbrot_limit() -> i32 {
    24
}

fn mandelbrot_cell_iter(x: i32, y: i32) -> i32 {
    let cx = x * 50 - 200;
    let cy = y * 50 - 175;
    let mut zx = 0;
    let mut zy = 0;
    let mut iter = 0;

    while iter < mandelbrot_limit() {
        let zx2 = zx * zx;
        let zy2 = zy * zy;
        if zx2 + zy2 >= 40000 {
            break;
        }
        let next_zx = zx2 / 100 - zy2 / 100 + cx;
        let next_zy = zx * zy * 2 / 100 + cy;
        zx = next_zx;
        zy = next_zy;
        iter += 1;
    }

    iter
}

fn mandelbrot_color(iter: i32) -> u32 {
    if iter == mandelbrot_limit() {
        return 0x000000;
    }
    let shade = (iter * 10).clamp(0, 255) as u32;
    rgb(shade, shade, 255)
}

fn render_life_frame() -> GuiFrame {
    let width = 5;
    let height = 5;
    let cell_size = 28;
    let step = 3;
    let mut rects = Vec::with_capacity(width * height);
    let mut live_cells = 0;
    let mut checksum = 0;

    for y in 0..height {
        for x in 0..width {
            let alive = life_cell_at_step(x as i32, y as i32, step);
            if alive {
                live_cells += 1;
                checksum += (x + 1) * (y + 1);
            }
            rects.push(RectCommand {
                x: x * cell_size,
                y: y * cell_size,
                width: cell_size - 2,
                height: cell_size - 2,
                color: if alive { 0x00b4b4 } else { 0x181818 },
            });
        }
    }

    GuiFrame {
        demo: GuiDemo::Life,
        width: width * cell_size - 2,
        height: height * cell_size - 2,
        metrics: GuiMetrics {
            command_count: rects.len(),
            inside_count: None,
            live_cells: Some(live_cells),
            checksum: Some(checksum),
            counter_value: None,
            action_id: None,
            redraw_target: None,
        },
        rects,
        counter_hit_target: None,
    }
}

fn life_initial_cell(x: i32, y: i32) -> bool {
    (x == 1 && y == 0)
        || (x == 2 && y == 1)
        || (x == 0 && y == 2)
        || (x == 1 && y == 2)
        || (x == 2 && y == 2)
}

fn life_cell_at_step(x: i32, y: i32, step: i32) -> bool {
    if !(0..5).contains(&x) || !(0..5).contains(&y) {
        return false;
    }
    let mut grid = [[false; 5]; 5];
    for row in 0..5 {
        for col in 0..5 {
            grid[row][col] = life_initial_cell(col as i32, row as i32);
        }
    }
    for _ in 0..step {
        let mut next = [[false; 5]; 5];
        for row in 0..5 {
            for col in 0..5 {
                let alive = grid[row][col];
                let neighbors = life_neighbor_count(&grid, col as i32, row as i32);
                next[row][col] = if alive {
                    neighbors == 2 || neighbors == 3
                } else {
                    neighbors == 3
                };
            }
        }
        grid = next;
    }
    grid[y as usize][x as usize]
}

fn life_neighbor_count(grid: &[[bool; 5]; 5], x: i32, y: i32) -> i32 {
    let mut count = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;
            if (0..5).contains(&nx) && (0..5).contains(&ny) && grid[ny as usize][nx as usize] {
                count += 1;
            }
        }
    }
    count
}

fn render_counter_frame(counter_value: i32) -> GuiFrame {
    let value = counter_value.max(0);
    let mut rects = vec![
        RectCommand {
            x: 0,
            y: 0,
            width: 220,
            height: 142,
            color: 0x101820,
        },
        RectCommand {
            x: 18,
            y: 20,
            width: 184,
            height: 50,
            color: 0x1d2b35,
        },
        RectCommand {
            x: 18,
            y: 88,
            width: 184,
            height: 34,
            color: 0x2d7d6f,
        },
    ];
    push_digit_rects(&mut rects, value, 92, 28, 6, 0xf2f7f5);
    let hit_target = RectCommand {
        x: 18,
        y: 88,
        width: 184,
        height: 34,
        color: 0,
    };

    GuiFrame {
        demo: GuiDemo::Counter,
        width: 220,
        height: 142,
        metrics: GuiMetrics {
            command_count: rects.len(),
            inside_count: None,
            live_cells: None,
            checksum: None,
            counter_value: Some(value),
            action_id: Some(1),
            redraw_target: Some(0),
        },
        rects,
        counter_hit_target: Some(hit_target),
    }
}

fn push_digit_rects(
    rects: &mut Vec<RectCommand>,
    value: i32,
    x: usize,
    y: usize,
    thickness: usize,
    color: u32,
) {
    let digit = (value % 10) as usize;
    let segments = [
        [true, true, true, true, true, true, false],
        [false, true, true, false, false, false, false],
        [true, true, false, true, true, false, true],
        [true, true, true, true, false, false, true],
        [false, true, true, false, false, true, true],
        [true, false, true, true, false, true, true],
        [true, false, true, true, true, true, true],
        [true, true, true, false, false, false, false],
        [true, true, true, true, true, true, true],
        [true, true, true, true, false, true, true],
    ][digit];
    let width = 34;
    let height = 40;
    let specs = [
        RectCommand {
            x: x + thickness,
            y,
            width: width - thickness * 2,
            height: thickness,
            color,
        },
        RectCommand {
            x: x + width - thickness,
            y: y + thickness,
            width: thickness,
            height: height / 2 - thickness,
            color,
        },
        RectCommand {
            x: x + width - thickness,
            y: y + height / 2,
            width: thickness,
            height: height / 2 - thickness,
            color,
        },
        RectCommand {
            x: x + thickness,
            y: y + height - thickness,
            width: width - thickness * 2,
            height: thickness,
            color,
        },
        RectCommand {
            x,
            y: y + height / 2,
            width: thickness,
            height: height / 2 - thickness,
            color,
        },
        RectCommand {
            x,
            y: y + thickness,
            width: thickness,
            height: height / 2 - thickness,
            color,
        },
        RectCommand {
            x: x + thickness,
            y: y + height / 2 - thickness / 2,
            width: width - thickness * 2,
            height: thickness,
            color,
        },
    ];
    for (enabled, rect) in segments.into_iter().zip(specs) {
        if enabled {
            rects.push(rect);
        }
    }
}

fn fill_rect(pixels: &mut [u32], width: usize, height: usize, scale: usize, rect: &RectCommand) {
    let x0 = rect.x * scale;
    let y0 = rect.y * scale;
    let x1 = (rect.x + rect.width).saturating_mul(scale).min(width);
    let y1 = (rect.y + rect.height).saturating_mul(scale).min(height);
    for y in y0..y1 {
        let row = y * width;
        for x in x0..x1 {
            pixels[row + x] = rect.color;
        }
    }
}

fn rgb(red: u32, green: u32, blue: u32) -> u32 {
    (red << 16) | (green << 8) | blue
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mandelbrot_metrics_match_gui_example_contract() {
        let frame = render_demo_frame(GuiDemo::Mandelbrot, 0);
        assert_eq!(frame.metrics.command_count, 64);
        assert_eq!(frame.metrics.inside_count, Some(8));
    }

    #[test]
    fn life_metrics_match_gui_example_contract() {
        let frame = render_demo_frame(GuiDemo::Life, 0);
        assert_eq!(frame.metrics.command_count, 25);
        assert_eq!(frame.metrics.live_cells, Some(5));
        assert_eq!(frame.metrics.checksum, Some(45));
    }

    #[test]
    fn counter_keeps_action_and_redraw_contract() {
        let frame = render_demo_frame(GuiDemo::Counter, 2);
        assert_eq!(frame.metrics.counter_value, Some(2));
        assert_eq!(frame.metrics.action_id, Some(1));
        assert_eq!(frame.metrics.redraw_target, Some(0));
        assert!(counter_hit(&frame, 20, 90));
        assert!(!counter_hit(&frame, 1, 1));
    }

    #[test]
    fn rasterize_checksum_is_stable() {
        let frame = render_demo_frame(GuiDemo::Mandelbrot, 0);
        let image = rasterize_frame(&frame, 2);
        assert_eq!(image.width, 288);
        assert_eq!(image.height, 288);
        assert_eq!(checksum_pixels(&image.pixels), 17_705_978_859_225_436_581);
    }

    #[test]
    fn native_surface_placement_preserves_aspect_ratio_inside_window() {
        let state = native_aspect_ratio_placement(800, 600, 400, 400);
        assert_eq!(
            state,
            NativeSurfaceState::Drawable(NativeSurfacePlacement {
                x: 100,
                y: 0,
                width: 600,
                height: 600,
            })
        );
    }

    #[test]
    fn native_surface_placement_reports_unavailable_zero_surface() {
        assert_eq!(
            native_aspect_ratio_placement(0, 600, 400, 400),
            NativeSurfaceState::Unavailable
        );
        assert_eq!(
            native_aspect_ratio_placement(800, 600, 0, 400),
            NativeSurfaceState::Unavailable
        );
    }

    #[test]
    fn native_window_point_mapping_rejects_letterbox_and_maps_to_image() {
        assert_eq!(
            map_native_window_point_to_image(800, 600, 400, 400, 99.0, 10.0),
            None
        );
        assert_eq!(
            map_native_window_point_to_image(800, 600, 400, 400, 100.0, 0.0),
            Some((0, 0))
        );
        assert_eq!(
            map_native_window_point_to_image(800, 600, 400, 400, 699.0, 599.0),
            Some((399, 399))
        );
    }

    #[test]
    fn native_window_point_mapping_handles_shrunken_window() {
        assert_eq!(
            map_native_window_point_to_image(100, 50, 400, 200, 99.0, 49.0),
            Some((396, 196))
        );
    }

    #[test]
    fn native_window_point_mapping_rejects_top_bottom_letterbox() {
        assert_eq!(
            map_native_window_point_to_image(600, 800, 400, 400, 10.0, 99.0),
            None
        );
        assert_eq!(
            map_native_window_point_to_image(600, 800, 400, 400, 0.0, 100.0),
            Some((0, 0))
        );
        assert_eq!(
            map_native_window_point_to_image(600, 800, 400, 400, 599.0, 699.0),
            Some((399, 399))
        );
    }

    #[test]
    fn native_window_point_mapping_rejects_unavailable_and_invalid_points() {
        assert_eq!(
            map_native_window_point_to_image(0, 600, 400, 400, 10.0, 10.0),
            None
        );
        assert_eq!(
            map_native_window_point_to_image(800, 600, 400, 400, -1.0, 10.0),
            None
        );
        assert_eq!(
            map_native_window_point_to_image(800, 600, 400, 400, f32::NAN, 10.0),
            None
        );
        assert_eq!(
            map_native_window_point_to_image(800, 600, 400, 400, f32::INFINITY, 10.0),
            None
        );
    }

    #[test]
    fn native_monotonic_clock_elapsed_conversion_checks_i32_range() {
        assert_eq!(native_monotonic_clock_ms_from_elapsed_ms(0), 0);
        assert_eq!(
            native_monotonic_clock_ms_from_elapsed_ms(GUI_NATIVE_BACKEND_CLOCK_I32_MAX_MS),
            i32::MAX
        );
        assert_eq!(
            native_monotonic_clock_ms_from_elapsed_ms(GUI_NATIVE_BACKEND_CLOCK_I32_MAX_MS + 1),
            GUI_NATIVE_BACKEND_CLOCK_STATUS_BACKEND_FAILURE
        );
    }

    #[test]
    fn native_monotonic_clock_since_uses_instant_source() {
        let start = Instant::now();
        let sample = native_monotonic_clock_ms_since(&start);
        assert!(sample >= 0);
    }
}
