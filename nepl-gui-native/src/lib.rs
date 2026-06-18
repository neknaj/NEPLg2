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

pub const GUI_NATIVE_SPAN_OPERATION_STATUS_OK: i32 = 0;
pub const GUI_NATIVE_SPAN_OPERATION_STATUS_UNSUPPORTED: i32 = -1;
pub const GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT: i32 = -2;
pub const GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED: i32 = -3;
pub const GUI_NATIVE_SPAN_OPERATION_STATUS_NO_WRITABLE_SLOT: i32 = -4;
pub const GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE: i32 = -5;
pub const GUI_NATIVE_SPAN_OPERATION_STATUS_STALE_FRAME: i32 = -6;

pub const GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW: i32 = 1;
pub const GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_OFFSCREEN: i32 = 2;
pub const GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_DEVICE: i32 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSpanOperationTarget {
    Window { window_id: i32 },
    Offscreen,
    Device,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSpanOperationDescriptor {
    pub target: NativeSpanOperationTarget,
    pub surface_id: i32,
    pub frame_id: i32,
    pub packet_frame_id: i32,
    pub batch_index: i32,
    pub tile_index: i32,
    pub plan_row_start: i32,
    pub plan_row_count: i32,
    pub row_start: i32,
    pub row_count: i32,
    pub width: i32,
    pub height: i32,
    pub stride_bytes: i32,
    pub tile_rows: i32,
    pub tile_count: i32,
    pub pixel_count: i32,
    pub total_run_count: i32,
    pub encoded_byte_count: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSpanOperationRunSpan {
    pub target: NativeSpanOperationTarget,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSpanOperation {
    Begin(NativeSpanOperationDescriptor),
    RunSpan(NativeSpanOperationRunSpan),
    End(NativeSpanOperationDescriptor),
}

/// Receives already validated native span operations from the Wasm host ABI.
pub trait NativeSpanOperationSink {
    fn execute_span_operation(&mut self, operation: NativeSpanOperation) -> i32;
}

pub fn normalize_native_span_operation_status(status: i32) -> i32 {
    match status {
        GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        | GUI_NATIVE_SPAN_OPERATION_STATUS_UNSUPPORTED
        | GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        | GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED
        | GUI_NATIVE_SPAN_OPERATION_STATUS_NO_WRITABLE_SLOT
        | GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE
        | GUI_NATIVE_SPAN_OPERATION_STATUS_STALE_FRAME => status,
        _ => GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE,
    }
}

pub fn execute_native_span_operation_begin<S: NativeSpanOperationSink>(
    sink: &mut S,
    target_kind: i32,
    window_raw: i32,
    surface_raw: i32,
    frame_raw: i32,
    packet_frame_id: i32,
    batch_index: i32,
    tile_index: i32,
    plan_row_start: i32,
    plan_row_count: i32,
    row_start: i32,
    row_count: i32,
    width: i32,
    height: i32,
    stride_bytes: i32,
    tile_rows: i32,
    tile_count: i32,
    pixel_count: i32,
    total_run_count: i32,
    encoded_byte_count: i32,
) -> i32 {
    let descriptor = match validate_native_span_operation_descriptor(
        target_kind,
        window_raw,
        surface_raw,
        frame_raw,
        packet_frame_id,
        batch_index,
        tile_index,
        plan_row_start,
        plan_row_count,
        row_start,
        row_count,
        width,
        height,
        stride_bytes,
        tile_rows,
        tile_count,
        pixel_count,
        total_run_count,
        encoded_byte_count,
    ) {
        Ok(descriptor) => descriptor,
        Err(status) => return status,
    };

    normalize_native_span_operation_status(
        sink.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
    )
}

pub fn execute_native_span_operation_run<S: NativeSpanOperationSink>(
    sink: &mut S,
    target_kind: i32,
    window_raw: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    r: i32,
    g: i32,
    b: i32,
    a: i32,
) -> i32 {
    let run_span = match validate_native_span_operation_run_span(
        target_kind,
        window_raw,
        x,
        y,
        width,
        height,
        r,
        g,
        b,
        a,
    ) {
        Ok(run_span) => run_span,
        Err(status) => return status,
    };

    normalize_native_span_operation_status(
        sink.execute_span_operation(NativeSpanOperation::RunSpan(run_span)),
    )
}

pub fn execute_native_span_operation_end<S: NativeSpanOperationSink>(
    sink: &mut S,
    target_kind: i32,
    window_raw: i32,
    surface_raw: i32,
    frame_raw: i32,
    packet_frame_id: i32,
    batch_index: i32,
    tile_index: i32,
    plan_row_start: i32,
    plan_row_count: i32,
    row_start: i32,
    row_count: i32,
    width: i32,
    height: i32,
    stride_bytes: i32,
    tile_rows: i32,
    tile_count: i32,
    pixel_count: i32,
    total_run_count: i32,
    encoded_byte_count: i32,
) -> i32 {
    let descriptor = match validate_native_span_operation_descriptor(
        target_kind,
        window_raw,
        surface_raw,
        frame_raw,
        packet_frame_id,
        batch_index,
        tile_index,
        plan_row_start,
        plan_row_count,
        row_start,
        row_count,
        width,
        height,
        stride_bytes,
        tile_rows,
        tile_count,
        pixel_count,
        total_run_count,
        encoded_byte_count,
    ) {
        Ok(descriptor) => descriptor,
        Err(status) => return status,
    };

    normalize_native_span_operation_status(
        sink.execute_span_operation(NativeSpanOperation::End(descriptor)),
    )
}

fn validate_native_span_operation_descriptor(
    target_kind: i32,
    window_raw: i32,
    surface_raw: i32,
    frame_raw: i32,
    packet_frame_id: i32,
    batch_index: i32,
    tile_index: i32,
    plan_row_start: i32,
    plan_row_count: i32,
    row_start: i32,
    row_count: i32,
    width: i32,
    height: i32,
    stride_bytes: i32,
    tile_rows: i32,
    tile_count: i32,
    pixel_count: i32,
    total_run_count: i32,
    encoded_byte_count: i32,
) -> Result<NativeSpanOperationDescriptor, i32> {
    let target = validate_native_span_operation_target(target_kind, window_raw)?;
    let surface_id = require_positive_i32(surface_raw)?;
    let frame_id = require_positive_i32(frame_raw)?;
    let packet_frame_id = require_positive_i32(packet_frame_id)?;
    if packet_frame_id != frame_id {
        return Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT);
    }

    let batch_index = require_non_negative_i32(batch_index)?;
    let tile_index = require_non_negative_i32(tile_index)?;
    let plan_row_start = require_non_negative_i32(plan_row_start)?;
    let row_start = require_non_negative_i32(row_start)?;
    let plan_row_count = require_positive_i32(plan_row_count)?;
    let row_count = require_positive_i32(row_count)?;
    let width = require_positive_i32(width)?;
    let height = require_positive_i32(height)?;
    let stride_bytes = require_positive_i32(stride_bytes)?;
    let tile_rows = require_positive_i32(tile_rows)?;
    let tile_count = require_positive_i32(tile_count)?;
    let pixel_count = require_positive_i32(pixel_count)?;
    let total_run_count = require_positive_i32(total_run_count)?;
    let encoded_byte_count = require_positive_i32(encoded_byte_count)?;

    let plan_row_end = checked_extent_end(plan_row_start, plan_row_count)?;
    let row_end = checked_extent_end(row_start, row_count)?;
    if plan_row_end > height
        || row_end > height
        || row_start < plan_row_start
        || row_end > plan_row_end
    {
        return Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT);
    }

    let expected_stride = checked_mul_i32(width, 4)?;
    if stride_bytes != expected_stride {
        return Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT);
    }

    let expected_pixel_count = checked_mul_i32(width, row_count)?;
    if pixel_count != expected_pixel_count {
        return Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT);
    }

    let expected_encoded_byte_count = checked_mul_i32(total_run_count, 12)?;
    if encoded_byte_count != expected_encoded_byte_count {
        return Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT);
    }

    let expected_tile_count = checked_ceil_div_i32(plan_row_count, tile_rows)?;
    if tile_count != expected_tile_count || tile_index >= tile_count {
        return Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT);
    }

    Ok(NativeSpanOperationDescriptor {
        target,
        surface_id,
        frame_id,
        packet_frame_id,
        batch_index,
        tile_index,
        plan_row_start,
        plan_row_count,
        row_start,
        row_count,
        width,
        height,
        stride_bytes,
        tile_rows,
        tile_count,
        pixel_count,
        total_run_count,
        encoded_byte_count,
    })
}

