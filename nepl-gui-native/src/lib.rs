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
pub enum RasterizeSurfaceError {
    InvalidDimensions,
    CommandOutOfBounds,
    DimensionOverflow,
    ResourceExhausted,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSpanOperationStatus {
    Ok,
    Unsupported,
    InvalidArgument,
    ResourceExhausted,
    NoWritableSlot,
    BackendFailure,
    StaleFrame,
}

pub const GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW: i32 = 1;
pub const GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_OFFSCREEN: i32 = 2;
pub const GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_DEVICE: i32 = 3;
pub const NATIVE_RGB0_HIGH_BYTE_MASK: u32 = 0xff000000;

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

impl NativeSpanOperationStatus {
    pub fn from_raw(status: i32) -> Self {
        match normalize_native_span_operation_status(status) {
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK => NativeSpanOperationStatus::Ok,
            GUI_NATIVE_SPAN_OPERATION_STATUS_UNSUPPORTED => NativeSpanOperationStatus::Unsupported,
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT => {
                NativeSpanOperationStatus::InvalidArgument
            }
            GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED => {
                NativeSpanOperationStatus::ResourceExhausted
            }
            GUI_NATIVE_SPAN_OPERATION_STATUS_NO_WRITABLE_SLOT => {
                NativeSpanOperationStatus::NoWritableSlot
            }
            GUI_NATIVE_SPAN_OPERATION_STATUS_STALE_FRAME => NativeSpanOperationStatus::StaleFrame,
            _ => NativeSpanOperationStatus::BackendFailure,
        }
    }

    pub fn as_raw(self) -> i32 {
        match self {
            NativeSpanOperationStatus::Ok => GUI_NATIVE_SPAN_OPERATION_STATUS_OK,
            NativeSpanOperationStatus::Unsupported => GUI_NATIVE_SPAN_OPERATION_STATUS_UNSUPPORTED,
            NativeSpanOperationStatus::InvalidArgument => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
            }
            NativeSpanOperationStatus::ResourceExhausted => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED
            }
            NativeSpanOperationStatus::NoWritableSlot => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_NO_WRITABLE_SLOT
            }
            NativeSpanOperationStatus::BackendFailure => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE
            }
            NativeSpanOperationStatus::StaleFrame => GUI_NATIVE_SPAN_OPERATION_STATUS_STALE_FRAME,
        }
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

pub fn execute_native_window_presenter_session_operation(
    session: &mut NativeWindowPresenterSession,
    operation: NativeSpanOperation,
) -> Result<NativeWindowPresenterSessionOutcome, NativeWindowPresenterSessionHostError> {
    session
        .execute_span_operation(operation)
        .map_err(NativeWindowPresenterSessionHostError::SessionFailed)
}

pub fn execute_native_window_presenter_session_begin(
    session: &mut NativeWindowPresenterSession,
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
) -> Result<NativeWindowPresenterSessionOutcome, NativeWindowPresenterSessionHostError> {
    let descriptor = validate_native_span_operation_descriptor(
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
    )
    .map_err(NativeWindowPresenterSessionHostError::from_validation_status)?;

    execute_native_window_presenter_session_operation(
        session,
        NativeSpanOperation::Begin(descriptor),
    )
}

pub fn execute_native_window_presenter_session_run(
    session: &mut NativeWindowPresenterSession,
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
) -> Result<NativeWindowPresenterSessionOutcome, NativeWindowPresenterSessionHostError> {
    let run_span = validate_native_span_operation_run_span(
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
    )
    .map_err(NativeWindowPresenterSessionHostError::from_validation_status)?;

    execute_native_window_presenter_session_operation(
        session,
        NativeSpanOperation::RunSpan(run_span),
    )
}

pub fn execute_native_window_presenter_session_end(
    session: &mut NativeWindowPresenterSession,
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
) -> Result<NativeWindowPresenterSessionOutcome, NativeWindowPresenterSessionHostError> {
    let descriptor = validate_native_span_operation_descriptor(
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
    )
    .map_err(NativeWindowPresenterSessionHostError::from_validation_status)?;

    execute_native_window_presenter_session_operation(session, NativeSpanOperation::End(descriptor))
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

pub const NATIVE_RGBA8888_PIXEL_TRANSPARENT: u32 = 0x00000000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSpanFramebufferError {
    InvalidDimensions,
    DimensionOverflow,
    ResourceExhausted,
    SequenceAlreadyActive,
    SequenceMissing,
    FramebufferDescriptorMismatch,
    DescriptorMismatch,
    TargetMismatch,
    RunCountExceeded,
    RunExtentOutOfBounds,
    RunCountMismatch,
    InternalIndexOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeSpanFramebufferActiveSequence {
    pub descriptor: NativeSpanOperationDescriptor,
    pub seen_run_count: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeRgba8888FrameBuffer {
    width: i32,
    height: i32,
    stride_bytes: i32,
    pixels: Vec<u32>,
    active_sequence: Option<NativeSpanFramebufferActiveSequence>,
}

impl NativeSpanFramebufferError {
    pub fn status(self) -> i32 {
        match self {
            NativeSpanFramebufferError::ResourceExhausted => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED
            }
            NativeSpanFramebufferError::InternalIndexOverflow => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE
            }
            NativeSpanFramebufferError::InvalidDimensions
            | NativeSpanFramebufferError::DimensionOverflow
            | NativeSpanFramebufferError::SequenceAlreadyActive
            | NativeSpanFramebufferError::SequenceMissing
            | NativeSpanFramebufferError::FramebufferDescriptorMismatch
            | NativeSpanFramebufferError::DescriptorMismatch
            | NativeSpanFramebufferError::TargetMismatch
            | NativeSpanFramebufferError::RunCountExceeded
            | NativeSpanFramebufferError::RunExtentOutOfBounds
            | NativeSpanFramebufferError::RunCountMismatch => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
            }
        }
    }
}