fn validate_native_span_operation_run_span(
    target_kind: i32,
    window_raw: i32,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    r: i32,
    g: i32,
    b: i32,
    a: i32,
) -> Result<NativeSpanOperationRunSpan, i32> {
    let target = validate_native_span_operation_target(target_kind, window_raw)?;
    let x = require_non_negative_i32(x)?;
    let y = require_non_negative_i32(y)?;
    let width = require_positive_i32(width)?;
    if height != 1 {
        return Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT);
    }
    let r = require_rgba_channel(r)?;
    let g = require_rgba_channel(g)?;
    let b = require_rgba_channel(b)?;
    let a = require_rgba_channel(a)?;

    Ok(NativeSpanOperationRunSpan {
        target,
        x,
        y,
        width,
        height,
        r,
        g,
        b,
        a,
    })
}

fn validate_native_span_operation_target(
    target_kind: i32,
    window_raw: i32,
) -> Result<NativeSpanOperationTarget, i32> {
    match target_kind {
        GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW => {
            let window_id = require_positive_i32(window_raw)?;
            Ok(NativeSpanOperationTarget::Window { window_id })
        }
        GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_OFFSCREEN => {
            if window_raw == 0 {
                Ok(NativeSpanOperationTarget::Offscreen)
            } else {
                Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT)
            }
        }
        GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_DEVICE => {
            if window_raw == 0 {
                Ok(NativeSpanOperationTarget::Device)
            } else {
                Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT)
            }
        }
        _ => Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT),
    }
}

fn require_positive_i32(value: i32) -> Result<i32, i32> {
    if value > 0 {
        Ok(value)
    } else {
        Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT)
    }
}

fn require_non_negative_i32(value: i32) -> Result<i32, i32> {
    if value >= 0 {
        Ok(value)
    } else {
        Err(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT)
    }
}

fn require_rgba_channel(value: i32) -> Result<u8, i32> {
    u8::try_from(value).map_err(|_| GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT)
}

fn checked_extent_end(start: i32, count: i32) -> Result<i32, i32> {
    start
        .checked_add(count)
        .ok_or(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT)
}

fn checked_mul_i32(left: i32, right: i32) -> Result<i32, i32> {
    left.checked_mul(right)
        .ok_or(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT)
}

fn checked_ceil_div_i32(value: i32, divisor: i32) -> Result<i32, i32> {
    let adjusted = value
        .checked_add(divisor - 1)
        .ok_or(GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT)?;
    Ok(adjusted / divisor)
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

    #[derive(Debug, Default)]
    struct RecordingSpanOperationSink {
        operations: Vec<NativeSpanOperation>,
        next_status: i32,
    }

    impl RecordingSpanOperationSink {
        fn with_next_status(next_status: i32) -> Self {
            Self {
                operations: Vec::new(),
                next_status,
            }
        }
    }

    impl NativeSpanOperationSink for RecordingSpanOperationSink {
        fn execute_span_operation(&mut self, operation: NativeSpanOperation) -> i32 {
            self.operations.push(operation);
            self.next_status
        }
    }

    fn execute_valid_begin(sink: &mut RecordingSpanOperationSink) -> i32 {
        execute_native_span_operation_begin(
            sink,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            10,
            11,
            11,
            0,
            1,
            2,
            4,
            4,
            2,
            4,
            8,
            16,
            2,
            2,
            8,
            3,
            36,
        )
    }

    #[test]
    fn native_span_operation_records_valid_begin_run_end() {
        let mut sink = RecordingSpanOperationSink::default();

        assert_eq!(
            execute_valid_begin(&mut sink),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            execute_native_span_operation_run(
                &mut sink,
                GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
                7,
                1,
                4,
                2,
                1,
                10,
                20,
                30,
                255,
            ),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            execute_native_span_operation_end(
                &mut sink,
                GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
                7,
                10,
                11,
                11,
                0,
                1,
                2,
                4,
                4,
                2,
                4,
                8,
                16,
                2,
                2,
                8,
                3,
                36,
            ),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );

        assert_eq!(sink.operations.len(), 3);
        assert!(matches!(sink.operations[0], NativeSpanOperation::Begin(_)));
        assert_eq!(
            sink.operations[1],
            NativeSpanOperation::RunSpan(NativeSpanOperationRunSpan {
                target: NativeSpanOperationTarget::Window { window_id: 7 },
                x: 1,
                y: 4,
                width: 2,
                height: 1,
                r: 10,
                g: 20,
                b: 30,
                a: 255,
            })
        );
        assert!(matches!(sink.operations[2], NativeSpanOperation::End(_)));
    }

    #[test]
    fn native_span_operation_rejects_invalid_descriptor_before_sink() {
        let mut sink = RecordingSpanOperationSink::default();
        let status = execute_native_span_operation_begin(
            &mut sink,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            10,
            11,
            11,
            0,
            1,
            2,
            4,
            4,
            2,
            4,
            8,
            20,
            2,
            2,
            8,
            3,
            36,
        );

        assert_eq!(status, GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT);
        assert!(sink.operations.is_empty());
    }

    #[test]
    fn native_span_operation_requires_exact_tile_count_and_frame_id() {
        let mut sink = RecordingSpanOperationSink::default();
        let wrong_tile_count = execute_native_span_operation_begin(
            &mut sink,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            10,
            11,
            11,
            0,
            2,
            2,
            4,
            4,
            2,
            4,
            8,
            16,
            2,
            3,
            8,
            3,
            36,
        );
        let wrong_packet_frame = execute_native_span_operation_begin(
            &mut sink,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            10,
            11,
            12,
            0,
            1,
            2,
            4,
            4,
            2,
            4,
            8,
            16,
            2,
            2,
            8,
            3,
            36,
        );

        assert_eq!(
            wrong_tile_count,
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(
            wrong_packet_frame,
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert!(sink.operations.is_empty());
    }

    #[test]
    fn native_span_operation_rejects_invalid_run_span_before_sink() {
        let mut sink = RecordingSpanOperationSink::default();
        let wrong_height = execute_native_span_operation_run(
            &mut sink,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            1,
            4,
            2,
            2,
            10,
            20,
            30,
            255,
        );
        let wrong_channel = execute_native_span_operation_run(
            &mut sink,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            1,
            4,
            2,
            1,
            10,
            20,
            30,
            256,
        );

        assert_eq!(
            wrong_height,
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(
            wrong_channel,
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert!(sink.operations.is_empty());
    }

    #[test]
    fn native_span_operation_normalizes_sink_status() {
        let mut resource_exhausted = RecordingSpanOperationSink::with_next_status(
            GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED,
        );
        assert_eq!(
            execute_valid_begin(&mut resource_exhausted),
            GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED
        );
        assert_eq!(resource_exhausted.operations.len(), 1);

        let mut unknown_positive = RecordingSpanOperationSink::with_next_status(99);
        assert_eq!(
            execute_valid_begin(&mut unknown_positive),
            GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE
        );
        assert_eq!(unknown_positive.operations.len(), 1);

        let mut unknown_negative = RecordingSpanOperationSink::with_next_status(-99);
        assert_eq!(
            execute_valid_begin(&mut unknown_negative),
            GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE
        );
        assert_eq!(unknown_negative.operations.len(), 1);
    }
}