impl NativeRgba8888FrameBuffer {
    /// Creates a checked logical RGBA8888 framebuffer.
    ///
    /// Pixels are semantic `0xRRGGBBAA` values. The storage is not a native-endian
    /// byte view, so presenters must explicitly convert when a host surface uses a
    /// different pixel contract.
    pub fn new(width: i32, height: i32) -> Result<Self, NativeSpanFramebufferError> {
        if width <= 0 || height <= 0 {
            return Err(NativeSpanFramebufferError::InvalidDimensions);
        }
        let stride_bytes = width
            .checked_mul(4)
            .ok_or(NativeSpanFramebufferError::DimensionOverflow)?;
        let pixel_count = width
            .checked_mul(height)
            .ok_or(NativeSpanFramebufferError::DimensionOverflow)?;
        let pixel_count = usize::try_from(pixel_count)
            .map_err(|_| NativeSpanFramebufferError::DimensionOverflow)?;
        let mut pixels = Vec::new();
        pixels
            .try_reserve_exact(pixel_count)
            .map_err(|_| NativeSpanFramebufferError::ResourceExhausted)?;
        pixels.resize(pixel_count, NATIVE_RGBA8888_PIXEL_TRANSPARENT);

        Ok(Self {
            width,
            height,
            stride_bytes,
            pixels,
            active_sequence: None,
        })
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn stride_bytes(&self) -> i32 {
        self.stride_bytes
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub fn active_sequence(&self) -> Option<NativeSpanFramebufferActiveSequence> {
        self.active_sequence
    }

    pub fn pixel_at(&self, x: i32, y: i32) -> Option<u32> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        let index = native_span_framebuffer_index(self.width, x, y).ok()?;
        self.pixels.get(index).copied()
    }

    fn begin_sequence(
        &mut self,
        descriptor: NativeSpanOperationDescriptor,
    ) -> Result<(), NativeSpanFramebufferError> {
        if self.active_sequence.is_some() {
            return Err(NativeSpanFramebufferError::SequenceAlreadyActive);
        }
        if descriptor.width != self.width
            || descriptor.height != self.height
            || descriptor.stride_bytes != self.stride_bytes
        {
            return Err(NativeSpanFramebufferError::FramebufferDescriptorMismatch);
        }
        self.active_sequence = Some(NativeSpanFramebufferActiveSequence {
            descriptor,
            seen_run_count: 0,
        });
        Ok(())
    }

    fn write_run_span(
        &mut self,
        run_span: NativeSpanOperationRunSpan,
    ) -> Result<(), NativeSpanFramebufferError> {
        let active = self
            .active_sequence
            .ok_or(NativeSpanFramebufferError::SequenceMissing)?;
        let descriptor = active.descriptor;
        if run_span.target != descriptor.target {
            return Err(NativeSpanFramebufferError::TargetMismatch);
        }
        if run_span.x < 0 || run_span.width <= 0 || run_span.height != 1 {
            return Err(NativeSpanFramebufferError::RunExtentOutOfBounds);
        }
        if active.seen_run_count >= descriptor.total_run_count {
            return Err(NativeSpanFramebufferError::RunCountExceeded);
        }
        let row_end = descriptor
            .row_start
            .checked_add(descriptor.row_count)
            .ok_or(NativeSpanFramebufferError::InternalIndexOverflow)?;
        let run_x_end = run_span
            .x
            .checked_add(run_span.width)
            .ok_or(NativeSpanFramebufferError::RunExtentOutOfBounds)?;
        if run_span.y < descriptor.row_start
            || run_span.y >= row_end
            || run_x_end > descriptor.width
        {
            return Err(NativeSpanFramebufferError::RunExtentOutOfBounds);
        }

        let start = native_span_framebuffer_index(self.width, run_span.x, run_span.y)?;
        let run_width = usize::try_from(run_span.width)
            .map_err(|_| NativeSpanFramebufferError::RunExtentOutOfBounds)?;
        let end = start
            .checked_add(run_width)
            .ok_or(NativeSpanFramebufferError::InternalIndexOverflow)?;
        if end > self.pixels.len() {
            return Err(NativeSpanFramebufferError::InternalIndexOverflow);
        }

        let pixel = native_pack_rgba8888_pixel(run_span.r, run_span.g, run_span.b, run_span.a);
        for value in &mut self.pixels[start..end] {
            *value = pixel;
        }
        self.active_sequence = Some(NativeSpanFramebufferActiveSequence {
            descriptor,
            seen_run_count: active.seen_run_count + 1,
        });
        Ok(())
    }

    fn end_sequence(
        &mut self,
        descriptor: NativeSpanOperationDescriptor,
    ) -> Result<(), NativeSpanFramebufferError> {
        let active = self
            .active_sequence
            .ok_or(NativeSpanFramebufferError::SequenceMissing)?;
        if descriptor != active.descriptor {
            return Err(NativeSpanFramebufferError::DescriptorMismatch);
        }
        if active.seen_run_count != descriptor.total_run_count {
            return Err(NativeSpanFramebufferError::RunCountMismatch);
        }
        self.active_sequence = None;
        Ok(())
    }

    fn end_sequence_to_rgb0_present_buffer(
        &mut self,
        descriptor: NativeSpanOperationDescriptor,
        background: NativeRgbColor,
    ) -> Result<NativeRgb0PresentBuffer, NativeSpanFramebufferError> {
        let active = self
            .active_sequence
            .ok_or(NativeSpanFramebufferError::SequenceMissing)?;
        if descriptor != active.descriptor {
            return Err(NativeSpanFramebufferError::DescriptorMismatch);
        }
        if active.seen_run_count != descriptor.total_run_count {
            return Err(NativeSpanFramebufferError::RunCountMismatch);
        }
        let present_buffer = native_rgb0_present_buffer_from_rgba8888_parts(
            self.width,
            self.height,
            &self.pixels,
            background,
        )?;
        self.active_sequence = None;
        Ok(present_buffer)
    }
}

impl NativeSpanOperationSink for NativeRgba8888FrameBuffer {
    fn execute_span_operation(&mut self, operation: NativeSpanOperation) -> i32 {
        let result = match operation {
            NativeSpanOperation::Begin(descriptor) => self.begin_sequence(descriptor),
            NativeSpanOperation::RunSpan(run_span) => self.write_run_span(run_span),
            NativeSpanOperation::End(descriptor) => self.end_sequence(descriptor),
        };
        match result {
            Ok(()) => GUI_NATIVE_SPAN_OPERATION_STATUS_OK,
            Err(error) => error.status(),
        }
    }
}

pub fn native_pack_rgba8888_pixel(r: u8, g: u8, b: u8, a: u8) -> u32 {
    (u32::from(r) << 24) | (u32::from(g) << 16) | (u32::from(b) << 8) | u32::from(a)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeRgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeRgb0PresentBuffer {
    width: i32,
    height: i32,
    pixels: Vec<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeRgb0PresenterSink {
    frame_buffer: NativeRgba8888FrameBuffer,
    background: NativeRgbColor,
    last_present_buffer: Option<NativeRgb0PresentBuffer>,
    last_presented_frame_id: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowPresenterSurfaceState {
    Drawable { width: usize, height: usize },
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowPresenterError {
    InvalidSurfaceDimensions,
    FrameMissing,
    FrameIdMissing,
    InvalidFrameId,
    PresenterFrameValidationFailed(NativePresenterFrameError),
    ResourceExhausted,
    DimensionOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeWindowPresenterState {
    surface_state: NativeWindowPresenterSurfaceState,
    last_frame_id: Option<i32>,
    last_frame_width: usize,
    last_frame_height: usize,
    last_pixels: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeRgb0PresenterSinkOutcome {
    Accepted,
    Completed { frame_id: i32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeWindowPresenterSession {
    sink: NativeRgb0PresenterSink,
    presenter_state: NativeWindowPresenterState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowPresenterSessionOutcome {
    NotPresented,
    Presented {
        frame_id: i32,
        width: usize,
        height: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowPresenterSessionError {
    SinkFailed(NativeSpanFramebufferError),
    PresenterFailed(NativeWindowPresenterError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowPresenterSessionHostError {
    ValidationFailed(NativeSpanOperationStatus),
    SessionFailed(NativeWindowPresenterSessionError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowSize {
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowEventPumpCloseState {
    Open,
    OsCloseRequested,
    ExitShortcutRequested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowPointerButtonTransition {
    Unchanged,
    Pressed,
    Released,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NativeWindowPointerSample {
    Unavailable,
    Available { x: f32, y: f32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowEventPumpError {
    InvalidPointerSample,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowEventPumpInput {
    pub previous_size: NativeWindowSize,
    pub previous_mouse_down: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeWindowEventPumpSnapshot {
    pub close_state: NativeWindowEventPumpCloseState,
    pub window_size: NativeWindowSize,
    pub surface_state: NativeWindowPresenterSurfaceState,
    pub size_changed: bool,
    pub mouse_down: bool,
    pub mouse_left_transition: NativeWindowPointerButtonTransition,
    pub pointer_sample: NativeWindowPointerSample,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativePresenterFrameError {
    InvalidDimensions,
    DimensionOverflow,
    PixelCountMismatch,
    PixelFormatMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativePresenterFrame<'a> {
    width: usize,
    height: usize,
    pixels: &'a [u32],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowBackendLoopPresentation {
    pub frame_id: i32,
    pub width: usize,
    pub height: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowBackendLoopPointerAction {
    None,
    PressedUnavailable,
    PressedOutside,
    CounterIncremented {
        value: i32,
        presentation: NativeWindowBackendLoopPresentation,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowBackendLoopDrawableStep {
    pub window_size: NativeWindowSize,
    pub size_changed: bool,
    pub resize_redraw: Option<NativeWindowBackendLoopPresentation>,
    pub pointer_action: NativeWindowBackendLoopPointerAction,
    pub final_frame: NativeWindowBackendLoopPresentation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowBackendLoopStepOutcome {
    CloseRequested {
        close_state: NativeWindowEventPumpCloseState,
    },
    Unavailable {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    Drawable(NativeWindowBackendLoopDrawableStep),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostTerminalReason {
    OsCloseRequested,
    ExitShortcutRequested,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostAction {
    Terminate {
        reason: NativeWindowHostTerminalReason,
    },
    PumpEventsOnly {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    PresentFrame {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostActionError {
    UnsupportedCloseState {
        close_state: NativeWindowEventPumpCloseState,
    },
    StepFailed(NativeWindowBackendLoopError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowTargetFps {
    value: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowTargetFpsInvalidReason {
    Zero,
    TooHigh { max: u16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowTargetFpsError {
    pub value: usize,
    pub reason: NativeWindowTargetFpsInvalidReason,
}

pub const NATIVE_WINDOW_RUN_LOOP_MIN_TARGET_FPS: u16 = 1;
pub const NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS: u16 = 240;
pub const NATIVE_WINDOW_RUN_LOOP_DEFAULT_TARGET_FPS: u16 = 60;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopTurnSlice {
    value: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopTurnSliceInvalidReason {
    Zero,
    TooHigh { max: u16 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopTurnSliceError {
    pub value: usize,
    pub reason: NativeWindowHostLoopTurnSliceInvalidReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopRunPolicy {
    pub turn_slice: NativeWindowHostLoopTurnSlice,
}

pub const NATIVE_WINDOW_HOST_LOOP_MIN_TURN_SLICE: u16 = 1;
pub const NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE: u16 = 4096;
pub const NATIVE_WINDOW_HOST_LOOP_DEFAULT_TURN_SLICE: u16 = 1;

impl NativeWindowTargetFps {
    pub fn new(value: usize) -> Result<Self, NativeWindowTargetFpsError> {
        if value == 0 {
            return Err(NativeWindowTargetFpsError {
                value,
                reason: NativeWindowTargetFpsInvalidReason::Zero,
            });
        }
        let Ok(value) = u16::try_from(value) else {
            return Err(NativeWindowTargetFpsError {
                value,
                reason: NativeWindowTargetFpsInvalidReason::TooHigh {
                    max: NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS,
                },
            });
        };
        if value > NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS {
            return Err(NativeWindowTargetFpsError {
                value: usize::from(value),
                reason: NativeWindowTargetFpsInvalidReason::TooHigh {
                    max: NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS,
                },
            });
        }
        Ok(Self { value })
    }

    pub fn value(self) -> u16 {
        self.value
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.value)
    }
}

impl Default for NativeWindowTargetFps {
    fn default() -> Self {
        Self {
            value: NATIVE_WINDOW_RUN_LOOP_DEFAULT_TARGET_FPS,
        }
    }
}

impl NativeWindowHostLoopTurnSlice {
    pub fn new(value: usize) -> Result<Self, NativeWindowHostLoopTurnSliceError> {
        if value == 0 {
            return Err(NativeWindowHostLoopTurnSliceError {
                value,
                reason: NativeWindowHostLoopTurnSliceInvalidReason::Zero,
            });
        }
        let Ok(value) = u16::try_from(value) else {
            return Err(NativeWindowHostLoopTurnSliceError {
                value,
                reason: NativeWindowHostLoopTurnSliceInvalidReason::TooHigh {
                    max: NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE,
                },
            });
        };
        if value > NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE {
            return Err(NativeWindowHostLoopTurnSliceError {
                value: usize::from(value),
                reason: NativeWindowHostLoopTurnSliceInvalidReason::TooHigh {
                    max: NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE,
                },
            });
        }
        Ok(Self { value })
    }

    pub fn value(self) -> u16 {
        self.value
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.value)
    }
}

impl Default for NativeWindowHostLoopTurnSlice {
    fn default() -> Self {
        Self {
            value: NATIVE_WINDOW_HOST_LOOP_DEFAULT_TURN_SLICE,
        }
    }
}

impl NativeWindowHostLoopRunPolicy {
    pub fn new(turn_slice: NativeWindowHostLoopTurnSlice) -> Self {
        Self { turn_slice }
    }
}

impl Default for NativeWindowHostLoopRunPolicy {
    fn default() -> Self {
        Self {
            turn_slice: NativeWindowHostLoopTurnSlice::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowRunLoopConfig {
    pub demo: GuiDemo,
    pub counter_value: i32,
    pub scale: usize,
    pub target_fps: NativeWindowTargetFps,
    pub host_loop_policy: NativeWindowHostLoopRunPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowRunLoopExit {
    pub reason: NativeWindowHostTerminalReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowRunLoopError {
    TargetFpsInvalid {
        value: usize,
        reason: NativeWindowTargetFpsInvalidReason,
    },
    BackendLoopInitializationFailed(NativeWindowBackendLoopError),
    WindowCreationFailed {
        message: String,
    },
    EventPumpFailed(NativeWindowEventPumpError),
    HostActionFailed(NativeWindowHostActionError),
    PresenterFrameUnavailable(NativeWindowBackendLoopError),
    WindowPresentFailed {
        message: String,
    },
    WaitDecisionMissing,
}

pub trait NativeWindowRunLoopHost {
    type EventError;
    type PresentError;
    type WaitError;

    fn poll_event_snapshot(
        &mut self,
        input: NativeWindowEventPumpInput,
    ) -> Result<NativeWindowEventPumpSnapshot, Self::EventError>;

    fn set_window_title(&mut self, title: &str);

    fn pump_events_only(&mut self);

    fn present_frame(&mut self, frame: NativePresenterFrame<'_>) -> Result<(), Self::PresentError>;

    fn wait_after_budget_exhausted(
        &mut self,
        request: NativeWindowHostLoopWaitRequest,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopError<EventError, PresentError, WaitError> {
    HostEventPumpFailed(EventError),
    HostActionFailed(NativeWindowHostActionError),
    PresenterFrameUnavailable(NativeWindowBackendLoopError),
    HostPresentFailed(PresentError),
    HostWaitFailed(WaitError),
    WaitDecisionMissing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopContinueEvidence {
    PumpedEventsOnly {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    PresentedFrame {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopWaitDecision {
    WaitForHostEvent {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    WaitForFrameInterval {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
    },
}

pub const NATIVE_WINDOW_NANOS_PER_SECOND: u32 = 1_000_000_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowFrameIntervalRequest {
    target_fps: NativeWindowTargetFps,
    nanos_per_frame: u32,
    remainder_nanos_per_second: u32,
}

impl NativeWindowFrameIntervalRequest {
    pub fn target_fps(self) -> NativeWindowTargetFps {
        self.target_fps
    }

    pub fn nanos_per_frame(self) -> u32 {
        self.nanos_per_frame
    }

    pub fn remainder_nanos_per_second(self) -> u32 {
        self.remainder_nanos_per_second
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopWaitRequest {
    WaitForHostEvent {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    WaitForFrameInterval {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        frame_interval: NativeWindowFrameIntervalRequest,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopWaitOutcome {
    HostEventPumpAlreadyPaced {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    FramePresentAlreadyPaced {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopTurn {
    Continue(NativeWindowHostLoopContinueEvidence),
    Exit(NativeWindowRunLoopExit),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopRunnerState {
    title_initialized: bool,
}

impl NativeWindowHostLoopRunnerState {
    pub fn new() -> Self {
        Self {
            title_initialized: false,
        }
    }

    pub fn title_initialized(&self) -> bool {
        self.title_initialized
    }
}

impl Default for NativeWindowHostLoopRunnerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopSchedulerState {
    runner_state: NativeWindowHostLoopRunnerState,
}

impl NativeWindowHostLoopSchedulerState {
    pub fn new() -> Self {
        Self {
            runner_state: NativeWindowHostLoopRunnerState::new(),
        }
    }

    pub fn title_initialized(&self) -> bool {
        self.runner_state.title_initialized()
    }
}

impl Default for NativeWindowHostLoopSchedulerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopInitialization {
    Initialized,
    AlreadyInitialized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopBoundedRunResult {
    Exited {
        exit: NativeWindowRunLoopExit,
        completed_turns: usize,
    },
    BudgetExhausted {
        completed_turns: usize,
        last_wait_decision: Option<NativeWindowHostLoopWaitDecision>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopSchedulerSliceResult {
    Exited {
        exit: NativeWindowRunLoopExit,
        completed_turns: usize,
    },
    Waited {
        completed_turns: usize,
        decision: NativeWindowHostLoopWaitDecision,
        request: NativeWindowHostLoopWaitRequest,
        outcome: NativeWindowHostLoopWaitOutcome,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowBackendLoopError {
    InitialScaleInvalid,
    InitialSizeOverflow,
    InitialSurfaceInvalid,
    FrameIdOverflow {
        previous: i32,
    },
    CounterValueOverflow {
        previous: i32,
    },
    CounterFrameIdMissing,
    RasterizeFailed(RasterizeSurfaceError),
    PresentBufferInvalid(NativePresenterFrameError),
    PresenterFailed(NativeWindowPresenterError),
    FrameMissing,
    SurfaceUnavailable,
    FrameWindowMismatch {
        frame_width: usize,
        frame_height: usize,
        window_width: usize,
        window_height: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeWindowBackendLoopState {
    demo: GuiDemo,
    counter_value: i32,
    frame: GuiFrame,
    presenter_frame_id: i32,
    previous_size: NativeWindowSize,
    previous_mouse_down: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeWindowBackendLoop {
    state: NativeWindowBackendLoopState,
    presenter_state: NativeWindowPresenterState,
}

impl NativeRgb0PresentBuffer {
    /// Converts a completed semantic RGBA8888 framebuffer into `0x00RRGGBB`.
    ///
    /// The conversion performs source-over alpha composition with an explicit
    /// background color and does not expose a native-endian byte representation.
    pub fn from_rgba8888_framebuffer(
        frame_buffer: &NativeRgba8888FrameBuffer,
        background: NativeRgbColor,
    ) -> Result<Self, NativeSpanFramebufferError> {
        if frame_buffer.active_sequence().is_some() {
            return Err(NativeSpanFramebufferError::SequenceAlreadyActive);
        }
        native_rgb0_present_buffer_from_rgba8888_parts(
            frame_buffer.width,
            frame_buffer.height,
            &frame_buffer.pixels,
            background,
        )
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    /// Imports smoke/demo pixels that are already semantic `0x00RRGGBB`.
    ///
    /// This constructor is not the formal NEPL span presentation path. It exists
    /// to keep the native smoke runner on the same presenter-side pixel contract
    /// while the formal host import path is still being connected.
    pub fn from_rgb0_pixels_for_smoke_demo(
        width: usize,
        height: usize,
        pixels: Vec<u32>,
    ) -> Result<Self, NativePresenterFrameError> {
        if width == 0 || height == 0 {
            return Err(NativePresenterFrameError::InvalidDimensions);
        }
        let pixel_count = width
            .checked_mul(height)
            .ok_or(NativePresenterFrameError::DimensionOverflow)?;
        if pixels.len() != pixel_count {
            return Err(NativePresenterFrameError::PixelCountMismatch);
        }
        if pixels
            .iter()
            .any(|pixel| pixel & NATIVE_RGB0_HIGH_BYTE_MASK != 0)
        {
            return Err(NativePresenterFrameError::PixelFormatMismatch);
        }
        let width =
            i32::try_from(width).map_err(|_| NativePresenterFrameError::DimensionOverflow)?;
        let height =
            i32::try_from(height).map_err(|_| NativePresenterFrameError::DimensionOverflow)?;
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
}

impl NativeRgb0PresenterSink {
    pub fn new(
        width: i32,
        height: i32,
        background: NativeRgbColor,
    ) -> Result<Self, NativeSpanFramebufferError> {
        Ok(Self {
            frame_buffer: NativeRgba8888FrameBuffer::new(width, height)?,
            background,
            last_present_buffer: None,
            last_presented_frame_id: None,
        })
    }

    pub fn background(&self) -> NativeRgbColor {
        self.background
    }

    pub fn frame_buffer(&self) -> &NativeRgba8888FrameBuffer {
        &self.frame_buffer
    }

    pub fn last_presented_frame_id(&self) -> Option<i32> {
        self.last_presented_frame_id
    }

    pub fn last_present_frame(
        &self,
    ) -> Result<Option<NativePresenterFrame<'_>>, NativePresenterFrameError> {
        match &self.last_present_buffer {
            Some(buffer) => NativePresenterFrame::from_rgb0_present_buffer(buffer).map(Some),
            None => Ok(None),
        }
    }

    pub fn execute_span_operation_typed(
        &mut self,
        operation: NativeSpanOperation,
    ) -> Result<NativeRgb0PresenterSinkOutcome, NativeSpanFramebufferError> {
        match operation {
            NativeSpanOperation::Begin(descriptor) => {
                self.frame_buffer.begin_sequence(descriptor)?;
                Ok(NativeRgb0PresenterSinkOutcome::Accepted)
            }
            NativeSpanOperation::RunSpan(run_span) => {
                self.frame_buffer.write_run_span(run_span)?;
                Ok(NativeRgb0PresenterSinkOutcome::Accepted)
            }
            NativeSpanOperation::End(descriptor) => {
                let frame_id = descriptor.frame_id;
                let present_buffer = self
                    .frame_buffer
                    .end_sequence_to_rgb0_present_buffer(descriptor, self.background)?;
                self.last_present_buffer = Some(present_buffer);
                self.last_presented_frame_id = Some(frame_id);
                Ok(NativeRgb0PresenterSinkOutcome::Completed { frame_id })
            }
        }
    }
}

impl NativeWindowPresenterError {
    pub fn status(self) -> i32 {
        match self {
            NativeWindowPresenterError::ResourceExhausted => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_RESOURCE_EXHAUSTED
            }
            NativeWindowPresenterError::DimensionOverflow => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_BACKEND_FAILURE
            }
            NativeWindowPresenterError::FrameMissing
            | NativeWindowPresenterError::FrameIdMissing => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_STALE_FRAME
            }
            NativeWindowPresenterError::InvalidSurfaceDimensions
            | NativeWindowPresenterError::InvalidFrameId
            | NativeWindowPresenterError::PresenterFrameValidationFailed(_) => {
                GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
            }
        }
    }
}

impl NativeWindowPresenterSessionError {
    pub fn status(self) -> i32 {
        match self {
            NativeWindowPresenterSessionError::SinkFailed(error) => error.status(),
            NativeWindowPresenterSessionError::PresenterFailed(error) => error.status(),
        }
    }
}

impl NativeWindowPresenterSessionHostError {
    fn from_validation_status(status: i32) -> Self {
        NativeWindowPresenterSessionHostError::ValidationFailed(
            NativeSpanOperationStatus::from_raw(status),
        )
    }

    pub fn status(self) -> i32 {
        match self {
            NativeWindowPresenterSessionHostError::ValidationFailed(status) => status.as_raw(),
            NativeWindowPresenterSessionHostError::SessionFailed(error) => error.status(),
        }
    }
}

impl NativeWindowSize {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    pub fn from_tuple(size: (usize, usize)) -> Self {
        Self {
            width: size.0,
            height: size.1,
        }
    }

    pub fn as_tuple(self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn presenter_surface_state(self) -> NativeWindowPresenterSurfaceState {
        if self.width == 0 || self.height == 0 {
            NativeWindowPresenterSurfaceState::Unavailable
        } else {
            NativeWindowPresenterSurfaceState::Drawable {
                width: self.width,
                height: self.height,
            }
        }
    }
}

pub fn native_window_pointer_sample_from_raw(
    x: f32,
    y: f32,
) -> Result<NativeWindowPointerSample, NativeWindowEventPumpError> {
    if !x.is_finite() || !y.is_finite() {
        return Err(NativeWindowEventPumpError::InvalidPointerSample);
    }
    Ok(NativeWindowPointerSample::Available { x, y })
}

pub fn build_native_window_event_pump_snapshot(
    input: NativeWindowEventPumpInput,
    os_close_requested: bool,
    exit_shortcut_requested: bool,
    current_size: NativeWindowSize,
    mouse_down: bool,
    pointer_sample: NativeWindowPointerSample,
) -> NativeWindowEventPumpSnapshot {
    let close_state = if os_close_requested {
        NativeWindowEventPumpCloseState::OsCloseRequested
    } else if exit_shortcut_requested {
        NativeWindowEventPumpCloseState::ExitShortcutRequested
    } else {
        NativeWindowEventPumpCloseState::Open
    };
    let mouse_left_transition = match (input.previous_mouse_down, mouse_down) {
        (false, true) => NativeWindowPointerButtonTransition::Pressed,
        (true, false) => NativeWindowPointerButtonTransition::Released,
        _ => NativeWindowPointerButtonTransition::Unchanged,
    };

    NativeWindowEventPumpSnapshot {
        close_state,
        window_size: current_size,
        surface_state: current_size.presenter_surface_state(),
        size_changed: current_size != input.previous_size,
        mouse_down,
        mouse_left_transition,
        pointer_sample,
    }
}

pub fn build_native_window_event_pump_snapshot_from_raw(
    input: NativeWindowEventPumpInput,
    os_close_requested: bool,
    exit_shortcut_requested: bool,
    current_size: NativeWindowSize,
    mouse_down: bool,
    pointer_raw: Option<(f32, f32)>,
) -> Result<NativeWindowEventPumpSnapshot, NativeWindowEventPumpError> {
    let pointer_sample = match pointer_raw {
        Some((x, y)) => native_window_pointer_sample_from_raw(x, y)?,
        None => NativeWindowPointerSample::Unavailable,
    };
    Ok(build_native_window_event_pump_snapshot(
        input,
        os_close_requested,
        exit_shortcut_requested,
        current_size,
        mouse_down,
        pointer_sample,
    ))
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
pub fn poll_minifb_window_event_pump(
    window: &minifb::Window,
    input: NativeWindowEventPumpInput,
) -> Result<NativeWindowEventPumpSnapshot, NativeWindowEventPumpError> {
    build_native_window_event_pump_snapshot_from_raw(
        input,
        !window.is_open(),
        window.is_key_down(minifb::Key::Escape),
        NativeWindowSize::from_tuple(window.get_size()),
        window.get_mouse_down(minifb::MouseButton::Left),
        window.get_unscaled_mouse_pos(minifb::MouseMode::Discard),
    )
}

impl NativeWindowPresenterState {
    pub fn new(
        surface_width: usize,
        surface_height: usize,
    ) -> Result<Self, NativeWindowPresenterError> {
        if surface_width == 0 || surface_height == 0 {
            return Err(NativeWindowPresenterError::InvalidSurfaceDimensions);
        }
        Ok(Self {
            surface_state: NativeWindowPresenterSurfaceState::Drawable {
                width: surface_width,
                height: surface_height,
            },
            last_frame_id: None,
            last_frame_width: 0,
            last_frame_height: 0,
            last_pixels: Vec::new(),
        })
    }

    pub fn surface_state(&self) -> NativeWindowPresenterSurfaceState {
        self.surface_state
    }

    pub fn resize_surface(
        &mut self,
        surface_width: usize,
        surface_height: usize,
    ) -> Result<(), NativeWindowPresenterError> {
        self.surface_state = if surface_width == 0 || surface_height == 0 {
            NativeWindowPresenterSurfaceState::Unavailable
        } else {
            NativeWindowPresenterSurfaceState::Drawable {
                width: surface_width,
                height: surface_height,
            }
        };
        Ok(())
    }

    pub fn last_frame_id(&self) -> Option<i32> {
        self.last_frame_id
    }

    pub fn last_frame_size(&self) -> Option<(usize, usize)> {
        self.last_frame_id
            .map(|_| (self.last_frame_width, self.last_frame_height))
    }

    pub fn last_present_frame(
        &self,
    ) -> Result<Option<NativePresenterFrame<'_>>, NativeWindowPresenterError> {
        if self.last_frame_id.is_none() {
            return Ok(None);
        }
        native_presenter_frame_from_rgb0_parts(
            self.last_frame_width,
            self.last_frame_height,
            &self.last_pixels,
        )
        .map(Some)
        .map_err(NativeWindowPresenterError::PresenterFrameValidationFailed)
    }

    pub fn last_present_frame_required(
        &self,
    ) -> Result<NativePresenterFrame<'_>, NativeWindowPresenterError> {
        self.last_present_frame()?
            .ok_or(NativeWindowPresenterError::FrameMissing)
    }

    pub fn present_buffer(
        &mut self,
        frame_id: i32,
        buffer: &NativeRgb0PresentBuffer,
    ) -> Result<(), NativeWindowPresenterError> {
        let source_frame = NativePresenterFrame::from_rgb0_present_buffer(buffer)
            .map_err(NativeWindowPresenterError::PresenterFrameValidationFailed)?;
        self.present_frame(frame_id, source_frame)
    }

    pub fn present_frame(
        &mut self,
        frame_id: i32,
        source_frame: NativePresenterFrame<'_>,
    ) -> Result<(), NativeWindowPresenterError> {
        if frame_id <= 0 {
            return Err(NativeWindowPresenterError::InvalidFrameId);
        }
        let pixel_count = source_frame
            .width()
            .checked_mul(source_frame.height())
            .ok_or(NativeWindowPresenterError::DimensionOverflow)?;
        if pixel_count != source_frame.pixels().len() {
            return Err(NativeWindowPresenterError::PresenterFrameValidationFailed(
                NativePresenterFrameError::PixelCountMismatch,
            ));
        }

        let mut next_pixels = Vec::new();
        next_pixels
            .try_reserve_exact(pixel_count)
            .map_err(|_| NativeWindowPresenterError::ResourceExhausted)?;
        next_pixels.extend_from_slice(source_frame.pixels());

        self.last_pixels = next_pixels;
        self.last_frame_width = source_frame.width();
        self.last_frame_height = source_frame.height();
        self.last_frame_id = Some(frame_id);
        Ok(())
    }

    pub fn present_sink_frame(
        &mut self,
        sink: &NativeRgb0PresenterSink,
    ) -> Result<NativePresenterFrame<'_>, NativeWindowPresenterError> {
        let source_frame = sink
            .last_present_frame()
            .map_err(NativeWindowPresenterError::PresenterFrameValidationFailed)?
            .ok_or(NativeWindowPresenterError::FrameMissing)?;
        let frame_id = sink
            .last_presented_frame_id()
            .ok_or(NativeWindowPresenterError::FrameIdMissing)?;
        self.present_frame(frame_id, source_frame)?;
        self.last_present_frame_required()
    }
}

enum NativeWindowBackendLoopCounterIntent {
    None,
    PressedUnavailable,
    PressedOutside,
    Hit { value: i32, frame: GuiFrame },
}

impl NativeWindowBackendLoop {
    pub fn new_for_scale(
        demo: GuiDemo,
        counter_value: i32,
        scale: usize,
    ) -> Result<Self, NativeWindowBackendLoopError> {
        let frame = render_demo_frame(demo, counter_value);
        let initial_size = native_window_backend_loop_initial_size_for_frame(&frame, scale)?;
        let mut presenter_state =
            NativeWindowPresenterState::new(initial_size.width, initial_size.height)
                .map_err(|_| NativeWindowBackendLoopError::InitialSurfaceInvalid)?;
        let buffer = native_window_backend_loop_present_buffer_for_frame(
            &frame,
            initial_size.width,
            initial_size.height,
        )?;
        presenter_state
            .present_buffer(NATIVE_WINDOW_BACKEND_LOOP_INITIAL_FRAME_ID, &buffer)
            .map_err(NativeWindowBackendLoopError::PresenterFailed)?;

        Ok(Self {
            state: NativeWindowBackendLoopState {
                demo,
                counter_value,
                frame,
                presenter_frame_id: NATIVE_WINDOW_BACKEND_LOOP_INITIAL_FRAME_ID,
                previous_size: initial_size,
                previous_mouse_down: false,
            },
            presenter_state,
        })
    }

    pub fn initial_size(&self) -> NativeWindowSize {
        self.state.previous_size
    }

    pub fn event_pump_input(&self) -> NativeWindowEventPumpInput {
        NativeWindowEventPumpInput {
            previous_size: self.state.previous_size,
            previous_mouse_down: self.state.previous_mouse_down,
        }
    }

    pub fn demo(&self) -> GuiDemo {
        self.state.demo
    }

    pub fn counter_value(&self) -> i32 {
        self.state.counter_value
    }

    pub fn presenter_frame_id(&self) -> i32 {
        self.state.presenter_frame_id
    }

    pub fn presenter_state(&self) -> &NativeWindowPresenterState {
        &self.presenter_state
    }

    pub fn current_present_frame_for_window(
        &self,
    ) -> Result<NativePresenterFrame<'_>, NativeWindowBackendLoopError> {
        let NativeWindowPresenterSurfaceState::Drawable { width, height } =
            self.presenter_state.surface_state()
        else {
            return Err(NativeWindowBackendLoopError::SurfaceUnavailable);
        };
        let frame = self
            .presenter_state
            .last_present_frame_required()
            .map_err(native_window_backend_loop_frame_error)?;
        if frame.width() != width || frame.height() != height {
            return Err(NativeWindowBackendLoopError::FrameWindowMismatch {
                frame_width: frame.width(),
                frame_height: frame.height(),
                window_width: width,
                window_height: height,
            });
        }
        Ok(frame)
    }

    pub fn step(
        &mut self,
        snapshot: NativeWindowEventPumpSnapshot,
    ) -> Result<NativeWindowBackendLoopStepOutcome, NativeWindowBackendLoopError> {
        match snapshot.close_state {
            NativeWindowEventPumpCloseState::Open => {}
            NativeWindowEventPumpCloseState::OsCloseRequested
            | NativeWindowEventPumpCloseState::ExitShortcutRequested => {
                return Ok(NativeWindowBackendLoopStepOutcome::CloseRequested {
                    close_state: snapshot.close_state,
                });
            }
        }

        let NativeWindowPresenterSurfaceState::Drawable { width, height } = snapshot.surface_state
        else {
            self.presenter_state
                .resize_surface(snapshot.window_size.width, snapshot.window_size.height)
                .map_err(NativeWindowBackendLoopError::PresenterFailed)?;
            self.state.previous_size = snapshot.window_size;
            self.state.previous_mouse_down = snapshot.mouse_down;
            return Ok(NativeWindowBackendLoopStepOutcome::Unavailable {
                window_size: snapshot.window_size,
                size_changed: snapshot.size_changed,
            });
        };

        let counter_intent = self.counter_intent(width, height, snapshot)?;
        let resize_frame_id = if snapshot.size_changed {
            Some(native_window_backend_loop_next_frame_id(
                self.state.presenter_frame_id,
            )?)
        } else {
            None
        };
        let counter_frame_id = match counter_intent {
            NativeWindowBackendLoopCounterIntent::Hit { .. } => {
                let previous = resize_frame_id.unwrap_or(self.state.presenter_frame_id);
                Some(native_window_backend_loop_next_frame_id(previous)?)
            }
            NativeWindowBackendLoopCounterIntent::None
            | NativeWindowBackendLoopCounterIntent::PressedUnavailable
            | NativeWindowBackendLoopCounterIntent::PressedOutside => None,
        };

        let resize_redraw = match resize_frame_id {
            Some(frame_id) => {
                let frame = self.state.frame.clone();
                self.present_frame_to_surface_after_success(frame_id, &frame, width, height)?;
                self.presenter_state
                    .resize_surface(width, height)
                    .map_err(NativeWindowBackendLoopError::PresenterFailed)?;
                self.state.presenter_frame_id = frame_id;
                self.state.previous_size = snapshot.window_size;
                Some(NativeWindowBackendLoopPresentation {
                    frame_id,
                    width,
                    height,
                })
            }
            None => None,
        };

        let pointer_action = match counter_intent {
            NativeWindowBackendLoopCounterIntent::None => {
                NativeWindowBackendLoopPointerAction::None
            }
            NativeWindowBackendLoopCounterIntent::PressedUnavailable => {
                NativeWindowBackendLoopPointerAction::PressedUnavailable
            }
            NativeWindowBackendLoopCounterIntent::PressedOutside => {
                NativeWindowBackendLoopPointerAction::PressedOutside
            }
            NativeWindowBackendLoopCounterIntent::Hit { value, frame } => {
                let Some(frame_id) = counter_frame_id else {
                    return Err(NativeWindowBackendLoopError::CounterFrameIdMissing);
                };
                self.present_frame_to_surface_after_success(frame_id, &frame, width, height)?;
                self.state.counter_value = value;
                self.state.frame = frame;
                self.state.presenter_frame_id = frame_id;
                NativeWindowBackendLoopPointerAction::CounterIncremented {
                    value,
                    presentation: NativeWindowBackendLoopPresentation {
                        frame_id,
                        width,
                        height,
                    },
                }
            }
        };

        self.state.previous_size = snapshot.window_size;
        self.state.previous_mouse_down = snapshot.mouse_down;
        let final_frame = self.current_presentation_for_window(width, height)?;
        Ok(NativeWindowBackendLoopStepOutcome::Drawable(
            NativeWindowBackendLoopDrawableStep {
                window_size: snapshot.window_size,
                size_changed: snapshot.size_changed,
                resize_redraw,
                pointer_action,
                final_frame,
            },
        ))
    }

    pub fn step_host_action(
        &mut self,
        snapshot: NativeWindowEventPumpSnapshot,
    ) -> Result<NativeWindowHostAction, NativeWindowHostActionError> {
        let outcome = self
            .step(snapshot)
            .map_err(NativeWindowHostActionError::StepFailed)?;
        native_window_host_action_from_backend_loop_outcome(outcome)
    }

    fn counter_intent(
        &self,
        surface_width: usize,
        surface_height: usize,
        snapshot: NativeWindowEventPumpSnapshot,
    ) -> Result<NativeWindowBackendLoopCounterIntent, NativeWindowBackendLoopError> {
        if self.state.demo != GuiDemo::Counter
            || snapshot.mouse_left_transition != NativeWindowPointerButtonTransition::Pressed
        {
            return Ok(NativeWindowBackendLoopCounterIntent::None);
        }

        let NativeWindowPointerSample::Available {
            x: mouse_x,
            y: mouse_y,
        } = snapshot.pointer_sample
        else {
            return Ok(NativeWindowBackendLoopCounterIntent::PressedUnavailable);
        };

        let Some((image_x, image_y)) = map_native_window_point_to_image(
            surface_width,
            surface_height,
            self.state.frame.width,
            self.state.frame.height,
            mouse_x,
            mouse_y,
        ) else {
            return Ok(NativeWindowBackendLoopCounterIntent::PressedOutside);
        };
        if !counter_hit(&self.state.frame, image_x, image_y) {
            return Ok(NativeWindowBackendLoopCounterIntent::PressedOutside);
        }

        let value = self.state.counter_value.checked_add(1).ok_or(
            NativeWindowBackendLoopError::CounterValueOverflow {
                previous: self.state.counter_value,
            },
        )?;
        Ok(NativeWindowBackendLoopCounterIntent::Hit {
            value,
            frame: render_demo_frame(self.state.demo, value),
        })
    }

    fn present_frame_to_surface_after_success(
        &mut self,
        frame_id: i32,
        frame: &GuiFrame,
        surface_width: usize,
        surface_height: usize,
    ) -> Result<(), NativeWindowBackendLoopError> {
        let buffer = native_window_backend_loop_present_buffer_for_frame(
            frame,
            surface_width,
            surface_height,
        )?;
        self.presenter_state
            .present_buffer(frame_id, &buffer)
            .map_err(NativeWindowBackendLoopError::PresenterFailed)
    }

    fn current_presentation_for_window(
        &self,
        window_width: usize,
        window_height: usize,
    ) -> Result<NativeWindowBackendLoopPresentation, NativeWindowBackendLoopError> {
        let frame = self
            .presenter_state
            .last_present_frame_required()
            .map_err(native_window_backend_loop_frame_error)?;
        if frame.width() != window_width || frame.height() != window_height {
            return Err(NativeWindowBackendLoopError::FrameWindowMismatch {
                frame_width: frame.width(),
                frame_height: frame.height(),
                window_width,
                window_height,
            });
        }
        Ok(NativeWindowBackendLoopPresentation {
            frame_id: self.state.presenter_frame_id,
            width: frame.width(),
            height: frame.height(),
        })
    }
}

fn native_window_host_action_from_backend_loop_outcome(
    outcome: NativeWindowBackendLoopStepOutcome,
) -> Result<NativeWindowHostAction, NativeWindowHostActionError> {
    match outcome {
        NativeWindowBackendLoopStepOutcome::CloseRequested { close_state } => {
            let reason = match close_state {
                NativeWindowEventPumpCloseState::OsCloseRequested => {
                    NativeWindowHostTerminalReason::OsCloseRequested
                }
                NativeWindowEventPumpCloseState::ExitShortcutRequested => {
                    NativeWindowHostTerminalReason::ExitShortcutRequested
                }
                NativeWindowEventPumpCloseState::Open => {
                    return Err(NativeWindowHostActionError::UnsupportedCloseState { close_state });
                }
            };
            Ok(NativeWindowHostAction::Terminate { reason })
        }
        NativeWindowBackendLoopStepOutcome::Unavailable {
            window_size,
            size_changed,
        } => Ok(NativeWindowHostAction::PumpEventsOnly {
            window_size,
            size_changed,
        }),
        NativeWindowBackendLoopStepOutcome::Drawable(drawable) => {
            Ok(NativeWindowHostAction::PresentFrame {
                presentation: drawable.final_frame,
                window_size: drawable.window_size,
                size_changed: drawable.size_changed,
            })
        }
    }
}

const NATIVE_WINDOW_BACKEND_LOOP_INITIAL_FRAME_ID: i32 = 1;

fn native_window_backend_loop_next_frame_id(
    previous: i32,
) -> Result<i32, NativeWindowBackendLoopError> {
    previous
        .checked_add(1)
        .ok_or(NativeWindowBackendLoopError::FrameIdOverflow { previous })
}

fn native_window_backend_loop_initial_size_for_frame(
    frame: &GuiFrame,
    scale: usize,
) -> Result<NativeWindowSize, NativeWindowBackendLoopError> {
    if scale == 0 {
        return Err(NativeWindowBackendLoopError::InitialScaleInvalid);
    }
    let width = frame
        .width
        .checked_mul(scale)
        .ok_or(NativeWindowBackendLoopError::InitialSizeOverflow)?;
    let height = frame
        .height
        .checked_mul(scale)
        .ok_or(NativeWindowBackendLoopError::InitialSizeOverflow)?;
    if width == 0 || height == 0 {
        return Err(NativeWindowBackendLoopError::InitialSurfaceInvalid);
    }
    Ok(NativeWindowSize::new(width, height))
}

fn native_window_backend_loop_present_buffer_for_frame(
    frame: &GuiFrame,
    surface_width: usize,
    surface_height: usize,
) -> Result<NativeRgb0PresentBuffer, NativeWindowBackendLoopError> {
    let image = rasterize_frame_to_surface(frame, surface_width, surface_height)
        .map_err(NativeWindowBackendLoopError::RasterizeFailed)?;
    NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(
        image.width,
        image.height,
        image.pixels,
    )
    .map_err(NativeWindowBackendLoopError::PresentBufferInvalid)
}

fn native_window_backend_loop_frame_error(
    error: NativeWindowPresenterError,
) -> NativeWindowBackendLoopError {
    match error {
        NativeWindowPresenterError::FrameMissing => NativeWindowBackendLoopError::FrameMissing,
        other => NativeWindowBackendLoopError::PresenterFailed(other),
    }
}

impl NativeWindowPresenterSession {
    pub fn new(
        framebuffer_width: i32,
        framebuffer_height: i32,
        background: NativeRgbColor,
        surface_width: usize,
        surface_height: usize,
    ) -> Result<Self, NativeWindowPresenterSessionError> {
        let sink = NativeRgb0PresenterSink::new(framebuffer_width, framebuffer_height, background)
            .map_err(NativeWindowPresenterSessionError::SinkFailed)?;
        let presenter_state = NativeWindowPresenterState::new(surface_width, surface_height)
            .map_err(NativeWindowPresenterSessionError::PresenterFailed)?;
        Ok(Self {
            sink,
            presenter_state,
        })
    }

    pub fn sink(&self) -> &NativeRgb0PresenterSink {
        &self.sink
    }

    pub fn presenter_state(&self) -> &NativeWindowPresenterState {
        &self.presenter_state
    }

    pub fn resize_surface(
        &mut self,
        surface_width: usize,
        surface_height: usize,
    ) -> Result<(), NativeWindowPresenterSessionError> {
        self.presenter_state
            .resize_surface(surface_width, surface_height)
            .map_err(NativeWindowPresenterSessionError::PresenterFailed)
    }

    pub fn execute_span_operation(
        &mut self,
        operation: NativeSpanOperation,
    ) -> Result<NativeWindowPresenterSessionOutcome, NativeWindowPresenterSessionError> {
        match self
            .sink
            .execute_span_operation_typed(operation)
            .map_err(NativeWindowPresenterSessionError::SinkFailed)?
        {
            NativeRgb0PresenterSinkOutcome::Accepted => {
                Ok(NativeWindowPresenterSessionOutcome::NotPresented)
            }
            NativeRgb0PresenterSinkOutcome::Completed { frame_id } => {
                let (width, height) = {
                    let frame = self
                        .presenter_state
                        .present_sink_frame(&self.sink)
                        .map_err(NativeWindowPresenterSessionError::PresenterFailed)?;
                    (frame.width(), frame.height())
                };
                Ok(NativeWindowPresenterSessionOutcome::Presented {
                    frame_id,
                    width,
                    height,
                })
            }
        }
    }
}

impl NativeSpanOperationSink for NativeRgb0PresenterSink {
    fn execute_span_operation(&mut self, operation: NativeSpanOperation) -> i32 {
        match self.execute_span_operation_typed(operation) {
            Ok(_) => GUI_NATIVE_SPAN_OPERATION_STATUS_OK,
            Err(error) => error.status(),
        }
    }
}

impl<'a> NativePresenterFrame<'a> {
    /// Borrows a checked RGB0 buffer as a presenter-ready immutable frame.
    pub fn from_rgb0_present_buffer(
        buffer: &'a NativeRgb0PresentBuffer,
    ) -> Result<Self, NativePresenterFrameError> {
        if buffer.width <= 0 || buffer.height <= 0 {
            return Err(NativePresenterFrameError::InvalidDimensions);
        }
        let width = usize::try_from(buffer.width)
            .map_err(|_| NativePresenterFrameError::DimensionOverflow)?;
        let height = usize::try_from(buffer.height)
            .map_err(|_| NativePresenterFrameError::DimensionOverflow)?;
        native_presenter_frame_from_rgb0_parts(width, height, &buffer.pixels)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &'a [u32] {
        self.pixels
    }
}

fn native_presenter_frame_from_rgb0_parts(
    width: usize,
    height: usize,
    pixels: &[u32],
) -> Result<NativePresenterFrame<'_>, NativePresenterFrameError> {
    if width == 0 || height == 0 {
        return Err(NativePresenterFrameError::InvalidDimensions);
    }
    let pixel_count = width
        .checked_mul(height)
        .ok_or(NativePresenterFrameError::DimensionOverflow)?;
    if pixels.len() != pixel_count {
        return Err(NativePresenterFrameError::PixelCountMismatch);
    }
    if pixels
        .iter()
        .any(|pixel| pixel & NATIVE_RGB0_HIGH_BYTE_MASK != 0)
    {
        return Err(NativePresenterFrameError::PixelFormatMismatch);
    }
    Ok(NativePresenterFrame {
        width,
        height,
        pixels,
    })
}

fn native_rgb0_present_buffer_from_rgba8888_parts(
    width: i32,
    height: i32,
    rgba8888_pixels: &[u32],
    background: NativeRgbColor,
) -> Result<NativeRgb0PresentBuffer, NativeSpanFramebufferError> {
    if width <= 0 || height <= 0 {
        return Err(NativeSpanFramebufferError::InvalidDimensions);
    }
    let pixel_count = width
        .checked_mul(height)
        .ok_or(NativeSpanFramebufferError::DimensionOverflow)?;
    let pixel_count =
        usize::try_from(pixel_count).map_err(|_| NativeSpanFramebufferError::DimensionOverflow)?;
    if pixel_count != rgba8888_pixels.len() {
        return Err(NativeSpanFramebufferError::InternalIndexOverflow);
    }

    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(pixel_count)
        .map_err(|_| NativeSpanFramebufferError::ResourceExhausted)?;
    for pixel in rgba8888_pixels {
        pixels.push(native_rgba8888_to_rgb0_over_background(*pixel, background));
    }

    Ok(NativeRgb0PresentBuffer {
        width,
        height,
        pixels,
    })
}

pub fn native_pack_rgb0_pixel(r: u8, g: u8, b: u8) -> u32 {
    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
}

pub fn native_rgba8888_to_rgb0_over_background(rgba8888: u32, background: NativeRgbColor) -> u32 {
    let source_r = ((rgba8888 >> 24) & 0xff) as u8;
    let source_g = ((rgba8888 >> 16) & 0xff) as u8;
    let source_b = ((rgba8888 >> 8) & 0xff) as u8;
    let source_a = (rgba8888 & 0xff) as u8;
    native_pack_rgb0_pixel(
        native_blend_rgba8888_channel(source_r, background.r, source_a),
        native_blend_rgba8888_channel(source_g, background.g, source_a),
        native_blend_rgba8888_channel(source_b, background.b, source_a),
    )
}

fn native_blend_rgba8888_channel(source: u8, background: u8, alpha: u8) -> u8 {
    let alpha = u32::from(alpha);
    let inverse_alpha = 255 - alpha;
    let blended = (u32::from(source) * alpha + u32::from(background) * inverse_alpha + 127) / 255;
    blended as u8
}

fn native_span_framebuffer_index(
    frame_width: i32,
    x: i32,
    y: i32,
) -> Result<usize, NativeSpanFramebufferError> {
    let row_offset = y
        .checked_mul(frame_width)
        .ok_or(NativeSpanFramebufferError::InternalIndexOverflow)?;
    let index = row_offset
        .checked_add(x)
        .ok_or(NativeSpanFramebufferError::InternalIndexOverflow)?;
    usize::try_from(index).map_err(|_| NativeSpanFramebufferError::InternalIndexOverflow)
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

impl NativeWindowRunLoopConfig {
    pub fn new(demo: GuiDemo, counter_value: i32, scale: usize) -> Self {
        Self {
            demo,
            counter_value,
            scale,
            target_fps: NativeWindowTargetFps::default(),
            host_loop_policy: NativeWindowHostLoopRunPolicy::default(),
        }
    }

    pub fn new_with_target_fps(
        demo: GuiDemo,
        counter_value: i32,
        scale: usize,
        target_fps: NativeWindowTargetFps,
    ) -> Self {
        Self {
            demo,
            counter_value,
            scale,
            target_fps,
            host_loop_policy: NativeWindowHostLoopRunPolicy::default(),
        }
    }

    pub fn new_with_target_fps_and_host_loop_policy(
        demo: GuiDemo,
        counter_value: i32,
        scale: usize,
        target_fps: NativeWindowTargetFps,
        host_loop_policy: NativeWindowHostLoopRunPolicy,
    ) -> Self {
        Self {
            demo,
            counter_value,
            scale,
            target_fps,
            host_loop_policy,
        }
    }

    pub fn try_new_with_raw_target_fps(
        demo: GuiDemo,
        counter_value: i32,
        scale: usize,
        target_fps: usize,
    ) -> Result<Self, NativeWindowRunLoopError> {
        match NativeWindowTargetFps::new(target_fps) {
            Ok(target_fps) => Ok(Self::new_with_target_fps(
                demo,
                counter_value,
                scale,
                target_fps,
            )),
            Err(error) => Err(NativeWindowRunLoopError::TargetFpsInvalid {
                value: error.value,
                reason: error.reason,
            }),
        }
    }
}

pub fn native_window_title(demo: GuiDemo, size: NativeWindowSize) -> String {
    if size.width == 0 || size.height == 0 {
        format!(
            "NEPLg2 GUI native preview - {:?} - surface unavailable",
            demo
        )
    } else {
        format!(
            "NEPLg2 GUI native preview - {:?} - {}x{}",
            demo, size.width, size.height
        )
    }
}

pub fn initialize_native_window_host_loop<Host>(
    runner_state: &mut NativeWindowHostLoopRunnerState,
    backend_loop: &NativeWindowBackendLoop,
    host: &mut Host,
) -> NativeWindowHostLoopInitialization
where
    Host: NativeWindowRunLoopHost,
{
    if runner_state.title_initialized {
        return NativeWindowHostLoopInitialization::AlreadyInitialized;
    }
    let initial_title = native_window_title(backend_loop.demo(), backend_loop.initial_size());
    host.set_window_title(&initial_title);
    runner_state.title_initialized = true;
    NativeWindowHostLoopInitialization::Initialized
}

pub fn native_window_host_loop_wait_decision(
    evidence: NativeWindowHostLoopContinueEvidence,
) -> NativeWindowHostLoopWaitDecision {
    match evidence {
        NativeWindowHostLoopContinueEvidence::PumpedEventsOnly {
            window_size,
            size_changed,
        } => NativeWindowHostLoopWaitDecision::WaitForHostEvent {
            window_size,
            size_changed,
        },
        NativeWindowHostLoopContinueEvidence::PresentedFrame {
            presentation,
            window_size,
            size_changed,
        } => NativeWindowHostLoopWaitDecision::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
        },
    }
}

pub fn native_window_frame_interval_request(
    target_fps: NativeWindowTargetFps,
) -> NativeWindowFrameIntervalRequest {
    let target_fps_value = u32::from(target_fps.value());
    NativeWindowFrameIntervalRequest {
        target_fps,
        nanos_per_frame: NATIVE_WINDOW_NANOS_PER_SECOND / target_fps_value,
        remainder_nanos_per_second: NATIVE_WINDOW_NANOS_PER_SECOND % target_fps_value,
    }
}

pub fn native_window_host_loop_wait_request(
    decision: NativeWindowHostLoopWaitDecision,
    target_fps: NativeWindowTargetFps,
) -> NativeWindowHostLoopWaitRequest {
    match decision {
        NativeWindowHostLoopWaitDecision::WaitForHostEvent {
            window_size,
            size_changed,
        } => NativeWindowHostLoopWaitRequest::WaitForHostEvent {
            window_size,
            size_changed,
        },
        NativeWindowHostLoopWaitDecision::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
        } => NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
            frame_interval: native_window_frame_interval_request(target_fps),
        },
    }
}

pub fn run_native_window_host_loop_bounded<Host>(
    runner_state: &mut NativeWindowHostLoopRunnerState,
    backend_loop: &mut NativeWindowBackendLoop,
    host: &mut Host,
    max_turn_count: usize,
) -> Result<
    NativeWindowHostLoopBoundedRunResult,
    NativeWindowHostLoopError<Host::EventError, Host::PresentError, Host::WaitError>,
>
where
    Host: NativeWindowRunLoopHost,
{
    match initialize_native_window_host_loop(runner_state, backend_loop, host) {
        NativeWindowHostLoopInitialization::Initialized
        | NativeWindowHostLoopInitialization::AlreadyInitialized => {}
    }
    let mut completed_turns = 0usize;
    let mut last_wait_decision = None;
    while completed_turns < max_turn_count {
        match step_native_window_host_loop(backend_loop, host)? {
            NativeWindowHostLoopTurn::Continue(evidence) => {
                last_wait_decision = Some(native_window_host_loop_wait_decision(evidence));
                completed_turns += 1;
            }
            NativeWindowHostLoopTurn::Exit(exit) => {
                return Ok(NativeWindowHostLoopBoundedRunResult::Exited {
                    exit,
                    completed_turns: completed_turns + 1,
                });
            }
        }
    }
    Ok(NativeWindowHostLoopBoundedRunResult::BudgetExhausted {
        completed_turns,
        last_wait_decision,
    })
}

pub fn run_native_window_host_loop_scheduler_slice_with_policy<Host>(
    scheduler_state: &mut NativeWindowHostLoopSchedulerState,
    backend_loop: &mut NativeWindowBackendLoop,
    host: &mut Host,
    policy: NativeWindowHostLoopRunPolicy,
) -> Result<
    NativeWindowHostLoopSchedulerSliceResult,
    NativeWindowHostLoopError<Host::EventError, Host::PresentError, Host::WaitError>,
>
where
    Host: NativeWindowRunLoopHost,
{
    run_native_window_host_loop_scheduler_slice_with_policy_and_target_fps(
        scheduler_state,
        backend_loop,
        host,
        policy,
        NativeWindowTargetFps::default(),
    )
}

pub fn run_native_window_host_loop_scheduler_slice_with_policy_and_target_fps<Host>(
    scheduler_state: &mut NativeWindowHostLoopSchedulerState,
    backend_loop: &mut NativeWindowBackendLoop,
    host: &mut Host,
    policy: NativeWindowHostLoopRunPolicy,
    target_fps: NativeWindowTargetFps,
) -> Result<
    NativeWindowHostLoopSchedulerSliceResult,
    NativeWindowHostLoopError<Host::EventError, Host::PresentError, Host::WaitError>,
>
where
    Host: NativeWindowRunLoopHost,
{
    let max_turn_count = policy.turn_slice.as_usize();
    match run_native_window_host_loop_bounded(
        &mut scheduler_state.runner_state,
        backend_loop,
        host,
        max_turn_count,
    )? {
        NativeWindowHostLoopBoundedRunResult::Exited {
            exit,
            completed_turns,
        } => Ok(NativeWindowHostLoopSchedulerSliceResult::Exited {
            exit,
            completed_turns,
        }),
        NativeWindowHostLoopBoundedRunResult::BudgetExhausted {
            completed_turns,
            last_wait_decision: Some(decision),
        } => {
            let request = native_window_host_loop_wait_request(decision.clone(), target_fps);
            let outcome = host
                .wait_after_budget_exhausted(request.clone())
                .map_err(NativeWindowHostLoopError::HostWaitFailed)?;
            Ok(NativeWindowHostLoopSchedulerSliceResult::Waited {
                completed_turns,
                decision,
                request,
                outcome,
            })
        }
        NativeWindowHostLoopBoundedRunResult::BudgetExhausted {
            last_wait_decision: None,
            ..
        } => Err(NativeWindowHostLoopError::WaitDecisionMissing),
    }
}

pub fn run_native_window_host_loop<Host>(
    backend_loop: &mut NativeWindowBackendLoop,
    host: &mut Host,
) -> Result<
    NativeWindowRunLoopExit,
    NativeWindowHostLoopError<Host::EventError, Host::PresentError, Host::WaitError>,
>
where
    Host: NativeWindowRunLoopHost,
{
    run_native_window_host_loop_with_policy(
        backend_loop,
        host,
        NativeWindowHostLoopRunPolicy::default(),
    )
}

pub fn run_native_window_host_loop_with_policy<Host>(
    backend_loop: &mut NativeWindowBackendLoop,
    host: &mut Host,
    policy: NativeWindowHostLoopRunPolicy,
) -> Result<
    NativeWindowRunLoopExit,
    NativeWindowHostLoopError<Host::EventError, Host::PresentError, Host::WaitError>,
>
where
    Host: NativeWindowRunLoopHost,
{
    run_native_window_host_loop_with_policy_and_target_fps(
        backend_loop,
        host,
        policy,
        NativeWindowTargetFps::default(),
    )
}

pub fn run_native_window_host_loop_with_policy_and_target_fps<Host>(
    backend_loop: &mut NativeWindowBackendLoop,
    host: &mut Host,
    policy: NativeWindowHostLoopRunPolicy,
    target_fps: NativeWindowTargetFps,
) -> Result<
    NativeWindowRunLoopExit,
    NativeWindowHostLoopError<Host::EventError, Host::PresentError, Host::WaitError>,
>
where
    Host: NativeWindowRunLoopHost,
{
    let mut scheduler_state = NativeWindowHostLoopSchedulerState::new();
    loop {
        match run_native_window_host_loop_scheduler_slice_with_policy_and_target_fps(
            &mut scheduler_state,
            backend_loop,
            host,
            policy,
            target_fps,
        )? {
            NativeWindowHostLoopSchedulerSliceResult::Exited { exit, .. } => return Ok(exit),
            NativeWindowHostLoopSchedulerSliceResult::Waited { .. } => {}
        }
    }
}

pub fn step_native_window_host_loop<Host>(
    backend_loop: &mut NativeWindowBackendLoop,
    host: &mut Host,
) -> Result<
    NativeWindowHostLoopTurn,
    NativeWindowHostLoopError<Host::EventError, Host::PresentError, Host::WaitError>,
>
where
    Host: NativeWindowRunLoopHost,
{
    let event_snapshot = host
        .poll_event_snapshot(backend_loop.event_pump_input())
        .map_err(NativeWindowHostLoopError::HostEventPumpFailed)?;
    let action = backend_loop
        .step_host_action(event_snapshot)
        .map_err(NativeWindowHostLoopError::HostActionFailed)?;
    match action {
        NativeWindowHostAction::Terminate { reason } => {
            Ok(NativeWindowHostLoopTurn::Exit(NativeWindowRunLoopExit {
                reason,
            }))
        }
        NativeWindowHostAction::PumpEventsOnly {
            window_size,
            size_changed,
        } => {
            if size_changed {
                let title = native_window_title(backend_loop.demo(), window_size);
                host.set_window_title(&title);
            }
            host.pump_events_only();
            Ok(NativeWindowHostLoopTurn::Continue(
                NativeWindowHostLoopContinueEvidence::PumpedEventsOnly {
                    window_size,
                    size_changed,
                },
            ))
        }
        NativeWindowHostAction::PresentFrame {
            presentation,
            window_size,
            size_changed,
        } => {
            if size_changed {
                let title = native_window_title(backend_loop.demo(), window_size);
                host.set_window_title(&title);
            }
            let present_frame = backend_loop
                .current_present_frame_for_window()
                .map_err(NativeWindowHostLoopError::PresenterFrameUnavailable)?;
            host.present_frame(present_frame)
                .map_err(NativeWindowHostLoopError::HostPresentFailed)?;
            Ok(NativeWindowHostLoopTurn::Continue(
                NativeWindowHostLoopContinueEvidence::PresentedFrame {
                    presentation,
                    window_size,
                    size_changed,
                },
            ))
        }
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
struct MinifbNativeWindowRunLoopHost<'window> {
    window: &'window mut minifb::Window,
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
impl NativeWindowRunLoopHost for MinifbNativeWindowRunLoopHost<'_> {
    type EventError = NativeWindowEventPumpError;
    type PresentError = String;
    type WaitError = std::convert::Infallible;

    fn poll_event_snapshot(
        &mut self,
        input: NativeWindowEventPumpInput,
    ) -> Result<NativeWindowEventPumpSnapshot, Self::EventError> {
        poll_minifb_window_event_pump(self.window, input)
    }

    fn set_window_title(&mut self, title: &str) {
        self.window.set_title(title);
    }

    fn pump_events_only(&mut self) {
        self.window.update();
    }

    fn present_frame(&mut self, frame: NativePresenterFrame<'_>) -> Result<(), Self::PresentError> {
        self.window
            .update_with_buffer(frame.pixels(), frame.width(), frame.height())
            .map_err(|error| error.to_string())
    }

    fn wait_after_budget_exhausted(
        &mut self,
        request: NativeWindowHostLoopWaitRequest,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
        Ok(match request {
            NativeWindowHostLoopWaitRequest::WaitForHostEvent {
                window_size,
                size_changed,
            } => NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed,
            },
            NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
                presentation,
                window_size,
                size_changed,
                frame_interval: _,
            } => NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
                presentation,
                window_size,
                size_changed,
            },
        })
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
pub fn run_minifb_window_loop(
    config: NativeWindowRunLoopConfig,
) -> Result<NativeWindowRunLoopExit, NativeWindowRunLoopError> {
    use minifb::{ScaleMode, Window, WindowOptions};

    let target_fps = config.target_fps.as_usize();
    let mut backend_loop =
        NativeWindowBackendLoop::new_for_scale(config.demo, config.counter_value, config.scale)
            .map_err(NativeWindowRunLoopError::BackendLoopInitializationFailed)?;
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
    .map_err(|error| NativeWindowRunLoopError::WindowCreationFailed {
        message: error.to_string(),
    })?;
    window.set_target_fps(target_fps);
    window.set_background_color(9, 13, 18);

    let mut host = MinifbNativeWindowRunLoopHost {
        window: &mut window,
    };
    run_native_window_host_loop_with_policy_and_target_fps(
        &mut backend_loop,
        &mut host,
        config.host_loop_policy,
        config.target_fps,
    )
    .map_err(native_window_run_loop_error_from_host_loop)
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn native_window_run_loop_error_from_host_loop(
    error: NativeWindowHostLoopError<NativeWindowEventPumpError, String, std::convert::Infallible>,
) -> NativeWindowRunLoopError {
    match error {
        NativeWindowHostLoopError::HostEventPumpFailed(error) => {
            NativeWindowRunLoopError::EventPumpFailed(error)
        }
        NativeWindowHostLoopError::HostActionFailed(error) => {
            NativeWindowRunLoopError::HostActionFailed(error)
        }
        NativeWindowHostLoopError::PresenterFrameUnavailable(error) => {
            NativeWindowRunLoopError::PresenterFrameUnavailable(error)
        }
        NativeWindowHostLoopError::HostPresentFailed(message) => {
            NativeWindowRunLoopError::WindowPresentFailed { message }
        }
        NativeWindowHostLoopError::HostWaitFailed(error) => match error {},
        NativeWindowHostLoopError::WaitDecisionMissing => {
            NativeWindowRunLoopError::WaitDecisionMissing
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

pub fn rasterize_frame_to_surface(
    frame: &GuiFrame,
    surface_width: usize,
    surface_height: usize,
) -> Result<RasterImage, RasterizeSurfaceError> {
    if frame.width == 0 || frame.height == 0 || surface_width == 0 || surface_height == 0 {
        return Err(RasterizeSurfaceError::InvalidDimensions);
    }
    let pixel_count = surface_width
        .checked_mul(surface_height)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(pixel_count)
        .map_err(|_| RasterizeSurfaceError::ResourceExhausted)?;
    pixels.resize(pixel_count, 0x0d1117);

    let NativeSurfaceState::Drawable(placement) =
        native_aspect_ratio_placement(surface_width, surface_height, frame.width, frame.height)
    else {
        return Err(RasterizeSurfaceError::InvalidDimensions);
    };

    for rect in &frame.rects {
        fill_surface_rect(
            &mut pixels,
            surface_width,
            surface_height,
            placement,
            frame.width,
            frame.height,
            rect,
        )?;
    }

    Ok(RasterImage {
        width: surface_width,
        height: surface_height,
        pixels,
    })
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
        let width = ceil_div_u128(
            (height as u128) * (image_width as u128),
            image_height as u128,
        )
        .min(window_width as u128) as usize;
        (width, height)
    } else {
        let width = window_width;
        let height = ceil_div_u128(
            (width as u128) * (image_height as u128),
            image_width as u128,
        )
        .min(window_height as u128) as usize;
        (width, height)
    };
    if width == 0 || height == 0 {
        return NativeSurfaceState::Unavailable;
    }

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

fn fill_surface_rect(
    pixels: &mut [u32],
    surface_width: usize,
    surface_height: usize,
    placement: NativeSurfacePlacement,
    frame_width: usize,
    frame_height: usize,
    rect: &RectCommand,
) -> Result<(), RasterizeSurfaceError> {
    if rect.width == 0 || rect.height == 0 {
        return Err(RasterizeSurfaceError::CommandOutOfBounds);
    }
    let rect_x_end = rect
        .x
        .checked_add(rect.width)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    let rect_y_end = rect
        .y
        .checked_add(rect.height)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    if rect_x_end > frame_width || rect_y_end > frame_height {
        return Err(RasterizeSurfaceError::CommandOutOfBounds);
    }

    let x0 = placement
        .x
        .checked_add(scale_floor(rect.x, placement.width, frame_width)?)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    let y0 = placement
        .y
        .checked_add(scale_floor(rect.y, placement.height, frame_height)?)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    let x1 = placement
        .x
        .checked_add(scale_ceil(rect_x_end, placement.width, frame_width)?)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    let y1 = placement
        .y
        .checked_add(scale_ceil(rect_y_end, placement.height, frame_height)?)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    if x1 > surface_width || y1 > surface_height || x0 >= x1 || y0 >= y1 {
        return Err(RasterizeSurfaceError::CommandOutOfBounds);
    }

    for y in y0..y1 {
        let row = y
            .checked_mul(surface_width)
            .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
        for x in x0..x1 {
            pixels[row + x] = rect.color;
        }
    }
    Ok(())
}

fn scale_floor(
    value: usize,
    destination_span: usize,
    source_span: usize,
) -> Result<usize, RasterizeSurfaceError> {
    if source_span == 0 {
        return Err(RasterizeSurfaceError::InvalidDimensions);
    }
    let numerator = (value as u128)
        .checked_mul(destination_span as u128)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    usize::try_from(numerator / source_span as u128)
        .map_err(|_| RasterizeSurfaceError::DimensionOverflow)
}

fn scale_ceil(
    value: usize,
    destination_span: usize,
    source_span: usize,
) -> Result<usize, RasterizeSurfaceError> {
    if source_span == 0 {
        return Err(RasterizeSurfaceError::InvalidDimensions);
    }
    let numerator = (value as u128)
        .checked_mul(destination_span as u128)
        .ok_or(RasterizeSurfaceError::DimensionOverflow)?;
    usize::try_from(ceil_div_u128(numerator, source_span as u128))
        .map_err(|_| RasterizeSurfaceError::DimensionOverflow)
}

fn ceil_div_u128(numerator: u128, denominator: u128) -> u128 {
    if numerator == 0 {
        0
    } else {
        ((numerator - 1) / denominator) + 1
    }
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
    fn rasterize_frame_to_surface_matches_drawable_size() {
        let frame = render_demo_frame(GuiDemo::Counter, 3);
        let image = rasterize_frame_to_surface(&frame, 640, 480).unwrap();

        assert_eq!(image.width, 640);
        assert_eq!(image.height, 480);
        assert_eq!(image.pixels.len(), 640 * 480);
        assert_eq!(image.pixels[0], 0x0d1117);
    }

    #[test]
    fn rasterize_frame_to_surface_keeps_counter_hit_mapping() {
        let frame = render_demo_frame(GuiDemo::Counter, 3);
        let NativeSurfaceState::Drawable(placement) =
            native_aspect_ratio_placement(640, 480, frame.width, frame.height)
        else {
            panic!("counter frame should be drawable");
        };
        let scene_x = 20usize;
        let scene_y = 90usize;
        let window_x = placement.x + (scene_x * placement.width / frame.width);
        let window_y = placement.y + (scene_y * placement.height / frame.height);

        let Some((mapped_x, mapped_y)) = map_native_window_point_to_image(
            640,
            480,
            frame.width,
            frame.height,
            window_x as f32,
            window_y as f32,
        ) else {
            panic!("counter button point should map into frame");
        };

        assert!(counter_hit(&frame, mapped_x, mapped_y));
    }

    #[test]
    fn rasterize_frame_to_surface_rejects_invalid_surface() {
        let frame = render_demo_frame(GuiDemo::Counter, 0);

        assert_eq!(
            rasterize_frame_to_surface(&frame, 0, 480).unwrap_err(),
            RasterizeSurfaceError::InvalidDimensions
        );
        assert_eq!(
            rasterize_frame_to_surface(&frame, 640, 0).unwrap_err(),
            RasterizeSurfaceError::InvalidDimensions
        );
    }

    #[test]
    fn rasterize_frame_to_surface_rejects_out_of_bounds_command() {
        let mut frame = render_demo_frame(GuiDemo::Counter, 0);
        frame.rects.push(RectCommand {
            x: frame.width,
            y: 0,
            width: 1,
            height: 1,
            color: 0x00ff00,
        });

        assert_eq!(
            rasterize_frame_to_surface(&frame, 640, 480).unwrap_err(),
            RasterizeSurfaceError::CommandOutOfBounds
        );
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

    fn native_framebuffer_descriptor(total_run_count: i32) -> NativeSpanOperationDescriptor {
        NativeSpanOperationDescriptor {
            target: NativeSpanOperationTarget::Window { window_id: 7 },
            surface_id: 10,
            frame_id: 11,
            packet_frame_id: 11,
            batch_index: 0,
            tile_index: 0,
            plan_row_start: 0,
            plan_row_count: 2,
            row_start: 0,
            row_count: 2,
            width: 4,
            height: 3,
            stride_bytes: 16,
            tile_rows: 2,
            tile_count: 1,
            pixel_count: 8,
            total_run_count,
            encoded_byte_count: total_run_count * 12,
        }
    }

    fn native_framebuffer_run(x: i32, y: i32, width: i32, r: u8) -> NativeSpanOperationRunSpan {
        NativeSpanOperationRunSpan {
            target: NativeSpanOperationTarget::Window { window_id: 7 },
            x,
            y,
            width,
            height: 1,
            r,
            g: 20,
            b: 30,
            a: 255,
        }
    }

    #[test]
    fn native_span_framebuffer_constructor_checks_dimensions_and_layout() {
        assert_eq!(
            NativeRgba8888FrameBuffer::new(0, 2).unwrap_err(),
            NativeSpanFramebufferError::InvalidDimensions
        );
        assert_eq!(
            NativeRgba8888FrameBuffer::new(i32::MAX, 2).unwrap_err(),
            NativeSpanFramebufferError::DimensionOverflow
        );

        let frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();
        assert_eq!(frame_buffer.width(), 4);
        assert_eq!(frame_buffer.height(), 3);
        assert_eq!(frame_buffer.stride_bytes(), 16);
        assert_eq!(frame_buffer.pixels().len(), 12);
        assert!(frame_buffer
            .pixels()
            .iter()
            .all(|pixel| *pixel == NATIVE_RGBA8888_PIXEL_TRANSPARENT));
        assert_eq!(frame_buffer.active_sequence(), None);
    }

    #[test]
    fn native_span_framebuffer_writes_complete_sequence() {
        let descriptor = native_framebuffer_descriptor(2);
        let mut frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();

        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(1, 0, 2, 10),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(frame_buffer.active_sequence().unwrap().seen_run_count, 1);
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(0, 1, 4, 40),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );

        assert_eq!(frame_buffer.active_sequence(), None);
        assert_eq!(
            frame_buffer.pixel_at(0, 0),
            Some(NATIVE_RGBA8888_PIXEL_TRANSPARENT)
        );
        assert_eq!(
            frame_buffer.pixel_at(1, 0),
            Some(native_pack_rgba8888_pixel(10, 20, 30, 255))
        );
        assert_eq!(
            frame_buffer.pixel_at(2, 0),
            Some(native_pack_rgba8888_pixel(10, 20, 30, 255))
        );
        assert_eq!(
            frame_buffer.pixel_at(3, 1),
            Some(native_pack_rgba8888_pixel(40, 20, 30, 255))
        );
    }

    #[test]
    fn native_span_framebuffer_rejects_missing_and_nested_sequence() {
        let descriptor = native_framebuffer_descriptor(1);
        let mut frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();

        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(0, 0, 1, 10),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(
            frame_buffer.active_sequence().unwrap(),
            NativeSpanFramebufferActiveSequence {
                descriptor,
                seen_run_count: 0,
            }
        );
    }

    #[test]
    fn native_span_framebuffer_rejects_invalid_run_without_partial_write() {
        let descriptor = native_framebuffer_descriptor(1);
        let mut frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        let before = frame_buffer.pixels().to_vec();

        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(3, 0, 2, 10),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(frame_buffer.pixels(), before.as_slice());
        assert_eq!(frame_buffer.active_sequence().unwrap().seen_run_count, 0);

        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(-1, 1, 1, 10),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(frame_buffer.pixels(), before.as_slice());
        assert_eq!(frame_buffer.active_sequence().unwrap().seen_run_count, 0);

        let wrong_height = NativeSpanOperationRunSpan {
            height: 2,
            ..native_framebuffer_run(0, 0, 1, 10)
        };
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(wrong_height)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(frame_buffer.pixels(), before.as_slice());
        assert_eq!(frame_buffer.active_sequence().unwrap().seen_run_count, 0);

        let wrong_target = NativeSpanOperationRunSpan {
            target: NativeSpanOperationTarget::Window { window_id: 8 },
            ..native_framebuffer_run(0, 0, 1, 10)
        };
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(wrong_target)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(frame_buffer.pixels(), before.as_slice());
        assert_eq!(frame_buffer.active_sequence().unwrap().seen_run_count, 0);
    }

    #[test]
    fn native_span_framebuffer_requires_exact_run_count_before_end() {
        let descriptor = native_framebuffer_descriptor(2);
        let mut frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(0, 0, 1, 10),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(frame_buffer.active_sequence().unwrap().seen_run_count, 1);

        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(1, 0, 1, 20),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(2, 0, 1, 30),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(frame_buffer.active_sequence().unwrap().seen_run_count, 2);
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
    }

    #[test]
    fn native_span_framebuffer_rejects_end_descriptor_mismatch_and_keeps_active() {
        let descriptor = native_framebuffer_descriptor(1);
        let mismatched = NativeSpanOperationDescriptor {
            batch_index: 1,
            ..descriptor
        };
        let mut frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::End(mismatched)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(
            frame_buffer.active_sequence().unwrap(),
            NativeSpanFramebufferActiveSequence {
                descriptor,
                seen_run_count: 0,
            }
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(0, 0, 1, 10),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
    }

    #[test]
    fn native_present_buffer_packs_rgb0_and_blends_alpha() {
        let background = NativeRgbColor { r: 0, g: 0, b: 255 };
        assert_eq!(native_pack_rgb0_pixel(1, 2, 3), 0x00010203);
        assert_eq!(
            native_rgba8888_to_rgb0_over_background(
                native_pack_rgba8888_pixel(255, 0, 0, 255),
                background,
            ),
            0x00ff0000
        );
        assert_eq!(
            native_rgba8888_to_rgb0_over_background(
                native_pack_rgba8888_pixel(255, 0, 0, 0),
                background,
            ),
            0x000000ff
        );
        assert_eq!(
            native_rgba8888_to_rgb0_over_background(
                native_pack_rgba8888_pixel(255, 0, 0, 128),
                background,
            ),
            0x0080007f
        );
    }

    #[test]
    fn native_present_buffer_converts_completed_framebuffer() {
        let descriptor = native_framebuffer_descriptor(2);
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(0, 0, 1, 255),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                NativeSpanOperationRunSpan {
                    a: 0,
                    ..native_framebuffer_run(1, 0, 1, 200)
                },
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );

        let present_buffer =
            NativeRgb0PresentBuffer::from_rgba8888_framebuffer(&frame_buffer, background).unwrap();
        assert_eq!(present_buffer.width(), 4);
        assert_eq!(present_buffer.height(), 3);
        assert_eq!(present_buffer.pixels().len(), 12);
        assert_eq!(present_buffer.pixels()[0], 0x00ff141e);
        assert_eq!(present_buffer.pixels()[1], native_pack_rgb0_pixel(1, 2, 3));
    }

    #[test]
    fn native_present_buffer_rejects_active_framebuffer_sequence() {
        let descriptor = native_framebuffer_descriptor(1);
        let background = NativeRgbColor { r: 0, g: 0, b: 0 };
        let mut frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            NativeRgb0PresentBuffer::from_rgba8888_framebuffer(&frame_buffer, background)
                .unwrap_err(),
            NativeSpanFramebufferError::SequenceAlreadyActive
        );
        assert_eq!(
            frame_buffer.active_sequence().unwrap(),
            NativeSpanFramebufferActiveSequence {
                descriptor,
                seen_run_count: 0,
            }
        );
    }

    #[test]
    fn native_presenter_frame_imports_smoke_rgb0_pixels() {
        let present_buffer = NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(
            2,
            2,
            vec![0x00010203, 0x00040506, 0x00070809, 0x000a0b0c],
        )
        .unwrap();
        let present_frame =
            NativePresenterFrame::from_rgb0_present_buffer(&present_buffer).unwrap();

        assert_eq!(present_frame.width(), 2);
        assert_eq!(present_frame.height(), 2);
        assert_eq!(present_frame.pixels(), present_buffer.pixels());
    }

    #[test]
    fn native_presenter_frame_rejects_invalid_rgb0_import() {
        assert_eq!(
            NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(0, 1, vec![]).unwrap_err(),
            NativePresenterFrameError::InvalidDimensions
        );
        assert_eq!(
            NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(2, 2, vec![0, 1, 2])
                .unwrap_err(),
            NativePresenterFrameError::PixelCountMismatch
        );
        assert_eq!(
            NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(1, 1, vec![0xff000001])
                .unwrap_err(),
            NativePresenterFrameError::PixelFormatMismatch
        );
    }

    #[test]
    fn native_presenter_frame_revalidates_buffer_contract() {
        let invalid_dimensions = NativeRgb0PresentBuffer {
            width: 0,
            height: 1,
            pixels: vec![0],
        };
        let mismatched_pixels = NativeRgb0PresentBuffer {
            width: 2,
            height: 2,
            pixels: vec![0, 1, 2],
        };
        let invalid_format = NativeRgb0PresentBuffer {
            width: 1,
            height: 1,
            pixels: vec![0x01000000],
        };

        assert_eq!(
            NativePresenterFrame::from_rgb0_present_buffer(&invalid_dimensions).unwrap_err(),
            NativePresenterFrameError::InvalidDimensions
        );
        assert_eq!(
            NativePresenterFrame::from_rgb0_present_buffer(&mismatched_pixels).unwrap_err(),
            NativePresenterFrameError::PixelCountMismatch
        );
        assert_eq!(
            NativePresenterFrame::from_rgb0_present_buffer(&invalid_format).unwrap_err(),
            NativePresenterFrameError::PixelFormatMismatch
        );
    }

    #[test]
    fn native_rgb0_presenter_sink_updates_last_frame_on_complete_sequence() {
        let descriptor = native_framebuffer_descriptor(1);
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut sink = NativeRgb0PresenterSink::new(4, 3, background).unwrap();

        assert_eq!(sink.background(), background);
        assert_eq!(sink.last_presented_frame_id(), None);
        assert_eq!(sink.last_present_frame().unwrap(), None);
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::RunSpan(native_framebuffer_run(
                0, 0, 1, 255,
            ))),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );

        assert_eq!(sink.last_presented_frame_id(), Some(descriptor.frame_id));
        assert_eq!(sink.frame_buffer().active_sequence(), None);
        let present_frame = sink.last_present_frame().unwrap().unwrap();
        assert_eq!(present_frame.width(), 4);
        assert_eq!(present_frame.height(), 3);
        assert_eq!(present_frame.pixels()[0], 0x00ff141e);
        assert_eq!(present_frame.pixels()[1], native_pack_rgb0_pixel(1, 2, 3));
    }

    #[test]
    fn native_rgb0_presenter_sink_keeps_previous_frame_on_invalid_sequence() {
        let descriptor = native_framebuffer_descriptor(1);
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut sink = NativeRgb0PresenterSink::new(4, 3, background).unwrap();
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::RunSpan(native_framebuffer_run(
                0, 0, 1, 40,
            ))),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        let previous_pixels = sink
            .last_present_frame()
            .unwrap()
            .unwrap()
            .pixels()
            .to_vec();

        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::RunSpan(native_framebuffer_run(
                3, 0, 2, 90,
            ))),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );

        assert_eq!(sink.last_presented_frame_id(), Some(descriptor.frame_id));
        assert_eq!(
            sink.last_present_frame().unwrap().unwrap().pixels(),
            previous_pixels.as_slice()
        );
        assert_eq!(
            sink.frame_buffer().active_sequence().unwrap(),
            NativeSpanFramebufferActiveSequence {
                descriptor,
                seen_run_count: 0,
            }
        );
    }

    fn native_complete_rgb0_presenter_sink(frame_id: i32, red: u8) -> NativeRgb0PresenterSink {
        let mut descriptor = native_framebuffer_descriptor(1);
        descriptor.frame_id = frame_id;
        descriptor.packet_frame_id = frame_id;
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut sink = NativeRgb0PresenterSink::new(4, 3, background).unwrap();
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::RunSpan(native_framebuffer_run(
                0, 0, 1, red,
            ))),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            sink.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        sink
    }

    #[test]
    fn native_window_presenter_state_requires_positive_initial_surface() {
        assert_eq!(
            NativeWindowPresenterState::new(0, 1).unwrap_err(),
            NativeWindowPresenterError::InvalidSurfaceDimensions
        );
        assert_eq!(
            NativeWindowPresenterState::new(1, 0).unwrap_err(),
            NativeWindowPresenterError::InvalidSurfaceDimensions
        );

        let state = NativeWindowPresenterState::new(640, 480).unwrap();
        assert_eq!(
            state.surface_state(),
            NativeWindowPresenterSurfaceState::Drawable {
                width: 640,
                height: 480,
            }
        );
        assert_eq!(state.last_frame_id(), None);
        assert_eq!(state.last_frame_size(), None);
        assert_eq!(state.last_present_frame().unwrap(), None);
    }

    #[test]
    fn native_window_presenter_state_presents_sink_frame_after_complete_sequence() {
        let sink = native_complete_rgb0_presenter_sink(21, 200);
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();

        let (frame_width, frame_height, first_pixel) = {
            let frame = state.present_sink_frame(&sink).unwrap();
            (frame.width(), frame.height(), frame.pixels()[0])
        };

        assert_eq!(frame_width, 4);
        assert_eq!(frame_height, 3);
        assert_eq!(first_pixel, 0x00c8141e);
        assert_eq!(state.last_frame_id(), Some(21));
        assert_eq!(state.last_frame_size(), Some((4, 3)));
    }

    #[test]
    fn native_window_presenter_state_presents_checked_buffer() {
        let present_buffer = NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(
            2,
            2,
            vec![
                native_pack_rgb0_pixel(1, 2, 3),
                native_pack_rgb0_pixel(4, 5, 6),
                native_pack_rgb0_pixel(7, 8, 9),
                native_pack_rgb0_pixel(10, 11, 12),
            ],
        )
        .unwrap();
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();

        state.present_buffer(51, &present_buffer).unwrap();

        assert_eq!(state.last_frame_id(), Some(51));
        assert_eq!(state.last_frame_size(), Some((2, 2)));
        assert_eq!(
            state.last_present_frame_required().unwrap().pixels(),
            present_buffer.pixels()
        );
    }

    #[test]
    fn native_window_presenter_state_requires_valid_frame_id() {
        let present_buffer = NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(
            1,
            1,
            vec![native_pack_rgb0_pixel(1, 2, 3)],
        )
        .unwrap();
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();

        assert_eq!(
            state.present_buffer(0, &present_buffer).unwrap_err(),
            NativeWindowPresenterError::InvalidFrameId
        );
        assert_eq!(
            state.present_buffer(-1, &present_buffer).unwrap_err(),
            NativeWindowPresenterError::InvalidFrameId
        );
        assert_eq!(state.last_frame_id(), None);
        assert_eq!(
            state.last_present_frame_required().unwrap_err(),
            NativeWindowPresenterError::FrameMissing
        );
    }

    #[test]
    fn native_window_presenter_state_rejects_missing_completed_frame() {
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let sink = NativeRgb0PresenterSink::new(4, 3, background).unwrap();
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();

        assert_eq!(
            state.present_sink_frame(&sink).unwrap_err(),
            NativeWindowPresenterError::FrameMissing
        );
        assert_eq!(state.last_frame_id(), None);
        assert_eq!(state.last_frame_size(), None);
    }

    #[test]
    fn native_window_presenter_state_rejects_missing_frame_id() {
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let sink = NativeRgb0PresenterSink {
            frame_buffer: NativeRgba8888FrameBuffer::new(1, 1).unwrap(),
            background,
            last_present_buffer: Some(NativeRgb0PresentBuffer {
                width: 1,
                height: 1,
                pixels: vec![native_pack_rgb0_pixel(1, 2, 3)],
            }),
            last_presented_frame_id: None,
        };
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();

        assert_eq!(
            state.present_sink_frame(&sink).unwrap_err(),
            NativeWindowPresenterError::FrameIdMissing
        );
        assert_eq!(state.last_frame_id(), None);
        assert_eq!(state.last_frame_size(), None);
    }

    #[test]
    fn native_window_presenter_state_rejects_invalid_sink_frame_id() {
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let sink = NativeRgb0PresenterSink {
            frame_buffer: NativeRgba8888FrameBuffer::new(1, 1).unwrap(),
            background,
            last_present_buffer: Some(NativeRgb0PresentBuffer {
                width: 1,
                height: 1,
                pixels: vec![native_pack_rgb0_pixel(1, 2, 3)],
            }),
            last_presented_frame_id: Some(0),
        };
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();

        assert_eq!(
            state.present_sink_frame(&sink).unwrap_err(),
            NativeWindowPresenterError::InvalidFrameId
        );
        assert_eq!(state.last_frame_id(), None);
        assert_eq!(state.last_frame_size(), None);
    }

    #[test]
    fn native_window_presenter_state_tracks_resize_without_stretching_last_frame() {
        let sink = native_complete_rgb0_presenter_sink(31, 90);
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();
        state.present_sink_frame(&sink).unwrap();
        let previous_id = state.last_frame_id();
        let previous_size = state.last_frame_size();
        let previous_pixels = state
            .last_present_frame()
            .unwrap()
            .unwrap()
            .pixels()
            .to_vec();

        state.resize_surface(800, 600).unwrap();
        assert_eq!(
            state.surface_state(),
            NativeWindowPresenterSurfaceState::Drawable {
                width: 800,
                height: 600,
            }
        );
        assert_eq!(state.last_frame_id(), previous_id);
        assert_eq!(state.last_frame_size(), previous_size);
        assert_eq!(
            state.last_present_frame().unwrap().unwrap().pixels(),
            previous_pixels.as_slice()
        );

        state.resize_surface(0, 600).unwrap();
        assert_eq!(
            state.surface_state(),
            NativeWindowPresenterSurfaceState::Unavailable
        );
        assert_eq!(state.last_frame_id(), previous_id);
        assert_eq!(state.last_frame_size(), previous_size);
        assert_eq!(
            state.last_present_frame().unwrap().unwrap().pixels(),
            previous_pixels.as_slice()
        );
    }

    #[test]
    fn native_window_presenter_state_failed_buffer_present_keeps_previous_frame() {
        let valid_buffer = NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(
            1,
            1,
            vec![native_pack_rgb0_pixel(1, 2, 3)],
        )
        .unwrap();
        let invalid_format = NativeRgb0PresentBuffer {
            width: 1,
            height: 1,
            pixels: vec![0xff000001],
        };
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();
        state.present_buffer(61, &valid_buffer).unwrap();
        let previous_id = state.last_frame_id();
        let previous_size = state.last_frame_size();
        let previous_pixels = state
            .last_present_frame_required()
            .unwrap()
            .pixels()
            .to_vec();

        assert_eq!(
            state.present_buffer(62, &invalid_format).unwrap_err(),
            NativeWindowPresenterError::PresenterFrameValidationFailed(
                NativePresenterFrameError::PixelFormatMismatch
            )
        );
        assert_eq!(state.last_frame_id(), previous_id);
        assert_eq!(state.last_frame_size(), previous_size);
        assert_eq!(
            state.last_present_frame_required().unwrap().pixels(),
            previous_pixels.as_slice()
        );
    }

    #[test]
    fn native_window_presenter_state_failed_present_keeps_previous_frame() {
        let valid_sink = native_complete_rgb0_presenter_sink(41, 20);
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut state = NativeWindowPresenterState::new(640, 480).unwrap();
        state.present_sink_frame(&valid_sink).unwrap();
        let previous_id = state.last_frame_id();
        let previous_size = state.last_frame_size();
        let previous_pixels = state
            .last_present_frame()
            .unwrap()
            .unwrap()
            .pixels()
            .to_vec();

        let malformed_sink = NativeRgb0PresenterSink {
            frame_buffer: NativeRgba8888FrameBuffer::new(4, 3).unwrap(),
            background,
            last_present_buffer: Some(NativeRgb0PresentBuffer {
                width: 4,
                height: 3,
                pixels: vec![NATIVE_RGBA8888_PIXEL_TRANSPARENT; 11],
            }),
            last_presented_frame_id: Some(42),
        };

        assert_eq!(
            state.present_sink_frame(&malformed_sink).unwrap_err(),
            NativeWindowPresenterError::PresenterFrameValidationFailed(
                NativePresenterFrameError::PixelCountMismatch
            )
        );
        assert_eq!(state.last_frame_id(), previous_id);
        assert_eq!(state.last_frame_size(), previous_size);
        assert_eq!(
            state.last_present_frame().unwrap().unwrap().pixels(),
            previous_pixels.as_slice()
        );
    }

    fn native_complete_window_presenter_session(
        frame_id: i32,
        red: u8,
    ) -> NativeWindowPresenterSession {
        let mut descriptor = native_framebuffer_descriptor(1);
        descriptor.frame_id = frame_id;
        descriptor.packet_frame_id = frame_id;
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut session = NativeWindowPresenterSession::new(4, 3, background, 640, 480).unwrap();

        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::Begin(descriptor))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::RunSpan(native_framebuffer_run(
                    0, 0, 1, red,
                )))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::End(descriptor))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::Presented {
                frame_id,
                width: 4,
                height: 3,
            }
        );
        session
    }

    fn execute_window_session_scalar_begin(
        session: &mut NativeWindowPresenterSession,
        descriptor: NativeSpanOperationDescriptor,
    ) -> Result<NativeWindowPresenterSessionOutcome, NativeWindowPresenterSessionHostError> {
        execute_native_window_presenter_session_begin(
            session,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            descriptor.surface_id,
            descriptor.frame_id,
            descriptor.packet_frame_id,
            descriptor.batch_index,
            descriptor.tile_index,
            descriptor.plan_row_start,
            descriptor.plan_row_count,
            descriptor.row_start,
            descriptor.row_count,
            descriptor.width,
            descriptor.height,
            descriptor.stride_bytes,
            descriptor.tile_rows,
            descriptor.tile_count,
            descriptor.pixel_count,
            descriptor.total_run_count,
            descriptor.encoded_byte_count,
        )
    }

    fn execute_window_session_scalar_end(
        session: &mut NativeWindowPresenterSession,
        descriptor: NativeSpanOperationDescriptor,
    ) -> Result<NativeWindowPresenterSessionOutcome, NativeWindowPresenterSessionHostError> {
        execute_native_window_presenter_session_end(
            session,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            descriptor.surface_id,
            descriptor.frame_id,
            descriptor.packet_frame_id,
            descriptor.batch_index,
            descriptor.tile_index,
            descriptor.plan_row_start,
            descriptor.plan_row_count,
            descriptor.row_start,
            descriptor.row_count,
            descriptor.width,
            descriptor.height,
            descriptor.stride_bytes,
            descriptor.tile_rows,
            descriptor.tile_count,
            descriptor.pixel_count,
            descriptor.total_run_count,
            descriptor.encoded_byte_count,
        )
    }

    fn execute_window_session_scalar_run(
        session: &mut NativeWindowPresenterSession,
        x: i32,
        y: i32,
        width: i32,
        red: i32,
    ) -> Result<NativeWindowPresenterSessionOutcome, NativeWindowPresenterSessionHostError> {
        execute_native_window_presenter_session_run(
            session,
            GUI_NATIVE_SPAN_OPERATION_TARGET_KIND_WINDOW,
            7,
            x,
            y,
            width,
            1,
            red,
            20,
            30,
            255,
        )
    }

    #[test]
    fn native_window_presenter_session_presents_only_after_end() {
        let descriptor = native_framebuffer_descriptor(1);
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut session = NativeWindowPresenterSession::new(4, 3, background, 640, 480).unwrap();

        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::Begin(descriptor))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(session.presenter_state().last_frame_id(), None);
        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::RunSpan(native_framebuffer_run(
                    0, 0, 1, 210,
                )))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(session.presenter_state().last_frame_id(), None);

        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::End(descriptor))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::Presented {
                frame_id: descriptor.frame_id,
                width: 4,
                height: 3,
            }
        );
        assert_eq!(
            session.sink().last_presented_frame_id(),
            Some(descriptor.frame_id)
        );
        assert_eq!(
            session.presenter_state().last_frame_id(),
            Some(descriptor.frame_id)
        );
        assert_eq!(
            session
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels()[0],
            0x00d2141e
        );
    }

    #[test]
    fn native_window_presenter_session_scalar_helper_presents_only_after_end() {
        let descriptor = native_framebuffer_descriptor(1);
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut session = NativeWindowPresenterSession::new(4, 3, background, 640, 480).unwrap();

        assert_eq!(
            execute_window_session_scalar_begin(&mut session, descriptor).unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(session.presenter_state().last_frame_id(), None);
        assert_eq!(
            execute_window_session_scalar_run(&mut session, 0, 0, 1, 210).unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(session.presenter_state().last_frame_id(), None);
        assert_eq!(
            execute_window_session_scalar_end(&mut session, descriptor).unwrap(),
            NativeWindowPresenterSessionOutcome::Presented {
                frame_id: descriptor.frame_id,
                width: 4,
                height: 3,
            }
        );
        assert_eq!(
            session.presenter_state().last_frame_id(),
            Some(descriptor.frame_id)
        );
        assert_eq!(
            session
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels()[0],
            0x00d2141e
        );
    }

    #[test]
    fn native_window_presenter_session_scalar_validation_keeps_session_state() {
        let background = NativeRgbColor { r: 1, g: 2, b: 3 };
        let mut session = NativeWindowPresenterSession::new(4, 3, background, 640, 480).unwrap();
        let mut invalid_descriptor = native_framebuffer_descriptor(1);
        invalid_descriptor.stride_bytes = 20;

        assert_eq!(
            execute_window_session_scalar_begin(&mut session, invalid_descriptor).unwrap_err(),
            NativeWindowPresenterSessionHostError::ValidationFailed(
                NativeSpanOperationStatus::InvalidArgument
            )
        );
        assert_eq!(session.sink().frame_buffer().active_sequence(), None);
        assert_eq!(session.presenter_state().last_frame_id(), None);
    }

    #[test]
    fn native_window_presenter_session_scalar_sink_failure_keeps_previous_frame() {
        let mut session = native_complete_window_presenter_session(101, 20);
        let previous_id = session.presenter_state().last_frame_id();
        let previous_size = session.presenter_state().last_frame_size();
        let previous_pixels = session
            .presenter_state()
            .last_present_frame_required()
            .unwrap()
            .pixels()
            .to_vec();
        let mut descriptor = native_framebuffer_descriptor(1);
        descriptor.frame_id = 102;
        descriptor.packet_frame_id = 102;

        assert_eq!(
            execute_window_session_scalar_begin(&mut session, descriptor).unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(
            execute_window_session_scalar_run(&mut session, 3, 0, 2, 90).unwrap_err(),
            NativeWindowPresenterSessionHostError::SessionFailed(
                NativeWindowPresenterSessionError::SinkFailed(
                    NativeSpanFramebufferError::RunExtentOutOfBounds
                )
            )
        );
        assert_eq!(
            execute_window_session_scalar_end(&mut session, descriptor).unwrap_err(),
            NativeWindowPresenterSessionHostError::SessionFailed(
                NativeWindowPresenterSessionError::SinkFailed(
                    NativeSpanFramebufferError::RunCountMismatch
                )
            )
        );
        assert_eq!(session.presenter_state().last_frame_id(), previous_id);
        assert_eq!(session.presenter_state().last_frame_size(), previous_size);
        assert_eq!(
            session
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels(),
            previous_pixels.as_slice()
        );
    }

    #[test]
    fn native_window_presenter_session_host_error_separates_presenter_failure() {
        let mut session = native_complete_window_presenter_session(111, 30);
        let previous_id = session.presenter_state().last_frame_id();
        let previous_pixels = session
            .presenter_state()
            .last_present_frame_required()
            .unwrap()
            .pixels()
            .to_vec();
        let mut descriptor = native_framebuffer_descriptor(1);
        descriptor.frame_id = 0;
        descriptor.packet_frame_id = 0;

        assert_eq!(
            execute_native_window_presenter_session_operation(
                &mut session,
                NativeSpanOperation::Begin(descriptor),
            )
            .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(
            execute_native_window_presenter_session_operation(
                &mut session,
                NativeSpanOperation::RunSpan(native_framebuffer_run(0, 0, 1, 40)),
            )
            .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(
            execute_native_window_presenter_session_operation(
                &mut session,
                NativeSpanOperation::End(descriptor),
            )
            .unwrap_err(),
            NativeWindowPresenterSessionHostError::SessionFailed(
                NativeWindowPresenterSessionError::PresenterFailed(
                    NativeWindowPresenterError::InvalidFrameId
                )
            )
        );
        assert_eq!(
            NativeWindowPresenterSessionHostError::SessionFailed(
                NativeWindowPresenterSessionError::PresenterFailed(
                    NativeWindowPresenterError::InvalidFrameId
                )
            )
            .status(),
            GUI_NATIVE_SPAN_OPERATION_STATUS_INVALID_ARGUMENT
        );
        assert_eq!(session.presenter_state().last_frame_id(), previous_id);
        assert_eq!(
            session
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels(),
            previous_pixels.as_slice()
        );
    }

    #[test]
    fn native_window_presenter_session_failed_sink_operation_keeps_previous_frame() {
        let mut session = native_complete_window_presenter_session(71, 20);
        let previous_id = session.presenter_state().last_frame_id();
        let previous_size = session.presenter_state().last_frame_size();
        let previous_pixels = session
            .presenter_state()
            .last_present_frame_required()
            .unwrap()
            .pixels()
            .to_vec();
        let mut descriptor = native_framebuffer_descriptor(1);
        descriptor.frame_id = 72;
        descriptor.packet_frame_id = 72;

        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::Begin(descriptor))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::RunSpan(native_framebuffer_run(
                    3, 0, 2, 90,
                )))
                .unwrap_err(),
            NativeWindowPresenterSessionError::SinkFailed(
                NativeSpanFramebufferError::RunExtentOutOfBounds
            )
        );
        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::End(descriptor))
                .unwrap_err(),
            NativeWindowPresenterSessionError::SinkFailed(
                NativeSpanFramebufferError::RunCountMismatch
            )
        );
        assert_eq!(session.presenter_state().last_frame_id(), previous_id);
        assert_eq!(session.presenter_state().last_frame_size(), previous_size);
        assert_eq!(
            session
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels(),
            previous_pixels.as_slice()
        );
    }

    #[test]
    fn native_window_presenter_session_failed_present_keeps_previous_frame() {
        let mut session = native_complete_window_presenter_session(81, 30);
        let previous_id = session.presenter_state().last_frame_id();
        let previous_size = session.presenter_state().last_frame_size();
        let previous_pixels = session
            .presenter_state()
            .last_present_frame_required()
            .unwrap()
            .pixels()
            .to_vec();
        let mut descriptor = native_framebuffer_descriptor(1);
        descriptor.frame_id = 0;
        descriptor.packet_frame_id = 0;

        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::Begin(descriptor))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::RunSpan(native_framebuffer_run(
                    0, 0, 1, 40,
                )))
                .unwrap(),
            NativeWindowPresenterSessionOutcome::NotPresented
        );
        assert_eq!(
            session
                .execute_span_operation(NativeSpanOperation::End(descriptor))
                .unwrap_err(),
            NativeWindowPresenterSessionError::PresenterFailed(
                NativeWindowPresenterError::InvalidFrameId
            )
        );
        assert_eq!(session.presenter_state().last_frame_id(), previous_id);
        assert_eq!(session.presenter_state().last_frame_size(), previous_size);
        assert_eq!(
            session
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels(),
            previous_pixels.as_slice()
        );
    }

    #[test]
    fn native_window_presenter_session_resize_keeps_frame_pixels_unscaled() {
        let mut session = native_complete_window_presenter_session(91, 60);
        let previous_id = session.presenter_state().last_frame_id();
        let previous_size = session.presenter_state().last_frame_size();
        let previous_pixels = session
            .presenter_state()
            .last_present_frame_required()
            .unwrap()
            .pixels()
            .to_vec();

        session.resize_surface(800, 600).unwrap();
        assert_eq!(
            session.presenter_state().surface_state(),
            NativeWindowPresenterSurfaceState::Drawable {
                width: 800,
                height: 600,
            }
        );
        assert_eq!(session.presenter_state().last_frame_id(), previous_id);
        assert_eq!(session.presenter_state().last_frame_size(), previous_size);
        assert_eq!(
            session
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels(),
            previous_pixels.as_slice()
        );

        session.resize_surface(0, 600).unwrap();
        assert_eq!(
            session.presenter_state().surface_state(),
            NativeWindowPresenterSurfaceState::Unavailable
        );
        assert_eq!(session.presenter_state().last_frame_id(), previous_id);
        assert_eq!(session.presenter_state().last_frame_size(), previous_size);
        assert_eq!(
            session
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels(),
            previous_pixels.as_slice()
        );
    }

    #[test]
    fn native_window_event_pump_tracks_positive_and_zero_resize() {
        let input = NativeWindowEventPumpInput {
            previous_size: NativeWindowSize::new(640, 480),
            previous_mouse_down: false,
        };

        let unchanged = build_native_window_event_pump_snapshot_from_raw(
            input,
            false,
            false,
            NativeWindowSize::new(640, 480),
            false,
            None,
        )
        .unwrap();
        assert_eq!(unchanged.close_state, NativeWindowEventPumpCloseState::Open);
        assert!(!unchanged.size_changed);
        assert_eq!(
            unchanged.surface_state,
            NativeWindowPresenterSurfaceState::Drawable {
                width: 640,
                height: 480,
            }
        );

        let resized = build_native_window_event_pump_snapshot_from_raw(
            input,
            false,
            false,
            NativeWindowSize::new(1280, 720),
            false,
            None,
        )
        .unwrap();
        assert!(resized.size_changed);
        assert_eq!(
            resized.surface_state,
            NativeWindowPresenterSurfaceState::Drawable {
                width: 1280,
                height: 720,
            }
        );

        let unavailable = build_native_window_event_pump_snapshot_from_raw(
            input,
            false,
            false,
            NativeWindowSize::new(0, 720),
            false,
            None,
        )
        .unwrap();
        assert!(unavailable.size_changed);
        assert_eq!(
            unavailable.surface_state,
            NativeWindowPresenterSurfaceState::Unavailable
        );

        let restored = build_native_window_event_pump_snapshot_from_raw(
            NativeWindowEventPumpInput {
                previous_size: NativeWindowSize::new(0, 720),
                previous_mouse_down: true,
            },
            false,
            false,
            NativeWindowSize::new(1280, 720),
            false,
            None,
        )
        .unwrap();
        assert!(restored.size_changed);
        assert_eq!(
            restored.surface_state,
            NativeWindowPresenterSurfaceState::Drawable {
                width: 1280,
                height: 720,
            }
        );
        assert_eq!(
            restored.mouse_left_transition,
            NativeWindowPointerButtonTransition::Released
        );
    }

    #[test]
    fn native_window_event_pump_tracks_pointer_button_transitions() {
        let size = NativeWindowSize::new(640, 480);
        let idle = build_native_window_event_pump_snapshot_from_raw(
            NativeWindowEventPumpInput {
                previous_size: size,
                previous_mouse_down: false,
            },
            false,
            false,
            size,
            false,
            None,
        )
        .unwrap();
        assert_eq!(
            idle.mouse_left_transition,
            NativeWindowPointerButtonTransition::Unchanged
        );

        let pressed = build_native_window_event_pump_snapshot_from_raw(
            NativeWindowEventPumpInput {
                previous_size: size,
                previous_mouse_down: false,
            },
            false,
            false,
            size,
            true,
            Some((12.0, 34.0)),
        )
        .unwrap();
        assert_eq!(
            pressed.mouse_left_transition,
            NativeWindowPointerButtonTransition::Pressed
        );
        assert_eq!(
            pressed.pointer_sample,
            NativeWindowPointerSample::Available { x: 12.0, y: 34.0 }
        );

        let held = build_native_window_event_pump_snapshot_from_raw(
            NativeWindowEventPumpInput {
                previous_size: size,
                previous_mouse_down: true,
            },
            false,
            false,
            size,
            true,
            None,
        )
        .unwrap();
        assert_eq!(
            held.mouse_left_transition,
            NativeWindowPointerButtonTransition::Unchanged
        );

        let released = build_native_window_event_pump_snapshot_from_raw(
            NativeWindowEventPumpInput {
                previous_size: size,
                previous_mouse_down: true,
            },
            false,
            false,
            size,
            false,
            None,
        )
        .unwrap();
        assert_eq!(
            released.mouse_left_transition,
            NativeWindowPointerButtonTransition::Released
        );
    }

    #[test]
    fn native_window_event_pump_rejects_non_finite_pointer_sample() {
        let input = NativeWindowEventPumpInput {
            previous_size: NativeWindowSize::new(640, 480),
            previous_mouse_down: false,
        };

        assert_eq!(
            build_native_window_event_pump_snapshot_from_raw(
                input,
                false,
                false,
                NativeWindowSize::new(640, 480),
                true,
                Some((f32::NAN, 10.0)),
            )
            .unwrap_err(),
            NativeWindowEventPumpError::InvalidPointerSample
        );
        assert_eq!(
            build_native_window_event_pump_snapshot_from_raw(
                input,
                false,
                false,
                NativeWindowSize::new(640, 480),
                true,
                Some((10.0, f32::INFINITY)),
            )
            .unwrap_err(),
            NativeWindowEventPumpError::InvalidPointerSample
        );
    }

    #[test]
    fn native_window_event_pump_separates_os_close_and_exit_shortcut() {
        let input = NativeWindowEventPumpInput {
            previous_size: NativeWindowSize::new(640, 480),
            previous_mouse_down: false,
        };

        let os_close = build_native_window_event_pump_snapshot_from_raw(
            input,
            true,
            false,
            NativeWindowSize::new(640, 480),
            false,
            None,
        )
        .unwrap();
        assert_eq!(
            os_close.close_state,
            NativeWindowEventPumpCloseState::OsCloseRequested
        );

        let shortcut = build_native_window_event_pump_snapshot_from_raw(
            input,
            false,
            true,
            NativeWindowSize::new(640, 480),
            false,
            None,
        )
        .unwrap();
        assert_eq!(
            shortcut.close_state,
            NativeWindowEventPumpCloseState::ExitShortcutRequested
        );

        let os_close_wins = build_native_window_event_pump_snapshot_from_raw(
            input,
            true,
            true,
            NativeWindowSize::new(640, 480),
            false,
            None,
        )
        .unwrap();
        assert_eq!(
            os_close_wins.close_state,
            NativeWindowEventPumpCloseState::OsCloseRequested
        );
    }

    fn native_window_backend_loop_counter() -> NativeWindowBackendLoop {
        NativeWindowBackendLoop::new_for_scale(GuiDemo::Counter, 0, 2).unwrap()
    }

    #[test]
    fn native_window_target_fps_accepts_default_and_custom_values() {
        assert_eq!(
            NativeWindowTargetFps::default().value(),
            NATIVE_WINDOW_RUN_LOOP_DEFAULT_TARGET_FPS
        );
        assert_eq!(NativeWindowTargetFps::new(144).unwrap().value(), 144);
        assert_eq!(NativeWindowTargetFps::new(144).unwrap().as_usize(), 144);
    }

    #[test]
    fn native_window_target_fps_rejects_zero_and_too_high_values() {
        assert_eq!(
            NativeWindowTargetFps::new(0).unwrap_err(),
            NativeWindowTargetFpsError {
                value: 0,
                reason: NativeWindowTargetFpsInvalidReason::Zero,
            }
        );
        assert_eq!(
            NativeWindowTargetFps::new(usize::from(NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS) + 1)
                .unwrap_err(),
            NativeWindowTargetFpsError {
                value: usize::from(NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS) + 1,
                reason: NativeWindowTargetFpsInvalidReason::TooHigh {
                    max: NATIVE_WINDOW_RUN_LOOP_MAX_TARGET_FPS
                },
            }
        );
    }

    #[test]
    fn native_window_host_loop_turn_slice_accepts_default_and_custom_values() {
        assert_eq!(
            NativeWindowHostLoopTurnSlice::default().value(),
            NATIVE_WINDOW_HOST_LOOP_DEFAULT_TURN_SLICE
        );
        assert_eq!(NativeWindowHostLoopTurnSlice::new(16).unwrap().value(), 16);
        assert_eq!(
            NativeWindowHostLoopTurnSlice::new(16).unwrap().as_usize(),
            16
        );
    }

    #[test]
    fn native_window_host_loop_turn_slice_rejects_zero_and_too_high_values() {
        assert_eq!(
            NativeWindowHostLoopTurnSlice::new(0).unwrap_err(),
            NativeWindowHostLoopTurnSliceError {
                value: 0,
                reason: NativeWindowHostLoopTurnSliceInvalidReason::Zero,
            }
        );
        assert_eq!(
            NativeWindowHostLoopTurnSlice::new(
                usize::from(NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE) + 1
            )
            .unwrap_err(),
            NativeWindowHostLoopTurnSliceError {
                value: usize::from(NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE) + 1,
                reason: NativeWindowHostLoopTurnSliceInvalidReason::TooHigh {
                    max: NATIVE_WINDOW_HOST_LOOP_MAX_TURN_SLICE
                },
            }
        );
    }

    #[test]
    fn native_window_run_loop_config_preserves_demo_state() {
        assert_eq!(
            NativeWindowRunLoopConfig::new(GuiDemo::Counter, 7, 3),
            NativeWindowRunLoopConfig {
                demo: GuiDemo::Counter,
                counter_value: 7,
                scale: 3,
                target_fps: NativeWindowTargetFps::default(),
                host_loop_policy: NativeWindowHostLoopRunPolicy::default(),
            }
        );
        let custom_fps = NativeWindowTargetFps::new(120).unwrap();
        assert_eq!(
            NativeWindowRunLoopConfig::new_with_target_fps(GuiDemo::Life, 11, 2, custom_fps),
            NativeWindowRunLoopConfig {
                demo: GuiDemo::Life,
                counter_value: 11,
                scale: 2,
                target_fps: custom_fps,
                host_loop_policy: NativeWindowHostLoopRunPolicy::default(),
            }
        );
        let custom_policy =
            NativeWindowHostLoopRunPolicy::new(NativeWindowHostLoopTurnSlice::new(8).unwrap());
        assert_eq!(
            NativeWindowRunLoopConfig::new_with_target_fps_and_host_loop_policy(
                GuiDemo::Life,
                11,
                2,
                custom_fps,
                custom_policy
            ),
            NativeWindowRunLoopConfig {
                demo: GuiDemo::Life,
                counter_value: 11,
                scale: 2,
                target_fps: custom_fps,
                host_loop_policy: custom_policy,
            }
        );
        assert_eq!(
            NativeWindowRunLoopConfig::try_new_with_raw_target_fps(GuiDemo::Life, 11, 2, 0)
                .unwrap_err(),
            NativeWindowRunLoopError::TargetFpsInvalid {
                value: 0,
                reason: NativeWindowTargetFpsInvalidReason::Zero,
            }
        );
    }

    #[test]
    fn native_window_title_reports_drawable_and_unavailable_surface() {
        assert_eq!(
            native_window_title(GuiDemo::Mandelbrot, NativeWindowSize::new(1280, 720)),
            "NEPLg2 GUI native preview - Mandelbrot - 1280x720"
        );
        assert_eq!(
            native_window_title(GuiDemo::Life, NativeWindowSize::new(0, 720)),
            "NEPLg2 GUI native preview - Life - surface unavailable"
        );
    }

    #[test]
    fn initialize_native_window_host_loop_reports_idempotent_title_state() {
        let loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(Vec::new());

        assert!(!runner_state.title_initialized());
        assert_eq!(
            initialize_native_window_host_loop(&mut runner_state, &loop_state, &mut host),
            NativeWindowHostLoopInitialization::Initialized
        );
        assert!(runner_state.title_initialized());
        assert_eq!(
            host.titles,
            vec![native_window_title(GuiDemo::Counter, initial_size)]
        );
        assert_eq!(
            initialize_native_window_host_loop(&mut runner_state, &loop_state, &mut host),
            NativeWindowHostLoopInitialization::AlreadyInitialized
        );
        assert_eq!(
            host.titles,
            vec![native_window_title(GuiDemo::Counter, initial_size)]
        );
        assert_eq!(host.cursor, 0);
    }

    #[test]
    fn native_window_host_loop_wait_decision_maps_continue_evidence() {
        let window_size = NativeWindowSize::new(0, 480);
        assert_eq!(
            native_window_host_loop_wait_decision(
                NativeWindowHostLoopContinueEvidence::PumpedEventsOnly {
                    window_size,
                    size_changed: true,
                }
            ),
            NativeWindowHostLoopWaitDecision::WaitForHostEvent {
                window_size,
                size_changed: true,
            }
        );

        let drawable_size = NativeWindowSize::new(640, 480);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 7,
            width: drawable_size.width,
            height: drawable_size.height,
        };
        assert_eq!(
            native_window_host_loop_wait_decision(
                NativeWindowHostLoopContinueEvidence::PresentedFrame {
                    presentation,
                    window_size: drawable_size,
                    size_changed: false,
                }
            ),
            NativeWindowHostLoopWaitDecision::WaitForFrameInterval {
                presentation,
                window_size: drawable_size,
                size_changed: false,
            }
        );
    }

    #[test]
    fn native_window_host_loop_wait_request_builds_typed_backend_plan() {
        let window_size = NativeWindowSize::new(320, 240);
        assert_eq!(
            native_window_host_loop_wait_request(
                NativeWindowHostLoopWaitDecision::WaitForHostEvent {
                    window_size,
                    size_changed: true,
                },
                NativeWindowTargetFps::new(120).unwrap(),
            ),
            NativeWindowHostLoopWaitRequest::WaitForHostEvent {
                window_size,
                size_changed: true,
            }
        );

        let target_fps = NativeWindowTargetFps::new(120).unwrap();
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 9,
            width: window_size.width,
            height: window_size.height,
        };
        let request = native_window_host_loop_wait_request(
            NativeWindowHostLoopWaitDecision::WaitForFrameInterval {
                presentation,
                window_size,
                size_changed: false,
            },
            target_fps,
        );
        assert_eq!(
            request,
            NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
                presentation,
                window_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(target_fps),
            }
        );
        match request {
            NativeWindowHostLoopWaitRequest::WaitForFrameInterval { frame_interval, .. } => {
                assert_eq!(frame_interval.target_fps(), target_fps);
                assert_eq!(frame_interval.nanos_per_frame(), 8_333_333);
                assert_eq!(frame_interval.remainder_nanos_per_second(), 40);
            }
            NativeWindowHostLoopWaitRequest::WaitForHostEvent { .. } => {
                panic!("frame interval request expected")
            }
        }
    }

    #[test]
    fn run_native_window_host_loop_bounded_zero_budget_initializes_without_polling() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(Vec::new());

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 0)
                .unwrap(),
            NativeWindowHostLoopBoundedRunResult::BudgetExhausted {
                completed_turns: 0,
                last_wait_decision: None,
            }
        );
        assert!(runner_state.title_initialized());
        assert_eq!(
            host.titles,
            vec![native_window_title(GuiDemo::Counter, initial_size)]
        );
        assert_eq!(host.cursor, 0);
        assert_eq!(host.pump_count, 0);
        assert!(host.present_frames.is_empty());
    }

    #[test]
    fn run_native_window_host_loop_bounded_counts_exit_turn() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::ExitShortcutRequested,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(close)]);

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 3)
                .unwrap(),
            NativeWindowHostLoopBoundedRunResult::Exited {
                exit: NativeWindowRunLoopExit {
                    reason: NativeWindowHostTerminalReason::ExitShortcutRequested
                },
                completed_turns: 1,
            }
        );
        assert_eq!(host.cursor, 1);
        assert_eq!(
            host.titles,
            vec![native_window_title(GuiDemo::Counter, initial_size)]
        );
    }

    #[test]
    fn run_native_window_host_loop_bounded_yields_after_continue_budget() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(unavailable)]);

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 1)
                .unwrap(),
            NativeWindowHostLoopBoundedRunResult::BudgetExhausted {
                completed_turns: 1,
                last_wait_decision: Some(NativeWindowHostLoopWaitDecision::WaitForHostEvent {
                    window_size: unavailable_size,
                    size_changed: true,
                }),
            }
        );
        assert_eq!(host.cursor, 1);
        assert_eq!(host.pump_count, 1);
        assert_eq!(
            host.titles,
            vec![
                native_window_title(GuiDemo::Counter, initial_size),
                native_window_title(GuiDemo::Counter, unavailable_size),
            ]
        );
    }

    #[test]
    fn run_native_window_host_loop_bounded_reports_last_wait_decision() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let restored = build_native_window_event_pump_snapshot(
            NativeWindowEventPumpInput {
                previous_size: unavailable_size,
                previous_mouse_down: true,
            },
            false,
            false,
            initial_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(unavailable), Ok(restored)]);

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 2)
                .unwrap(),
            NativeWindowHostLoopBoundedRunResult::BudgetExhausted {
                completed_turns: 2,
                last_wait_decision: Some(NativeWindowHostLoopWaitDecision::WaitForFrameInterval {
                    presentation: NativeWindowBackendLoopPresentation {
                        frame_id: 2,
                        width: initial_size.width,
                        height: initial_size.height,
                    },
                    window_size: initial_size,
                    size_changed: true,
                }),
            }
        );
        assert_eq!(host.cursor, 2);
        assert_eq!(host.pump_count, 1);
        assert_eq!(
            host.present_frames,
            vec![(initial_size.width, initial_size.height)]
        );
        assert_eq!(
            host.titles,
            vec![
                native_window_title(GuiDemo::Counter, initial_size),
                native_window_title(GuiDemo::Counter, unavailable_size),
                native_window_title(GuiDemo::Counter, initial_size),
            ]
        );
    }

    #[test]
    fn run_native_window_host_loop_bounded_keeps_initial_title_across_slices() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(close)]);

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 0)
                .unwrap(),
            NativeWindowHostLoopBoundedRunResult::BudgetExhausted {
                completed_turns: 0,
                last_wait_decision: None,
            }
        );
        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 1)
                .unwrap(),
            NativeWindowHostLoopBoundedRunResult::Exited {
                exit: NativeWindowRunLoopExit {
                    reason: NativeWindowHostTerminalReason::OsCloseRequested
                },
                completed_turns: 1,
            }
        );
        assert_eq!(
            host.titles,
            vec![native_window_title(GuiDemo::Counter, initial_size)]
        );
        assert_eq!(host.cursor, 1);
    }

    #[test]
    fn run_native_window_host_loop_bounded_preserves_event_pump_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Err("event failed")]);

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 1)
                .unwrap_err(),
            NativeWindowHostLoopError::HostEventPumpFailed("event failed")
        );
        assert!(runner_state.title_initialized());
    }

    #[test]
    fn run_native_window_host_loop_bounded_preserves_present_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable)])
            .with_present_error("present failed");

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 1)
                .unwrap_err(),
            NativeWindowHostLoopError::HostPresentFailed("present failed")
        );
    }

    #[test]
    fn run_native_window_host_loop_bounded_preserves_host_action_error() {
        let mut loop_state =
            NativeWindowBackendLoop::new_for_scale(GuiDemo::Counter, i32::MAX, 2).unwrap();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(440, 284),
            true,
            NativeWindowPointerSample::Available { x: 40.0, y: 180.0 },
        );
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 1)
                .unwrap_err(),
            NativeWindowHostLoopError::HostActionFailed(NativeWindowHostActionError::StepFailed(
                NativeWindowBackendLoopError::CounterValueOverflow { previous: i32::MAX }
            ))
        );
    }

    #[test]
    fn run_native_window_host_loop_bounded_preserves_presenter_frame_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        loop_state.presenter_state.resize_surface(0, 0).unwrap();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut runner_state = NativeWindowHostLoopRunnerState::new();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable)]);

        assert_eq!(
            run_native_window_host_loop_bounded(&mut runner_state, &mut loop_state, &mut host, 1)
                .unwrap_err(),
            NativeWindowHostLoopError::PresenterFrameUnavailable(
                NativeWindowBackendLoopError::SurfaceUnavailable
            )
        );
    }

    #[test]
    fn native_window_host_loop_scheduler_slice_waits_after_budget_exhaustion() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(unavailable)]);
        let mut scheduler_state = NativeWindowHostLoopSchedulerState::new();

        let result = run_native_window_host_loop_scheduler_slice_with_policy(
            &mut scheduler_state,
            &mut loop_state,
            &mut host,
            NativeWindowHostLoopRunPolicy::default(),
        )
        .unwrap();

        assert_eq!(
            result,
            NativeWindowHostLoopSchedulerSliceResult::Waited {
                completed_turns: 1,
                decision: NativeWindowHostLoopWaitDecision::WaitForHostEvent {
                    window_size: unavailable_size,
                    size_changed: true,
                },
                request: NativeWindowHostLoopWaitRequest::WaitForHostEvent {
                    window_size: unavailable_size,
                    size_changed: true,
                },
                outcome: NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                    window_size: unavailable_size,
                    size_changed: true,
                },
            }
        );
        assert!(scheduler_state.title_initialized());
        assert_eq!(host.cursor, 1);
        assert_eq!(host.pump_count, 1);
        assert_eq!(
            host.wait_requests,
            vec![NativeWindowHostLoopWaitRequest::WaitForHostEvent {
                window_size: unavailable_size,
                size_changed: true,
            }]
        );
    }

    #[test]
    fn native_window_host_loop_scheduler_slice_keeps_initial_title_across_calls() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            unavailable_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(unavailable), Ok(close)]);
        let mut scheduler_state = NativeWindowHostLoopSchedulerState::new();

        assert!(matches!(
            run_native_window_host_loop_scheduler_slice_with_policy(
                &mut scheduler_state,
                &mut loop_state,
                &mut host,
                NativeWindowHostLoopRunPolicy::default()
            )
            .unwrap(),
            NativeWindowHostLoopSchedulerSliceResult::Waited { .. }
        ));
        assert_eq!(
            run_native_window_host_loop_scheduler_slice_with_policy(
                &mut scheduler_state,
                &mut loop_state,
                &mut host,
                NativeWindowHostLoopRunPolicy::default()
            )
            .unwrap(),
            NativeWindowHostLoopSchedulerSliceResult::Exited {
                exit: NativeWindowRunLoopExit {
                    reason: NativeWindowHostTerminalReason::OsCloseRequested,
                },
                completed_turns: 1,
            }
        );
        assert_eq!(host.cursor, 2);
        assert_eq!(
            host.titles,
            vec![
                native_window_title(GuiDemo::Counter, initial_size),
                native_window_title(GuiDemo::Counter, unavailable_size),
            ]
        );
    }

    #[test]
    fn native_window_host_loop_scheduler_slice_preserves_wait_error_without_next_poll() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            unavailable_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(unavailable), Ok(close)])
            .with_wait_error("wait failed");
        let mut scheduler_state = NativeWindowHostLoopSchedulerState::new();

        assert_eq!(
            run_native_window_host_loop_scheduler_slice_with_policy(
                &mut scheduler_state,
                &mut loop_state,
                &mut host,
                NativeWindowHostLoopRunPolicy::default()
            )
            .unwrap_err(),
            NativeWindowHostLoopError::HostWaitFailed("wait failed")
        );
        assert_eq!(host.cursor, 1);
        assert_eq!(
            host.wait_requests,
            vec![NativeWindowHostLoopWaitRequest::WaitForHostEvent {
                window_size: unavailable_size,
                size_changed: true,
            }]
        );
        assert!(host.wait_outcomes.is_empty());
    }

    #[test]
    fn native_window_host_loop_scheduler_slice_exits_without_wait() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(close)]);
        let mut scheduler_state = NativeWindowHostLoopSchedulerState::new();

        assert_eq!(
            run_native_window_host_loop_scheduler_slice_with_policy(
                &mut scheduler_state,
                &mut loop_state,
                &mut host,
                NativeWindowHostLoopRunPolicy::default()
            )
            .unwrap(),
            NativeWindowHostLoopSchedulerSliceResult::Exited {
                exit: NativeWindowRunLoopExit {
                    reason: NativeWindowHostTerminalReason::OsCloseRequested,
                },
                completed_turns: 1,
            }
        );
        assert!(host.wait_requests.is_empty());
        assert!(host.wait_outcomes.is_empty());
    }

    #[test]
    fn native_window_host_loop_with_policy_exits_across_single_turn_slices() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            unavailable_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(unavailable), Ok(close)]);

        let exit = run_native_window_host_loop_with_policy(
            &mut loop_state,
            &mut host,
            NativeWindowHostLoopRunPolicy::default(),
        )
        .unwrap();

        assert_eq!(
            exit.reason,
            NativeWindowHostTerminalReason::OsCloseRequested
        );
        assert_eq!(host.cursor, 2);
        assert_eq!(host.pump_count, 1);
        assert!(host.present_frames.is_empty());
        assert_eq!(
            host.wait_requests,
            vec![NativeWindowHostLoopWaitRequest::WaitForHostEvent {
                window_size: unavailable_size,
                size_changed: true,
            }]
        );
        assert_eq!(
            host.wait_outcomes,
            vec![NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size: unavailable_size,
                size_changed: true,
            }]
        );
        assert_eq!(
            host.titles,
            vec![
                native_window_title(GuiDemo::Counter, initial_size),
                native_window_title(GuiDemo::Counter, unavailable_size),
            ]
        );
    }

    #[test]
    fn native_window_host_loop_with_policy_dispatches_frame_interval_wait() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable), Ok(close)]);

        let exit = run_native_window_host_loop_with_policy(
            &mut loop_state,
            &mut host,
            NativeWindowHostLoopRunPolicy::default(),
        )
        .unwrap();

        assert_eq!(
            exit.reason,
            NativeWindowHostTerminalReason::OsCloseRequested
        );
        assert_eq!(host.cursor, 2);
        assert_eq!(host.pump_count, 0);
        assert_eq!(
            host.present_frames,
            vec![(initial_size.width, initial_size.height)]
        );
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 1,
            width: initial_size.width,
            height: initial_size.height,
        };
        assert_eq!(
            host.wait_requests,
            vec![NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
                presentation,
                window_size: initial_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(
                    NativeWindowTargetFps::default()
                ),
            }]
        );
        assert_eq!(
            host.wait_outcomes,
            vec![NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
                presentation,
                window_size: initial_size,
                size_changed: false,
            }]
        );
    }

    #[test]
    fn native_window_host_loop_with_policy_uses_explicit_target_fps_for_wait_request() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable), Ok(close)]);
        let target_fps = NativeWindowTargetFps::new(120).unwrap();

        let exit = run_native_window_host_loop_with_policy_and_target_fps(
            &mut loop_state,
            &mut host,
            NativeWindowHostLoopRunPolicy::default(),
            target_fps,
        )
        .unwrap();

        assert_eq!(
            exit.reason,
            NativeWindowHostTerminalReason::OsCloseRequested
        );
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 1,
            width: initial_size.width,
            height: initial_size.height,
        };
        assert_eq!(
            host.wait_requests,
            vec![NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
                presentation,
                window_size: initial_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(target_fps),
            }]
        );
    }

    #[test]
    fn native_window_host_loop_with_policy_preserves_event_pump_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let policy =
            NativeWindowHostLoopRunPolicy::new(NativeWindowHostLoopTurnSlice::new(2).unwrap());
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Err("event failed")]);

        assert_eq!(
            run_native_window_host_loop_with_policy(&mut loop_state, &mut host, policy)
                .unwrap_err(),
            NativeWindowHostLoopError::HostEventPumpFailed("event failed")
        );
    }

    #[test]
    fn native_window_host_loop_with_policy_preserves_wait_error_without_next_poll() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            unavailable_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(unavailable), Ok(close)])
            .with_wait_error("wait failed");

        assert_eq!(
            run_native_window_host_loop_with_policy(
                &mut loop_state,
                &mut host,
                NativeWindowHostLoopRunPolicy::default()
            )
            .unwrap_err(),
            NativeWindowHostLoopError::HostWaitFailed("wait failed")
        );
        assert_eq!(host.cursor, 1);
        assert_eq!(host.pump_count, 1);
        assert_eq!(
            host.wait_requests,
            vec![NativeWindowHostLoopWaitRequest::WaitForHostEvent {
                window_size: unavailable_size,
                size_changed: true,
            }]
        );
        assert!(host.wait_outcomes.is_empty());
    }

    #[test]
    fn step_native_window_host_loop_close_turn_has_no_initial_title_or_present() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);

        assert_eq!(
            step_native_window_host_loop(&mut loop_state, &mut host).unwrap(),
            NativeWindowHostLoopTurn::Exit(NativeWindowRunLoopExit {
                reason: NativeWindowHostTerminalReason::OsCloseRequested
            })
        );
        assert!(host.titles.is_empty());
        assert_eq!(host.pump_count, 0);
        assert!(host.present_frames.is_empty());
    }

    #[test]
    fn step_native_window_host_loop_pump_only_resize_updates_title() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);

        assert_eq!(
            step_native_window_host_loop(&mut loop_state, &mut host).unwrap(),
            NativeWindowHostLoopTurn::Continue(
                NativeWindowHostLoopContinueEvidence::PumpedEventsOnly {
                    window_size: unavailable_size,
                    size_changed: true,
                }
            )
        );
        assert_eq!(
            host.titles,
            vec![native_window_title(GuiDemo::Counter, unavailable_size)]
        );
        assert_eq!(host.pump_count, 1);
        assert!(host.present_frames.is_empty());
    }

    #[test]
    fn step_native_window_host_loop_drawable_resize_presents_exact_frame() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let resized = NativeWindowSize::new(initial_size.width + 16, initial_size.height + 8);
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            resized,
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);

        assert_eq!(
            step_native_window_host_loop(&mut loop_state, &mut host).unwrap(),
            NativeWindowHostLoopTurn::Continue(
                NativeWindowHostLoopContinueEvidence::PresentedFrame {
                    presentation: NativeWindowBackendLoopPresentation {
                        frame_id: 2,
                        width: resized.width,
                        height: resized.height,
                    },
                    window_size: resized,
                    size_changed: true,
                }
            )
        );
        assert_eq!(
            host.titles,
            vec![native_window_title(GuiDemo::Counter, resized)]
        );
        assert_eq!(host.pump_count, 0);
        assert_eq!(host.present_frames, vec![(resized.width, resized.height)]);
    }

    #[test]
    fn step_native_window_host_loop_drawable_without_resize_keeps_title_empty() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);

        assert_eq!(
            step_native_window_host_loop(&mut loop_state, &mut host).unwrap(),
            NativeWindowHostLoopTurn::Continue(
                NativeWindowHostLoopContinueEvidence::PresentedFrame {
                    presentation: NativeWindowBackendLoopPresentation {
                        frame_id: 1,
                        width: initial_size.width,
                        height: initial_size.height,
                    },
                    window_size: initial_size,
                    size_changed: false,
                }
            )
        );
        assert!(host.titles.is_empty());
        assert_eq!(host.pump_count, 0);
        assert_eq!(
            host.present_frames,
            vec![(initial_size.width, initial_size.height)]
        );
    }

    #[test]
    fn step_native_window_host_loop_preserves_event_pump_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Err("event failed")]);

        assert_eq!(
            step_native_window_host_loop(&mut loop_state, &mut host).unwrap_err(),
            NativeWindowHostLoopError::HostEventPumpFailed("event failed")
        );
    }

    #[test]
    fn step_native_window_host_loop_preserves_present_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable)])
            .with_present_error("present failed");

        assert_eq!(
            step_native_window_host_loop(&mut loop_state, &mut host).unwrap_err(),
            NativeWindowHostLoopError::HostPresentFailed("present failed")
        );
    }

    #[test]
    fn step_native_window_host_loop_preserves_host_action_error() {
        let mut loop_state =
            NativeWindowBackendLoop::new_for_scale(GuiDemo::Counter, i32::MAX, 2).unwrap();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(440, 284),
            true,
            NativeWindowPointerSample::Available { x: 40.0, y: 180.0 },
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);

        assert_eq!(
            step_native_window_host_loop(&mut loop_state, &mut host).unwrap_err(),
            NativeWindowHostLoopError::HostActionFailed(NativeWindowHostActionError::StepFailed(
                NativeWindowBackendLoopError::CounterValueOverflow { previous: i32::MAX }
            ))
        );
    }

    #[test]
    fn step_native_window_host_loop_preserves_presenter_frame_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        loop_state.presenter_state.resize_surface(0, 0).unwrap();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable)]);

        assert_eq!(
            step_native_window_host_loop(&mut loop_state, &mut host).unwrap_err(),
            NativeWindowHostLoopError::PresenterFrameUnavailable(
                NativeWindowBackendLoopError::SurfaceUnavailable
            )
        );
    }

    #[derive(Debug)]
    struct ScriptedNativeWindowRunLoopHost {
        snapshots: Vec<Result<NativeWindowEventPumpSnapshot, &'static str>>,
        cursor: usize,
        titles: Vec<String>,
        pump_count: usize,
        present_frames: Vec<(usize, usize)>,
        wait_requests: Vec<NativeWindowHostLoopWaitRequest>,
        wait_outcomes: Vec<NativeWindowHostLoopWaitOutcome>,
        present_error: Option<&'static str>,
        wait_error: Option<&'static str>,
    }

    impl ScriptedNativeWindowRunLoopHost {
        fn new(snapshots: Vec<Result<NativeWindowEventPumpSnapshot, &'static str>>) -> Self {
            Self {
                snapshots,
                cursor: 0,
                titles: Vec::new(),
                pump_count: 0,
                present_frames: Vec::new(),
                wait_requests: Vec::new(),
                wait_outcomes: Vec::new(),
                present_error: None,
                wait_error: None,
            }
        }

        fn with_present_error(mut self, error: &'static str) -> Self {
            self.present_error = Some(error);
            self
        }

        fn with_wait_error(mut self, error: &'static str) -> Self {
            self.wait_error = Some(error);
            self
        }
    }

    impl NativeWindowRunLoopHost for ScriptedNativeWindowRunLoopHost {
        type EventError = &'static str;
        type PresentError = &'static str;
        type WaitError = &'static str;

        fn poll_event_snapshot(
            &mut self,
            _input: NativeWindowEventPumpInput,
        ) -> Result<NativeWindowEventPumpSnapshot, Self::EventError> {
            let Some(result) = self.snapshots.get(self.cursor).copied() else {
                return Err("event script exhausted");
            };
            self.cursor += 1;
            result
        }

        fn set_window_title(&mut self, title: &str) {
            self.titles.push(title.to_string());
        }

        fn pump_events_only(&mut self) {
            self.pump_count += 1;
        }

        fn present_frame(
            &mut self,
            frame: NativePresenterFrame<'_>,
        ) -> Result<(), Self::PresentError> {
            if let Some(error) = self.present_error {
                return Err(error);
            }
            self.present_frames.push((frame.width(), frame.height()));
            Ok(())
        }

        fn wait_after_budget_exhausted(
            &mut self,
            request: NativeWindowHostLoopWaitRequest,
        ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
            self.wait_requests.push(request.clone());
            if let Some(error) = self.wait_error {
                return Err(error);
            }
            let outcome = match request {
                NativeWindowHostLoopWaitRequest::WaitForHostEvent {
                    window_size,
                    size_changed,
                } => NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                    window_size,
                    size_changed,
                },
                NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
                    presentation,
                    window_size,
                    size_changed,
                    frame_interval: _,
                } => NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
                    presentation,
                    window_size,
                    size_changed,
                },
            };
            self.wait_outcomes.push(outcome.clone());
            Ok(outcome)
        }
    }

    #[test]
    fn native_window_host_loop_preserves_terminal_reason() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);

        let exit = run_native_window_host_loop(&mut loop_state, &mut host).unwrap();

        assert_eq!(
            exit.reason,
            NativeWindowHostTerminalReason::OsCloseRequested
        );
        assert_eq!(
            host.titles,
            vec![native_window_title(GuiDemo::Counter, initial_size)]
        );
        assert_eq!(host.pump_count, 0);
        assert!(host.present_frames.is_empty());
    }

    #[test]
    fn native_window_host_loop_pumps_unavailable_surface_without_presenting() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let unavailable_size = NativeWindowSize::new(0, initial_size.height);
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            unavailable_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::ExitShortcutRequested,
            unavailable_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(unavailable), Ok(close)]);

        let exit = run_native_window_host_loop(&mut loop_state, &mut host).unwrap();

        assert_eq!(
            exit.reason,
            NativeWindowHostTerminalReason::ExitShortcutRequested
        );
        assert_eq!(
            host.titles,
            vec![
                native_window_title(GuiDemo::Counter, initial_size),
                native_window_title(GuiDemo::Counter, unavailable_size),
            ]
        );
        assert_eq!(host.pump_count, 1);
        assert!(host.present_frames.is_empty());
    }

    #[test]
    fn native_window_host_loop_presents_exact_current_frame() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let close = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable), Ok(close)]);

        let exit = run_native_window_host_loop(&mut loop_state, &mut host).unwrap();

        assert_eq!(
            exit.reason,
            NativeWindowHostTerminalReason::OsCloseRequested
        );
        assert_eq!(host.pump_count, 0);
        assert_eq!(
            host.present_frames,
            vec![(initial_size.width, initial_size.height)]
        );
    }

    #[test]
    fn native_window_host_loop_preserves_event_pump_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Err("event failed")]);

        assert_eq!(
            run_native_window_host_loop(&mut loop_state, &mut host).unwrap_err(),
            NativeWindowHostLoopError::HostEventPumpFailed("event failed")
        );
    }

    #[test]
    fn native_window_host_loop_preserves_present_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable)])
            .with_present_error("present failed");

        assert_eq!(
            run_native_window_host_loop(&mut loop_state, &mut host).unwrap_err(),
            NativeWindowHostLoopError::HostPresentFailed("present failed")
        );
    }

    #[test]
    fn native_window_host_loop_preserves_host_action_error() {
        let mut loop_state =
            NativeWindowBackendLoop::new_for_scale(GuiDemo::Counter, i32::MAX, 2).unwrap();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(440, 284),
            true,
            NativeWindowPointerSample::Available { x: 40.0, y: 180.0 },
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);

        assert_eq!(
            run_native_window_host_loop(&mut loop_state, &mut host).unwrap_err(),
            NativeWindowHostLoopError::HostActionFailed(NativeWindowHostActionError::StepFailed(
                NativeWindowBackendLoopError::CounterValueOverflow { previous: i32::MAX }
            ))
        );
    }

    #[test]
    fn native_window_host_loop_preserves_presenter_frame_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        loop_state.presenter_state.resize_surface(0, 0).unwrap();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable)]);

        assert_eq!(
            run_native_window_host_loop(&mut loop_state, &mut host).unwrap_err(),
            NativeWindowHostLoopError::PresenterFrameUnavailable(
                NativeWindowBackendLoopError::SurfaceUnavailable
            )
        );
    }

    fn native_window_backend_loop_snapshot(
        loop_state: &NativeWindowBackendLoop,
        close_state: NativeWindowEventPumpCloseState,
        size: NativeWindowSize,
        mouse_down: bool,
        pointer_sample: NativeWindowPointerSample,
    ) -> NativeWindowEventPumpSnapshot {
        build_native_window_event_pump_snapshot(
            loop_state.event_pump_input(),
            close_state == NativeWindowEventPumpCloseState::OsCloseRequested,
            close_state == NativeWindowEventPumpCloseState::ExitShortcutRequested,
            size,
            mouse_down,
            pointer_sample,
        )
    }

    fn native_window_backend_loop_drawable(
        outcome: NativeWindowBackendLoopStepOutcome,
    ) -> NativeWindowBackendLoopDrawableStep {
        match outcome {
            NativeWindowBackendLoopStepOutcome::Drawable(drawable) => drawable,
            other => panic!("expected drawable outcome, got {other:?}"),
        }
    }

    #[test]
    fn native_window_backend_loop_initializes_scaled_surface() {
        assert_eq!(
            NativeWindowBackendLoop::new_for_scale(GuiDemo::Counter, 0, 0).unwrap_err(),
            NativeWindowBackendLoopError::InitialScaleInvalid
        );
        assert_eq!(
            NativeWindowBackendLoop::new_for_scale(GuiDemo::Counter, 0, usize::MAX).unwrap_err(),
            NativeWindowBackendLoopError::InitialSizeOverflow
        );

        let loop_state = native_window_backend_loop_counter();
        assert_eq!(loop_state.initial_size(), NativeWindowSize::new(440, 284));
        assert_eq!(loop_state.presenter_frame_id(), 1);
        let present_frame = loop_state.current_present_frame_for_window().unwrap();
        assert_eq!(present_frame.width(), 440);
        assert_eq!(present_frame.height(), 284);
    }

    #[test]
    fn native_window_backend_loop_close_does_not_progress_state() {
        let mut loop_state = native_window_backend_loop_counter();
        let input_before = loop_state.event_pump_input();
        let frame_id_before = loop_state.presenter_frame_id();
        let pixels_before = loop_state
            .current_present_frame_for_window()
            .unwrap()
            .pixels()
            .to_vec();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            NativeWindowSize::new(660, 426),
            true,
            NativeWindowPointerSample::Available { x: 60.0, y: 270.0 },
        );

        assert_eq!(
            loop_state.step(snapshot).unwrap(),
            NativeWindowBackendLoopStepOutcome::CloseRequested {
                close_state: NativeWindowEventPumpCloseState::OsCloseRequested,
            }
        );
        assert_eq!(loop_state.event_pump_input(), input_before);
        assert_eq!(loop_state.presenter_frame_id(), frame_id_before);
        assert_eq!(
            loop_state
                .current_present_frame_for_window()
                .unwrap()
                .pixels(),
            pixels_before.as_slice()
        );
    }

    #[test]
    fn native_window_backend_loop_host_action_preserves_terminal_reason() {
        let mut os_close_loop = native_window_backend_loop_counter();
        let os_close = native_window_backend_loop_snapshot(
            &os_close_loop,
            NativeWindowEventPumpCloseState::OsCloseRequested,
            NativeWindowSize::new(660, 426),
            false,
            NativeWindowPointerSample::Unavailable,
        );
        assert_eq!(
            os_close_loop.step_host_action(os_close).unwrap(),
            NativeWindowHostAction::Terminate {
                reason: NativeWindowHostTerminalReason::OsCloseRequested,
            }
        );

        let mut shortcut_loop = native_window_backend_loop_counter();
        let shortcut = native_window_backend_loop_snapshot(
            &shortcut_loop,
            NativeWindowEventPumpCloseState::ExitShortcutRequested,
            NativeWindowSize::new(660, 426),
            false,
            NativeWindowPointerSample::Unavailable,
        );
        assert_eq!(
            shortcut_loop.step_host_action(shortcut).unwrap(),
            NativeWindowHostAction::Terminate {
                reason: NativeWindowHostTerminalReason::ExitShortcutRequested,
            }
        );
    }

    #[test]
    fn native_window_backend_loop_host_action_rejects_impossible_open_close() {
        assert_eq!(
            native_window_host_action_from_backend_loop_outcome(
                NativeWindowBackendLoopStepOutcome::CloseRequested {
                    close_state: NativeWindowEventPumpCloseState::Open,
                }
            )
            .unwrap_err(),
            NativeWindowHostActionError::UnsupportedCloseState {
                close_state: NativeWindowEventPumpCloseState::Open,
            }
        );
    }

    #[test]
    fn native_window_backend_loop_host_action_unavailable_pumps_events_only() {
        let mut loop_state = native_window_backend_loop_counter();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(0, 284),
            true,
            NativeWindowPointerSample::Unavailable,
        );

        assert_eq!(
            loop_state.step_host_action(snapshot).unwrap(),
            NativeWindowHostAction::PumpEventsOnly {
                window_size: NativeWindowSize::new(0, 284),
                size_changed: true,
            }
        );
        assert_eq!(
            loop_state.current_present_frame_for_window().unwrap_err(),
            NativeWindowBackendLoopError::SurfaceUnavailable
        );
    }

    #[test]
    fn native_window_backend_loop_host_action_drawable_presents_final_frame_evidence() {
        let mut loop_state = native_window_backend_loop_counter();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(660, 426),
            false,
            NativeWindowPointerSample::Unavailable,
        );

        assert_eq!(
            loop_state.step_host_action(snapshot).unwrap(),
            NativeWindowHostAction::PresentFrame {
                presentation: NativeWindowBackendLoopPresentation {
                    frame_id: 2,
                    width: 660,
                    height: 426,
                },
                window_size: NativeWindowSize::new(660, 426),
                size_changed: true,
            }
        );
        let present_frame = loop_state.current_present_frame_for_window().unwrap();
        assert_eq!(present_frame.width(), 660);
        assert_eq!(present_frame.height(), 426);
    }

    #[test]
    fn native_window_backend_loop_unavailable_updates_observation_without_blank_frame() {
        let mut loop_state = native_window_backend_loop_counter();
        let frame_id_before = loop_state.presenter_frame_id();
        let pixels_before = loop_state
            .current_present_frame_for_window()
            .unwrap()
            .pixels()
            .to_vec();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(0, 284),
            true,
            NativeWindowPointerSample::Unavailable,
        );

        assert_eq!(
            loop_state.step(snapshot).unwrap(),
            NativeWindowBackendLoopStepOutcome::Unavailable {
                window_size: NativeWindowSize::new(0, 284),
                size_changed: true,
            }
        );
        assert_eq!(
            loop_state.event_pump_input(),
            NativeWindowEventPumpInput {
                previous_size: NativeWindowSize::new(0, 284),
                previous_mouse_down: true,
            }
        );
        assert_eq!(
            loop_state.presenter_state().surface_state(),
            NativeWindowPresenterSurfaceState::Unavailable
        );
        assert_eq!(loop_state.presenter_frame_id(), frame_id_before);
        assert_eq!(
            loop_state
                .presenter_state()
                .last_present_frame_required()
                .unwrap()
                .pixels(),
            pixels_before.as_slice()
        );
        assert_eq!(
            loop_state.current_present_frame_for_window().unwrap_err(),
            NativeWindowBackendLoopError::SurfaceUnavailable
        );
    }

    #[test]
    fn native_window_backend_loop_restores_positive_surface_after_zero_size() {
        let mut loop_state = native_window_backend_loop_counter();
        let unavailable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(0, 284),
            false,
            NativeWindowPointerSample::Unavailable,
        );
        loop_state.step(unavailable).unwrap();

        let restored = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(660, 426),
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let drawable = native_window_backend_loop_drawable(loop_state.step(restored).unwrap());
        assert_eq!(
            drawable.resize_redraw,
            Some(NativeWindowBackendLoopPresentation {
                frame_id: 2,
                width: 660,
                height: 426,
            })
        );
        assert_eq!(drawable.final_frame.frame_id, 2);
        assert_eq!(
            loop_state.presenter_state().surface_state(),
            NativeWindowPresenterSurfaceState::Drawable {
                width: 660,
                height: 426,
            }
        );
        assert_eq!(
            loop_state.event_pump_input().previous_size,
            NativeWindowSize::new(660, 426)
        );
    }

    #[test]
    fn native_window_backend_loop_resize_and_counter_report_both_presentations() {
        let mut loop_state = native_window_backend_loop_counter();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(660, 426),
            true,
            NativeWindowPointerSample::Available { x: 60.0, y: 270.0 },
        );
        let drawable = native_window_backend_loop_drawable(loop_state.step(snapshot).unwrap());

        assert_eq!(
            drawable.resize_redraw,
            Some(NativeWindowBackendLoopPresentation {
                frame_id: 2,
                width: 660,
                height: 426,
            })
        );
        assert_eq!(
            drawable.pointer_action,
            NativeWindowBackendLoopPointerAction::CounterIncremented {
                value: 1,
                presentation: NativeWindowBackendLoopPresentation {
                    frame_id: 3,
                    width: 660,
                    height: 426,
                },
            }
        );
        assert_eq!(
            drawable.final_frame,
            NativeWindowBackendLoopPresentation {
                frame_id: 3,
                width: 660,
                height: 426,
            }
        );
        assert_eq!(loop_state.counter_value(), 1);
        assert_eq!(loop_state.presenter_frame_id(), 3);
        let present_frame = loop_state.current_present_frame_for_window().unwrap();
        assert_eq!(present_frame.width(), 660);
        assert_eq!(present_frame.height(), 426);
    }

    #[test]
    fn native_window_backend_loop_distinguishes_pointer_miss_reasons() {
        let mut unavailable_loop = native_window_backend_loop_counter();
        let unavailable_snapshot = native_window_backend_loop_snapshot(
            &unavailable_loop,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(440, 284),
            true,
            NativeWindowPointerSample::Unavailable,
        );
        let unavailable = native_window_backend_loop_drawable(
            unavailable_loop.step(unavailable_snapshot).unwrap(),
        );
        assert_eq!(
            unavailable.pointer_action,
            NativeWindowBackendLoopPointerAction::PressedUnavailable
        );
        assert_eq!(unavailable_loop.counter_value(), 0);
        assert_eq!(unavailable_loop.presenter_frame_id(), 1);

        let mut outside_loop = native_window_backend_loop_counter();
        let outside_snapshot = native_window_backend_loop_snapshot(
            &outside_loop,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(440, 284),
            true,
            NativeWindowPointerSample::Available { x: 1.0, y: 1.0 },
        );
        let outside =
            native_window_backend_loop_drawable(outside_loop.step(outside_snapshot).unwrap());
        assert_eq!(
            outside.pointer_action,
            NativeWindowBackendLoopPointerAction::PressedOutside
        );
        assert_eq!(outside_loop.counter_value(), 0);
        assert_eq!(outside_loop.presenter_frame_id(), 1);
    }

    #[test]
    fn native_window_backend_loop_frame_id_overflow_happens_before_mutation() {
        let mut loop_state = native_window_backend_loop_counter();
        loop_state.state.presenter_frame_id = i32::MAX;
        let input_before = loop_state.event_pump_input();
        let pixels_before = loop_state
            .current_present_frame_for_window()
            .unwrap()
            .pixels()
            .to_vec();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(660, 426),
            false,
            NativeWindowPointerSample::Unavailable,
        );

        assert_eq!(
            loop_state.step(snapshot).unwrap_err(),
            NativeWindowBackendLoopError::FrameIdOverflow { previous: i32::MAX }
        );
        assert_eq!(loop_state.event_pump_input(), input_before);
        assert_eq!(loop_state.presenter_frame_id(), i32::MAX);
        assert_eq!(
            loop_state
                .current_present_frame_for_window()
                .unwrap()
                .pixels(),
            pixels_before.as_slice()
        );
    }

    #[test]
    fn native_window_backend_loop_counter_overflow_happens_before_mutation() {
        let mut loop_state =
            NativeWindowBackendLoop::new_for_scale(GuiDemo::Counter, i32::MAX, 2).unwrap();
        let input_before = loop_state.event_pump_input();
        let pixels_before = loop_state
            .current_present_frame_for_window()
            .unwrap()
            .pixels()
            .to_vec();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(440, 284),
            true,
            NativeWindowPointerSample::Available { x: 40.0, y: 180.0 },
        );

        assert_eq!(
            loop_state.step(snapshot).unwrap_err(),
            NativeWindowBackendLoopError::CounterValueOverflow { previous: i32::MAX }
        );
        assert_eq!(loop_state.event_pump_input(), input_before);
        assert_eq!(loop_state.counter_value(), i32::MAX);
        assert_eq!(loop_state.presenter_frame_id(), 1);
        assert_eq!(
            loop_state
                .current_present_frame_for_window()
                .unwrap()
                .pixels(),
            pixels_before.as_slice()
        );
    }

    #[test]
    fn native_window_backend_loop_rasterize_failure_preserves_old_surface_and_frame() {
        let mut loop_state = native_window_backend_loop_counter();
        let old_input = loop_state.event_pump_input();
        let old_surface = loop_state.presenter_state().surface_state();
        let old_frame_id = loop_state.presenter_frame_id();
        let old_pixels = loop_state
            .current_present_frame_for_window()
            .unwrap()
            .pixels()
            .to_vec();
        loop_state.state.frame.rects.push(RectCommand {
            x: loop_state.state.frame.width,
            y: 0,
            width: 1,
            height: 1,
            color: 0,
        });
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            NativeWindowSize::new(660, 426),
            false,
            NativeWindowPointerSample::Unavailable,
        );

        assert_eq!(
            loop_state.step(snapshot).unwrap_err(),
            NativeWindowBackendLoopError::RasterizeFailed(
                RasterizeSurfaceError::CommandOutOfBounds
            )
        );
        assert_eq!(loop_state.event_pump_input(), old_input);
        assert_eq!(loop_state.presenter_state().surface_state(), old_surface);
        assert_eq!(loop_state.presenter_frame_id(), old_frame_id);
        assert_eq!(
            loop_state
                .current_present_frame_for_window()
                .unwrap()
                .pixels(),
            old_pixels.as_slice()
        );
    }

    #[test]
    fn native_rgb0_presenter_private_helper_keeps_active_on_conversion_failure() {
        let descriptor = native_framebuffer_descriptor(1);
        let active = NativeSpanFramebufferActiveSequence {
            descriptor,
            seen_run_count: 1,
        };
        let mut frame_buffer = NativeRgba8888FrameBuffer {
            width: 4,
            height: 3,
            stride_bytes: 16,
            pixels: vec![NATIVE_RGBA8888_PIXEL_TRANSPARENT; 11],
            active_sequence: Some(active),
        };
        let background = NativeRgbColor { r: 0, g: 0, b: 0 };

        assert_eq!(
            frame_buffer
                .end_sequence_to_rgb0_present_buffer(descriptor, background)
                .unwrap_err(),
            NativeSpanFramebufferError::InternalIndexOverflow
        );
        assert_eq!(frame_buffer.active_sequence(), Some(active));
    }

    #[test]
    fn native_span_framebuffer_end_semantics_still_close_sequence() {
        let descriptor = native_framebuffer_descriptor(1);
        let mut frame_buffer = NativeRgba8888FrameBuffer::new(4, 3).unwrap();

        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::Begin(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::RunSpan(
                native_framebuffer_run(0, 0, 1, 10),
            )),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(
            frame_buffer.execute_span_operation(NativeSpanOperation::End(descriptor)),
            GUI_NATIVE_SPAN_OPERATION_STATUS_OK
        );
        assert_eq!(frame_buffer.active_sequence(), None);
    }
}
