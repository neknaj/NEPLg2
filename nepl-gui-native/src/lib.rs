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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowRunLoopFrameIntervalWaitBackend {
    MinifbInternalTargetFps,
    HostOwnedDeadlineTimer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowRunLoopWaitBackend {
    MinifbInternalTargetFps,
    HostOwnedDeadlineTimer,
    PlatformWait(NativeWindowHostLoopPlatformWaitBackendSelection),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowRunLoopFrameIntervalWaitBackendRunner {
    Minifb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowRunLoopFrameIntervalWaitBackendError {
    Unsupported {
        runner: NativeWindowRunLoopFrameIntervalWaitBackendRunner,
        requested: NativeWindowRunLoopWaitBackend,
        reason: NativeWindowFrameIntervalWaitAuthorityModeError,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowRunLoopPlatformWaitBackendConfigError {
    NotPlatformWaitBackend {
        requested: NativeWindowRunLoopWaitBackend,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowRunLoopPlatformWaitBackendFromConfigError {
    Config(NativeWindowRunLoopPlatformWaitBackendConfigError),
    Build(NativeWindowHostLoopPlatformWaitHostBuildError),
}

#[cfg(target_os = "windows")]
pub type NativeWindowWindowsPlatformWaitHostLoopError = NativeWindowHostLoopError<
    NativeWindowEventPumpError,
    String,
    NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError<
        NativeWindowHostLoopPlatformWaitBackendError<
            NativeWindowHostLoopWindowsWaitBackendError,
            NativeWindowHostLoopMacosRunLoopTimerBackendError,
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError,
        >,
    >,
>;

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

impl NativeWindowRunLoopFrameIntervalWaitBackend {
    pub fn authority_mode(
        self,
        target_fps: NativeWindowTargetFps,
    ) -> NativeWindowFrameIntervalWaitAuthorityMode {
        match self {
            NativeWindowRunLoopFrameIntervalWaitBackend::MinifbInternalTargetFps => {
                native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(
                    target_fps,
                )
            }
            NativeWindowRunLoopFrameIntervalWaitBackend::HostOwnedDeadlineTimer => {
                native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer()
            }
        }
    }
}

impl Default for NativeWindowRunLoopFrameIntervalWaitBackend {
    fn default() -> Self {
        Self::MinifbInternalTargetFps
    }
}

impl From<NativeWindowRunLoopFrameIntervalWaitBackend> for NativeWindowRunLoopWaitBackend {
    fn from(value: NativeWindowRunLoopFrameIntervalWaitBackend) -> Self {
        match value {
            NativeWindowRunLoopFrameIntervalWaitBackend::MinifbInternalTargetFps => {
                Self::MinifbInternalTargetFps
            }
            NativeWindowRunLoopFrameIntervalWaitBackend::HostOwnedDeadlineTimer => {
                Self::HostOwnedDeadlineTimer
            }
        }
    }
}

impl NativeWindowRunLoopWaitBackend {
    pub fn authority_mode(
        self,
        target_fps: NativeWindowTargetFps,
    ) -> NativeWindowFrameIntervalWaitAuthorityMode {
        match self {
            NativeWindowRunLoopWaitBackend::MinifbInternalTargetFps => {
                native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(
                    target_fps,
                )
            }
            NativeWindowRunLoopWaitBackend::HostOwnedDeadlineTimer
            | NativeWindowRunLoopWaitBackend::PlatformWait(_) => {
                native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer()
            }
        }
    }
}

impl Default for NativeWindowRunLoopWaitBackend {
    fn default() -> Self {
        Self::MinifbInternalTargetFps
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowRunLoopConfig {
    pub demo: GuiDemo,
    pub counter_value: i32,
    pub scale: usize,
    pub target_fps: NativeWindowTargetFps,
    pub host_loop_policy: NativeWindowHostLoopRunPolicy,
    pub wait_backend: NativeWindowRunLoopWaitBackend,
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
    HostWaitFailed {
        message: String,
    },
    PlatformWaitBackendFromConfigFailed(NativeWindowRunLoopPlatformWaitBackendFromConfigError),
    #[cfg(target_os = "windows")]
    WindowsPlatformWaitHostLoopFailed(NativeWindowWindowsPlatformWaitHostLoopError),
    FrameIntervalWaitBackendUnsupported(NativeWindowRunLoopFrameIntervalWaitBackendError),
    TimerFireResumeRequired {
        timer_registration_id: u32,
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
        instruction: NativeWindowHostLoopWaitInstruction,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Separates observed-input signal failures from the host wait implementation error.
pub enum NativeWindowHostEventSignalWaitError<WaitError> {
    HostEventSignalFailed(NativeWindowHostLoopLinuxHostEventSignalProducerError),
    DelegateWaitFailed(WaitError),
}

/// Exposes deferred signal failure state collected outside the wait call path.
pub trait NativeWindowHostEventSignalErrorState {
    fn take_host_event_signal_error(
        &mut self,
    ) -> Option<NativeWindowHostLoopLinuxHostEventSignalProducerError>;
}

/// Wraps a run-loop host and checks deferred signal failures before delegating wait.
pub struct NativeWindowHostEventSignalWaitGuardRunLoopHost<Host, SignalState> {
    host: Host,
    signal_state: SignalState,
}

impl<Host, SignalState> NativeWindowHostEventSignalWaitGuardRunLoopHost<Host, SignalState> {
    pub fn new(host: Host, signal_state: SignalState) -> Self {
        Self { host, signal_state }
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut Host {
        &mut self.host
    }

    pub fn signal_state(&self) -> &SignalState {
        &self.signal_state
    }

    pub fn signal_state_mut(&mut self) -> &mut SignalState {
        &mut self.signal_state
    }

    pub fn into_parts(self) -> (Host, SignalState) {
        (self.host, self.signal_state)
    }
}

impl<Host, SignalState> NativeWindowRunLoopHost
    for NativeWindowHostEventSignalWaitGuardRunLoopHost<Host, SignalState>
where
    Host: NativeWindowRunLoopHost,
    SignalState: NativeWindowHostEventSignalErrorState,
{
    type EventError = Host::EventError;
    type PresentError = Host::PresentError;
    type WaitError = NativeWindowHostEventSignalWaitError<Host::WaitError>;

    fn poll_event_snapshot(
        &mut self,
        input: NativeWindowEventPumpInput,
    ) -> Result<NativeWindowEventPumpSnapshot, Self::EventError> {
        self.host.poll_event_snapshot(input)
    }

    fn set_window_title(&mut self, title: &str) {
        self.host.set_window_title(title);
    }

    fn pump_events_only(&mut self) {
        self.host.pump_events_only();
    }

    fn present_frame(&mut self, frame: NativePresenterFrame<'_>) -> Result<(), Self::PresentError> {
        self.host.present_frame(frame)
    }

    fn wait_after_budget_exhausted(
        &mut self,
        instruction: NativeWindowHostLoopWaitInstruction,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
        if let Some(error) = self.signal_state.take_host_event_signal_error() {
            return Err(NativeWindowHostEventSignalWaitError::HostEventSignalFailed(
                error,
            ));
        }
        self.host
            .wait_after_budget_exhausted(instruction)
            .map_err(NativeWindowHostEventSignalWaitError::DelegateWaitFailed)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopError<EventError, PresentError, WaitError> {
    HostEventPumpFailed(EventError),
    HostActionFailed(NativeWindowHostActionError),
    PresenterFrameUnavailable(NativeWindowBackendLoopError),
    HostPresentFailed(PresentError),
    HostWaitFailed(WaitError),
    TimerFireResumeRequired {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        wait_nanos: u32,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    },
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
pub enum NativeWindowHostLoopWaitInstruction {
    WaitForHostEvent {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    WaitForFrameInterval {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        frame_interval: NativeWindowFrameIntervalRequest,
        wait_nanos: u32,
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
    FrameIntervalTimerRegistered {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        wait_nanos: u32,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    },
    FrameIntervalTimerFired {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        wait_nanos: u32,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopThreadWaitError<SleeperError> {
    HostEventWaitUnsupported {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    FrameIntervalWaitNanosMismatch {
        wait_nanos: u32,
        nanos_per_frame: u32,
    },
    SleeperFailed(SleeperError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopThreadWaitOutcome {
    FrameIntervalSlept {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        wait_nanos: u32,
    },
}

pub trait NativeWindowHostLoopThreadSleeper {
    type Error;

    fn sleep_for_nanos(&mut self, wait_nanos: u32) -> Result<(), Self::Error>;
}

pub fn execute_native_window_host_loop_thread_wait_with_sleeper<Sleeper>(
    instruction: NativeWindowHostLoopWaitInstruction,
    sleeper: &mut Sleeper,
) -> Result<
    NativeWindowHostLoopThreadWaitOutcome,
    NativeWindowHostLoopThreadWaitError<Sleeper::Error>,
>
where
    Sleeper: NativeWindowHostLoopThreadSleeper,
{
    match instruction {
        NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed,
        } => Err(
            NativeWindowHostLoopThreadWaitError::HostEventWaitUnsupported {
                window_size,
                size_changed,
            },
        ),
        NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
            frame_interval,
            wait_nanos,
        } => {
            let nanos_per_frame = frame_interval.nanos_per_frame();
            if wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame + 1 {
                return Err(
                    NativeWindowHostLoopThreadWaitError::FrameIntervalWaitNanosMismatch {
                        wait_nanos,
                        nanos_per_frame,
                    },
                );
            }
            sleeper
                .sleep_for_nanos(wait_nanos)
                .map_err(NativeWindowHostLoopThreadWaitError::SleeperFailed)?;
            Ok(NativeWindowHostLoopThreadWaitOutcome::FrameIntervalSlept {
                presentation,
                window_size,
                size_changed,
                wait_nanos,
            })
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StdNativeWindowHostLoopThreadSleeper;

#[cfg(not(target_arch = "wasm32"))]
impl NativeWindowHostLoopThreadSleeper for StdNativeWindowHostLoopThreadSleeper {
    type Error = std::convert::Infallible;

    fn sleep_for_nanos(&mut self, wait_nanos: u32) -> Result<(), Self::Error> {
        std::thread::sleep(std::time::Duration::from_nanos(u64::from(wait_nanos)));
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn execute_native_window_host_loop_thread_wait(
    instruction: NativeWindowHostLoopWaitInstruction,
) -> Result<
    NativeWindowHostLoopThreadWaitOutcome,
    NativeWindowHostLoopThreadWaitError<std::convert::Infallible>,
> {
    let mut sleeper = StdNativeWindowHostLoopThreadSleeper;
    execute_native_window_host_loop_thread_wait_with_sleeper(instruction, &mut sleeper)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopTimerRegistrationId {
    raw_id: u32,
}

impl NativeWindowHostLoopTimerRegistrationId {
    pub fn raw_id(self) -> u32 {
        self.raw_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopTimerRegistrationError<RegistrarError> {
    HostEventTimerRegistrationUnsupported {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    FrameIntervalWaitNanosMismatch {
        wait_nanos: u32,
        nanos_per_frame: u32,
    },
    InvalidTimerRegistrationId {
        raw_id: u32,
    },
    RegistrarFailed(RegistrarError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopTimerRegistrationOutcome {
    FrameIntervalTimerRegistered {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        wait_nanos: u32,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    },
}

pub trait NativeWindowHostLoopTimerRegistrar {
    type Error;

    fn register_timer_nanos(&mut self, wait_nanos: u32) -> Result<u32, Self::Error>;
}

pub fn execute_native_window_host_loop_timer_registration_with_registrar<Registrar>(
    instruction: NativeWindowHostLoopWaitInstruction,
    registrar: &mut Registrar,
) -> Result<
    NativeWindowHostLoopTimerRegistrationOutcome,
    NativeWindowHostLoopTimerRegistrationError<Registrar::Error>,
>
where
    Registrar: NativeWindowHostLoopTimerRegistrar,
{
    match instruction {
        NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed,
        } => Err(
            NativeWindowHostLoopTimerRegistrationError::HostEventTimerRegistrationUnsupported {
                window_size,
                size_changed,
            },
        ),
        NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
            frame_interval,
            wait_nanos,
        } => {
            let nanos_per_frame = frame_interval.nanos_per_frame();
            if wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame + 1 {
                return Err(
                    NativeWindowHostLoopTimerRegistrationError::FrameIntervalWaitNanosMismatch {
                        wait_nanos,
                        nanos_per_frame,
                    },
                );
            }
            let raw_id = registrar
                .register_timer_nanos(wait_nanos)
                .map_err(NativeWindowHostLoopTimerRegistrationError::RegistrarFailed)?;
            if raw_id == 0 {
                return Err(
                    NativeWindowHostLoopTimerRegistrationError::InvalidTimerRegistrationId {
                        raw_id,
                    },
                );
            }
            Ok(
                NativeWindowHostLoopTimerRegistrationOutcome::FrameIntervalTimerRegistered {
                    presentation,
                    window_size,
                    size_changed,
                    wait_nanos,
                    timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id },
                },
            )
        }
    }
}

pub fn execute_native_window_host_loop_timer_registration_wait_with_registrar<Registrar>(
    instruction: NativeWindowHostLoopWaitInstruction,
    registrar: &mut Registrar,
) -> Result<
    NativeWindowHostLoopWaitOutcome,
    NativeWindowHostLoopTimerRegistrationError<Registrar::Error>,
>
where
    Registrar: NativeWindowHostLoopTimerRegistrar,
{
    let registration =
        execute_native_window_host_loop_timer_registration_with_registrar(instruction, registrar)?;
    match registration {
        NativeWindowHostLoopTimerRegistrationOutcome::FrameIntervalTimerRegistered {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        } => Ok(
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
                presentation,
                window_size,
                size_changed,
                wait_nanos,
                timer_registration_id,
            },
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopTimerFireError<WaiterError> {
    HostEventPumpOutcomeUnsupported {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    FramePresentOutcomeUnsupported {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    InvalidFiredTimerRegistrationId {
        raw_id: u32,
    },
    FiredTimerRegistrationMismatch {
        expected_raw_id: u32,
        actual_raw_id: u32,
    },
    WaiterFailed(WaiterError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopTimerFireOutcome {
    FrameIntervalTimerFired {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        wait_nanos: u32,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    },
}

pub trait NativeWindowHostLoopTimerFireWaiter {
    type Error;

    fn wait_for_timer_fire(
        &mut self,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    ) -> Result<u32, Self::Error>;
}

pub fn execute_native_window_host_loop_timer_fire_wait_with_waiter<Waiter>(
    outcome: NativeWindowHostLoopWaitOutcome,
    waiter: &mut Waiter,
) -> Result<NativeWindowHostLoopTimerFireOutcome, NativeWindowHostLoopTimerFireError<Waiter::Error>>
where
    Waiter: NativeWindowHostLoopTimerFireWaiter,
{
    match outcome {
        NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
            window_size,
            size_changed,
        } => Err(
            NativeWindowHostLoopTimerFireError::HostEventPumpOutcomeUnsupported {
                window_size,
                size_changed,
            },
        ),
        NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
            presentation,
            window_size,
            size_changed,
        } => Err(
            NativeWindowHostLoopTimerFireError::FramePresentOutcomeUnsupported {
                presentation,
                window_size,
                size_changed,
            },
        ),
        NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        } => {
            let actual_raw_id = waiter
                .wait_for_timer_fire(timer_registration_id)
                .map_err(NativeWindowHostLoopTimerFireError::WaiterFailed)?;
            if actual_raw_id == 0 {
                return Err(
                    NativeWindowHostLoopTimerFireError::InvalidFiredTimerRegistrationId {
                        raw_id: actual_raw_id,
                    },
                );
            }
            let expected_raw_id = timer_registration_id.raw_id();
            if actual_raw_id != expected_raw_id {
                return Err(
                    NativeWindowHostLoopTimerFireError::FiredTimerRegistrationMismatch {
                        expected_raw_id,
                        actual_raw_id,
                    },
                );
            }
            Ok(
                NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
                    presentation,
                    window_size,
                    size_changed,
                    wait_nanos,
                    timer_registration_id,
                },
            )
        }
        NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        } => Ok(
            NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed,
                wait_nanos,
                timer_registration_id,
            },
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopTimerWakeError<RegistrarError, FireWaiterError> {
    RegistrationFailed(NativeWindowHostLoopTimerRegistrationError<RegistrarError>),
    FireFailed(NativeWindowHostLoopTimerFireError<FireWaiterError>),
}

pub fn execute_native_window_host_loop_timer_wakeup_with_backend<Registrar, Waiter>(
    instruction: NativeWindowHostLoopWaitInstruction,
    registrar: &mut Registrar,
    waiter: &mut Waiter,
) -> Result<
    NativeWindowHostLoopTimerFireOutcome,
    NativeWindowHostLoopTimerWakeError<Registrar::Error, Waiter::Error>,
>
where
    Registrar: NativeWindowHostLoopTimerRegistrar,
    Waiter: NativeWindowHostLoopTimerFireWaiter,
{
    let registration_outcome =
        execute_native_window_host_loop_timer_registration_wait_with_registrar(
            instruction,
            registrar,
        )
        .map_err(NativeWindowHostLoopTimerWakeError::RegistrationFailed)?;
    execute_native_window_host_loop_timer_fire_wait_with_waiter(registration_outcome, waiter)
        .map_err(NativeWindowHostLoopTimerWakeError::FireFailed)
}

pub fn native_window_host_loop_wait_outcome_from_timer_fire(
    outcome: NativeWindowHostLoopTimerFireOutcome,
) -> NativeWindowHostLoopWaitOutcome {
    match outcome {
        NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        } => NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        },
    }
}

pub fn execute_native_window_host_loop_timer_wakeup_wait_with_backend<Registrar, Waiter>(
    instruction: NativeWindowHostLoopWaitInstruction,
    registrar: &mut Registrar,
    waiter: &mut Waiter,
) -> Result<
    NativeWindowHostLoopWaitOutcome,
    NativeWindowHostLoopTimerWakeError<Registrar::Error, Waiter::Error>,
>
where
    Registrar: NativeWindowHostLoopTimerRegistrar,
    Waiter: NativeWindowHostLoopTimerFireWaiter,
{
    execute_native_window_host_loop_timer_wakeup_with_backend(instruction, registrar, waiter)
        .map(native_window_host_loop_wait_outcome_from_timer_fire)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopDeadlineTimerRecord {
    pub timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    pub deadline_nanos: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopDeadlineTimerAdapterError<ClockError, SleeperError> {
    ActiveTimerAlreadyRegistered {
        active_raw_id: u32,
    },
    NoActiveTimer {
        requested_raw_id: u32,
    },
    TimerRegistrationIdOverflow {
        last_raw_id: u32,
    },
    DeadlineNanosOverflow {
        now_nanos: u64,
        wait_nanos: u32,
    },
    ClockFailed(ClockError),
    SleeperFailed(SleeperError),
    FiredTimerRegistrationMismatch {
        expected_raw_id: u32,
        actual_raw_id: u32,
    },
}

pub trait NativeWindowHostLoopDeadlineTimerClock {
    type Error;

    fn now_nanos(&mut self) -> Result<u64, Self::Error>;
}

pub trait NativeWindowHostLoopDeadlineTimerSleeper {
    type Error;

    fn sleep_until_nanos(&mut self, deadline_nanos: u64) -> Result<(), Self::Error>;
}

pub struct NativeWindowHostLoopDeadlineTimerAdapter<Clock, Sleeper> {
    next_raw_id: u32,
    active_timer: Option<NativeWindowHostLoopDeadlineTimerRecord>,
    clock: Clock,
    sleeper: Sleeper,
}

impl<Clock, Sleeper> NativeWindowHostLoopDeadlineTimerAdapter<Clock, Sleeper> {
    pub fn new(clock: Clock, sleeper: Sleeper) -> Self {
        Self {
            next_raw_id: 1,
            active_timer: None,
            clock,
            sleeper,
        }
    }

    pub fn active_timer(&self) -> Option<NativeWindowHostLoopDeadlineTimerRecord> {
        self.active_timer
    }

    pub fn next_raw_id(&self) -> u32 {
        self.next_raw_id
    }
}

impl<Clock, Sleeper> NativeWindowHostLoopTimerRegistrar
    for NativeWindowHostLoopDeadlineTimerAdapter<Clock, Sleeper>
where
    Clock: NativeWindowHostLoopDeadlineTimerClock,
    Sleeper: NativeWindowHostLoopDeadlineTimerSleeper,
{
    type Error = NativeWindowHostLoopDeadlineTimerAdapterError<Clock::Error, Sleeper::Error>;

    fn register_timer_nanos(&mut self, wait_nanos: u32) -> Result<u32, Self::Error> {
        if let Some(active_timer) = self.active_timer {
            return Err(
                NativeWindowHostLoopDeadlineTimerAdapterError::ActiveTimerAlreadyRegistered {
                    active_raw_id: active_timer.timer_registration_id.raw_id(),
                },
            );
        }
        let raw_id = self.next_raw_id;
        let next_raw_id = raw_id.checked_add(1).ok_or(
            NativeWindowHostLoopDeadlineTimerAdapterError::TimerRegistrationIdOverflow {
                last_raw_id: raw_id,
            },
        )?;
        let now_nanos = self
            .clock
            .now_nanos()
            .map_err(NativeWindowHostLoopDeadlineTimerAdapterError::ClockFailed)?;
        let deadline_nanos = now_nanos.checked_add(u64::from(wait_nanos)).ok_or(
            NativeWindowHostLoopDeadlineTimerAdapterError::DeadlineNanosOverflow {
                now_nanos,
                wait_nanos,
            },
        )?;
        self.next_raw_id = next_raw_id;
        self.active_timer = Some(NativeWindowHostLoopDeadlineTimerRecord {
            timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id },
            deadline_nanos,
        });
        Ok(raw_id)
    }
}

impl<Clock, Sleeper> NativeWindowHostLoopTimerFireWaiter
    for NativeWindowHostLoopDeadlineTimerAdapter<Clock, Sleeper>
where
    Clock: NativeWindowHostLoopDeadlineTimerClock,
    Sleeper: NativeWindowHostLoopDeadlineTimerSleeper,
{
    type Error = NativeWindowHostLoopDeadlineTimerAdapterError<Clock::Error, Sleeper::Error>;

    fn wait_for_timer_fire(
        &mut self,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    ) -> Result<u32, Self::Error> {
        let Some(active_timer) = self.active_timer else {
            return Err(
                NativeWindowHostLoopDeadlineTimerAdapterError::NoActiveTimer {
                    requested_raw_id: timer_registration_id.raw_id(),
                },
            );
        };
        let expected_raw_id = active_timer.timer_registration_id.raw_id();
        let actual_raw_id = timer_registration_id.raw_id();
        if expected_raw_id != actual_raw_id {
            return Err(
                NativeWindowHostLoopDeadlineTimerAdapterError::FiredTimerRegistrationMismatch {
                    expected_raw_id,
                    actual_raw_id,
                },
            );
        }
        self.sleeper
            .sleep_until_nanos(active_timer.deadline_nanos)
            .map_err(NativeWindowHostLoopDeadlineTimerAdapterError::SleeperFailed)?;
        self.active_timer = None;
        Ok(actual_raw_id)
    }
}

pub type NativeWindowHostLoopDeadlineTimerWakeError<ClockError, SleeperError> =
    NativeWindowHostLoopTimerWakeError<
        NativeWindowHostLoopDeadlineTimerAdapterError<ClockError, SleeperError>,
        NativeWindowHostLoopDeadlineTimerAdapterError<ClockError, SleeperError>,
    >;

pub fn execute_native_window_host_loop_deadline_timer_wakeup_with_adapter<Clock, Sleeper>(
    instruction: NativeWindowHostLoopWaitInstruction,
    adapter: &mut NativeWindowHostLoopDeadlineTimerAdapter<Clock, Sleeper>,
) -> Result<
    NativeWindowHostLoopTimerFireOutcome,
    NativeWindowHostLoopDeadlineTimerWakeError<Clock::Error, Sleeper::Error>,
>
where
    Clock: NativeWindowHostLoopDeadlineTimerClock,
    Sleeper: NativeWindowHostLoopDeadlineTimerSleeper,
{
    let registration_outcome =
        execute_native_window_host_loop_timer_registration_wait_with_registrar(
            instruction,
            adapter,
        )
        .map_err(NativeWindowHostLoopTimerWakeError::RegistrationFailed)?;
    execute_native_window_host_loop_timer_fire_wait_with_waiter(registration_outcome, adapter)
        .map_err(NativeWindowHostLoopTimerWakeError::FireFailed)
}

pub fn execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter<Clock, Sleeper>(
    instruction: NativeWindowHostLoopWaitInstruction,
    adapter: &mut NativeWindowHostLoopDeadlineTimerAdapter<Clock, Sleeper>,
) -> Result<
    NativeWindowHostLoopWaitOutcome,
    NativeWindowHostLoopDeadlineTimerWakeError<Clock::Error, Sleeper::Error>,
>
where
    Clock: NativeWindowHostLoopDeadlineTimerClock,
    Sleeper: NativeWindowHostLoopDeadlineTimerSleeper,
{
    execute_native_window_host_loop_deadline_timer_wakeup_with_adapter(instruction, adapter)
        .map(native_window_host_loop_wait_outcome_from_timer_fire)
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StdNativeWindowHostLoopDeadlineTimerError {
    ElapsedNanosOverflow,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StdNativeWindowHostLoopDeadlineTimerClock {
    origin: std::time::Instant,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StdNativeWindowHostLoopDeadlineTimerSleeper {
    origin: std::time::Instant,
}

#[cfg(not(target_arch = "wasm32"))]
fn std_native_window_host_loop_elapsed_nanos(
    origin: std::time::Instant,
) -> Result<u64, StdNativeWindowHostLoopDeadlineTimerError> {
    u64::try_from(origin.elapsed().as_nanos())
        .map_err(|_| StdNativeWindowHostLoopDeadlineTimerError::ElapsedNanosOverflow)
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeWindowHostLoopDeadlineTimerClock for StdNativeWindowHostLoopDeadlineTimerClock {
    type Error = StdNativeWindowHostLoopDeadlineTimerError;

    fn now_nanos(&mut self) -> Result<u64, Self::Error> {
        std_native_window_host_loop_elapsed_nanos(self.origin)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeWindowHostLoopDeadlineTimerSleeper for StdNativeWindowHostLoopDeadlineTimerSleeper {
    type Error = StdNativeWindowHostLoopDeadlineTimerError;

    fn sleep_until_nanos(&mut self, deadline_nanos: u64) -> Result<(), Self::Error> {
        let now_nanos = std_native_window_host_loop_elapsed_nanos(self.origin)?;
        if deadline_nanos > now_nanos {
            std::thread::sleep(std::time::Duration::from_nanos(deadline_nanos - now_nanos));
        }
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn native_window_host_loop_std_deadline_timer_adapter(
) -> NativeWindowHostLoopDeadlineTimerAdapter<
    StdNativeWindowHostLoopDeadlineTimerClock,
    StdNativeWindowHostLoopDeadlineTimerSleeper,
> {
    let origin = std::time::Instant::now();
    NativeWindowHostLoopDeadlineTimerAdapter::new(
        StdNativeWindowHostLoopDeadlineTimerClock { origin },
        StdNativeWindowHostLoopDeadlineTimerSleeper { origin },
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopInterruptibleDeadlineWake {
    HostEventReady,
    DeadlineReached,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError<ClockError, WaiterError> {
    HostEventWaitFailed(WaiterError),
    FrameIntervalWaitNanosMismatch {
        wait_nanos: u32,
        nanos_per_frame: u32,
    },
    TimerRegistrationIdOverflow {
        last_raw_id: u32,
    },
    DeadlineNanosOverflow {
        now_nanos: u64,
        wait_nanos: u32,
    },
    ClockFailed(ClockError),
    FrameIntervalWaitFailed(WaiterError),
}

pub trait NativeWindowHostLoopInterruptibleDeadlineWaiter {
    type Error;

    fn wait_for_host_event(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<(), Self::Error>;

    fn wait_until_deadline_or_host_event(
        &mut self,
        deadline_nanos: u64,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<NativeWindowHostLoopInterruptibleDeadlineWake, Self::Error>;
}

pub struct NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock, Waiter> {
    next_raw_id: u32,
    clock: Clock,
    waiter: Waiter,
}

impl<Clock, Waiter> NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock, Waiter> {
    pub fn new(clock: Clock, waiter: Waiter) -> Self {
        Self {
            next_raw_id: 1,
            clock,
            waiter,
        }
    }

    pub fn next_raw_id(&self) -> u32 {
        self.next_raw_id
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn clock_mut(&mut self) -> &mut Clock {
        &mut self.clock
    }

    pub fn waiter(&self) -> &Waiter {
        &self.waiter
    }

    pub fn waiter_mut(&mut self) -> &mut Waiter {
        &mut self.waiter
    }

    pub fn into_parts(self) -> (Clock, Waiter) {
        (self.clock, self.waiter)
    }
}

pub fn execute_native_window_host_loop_interruptible_deadline_wait_with_adapter<Clock, Waiter>(
    instruction: NativeWindowHostLoopWaitInstruction,
    adapter: &mut NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock, Waiter>,
) -> Result<
    NativeWindowHostLoopWaitOutcome,
    NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError<Clock::Error, Waiter::Error>,
>
where
    Clock: NativeWindowHostLoopDeadlineTimerClock,
    Waiter: NativeWindowHostLoopInterruptibleDeadlineWaiter,
{
    match instruction {
        NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed,
        } => {
            adapter
                .waiter
                .wait_for_host_event(window_size, size_changed)
                .map_err(
                    NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::HostEventWaitFailed,
                )?;
            Ok(NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed,
            })
        }
        NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
            frame_interval,
            wait_nanos,
        } => {
            let nanos_per_frame = frame_interval.nanos_per_frame();
            if wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame + 1 {
                return Err(
                    NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitNanosMismatch {
                        wait_nanos,
                        nanos_per_frame,
                    },
                );
            }
            let raw_id = adapter.next_raw_id;
            let next_raw_id = raw_id.checked_add(1).ok_or(
                NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::TimerRegistrationIdOverflow {
                    last_raw_id: raw_id,
                },
            )?;
            let now_nanos = adapter
                .clock
                .now_nanos()
                .map_err(NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::ClockFailed)?;
            let deadline_nanos = now_nanos.checked_add(u64::from(wait_nanos)).ok_or(
                NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::DeadlineNanosOverflow {
                    now_nanos,
                    wait_nanos,
                },
            )?;
            adapter.next_raw_id = next_raw_id;
            match adapter
                .waiter
                .wait_until_deadline_or_host_event(deadline_nanos, window_size, size_changed)
                .map_err(
                    NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitFailed,
                )?
            {
                NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady => {
                    Ok(NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                        window_size,
                        size_changed,
                    })
                }
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached => {
                    Ok(NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                        presentation,
                        window_size,
                        size_changed,
                        wait_nanos,
                        timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id },
                    })
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError<BackendError> {
    HostEventWaitFailed(BackendError),
    FrameIntervalWaitNanosMismatch {
        wait_nanos: u32,
        nanos_per_frame: u32,
    },
    TimerRegistrationIdOverflow {
        last_raw_id: u32,
    },
    DeadlineNanosOverflow {
        now_nanos: u64,
        wait_nanos: u32,
    },
    ClockFailed(BackendError),
    FrameIntervalWaitFailed(BackendError),
}

pub struct NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend> {
    next_raw_id: u32,
    backend: Backend,
}

impl<Backend> NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend> {
    pub fn new(backend: Backend) -> Self {
        Self {
            next_raw_id: 1,
            backend,
        }
    }

    pub fn next_raw_id(&self) -> u32 {
        self.next_raw_id
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut Backend {
        &mut self.backend
    }

    pub fn into_backend(self) -> Backend {
        self.backend
    }
}

pub fn execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter<
    Backend,
>(
    instruction: NativeWindowHostLoopWaitInstruction,
    adapter: &mut NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend>,
) -> Result<
    NativeWindowHostLoopWaitOutcome,
    NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError<
        <Backend as NativeWindowHostLoopDeadlineTimerClock>::Error,
    >,
>
where
    Backend: NativeWindowHostLoopDeadlineTimerClock
        + NativeWindowHostLoopInterruptibleDeadlineWaiter<
            Error = <Backend as NativeWindowHostLoopDeadlineTimerClock>::Error,
        >,
{
    match instruction {
        NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed,
        } => {
            adapter
                .backend
                .wait_for_host_event(window_size, size_changed)
                .map_err(
                    NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::HostEventWaitFailed,
                )?;
            Ok(NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed,
            })
        }
        NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
            frame_interval,
            wait_nanos,
        } => {
            let nanos_per_frame = frame_interval.nanos_per_frame();
            if wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame + 1 {
                return Err(
                    NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitNanosMismatch {
                        wait_nanos,
                        nanos_per_frame,
                    },
                );
            }
            let raw_id = adapter.next_raw_id;
            let next_raw_id = raw_id.checked_add(1).ok_or(
                NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::TimerRegistrationIdOverflow {
                    last_raw_id: raw_id,
                },
            )?;
            let now_nanos = adapter.backend.now_nanos().map_err(
                NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::ClockFailed,
            )?;
            let deadline_nanos = now_nanos.checked_add(u64::from(wait_nanos)).ok_or(
                NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::DeadlineNanosOverflow {
                    now_nanos,
                    wait_nanos,
                },
            )?;
            adapter.next_raw_id = next_raw_id;
            match adapter
                .backend
                .wait_until_deadline_or_host_event(deadline_nanos, window_size, size_changed)
                .map_err(
                    NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitFailed,
                )?
            {
                NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady => {
                    Ok(NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                        window_size,
                        size_changed,
                    })
                }
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached => {
                    Ok(NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                        presentation,
                        window_size,
                        size_changed,
                        wait_nanos,
                        timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id },
                    })
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopEventQueueWaitError<WaiterError> {
    FrameIntervalEventQueueWaitUnsupported {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        frame_interval: NativeWindowFrameIntervalRequest,
        wait_nanos: u32,
    },
    WaiterFailed(WaiterError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopEventQueueWaitOutcome {
    HostEventReady {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
}

pub trait NativeWindowHostLoopEventQueueWaiter {
    type Error;

    fn wait_for_host_event(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<(), Self::Error>;
}

pub fn execute_native_window_host_loop_event_queue_wait_with_waiter<Waiter>(
    instruction: NativeWindowHostLoopWaitInstruction,
    waiter: &mut Waiter,
) -> Result<
    NativeWindowHostLoopEventQueueWaitOutcome,
    NativeWindowHostLoopEventQueueWaitError<Waiter::Error>,
>
where
    Waiter: NativeWindowHostLoopEventQueueWaiter,
{
    match instruction {
        NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed,
        } => {
            waiter
                .wait_for_host_event(window_size, size_changed)
                .map_err(NativeWindowHostLoopEventQueueWaitError::WaiterFailed)?;
            Ok(NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady {
                window_size,
                size_changed,
            })
        }
        NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
            frame_interval,
            wait_nanos,
        } => Err(
            NativeWindowHostLoopEventQueueWaitError::FrameIntervalEventQueueWaitUnsupported {
                presentation,
                window_size,
                size_changed,
                frame_interval,
                wait_nanos,
            },
        ),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopWaitOwnerError<EventQueueError, TimerClockError, TimerSleeperError> {
    EventQueueWaitFailed(NativeWindowHostLoopEventQueueWaitError<EventQueueError>),
    FrameIntervalAuthorityFailed(NativeWindowFrameIntervalWaitAuthorityModeError),
    FrameIntervalTimerWakeFailed(
        NativeWindowHostLoopDeadlineTimerWakeError<TimerClockError, TimerSleeperError>,
    ),
}

pub struct NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper> {
    event_queue_waiter: EventQueueWaiter,
    frame_interval_timer: NativeWindowHostLoopDeadlineTimerAdapter<TimerClock, TimerSleeper>,
}

impl<EventQueueWaiter, TimerClock, TimerSleeper>
    NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper>
{
    pub fn new(
        event_queue_waiter: EventQueueWaiter,
        frame_interval_timer: NativeWindowHostLoopDeadlineTimerAdapter<TimerClock, TimerSleeper>,
    ) -> Self {
        Self {
            event_queue_waiter,
            frame_interval_timer,
        }
    }

    pub fn event_queue_waiter(&self) -> &EventQueueWaiter {
        &self.event_queue_waiter
    }

    pub fn event_queue_waiter_mut(&mut self) -> &mut EventQueueWaiter {
        &mut self.event_queue_waiter
    }

    pub fn frame_interval_timer(
        &self,
    ) -> &NativeWindowHostLoopDeadlineTimerAdapter<TimerClock, TimerSleeper> {
        &self.frame_interval_timer
    }

    pub fn frame_interval_wait_authority_mode(&self) -> NativeWindowFrameIntervalWaitAuthorityMode {
        native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer()
    }

    pub fn frame_interval_timer_mut(
        &mut self,
    ) -> &mut NativeWindowHostLoopDeadlineTimerAdapter<TimerClock, TimerSleeper> {
        &mut self.frame_interval_timer
    }

    pub fn into_parts(
        self,
    ) -> (
        EventQueueWaiter,
        NativeWindowHostLoopDeadlineTimerAdapter<TimerClock, TimerSleeper>,
    ) {
        (self.event_queue_waiter, self.frame_interval_timer)
    }
}

pub fn execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode<
    EventQueueWaiter,
    TimerClock,
    TimerSleeper,
>(
    instruction: NativeWindowHostLoopWaitInstruction,
    owner: &mut NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper>,
    requested_authority_mode: NativeWindowFrameIntervalWaitAuthorityMode,
) -> Result<
    NativeWindowHostLoopWaitOutcome,
    NativeWindowHostLoopWaitOwnerError<
        EventQueueWaiter::Error,
        TimerClock::Error,
        TimerSleeper::Error,
    >,
>
where
    EventQueueWaiter: NativeWindowHostLoopEventQueueWaiter,
    TimerClock: NativeWindowHostLoopDeadlineTimerClock,
    TimerSleeper: NativeWindowHostLoopDeadlineTimerSleeper,
{
    match instruction {
        NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed,
        } => {
            let event_queue_outcome = execute_native_window_host_loop_event_queue_wait_with_waiter(
                NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
                    window_size,
                    size_changed,
                },
                owner.event_queue_waiter_mut(),
            )
            .map_err(NativeWindowHostLoopWaitOwnerError::EventQueueWaitFailed)?;
            match event_queue_outcome {
                NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady {
                    window_size,
                    size_changed,
                } => Ok(NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                    window_size,
                    size_changed,
                }),
            }
        }
        NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
            frame_interval,
            wait_nanos,
        } => {
            let active_authority_mode = owner.frame_interval_wait_authority_mode();
            let authority_mode = combine_native_window_frame_interval_wait_authority_mode(
                active_authority_mode,
                requested_authority_mode,
            )
            .map_err(NativeWindowHostLoopWaitOwnerError::FrameIntervalAuthorityFailed)?;
            validate_native_window_frame_interval_wait_authority_mode(
                authority_mode,
                frame_interval,
            )
            .map_err(NativeWindowHostLoopWaitOwnerError::FrameIntervalAuthorityFailed)?;
            execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter(
                NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                    presentation,
                    window_size,
                    size_changed,
                    frame_interval,
                    wait_nanos,
                },
                owner.frame_interval_timer_mut(),
            )
            .map_err(NativeWindowHostLoopWaitOwnerError::FrameIntervalTimerWakeFailed)
        }
    }
}

pub fn execute_native_window_host_loop_wait_with_owner<EventQueueWaiter, TimerClock, TimerSleeper>(
    instruction: NativeWindowHostLoopWaitInstruction,
    owner: &mut NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper>,
) -> Result<
    NativeWindowHostLoopWaitOutcome,
    NativeWindowHostLoopWaitOwnerError<
        EventQueueWaiter::Error,
        TimerClock::Error,
        TimerSleeper::Error,
    >,
>
where
    EventQueueWaiter: NativeWindowHostLoopEventQueueWaiter,
    TimerClock: NativeWindowHostLoopDeadlineTimerClock,
    TimerSleeper: NativeWindowHostLoopDeadlineTimerSleeper,
{
    let authority_mode = owner.frame_interval_wait_authority_mode();
    execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode(
        instruction,
        owner,
        authority_mode,
    )
}

pub struct NativeWindowHostOwnedDeadlineWaitRunLoopHost<
    Host,
    EventQueueWaiter,
    TimerClock,
    TimerSleeper,
> {
    host: Host,
    wait_owner: NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper>,
}

impl<Host, EventQueueWaiter, TimerClock, TimerSleeper>
    NativeWindowHostOwnedDeadlineWaitRunLoopHost<Host, EventQueueWaiter, TimerClock, TimerSleeper>
{
    pub fn new(
        host: Host,
        wait_owner: NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper>,
    ) -> Self {
        Self { host, wait_owner }
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut Host {
        &mut self.host
    }

    pub fn wait_owner(
        &self,
    ) -> &NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper> {
        &self.wait_owner
    }

    pub fn wait_owner_mut(
        &mut self,
    ) -> &mut NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper> {
        &mut self.wait_owner
    }

    pub fn into_parts(
        self,
    ) -> (
        Host,
        NativeWindowHostLoopWaitOwner<EventQueueWaiter, TimerClock, TimerSleeper>,
    ) {
        (self.host, self.wait_owner)
    }
}

impl<Host, EventQueueWaiter, TimerClock, TimerSleeper> NativeWindowRunLoopHost
    for NativeWindowHostOwnedDeadlineWaitRunLoopHost<
        Host,
        EventQueueWaiter,
        TimerClock,
        TimerSleeper,
    >
where
    Host: NativeWindowRunLoopHost,
    EventQueueWaiter: NativeWindowHostLoopEventQueueWaiter,
    TimerClock: NativeWindowHostLoopDeadlineTimerClock,
    TimerSleeper: NativeWindowHostLoopDeadlineTimerSleeper,
{
    type EventError = Host::EventError;
    type PresentError = Host::PresentError;
    type WaitError = NativeWindowHostLoopWaitOwnerError<
        EventQueueWaiter::Error,
        TimerClock::Error,
        TimerSleeper::Error,
    >;

    fn poll_event_snapshot(
        &mut self,
        input: NativeWindowEventPumpInput,
    ) -> Result<NativeWindowEventPumpSnapshot, Self::EventError> {
        self.host.poll_event_snapshot(input)
    }

    fn set_window_title(&mut self, title: &str) {
        self.host.set_window_title(title);
    }

    fn pump_events_only(&mut self) {
        self.host.pump_events_only();
    }

    fn present_frame(&mut self, frame: NativePresenterFrame<'_>) -> Result<(), Self::PresentError> {
        self.host.present_frame(frame)
    }

    fn wait_after_budget_exhausted(
        &mut self,
        instruction: NativeWindowHostLoopWaitInstruction,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
        execute_native_window_host_loop_wait_with_owner(instruction, &mut self.wait_owner)
    }
}

pub struct NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost<Host, Clock, Waiter> {
    host: Host,
    wait_adapter: NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock, Waiter>,
}

impl<Host, Clock, Waiter>
    NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost<Host, Clock, Waiter>
{
    pub fn new(
        host: Host,
        wait_adapter: NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock, Waiter>,
    ) -> Self {
        Self { host, wait_adapter }
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut Host {
        &mut self.host
    }

    pub fn wait_adapter(
        &self,
    ) -> &NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock, Waiter> {
        &self.wait_adapter
    }

    pub fn wait_adapter_mut(
        &mut self,
    ) -> &mut NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock, Waiter> {
        &mut self.wait_adapter
    }

    pub fn into_parts(
        self,
    ) -> (
        Host,
        NativeWindowHostLoopInterruptibleDeadlineWaitAdapter<Clock, Waiter>,
    ) {
        (self.host, self.wait_adapter)
    }
}

impl<Host, Clock, Waiter> NativeWindowRunLoopHost
    for NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost<Host, Clock, Waiter>
where
    Host: NativeWindowRunLoopHost,
    Clock: NativeWindowHostLoopDeadlineTimerClock,
    Waiter: NativeWindowHostLoopInterruptibleDeadlineWaiter,
{
    type EventError = Host::EventError;
    type PresentError = Host::PresentError;
    type WaitError =
        NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError<Clock::Error, Waiter::Error>;

    fn poll_event_snapshot(
        &mut self,
        input: NativeWindowEventPumpInput,
    ) -> Result<NativeWindowEventPumpSnapshot, Self::EventError> {
        self.host.poll_event_snapshot(input)
    }

    fn set_window_title(&mut self, title: &str) {
        self.host.set_window_title(title);
    }

    fn pump_events_only(&mut self) {
        self.host.pump_events_only();
    }

    fn present_frame(&mut self, frame: NativePresenterFrame<'_>) -> Result<(), Self::PresentError> {
        self.host.present_frame(frame)
    }

    fn wait_after_budget_exhausted(
        &mut self,
        instruction: NativeWindowHostLoopWaitInstruction,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
        execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
            instruction,
            &mut self.wait_adapter,
        )
    }
}

pub struct NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost<Host, Backend> {
    host: Host,
    wait_adapter: NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend>,
}

impl<Host, Backend>
    NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost<Host, Backend>
{
    pub fn new(
        host: Host,
        wait_adapter: NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend>,
    ) -> Self {
        Self { host, wait_adapter }
    }

    pub fn host(&self) -> &Host {
        &self.host
    }

    pub fn host_mut(&mut self) -> &mut Host {
        &mut self.host
    }

    pub fn wait_adapter(
        &self,
    ) -> &NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend> {
        &self.wait_adapter
    }

    pub fn wait_adapter_mut(
        &mut self,
    ) -> &mut NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend> {
        &mut self.wait_adapter
    }

    pub fn into_parts(
        self,
    ) -> (
        Host,
        NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter<Backend>,
    ) {
        (self.host, self.wait_adapter)
    }
}

impl<Host, Backend> NativeWindowRunLoopHost
    for NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost<Host, Backend>
where
    Host: NativeWindowRunLoopHost,
    Backend: NativeWindowHostLoopDeadlineTimerClock
        + NativeWindowHostLoopInterruptibleDeadlineWaiter<
            Error = <Backend as NativeWindowHostLoopDeadlineTimerClock>::Error,
        >,
{
    type EventError = Host::EventError;
    type PresentError = Host::PresentError;
    type WaitError = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError<
        <Backend as NativeWindowHostLoopDeadlineTimerClock>::Error,
    >;

    fn poll_event_snapshot(
        &mut self,
        input: NativeWindowEventPumpInput,
    ) -> Result<NativeWindowEventPumpSnapshot, Self::EventError> {
        self.host.poll_event_snapshot(input)
    }

    fn set_window_title(&mut self, title: &str) {
        self.host.set_window_title(title);
    }

    fn pump_events_only(&mut self) {
        self.host.pump_events_only();
    }

    fn present_frame(&mut self, frame: NativePresenterFrame<'_>) -> Result<(), Self::PresentError> {
        self.host.present_frame(frame)
    }

    fn wait_after_budget_exhausted(
        &mut self,
        instruction: NativeWindowHostLoopWaitInstruction,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
        execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
            instruction,
            &mut self.wait_adapter,
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopPlatformKind {
    Macos,
    Windows,
    Linux,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopPlatformWaitBackendKind {
    MacosRunLoopTimer,
    WindowsWaitableTimerMessageWait,
    LinuxSelectorTimerFd,
    HeadlessScripted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopPlatformWaitBackendSupportError {
    DefaultBackendUnsupportedPlatform {
        current: NativeWindowHostLoopPlatformKind,
    },
    RequestedBackendUnsupportedPlatform {
        current: NativeWindowHostLoopPlatformKind,
        requested: NativeWindowHostLoopPlatformWaitBackendKind,
    },
    BackendPlatformMismatch {
        current: NativeWindowHostLoopPlatformKind,
        requested: NativeWindowHostLoopPlatformWaitBackendKind,
    },
}

pub fn native_window_host_loop_current_platform_kind() -> NativeWindowHostLoopPlatformKind {
    #[cfg(target_os = "macos")]
    {
        NativeWindowHostLoopPlatformKind::Macos
    }
    #[cfg(target_os = "windows")]
    {
        NativeWindowHostLoopPlatformKind::Windows
    }
    #[cfg(target_os = "linux")]
    {
        NativeWindowHostLoopPlatformKind::Linux
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        NativeWindowHostLoopPlatformKind::Unsupported
    }
}

pub fn validate_native_window_host_loop_platform_wait_backend_kind_for_platform(
    current: NativeWindowHostLoopPlatformKind,
    requested: NativeWindowHostLoopPlatformWaitBackendKind,
) -> Result<
    NativeWindowHostLoopPlatformWaitBackendKind,
    NativeWindowHostLoopPlatformWaitBackendSupportError,
> {
    match (current, requested) {
        (NativeWindowHostLoopPlatformKind::Unsupported, requested) => {
            Err(NativeWindowHostLoopPlatformWaitBackendSupportError::RequestedBackendUnsupportedPlatform {
                current,
                requested,
            })
        }
        (
            NativeWindowHostLoopPlatformKind::Macos,
            NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
        )
        | (
            NativeWindowHostLoopPlatformKind::Windows,
            NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
        )
        | (
            NativeWindowHostLoopPlatformKind::Linux,
            NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
        ) => Ok(requested),
        (current, requested) => {
            Err(NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                current,
                requested,
            })
        }
    }
}

pub fn native_window_host_loop_default_platform_wait_backend_kind_for_platform(
    current: NativeWindowHostLoopPlatformKind,
) -> Result<
    NativeWindowHostLoopPlatformWaitBackendKind,
    NativeWindowHostLoopPlatformWaitBackendSupportError,
> {
    match current {
        NativeWindowHostLoopPlatformKind::Macos => Ok(
            NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
        ),
        NativeWindowHostLoopPlatformKind::Windows => Ok(
            NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
        ),
        NativeWindowHostLoopPlatformKind::Linux => Ok(
            NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
        ),
        NativeWindowHostLoopPlatformKind::Unsupported => {
            Err(NativeWindowHostLoopPlatformWaitBackendSupportError::DefaultBackendUnsupportedPlatform {
                current,
            })
        }
    }
}

pub fn native_window_host_loop_default_platform_wait_backend_kind() -> Result<
    NativeWindowHostLoopPlatformWaitBackendKind,
    NativeWindowHostLoopPlatformWaitBackendSupportError,
> {
    native_window_host_loop_default_platform_wait_backend_kind_for_platform(
        native_window_host_loop_current_platform_kind(),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopPlatformWaitBackendSelection {
    platform: NativeWindowHostLoopPlatformKind,
    backend: NativeWindowHostLoopPlatformWaitBackendKind,
}

impl NativeWindowHostLoopPlatformWaitBackendSelection {
    pub fn platform(&self) -> NativeWindowHostLoopPlatformKind {
        self.platform
    }

    pub fn backend(&self) -> NativeWindowHostLoopPlatformWaitBackendKind {
        self.backend
    }
}

pub fn validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
    platform: NativeWindowHostLoopPlatformKind,
    requested: NativeWindowHostLoopPlatformWaitBackendKind,
) -> Result<
    NativeWindowHostLoopPlatformWaitBackendSelection,
    NativeWindowHostLoopPlatformWaitBackendSupportError,
> {
    let backend = validate_native_window_host_loop_platform_wait_backend_kind_for_platform(
        platform, requested,
    )?;
    Ok(NativeWindowHostLoopPlatformWaitBackendSelection { platform, backend })
}

pub fn native_window_host_loop_default_platform_wait_backend_selection_for_platform(
    platform: NativeWindowHostLoopPlatformKind,
) -> Result<
    NativeWindowHostLoopPlatformWaitBackendSelection,
    NativeWindowHostLoopPlatformWaitBackendSupportError,
> {
    let requested =
        native_window_host_loop_default_platform_wait_backend_kind_for_platform(platform)?;
    validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
        platform, requested,
    )
}

pub fn native_window_host_loop_default_platform_wait_backend_selection() -> Result<
    NativeWindowHostLoopPlatformWaitBackendSelection,
    NativeWindowHostLoopPlatformWaitBackendSupportError,
> {
    native_window_host_loop_default_platform_wait_backend_selection_for_platform(
        native_window_host_loop_current_platform_kind(),
    )
}

pub fn native_window_run_loop_platform_wait_backend_selection(
    config: NativeWindowRunLoopConfig,
) -> Result<
    NativeWindowHostLoopPlatformWaitBackendSelection,
    NativeWindowRunLoopPlatformWaitBackendConfigError,
> {
    match config.wait_backend {
        NativeWindowRunLoopWaitBackend::PlatformWait(selection) => Ok(selection),
        requested => Err(
            NativeWindowRunLoopPlatformWaitBackendConfigError::NotPlatformWaitBackend { requested },
        ),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopPlatformWaitHostBuildError {
    BackendSupportFailed(NativeWindowHostLoopPlatformWaitBackendSupportError),
    BackendImplementationUnavailable {
        platform: NativeWindowHostLoopPlatformKind,
        backend: NativeWindowHostLoopPlatformWaitBackendKind,
    },
    WindowsWaitBackendFailed(NativeWindowHostLoopWindowsWaitBackendError),
    MacosRunLoopTimerBackendFailed(NativeWindowHostLoopMacosRunLoopTimerBackendError),
    LinuxSelectorTimerFdBackendFailed(NativeWindowHostLoopLinuxSelectorTimerFdBackendError),
}

pub fn build_native_window_host_loop_platform_wait_backend_from_selection(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
) -> Result<
    NativeWindowHostLoopPlatformWaitBackendSelection,
    NativeWindowHostLoopPlatformWaitHostBuildError,
> {
    Err(
        NativeWindowHostLoopPlatformWaitHostBuildError::BackendImplementationUnavailable {
            platform: selection.platform(),
            backend: selection.backend(),
        },
    )
}

pub fn build_native_window_host_loop_platform_wait_backend_for_platform(
    platform: NativeWindowHostLoopPlatformKind,
    requested: NativeWindowHostLoopPlatformWaitBackendKind,
) -> Result<
    NativeWindowHostLoopPlatformWaitBackendSelection,
    NativeWindowHostLoopPlatformWaitHostBuildError,
> {
    let selection = validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
        platform, requested,
    )
    .map_err(NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed)?;
    build_native_window_host_loop_platform_wait_backend_from_selection(selection)
}

#[derive(Debug)]
pub enum NativeWindowHostLoopPlatformWaitBackend<WindowsApi, MacosApi, LinuxApi>
where
    WindowsApi: NativeWindowHostLoopWindowsWaitRawApi,
    MacosApi: NativeWindowHostLoopMacosRunLoopTimerRawApi,
    LinuxApi: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    WindowsWaitableTimerMessageWait(NativeWindowHostLoopWindowsWaitBackend<WindowsApi>),
    MacosRunLoopTimer(NativeWindowHostLoopMacosRunLoopTimerBackend<MacosApi>),
    LinuxSelectorTimerFd(NativeWindowHostLoopLinuxSelectorTimerFdBackend<LinuxApi>),
}

pub type NativeWindowHostLoopWindowsOnlyPlatformWaitBackend<WindowsApi> =
    NativeWindowHostLoopPlatformWaitBackend<
        WindowsApi,
        NativeWindowHostLoopNeverMacosRunLoopTimerRawApi,
        NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi,
    >;

pub type NativeWindowHostLoopLinuxOnlyPlatformWaitBackend<LinuxApi> =
    NativeWindowHostLoopPlatformWaitBackend<
        NativeWindowHostLoopNeverWindowsWaitRawApi,
        NativeWindowHostLoopNeverMacosRunLoopTimerRawApi,
        LinuxApi,
    >;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopPlatformWaitBackendError<WindowsError, MacosError, LinuxError> {
    WindowsWaitableTimerMessageWait(WindowsError),
    MacosRunLoopTimer(MacosError),
    LinuxSelectorTimerFd(LinuxError),
}

#[derive(Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopMacosRunLoopTimerHandle {
    raw_handle: isize,
}

#[cfg(test)]
fn native_window_host_loop_macos_run_loop_timer_handle_raw(
    handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
) -> isize {
    handle.raw_handle
}

pub const NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED: u32 = 1;
pub const NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_HOST_EVENT_READY: u32 = 2;
pub const NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_FAILED: u32 = 0xFFFF_FFFF;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopMacosRunLoopDeadlinePlan {
    AlreadyReached,
    RelativeNanos(u64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopMacosRunLoopWake {
    TimerFired,
    HostEventReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopMacosRunLoopTimerBackendError {
    InvalidRawHandle { raw_handle: isize },
    CreateRunLoopTimerFailed { code: u32 },
    ScheduleRunLoopTimerFailed { code: u32 },
    RunLoopWaitFailed { code: u32 },
    UnexpectedRunLoopStatus { status: u32 },
    ElapsedNanosOverflow,
    DeadlineDeltaOverflow { now_nanos: u64, deadline_nanos: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopMacosRunLoopTimerBackendBuildError {
    BackendSupportFailed(NativeWindowHostLoopPlatformWaitBackendSupportError),
    RunLoopTimerBackendFailed(NativeWindowHostLoopMacosRunLoopTimerBackendError),
}

pub trait NativeWindowHostLoopMacosRunLoopTimerRawApi {
    fn create_run_loop_timer_raw(&mut self) -> isize;

    fn schedule_run_loop_timer_relative_nanos(
        &mut self,
        handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
        relative_nanos: u64,
    ) -> bool;

    fn run_loop_wait_for_timer_or_event_raw(
        &mut self,
        handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
    ) -> u32;

    fn run_loop_wait_for_event_raw(&mut self) -> u32;

    fn invalidate_run_loop_timer_raw(
        &mut self,
        handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
    ) -> bool;

    fn last_error_code(&mut self) -> u32;
}

pub fn native_window_host_loop_macos_run_loop_timer_handle_from_raw(
    raw_handle: isize,
) -> Result<
    NativeWindowHostLoopMacosRunLoopTimerHandle,
    NativeWindowHostLoopMacosRunLoopTimerBackendError,
> {
    if raw_handle == 0 || raw_handle == -1 {
        return Err(
            NativeWindowHostLoopMacosRunLoopTimerBackendError::InvalidRawHandle { raw_handle },
        );
    }
    Ok(NativeWindowHostLoopMacosRunLoopTimerHandle { raw_handle })
}

pub fn native_window_host_loop_macos_run_loop_deadline_plan(
    now_nanos: u64,
    deadline_nanos: u64,
) -> Result<
    NativeWindowHostLoopMacosRunLoopDeadlinePlan,
    NativeWindowHostLoopMacosRunLoopTimerBackendError,
> {
    if deadline_nanos <= now_nanos {
        return Ok(NativeWindowHostLoopMacosRunLoopDeadlinePlan::AlreadyReached);
    }
    let relative_nanos = deadline_nanos.checked_sub(now_nanos).ok_or(
        NativeWindowHostLoopMacosRunLoopTimerBackendError::DeadlineDeltaOverflow {
            now_nanos,
            deadline_nanos,
        },
    )?;
    Ok(NativeWindowHostLoopMacosRunLoopDeadlinePlan::RelativeNanos(
        relative_nanos,
    ))
}

pub fn native_window_host_loop_macos_run_loop_wake_from_timer_or_event_status(
    status: u32,
    last_error_code: u32,
) -> Result<NativeWindowHostLoopMacosRunLoopWake, NativeWindowHostLoopMacosRunLoopTimerBackendError>
{
    match status {
        NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED => {
            Ok(NativeWindowHostLoopMacosRunLoopWake::TimerFired)
        }
        NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_HOST_EVENT_READY => {
            Ok(NativeWindowHostLoopMacosRunLoopWake::HostEventReady)
        }
        NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_FAILED => Err(
            NativeWindowHostLoopMacosRunLoopTimerBackendError::RunLoopWaitFailed {
                code: last_error_code,
            },
        ),
        status => Err(
            NativeWindowHostLoopMacosRunLoopTimerBackendError::UnexpectedRunLoopStatus { status },
        ),
    }
}

pub fn native_window_host_loop_macos_run_loop_host_event_from_status(
    status: u32,
    last_error_code: u32,
) -> Result<(), NativeWindowHostLoopMacosRunLoopTimerBackendError> {
    match status {
        NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_HOST_EVENT_READY => Ok(()),
        NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_FAILED => Err(
            NativeWindowHostLoopMacosRunLoopTimerBackendError::RunLoopWaitFailed {
                code: last_error_code,
            },
        ),
        status => Err(
            NativeWindowHostLoopMacosRunLoopTimerBackendError::UnexpectedRunLoopStatus { status },
        ),
    }
}

#[derive(Debug)]
pub struct NativeWindowHostLoopMacosRunLoopTimerBackend<
    Api: NativeWindowHostLoopMacosRunLoopTimerRawApi,
> {
    origin: std::time::Instant,
    api: Api,
    handle: Option<NativeWindowHostLoopMacosRunLoopTimerHandle>,
}

impl<Api> NativeWindowHostLoopMacosRunLoopTimerBackend<Api>
where
    Api: NativeWindowHostLoopMacosRunLoopTimerRawApi,
{
    pub fn new(mut api: Api) -> Result<Self, NativeWindowHostLoopMacosRunLoopTimerBackendError> {
        let raw_handle = api.create_run_loop_timer_raw();
        let handle = match native_window_host_loop_macos_run_loop_timer_handle_from_raw(raw_handle)
        {
            Ok(handle) => handle,
            Err(NativeWindowHostLoopMacosRunLoopTimerBackendError::InvalidRawHandle { .. }) => {
                return Err(
                    NativeWindowHostLoopMacosRunLoopTimerBackendError::CreateRunLoopTimerFailed {
                        code: api.last_error_code(),
                    },
                );
            }
            Err(error) => return Err(error),
        };
        Ok(Self {
            origin: std::time::Instant::now(),
            api,
            handle: Some(handle),
        })
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn api_mut(&mut self) -> &mut Api {
        &mut self.api
    }

    pub fn is_handle_open(&self) -> bool {
        self.handle.is_some()
    }

    pub fn invalidate_handle_if_open(&mut self) -> bool {
        let Some(handle) = self.handle.take() else {
            return false;
        };
        let _ = self.api.invalidate_run_loop_timer_raw(&handle);
        true
    }

    fn elapsed_nanos(&self) -> Result<u64, NativeWindowHostLoopMacosRunLoopTimerBackendError> {
        u64::try_from(self.origin.elapsed().as_nanos())
            .map_err(|_| NativeWindowHostLoopMacosRunLoopTimerBackendError::ElapsedNanosOverflow)
    }

    pub fn wait_for_host_event(
        &mut self,
        _window_size: NativeWindowSize,
        _size_changed: bool,
    ) -> Result<(), NativeWindowHostLoopMacosRunLoopTimerBackendError> {
        let status = self.api.run_loop_wait_for_event_raw();
        let last_error_code = if status == NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_FAILED {
            self.api.last_error_code()
        } else {
            0
        };
        native_window_host_loop_macos_run_loop_host_event_from_status(status, last_error_code)
    }

    pub fn wait_until_deadline_or_host_event(
        &mut self,
        deadline_nanos: u64,
        _window_size: NativeWindowSize,
        _size_changed: bool,
    ) -> Result<
        NativeWindowHostLoopMacosRunLoopWake,
        NativeWindowHostLoopMacosRunLoopTimerBackendError,
    > {
        let now_nanos = self.elapsed_nanos()?;
        let plan = native_window_host_loop_macos_run_loop_deadline_plan(now_nanos, deadline_nanos)?;
        let NativeWindowHostLoopMacosRunLoopDeadlinePlan::RelativeNanos(relative_nanos) = plan
        else {
            return Ok(NativeWindowHostLoopMacosRunLoopWake::TimerFired);
        };
        let handle = self.handle.as_ref().ok_or(
            NativeWindowHostLoopMacosRunLoopTimerBackendError::InvalidRawHandle { raw_handle: 0 },
        )?;
        let api = &mut self.api;
        if !api.schedule_run_loop_timer_relative_nanos(handle, relative_nanos) {
            return Err(
                NativeWindowHostLoopMacosRunLoopTimerBackendError::ScheduleRunLoopTimerFailed {
                    code: api.last_error_code(),
                },
            );
        }
        let status = api.run_loop_wait_for_timer_or_event_raw(handle);
        let last_error_code = if status == NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_FAILED {
            api.last_error_code()
        } else {
            0
        };
        native_window_host_loop_macos_run_loop_wake_from_timer_or_event_status(
            status,
            last_error_code,
        )
    }
}

impl<Api> Drop for NativeWindowHostLoopMacosRunLoopTimerBackend<Api>
where
    Api: NativeWindowHostLoopMacosRunLoopTimerRawApi,
{
    fn drop(&mut self) {
        self.invalidate_handle_if_open();
    }
}

impl<Api> NativeWindowHostLoopDeadlineTimerClock
    for NativeWindowHostLoopMacosRunLoopTimerBackend<Api>
where
    Api: NativeWindowHostLoopMacosRunLoopTimerRawApi,
{
    type Error = NativeWindowHostLoopMacosRunLoopTimerBackendError;

    fn now_nanos(&mut self) -> Result<u64, Self::Error> {
        self.elapsed_nanos()
    }
}

impl<Api> NativeWindowHostLoopInterruptibleDeadlineWaiter
    for NativeWindowHostLoopMacosRunLoopTimerBackend<Api>
where
    Api: NativeWindowHostLoopMacosRunLoopTimerRawApi,
{
    type Error = NativeWindowHostLoopMacosRunLoopTimerBackendError;

    fn wait_for_host_event(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<(), Self::Error> {
        NativeWindowHostLoopMacosRunLoopTimerBackend::wait_for_host_event(
            self,
            window_size,
            size_changed,
        )
    }

    fn wait_until_deadline_or_host_event(
        &mut self,
        deadline_nanos: u64,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<NativeWindowHostLoopInterruptibleDeadlineWake, Self::Error> {
        let wake = NativeWindowHostLoopMacosRunLoopTimerBackend::wait_until_deadline_or_host_event(
            self,
            deadline_nanos,
            window_size,
            size_changed,
        )?;
        Ok(match wake {
            NativeWindowHostLoopMacosRunLoopWake::TimerFired => {
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached
            }
            NativeWindowHostLoopMacosRunLoopWake::HostEventReady => {
                NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady
            }
        })
    }
}

pub fn build_native_window_host_loop_macos_run_loop_timer_backend_from_selection<Api>(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
    api: Api,
) -> Result<
    NativeWindowHostLoopMacosRunLoopTimerBackend<Api>,
    NativeWindowHostLoopMacosRunLoopTimerBackendBuildError,
>
where
    Api: NativeWindowHostLoopMacosRunLoopTimerRawApi,
{
    let checked_selection =
        validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
            selection.platform(),
            selection.backend(),
        )
        .map_err(NativeWindowHostLoopMacosRunLoopTimerBackendBuildError::BackendSupportFailed)?;
    if checked_selection.platform() != NativeWindowHostLoopPlatformKind::Macos
        || checked_selection.backend()
            != NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer
    {
        return Err(
            NativeWindowHostLoopMacosRunLoopTimerBackendBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: checked_selection.platform(),
                    requested: checked_selection.backend(),
                },
            ),
        );
    }
    NativeWindowHostLoopMacosRunLoopTimerBackend::new(api)
        .map_err(NativeWindowHostLoopMacosRunLoopTimerBackendBuildError::RunLoopTimerBackendFailed)
}

#[derive(Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopLinuxSelectorFd {
    raw_fd: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopLinuxTimerFd {
    raw_fd: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopLinuxHostEventFd {
    raw_fd: i32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopLinuxHostEventSignalFd {
    raw_fd: i32,
}

#[cfg(any(test, target_os = "linux"))]
fn native_window_host_loop_linux_selector_fd_raw(
    handle: &NativeWindowHostLoopLinuxSelectorFd,
) -> i32 {
    handle.raw_fd
}

#[cfg(any(test, target_os = "linux"))]
fn native_window_host_loop_linux_timer_fd_raw(handle: &NativeWindowHostLoopLinuxTimerFd) -> i32 {
    handle.raw_fd
}

#[cfg(any(test, target_os = "linux"))]
fn native_window_host_loop_linux_host_event_fd_raw(
    handle: &NativeWindowHostLoopLinuxHostEventFd,
) -> i32 {
    handle.raw_fd
}

#[cfg(any(test, target_os = "linux"))]
fn native_window_host_loop_linux_host_event_signal_fd_raw(
    handle: &NativeWindowHostLoopLinuxHostEventSignalFd,
) -> i32 {
    handle.raw_fd
}

pub const NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED: u32 = 1;
pub const NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY: u32 = 2;
pub const NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED: u32 = 0xFFFF_FFFF;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopLinuxTimerFdTimespec {
    seconds: i64,
    nanoseconds: i64,
}

impl NativeWindowHostLoopLinuxTimerFdTimespec {
    pub fn seconds(self) -> i64 {
        self.seconds
    }

    pub fn nanoseconds(self) -> i64 {
        self.nanoseconds
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopLinuxSelectorTimerFdDeadlinePlan {
    AlreadyReached,
    RelativeTimespec(NativeWindowHostLoopLinuxTimerFdTimespec),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopLinuxSelectorTimerFdWake {
    TimerFired,
    HostEventReady,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopLinuxSelectorTimerFdBackendError {
    InvalidSelectorRawFd { raw_fd: i32 },
    InvalidTimerRawFd { raw_fd: i32 },
    InvalidHostEventRawFd { raw_fd: i32 },
    CreateSelectorFailed { code: u32 },
    CreateTimerFdFailed { code: u32 },
    RegisterTimerFdFailed { code: u32 },
    CreateHostEventFdFailed { code: u32 },
    RegisterHostEventFdFailed { code: u32 },
    SignalHostEventFdFailed { code: u32 },
    ArmTimerFdFailed { code: u32 },
    SelectorWaitFailed { code: u32 },
    UnexpectedSelectorStatus { status: u32 },
    ElapsedNanosOverflow,
    DeadlineDeltaOverflow { now_nanos: u64, deadline_nanos: u64 },
    TimespecSecondsOverflow { delta_nanos: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopLinuxHostEventSignalProducerError {
    InvalidHostEventRawFd { raw_fd: i32 },
    InvalidHostEventSignalRawFd { raw_fd: i32 },
    CreateHostEventSignalFdFailed { code: u32 },
    SignalHostEventSignalFdFailed { code: u32 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError {
    BackendSupportFailed(NativeWindowHostLoopPlatformWaitBackendSupportError),
    SelectorTimerFdBackendFailed(NativeWindowHostLoopLinuxSelectorTimerFdBackendError),
}

pub trait NativeWindowHostLoopLinuxSelectorTimerFdRawApi {
    fn create_selector_raw(&mut self) -> i32;

    fn create_timer_fd_raw(&mut self) -> i32;

    fn create_host_event_fd_raw(&mut self) -> i32;

    fn register_timer_fd_raw(
        &mut self,
        selector: &NativeWindowHostLoopLinuxSelectorFd,
        timer: &NativeWindowHostLoopLinuxTimerFd,
    ) -> bool;

    fn register_host_event_fd_raw(
        &mut self,
        selector: &NativeWindowHostLoopLinuxSelectorFd,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool;

    fn signal_host_event_fd_raw(
        &mut self,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool;

    fn arm_timer_fd_relative_timespec(
        &mut self,
        timer: &NativeWindowHostLoopLinuxTimerFd,
        timespec: NativeWindowHostLoopLinuxTimerFdTimespec,
    ) -> bool;

    fn selector_wait_for_timer_or_event_raw(
        &mut self,
        selector: &NativeWindowHostLoopLinuxSelectorFd,
        timer: &NativeWindowHostLoopLinuxTimerFd,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> u32;

    fn selector_wait_for_event_raw(
        &mut self,
        selector: &NativeWindowHostLoopLinuxSelectorFd,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> u32;

    fn close_selector_raw(&mut self, selector: &NativeWindowHostLoopLinuxSelectorFd) -> bool;

    fn close_timer_fd_raw(&mut self, timer: &NativeWindowHostLoopLinuxTimerFd) -> bool;

    fn close_host_event_fd_raw(
        &mut self,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool;

    fn last_error_code(&mut self) -> u32;
}

pub trait NativeWindowHostLoopLinuxHostEventSignalRawApi {
    fn clone_host_event_signal_fd_raw(
        &mut self,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> i32;

    fn signal_host_event_signal_fd_raw(
        &mut self,
        signal: &NativeWindowHostLoopLinuxHostEventSignalFd,
    ) -> bool;

    fn close_host_event_signal_fd_raw(
        &mut self,
        signal: &NativeWindowHostLoopLinuxHostEventSignalFd,
    ) -> bool;

    fn last_error_code(&mut self) -> u32;
}

pub fn native_window_host_loop_linux_selector_fd_from_raw(
    raw_fd: i32,
) -> Result<NativeWindowHostLoopLinuxSelectorFd, NativeWindowHostLoopLinuxSelectorTimerFdBackendError>
{
    if raw_fd < 0 {
        return Err(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidSelectorRawFd { raw_fd },
        );
    }
    Ok(NativeWindowHostLoopLinuxSelectorFd { raw_fd })
}

pub fn native_window_host_loop_linux_timer_fd_from_raw(
    raw_fd: i32,
) -> Result<NativeWindowHostLoopLinuxTimerFd, NativeWindowHostLoopLinuxSelectorTimerFdBackendError>
{
    if raw_fd < 0 {
        return Err(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidTimerRawFd { raw_fd },
        );
    }
    Ok(NativeWindowHostLoopLinuxTimerFd { raw_fd })
}

pub fn native_window_host_loop_linux_host_event_fd_from_raw(
    raw_fd: i32,
) -> Result<
    NativeWindowHostLoopLinuxHostEventFd,
    NativeWindowHostLoopLinuxSelectorTimerFdBackendError,
> {
    if raw_fd < 0 {
        return Err(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidHostEventRawFd { raw_fd },
        );
    }
    Ok(NativeWindowHostLoopLinuxHostEventFd { raw_fd })
}

pub fn native_window_host_loop_linux_host_event_signal_fd_from_raw(
    raw_fd: i32,
) -> Result<
    NativeWindowHostLoopLinuxHostEventSignalFd,
    NativeWindowHostLoopLinuxHostEventSignalProducerError,
> {
    if raw_fd < 0 {
        return Err(
            NativeWindowHostLoopLinuxHostEventSignalProducerError::InvalidHostEventSignalRawFd {
                raw_fd,
            },
        );
    }
    Ok(NativeWindowHostLoopLinuxHostEventSignalFd { raw_fd })
}

pub fn native_window_host_loop_linux_timer_fd_timespec_from_nanos(
    delta_nanos: u64,
) -> Result<
    NativeWindowHostLoopLinuxTimerFdTimespec,
    NativeWindowHostLoopLinuxSelectorTimerFdBackendError,
> {
    let seconds_u64 = delta_nanos / 1_000_000_000;
    let nanoseconds_u64 = delta_nanos % 1_000_000_000;
    let seconds = i64::try_from(seconds_u64).map_err(|_| {
        NativeWindowHostLoopLinuxSelectorTimerFdBackendError::TimespecSecondsOverflow {
            delta_nanos,
        }
    })?;
    let nanoseconds = i64::try_from(nanoseconds_u64).map_err(|_| {
        NativeWindowHostLoopLinuxSelectorTimerFdBackendError::TimespecSecondsOverflow {
            delta_nanos,
        }
    })?;
    Ok(NativeWindowHostLoopLinuxTimerFdTimespec {
        seconds,
        nanoseconds,
    })
}

pub fn native_window_host_loop_linux_selector_timer_fd_deadline_plan(
    now_nanos: u64,
    deadline_nanos: u64,
) -> Result<
    NativeWindowHostLoopLinuxSelectorTimerFdDeadlinePlan,
    NativeWindowHostLoopLinuxSelectorTimerFdBackendError,
> {
    if deadline_nanos <= now_nanos {
        return Ok(NativeWindowHostLoopLinuxSelectorTimerFdDeadlinePlan::AlreadyReached);
    }
    let delta_nanos = deadline_nanos.checked_sub(now_nanos).ok_or(
        NativeWindowHostLoopLinuxSelectorTimerFdBackendError::DeadlineDeltaOverflow {
            now_nanos,
            deadline_nanos,
        },
    )?;
    Ok(
        NativeWindowHostLoopLinuxSelectorTimerFdDeadlinePlan::RelativeTimespec(
            native_window_host_loop_linux_timer_fd_timespec_from_nanos(delta_nanos)?,
        ),
    )
}

pub fn native_window_host_loop_linux_selector_timer_fd_wake_from_status(
    status: u32,
    last_error_code: u32,
) -> Result<
    NativeWindowHostLoopLinuxSelectorTimerFdWake,
    NativeWindowHostLoopLinuxSelectorTimerFdBackendError,
> {
    match status {
        NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED => {
            Ok(NativeWindowHostLoopLinuxSelectorTimerFdWake::TimerFired)
        }
        NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY => {
            Ok(NativeWindowHostLoopLinuxSelectorTimerFdWake::HostEventReady)
        }
        NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED => Err(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::SelectorWaitFailed {
                code: last_error_code,
            },
        ),
        status => Err(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::UnexpectedSelectorStatus {
                status,
            },
        ),
    }
}

pub fn native_window_host_loop_linux_selector_timer_fd_host_event_from_status(
    status: u32,
    last_error_code: u32,
) -> Result<(), NativeWindowHostLoopLinuxSelectorTimerFdBackendError> {
    match status {
        NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY => Ok(()),
        NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED => Err(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::SelectorWaitFailed {
                code: last_error_code,
            },
        ),
        status => Err(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::UnexpectedSelectorStatus {
                status,
            },
        ),
    }
}

#[derive(Debug)]
pub struct NativeWindowHostLoopLinuxSelectorTimerFdBackend<
    Api: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
> {
    origin: std::time::Instant,
    api: Api,
    selector: Option<NativeWindowHostLoopLinuxSelectorFd>,
    timer: Option<NativeWindowHostLoopLinuxTimerFd>,
    host_event: Option<NativeWindowHostLoopLinuxHostEventFd>,
}

impl<Api> NativeWindowHostLoopLinuxSelectorTimerFdBackend<Api>
where
    Api: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    pub fn new(mut api: Api) -> Result<Self, NativeWindowHostLoopLinuxSelectorTimerFdBackendError> {
        let selector_raw_fd = api.create_selector_raw();
        let selector = match native_window_host_loop_linux_selector_fd_from_raw(selector_raw_fd) {
            Ok(selector) => selector,
            Err(NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidSelectorRawFd {
                ..
            }) => {
                return Err(
                    NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateSelectorFailed {
                        code: api.last_error_code(),
                    },
                );
            }
            Err(error) => return Err(error),
        };
        let timer_raw_fd = api.create_timer_fd_raw();
        let timer = match native_window_host_loop_linux_timer_fd_from_raw(timer_raw_fd) {
            Ok(timer) => timer,
            Err(NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidTimerRawFd {
                ..
            }) => {
                let code = api.last_error_code();
                let _ = api.close_selector_raw(&selector);
                return Err(
                    NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateTimerFdFailed {
                        code,
                    },
                );
            }
            Err(error) => {
                let _ = api.close_selector_raw(&selector);
                return Err(error);
            }
        };
        if !api.register_timer_fd_raw(&selector, &timer) {
            let code = api.last_error_code();
            let _ = api.close_timer_fd_raw(&timer);
            let _ = api.close_selector_raw(&selector);
            return Err(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::RegisterTimerFdFailed {
                    code,
                },
            );
        }
        let host_event_raw_fd = api.create_host_event_fd_raw();
        if host_event_raw_fd < 0 {
            let code = api.last_error_code();
            let _ = api.close_timer_fd_raw(&timer);
            let _ = api.close_selector_raw(&selector);
            return Err(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateHostEventFdFailed {
                    code,
                },
            );
        }
        let host_event = native_window_host_loop_linux_host_event_fd_from_raw(host_event_raw_fd)?;
        if !api.register_host_event_fd_raw(&selector, &host_event) {
            let code = api.last_error_code();
            let _ = api.close_host_event_fd_raw(&host_event);
            let _ = api.close_timer_fd_raw(&timer);
            let _ = api.close_selector_raw(&selector);
            return Err(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::RegisterHostEventFdFailed {
                    code,
                },
            );
        }
        Ok(Self {
            origin: std::time::Instant::now(),
            api,
            selector: Some(selector),
            timer: Some(timer),
            host_event: Some(host_event),
        })
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn api_mut(&mut self) -> &mut Api {
        &mut self.api
    }

    pub fn are_handles_open(&self) -> bool {
        self.selector.is_some() && self.timer.is_some() && self.host_event.is_some()
    }

    pub fn close_handles_if_open(&mut self) -> bool {
        let host_event = self.host_event.take();
        let timer = self.timer.take();
        let selector = self.selector.take();
        let mut closed = false;
        if let Some(host_event) = host_event {
            let _ = self.api.close_host_event_fd_raw(&host_event);
            closed = true;
        }
        if let Some(timer) = timer {
            let _ = self.api.close_timer_fd_raw(&timer);
            closed = true;
        }
        if let Some(selector) = selector {
            let _ = self.api.close_selector_raw(&selector);
            closed = true;
        }
        closed
    }

    pub fn create_host_event_signal_producer<ProducerApi>(
        &self,
        mut producer_api: ProducerApi,
    ) -> Result<
        NativeWindowHostLoopLinuxHostEventSignalProducer<ProducerApi>,
        NativeWindowHostLoopLinuxHostEventSignalProducerError,
    >
    where
        ProducerApi: NativeWindowHostLoopLinuxHostEventSignalRawApi,
    {
        let host_event = self.host_event.as_ref().ok_or(
            NativeWindowHostLoopLinuxHostEventSignalProducerError::InvalidHostEventRawFd {
                raw_fd: -1,
            },
        )?;
        let signal_raw_fd = producer_api.clone_host_event_signal_fd_raw(host_event);
        let signal = match native_window_host_loop_linux_host_event_signal_fd_from_raw(
            signal_raw_fd,
        ) {
            Ok(signal) => signal,
            Err(
                NativeWindowHostLoopLinuxHostEventSignalProducerError::InvalidHostEventSignalRawFd {
                    ..
                },
            ) => {
                return Err(
                    NativeWindowHostLoopLinuxHostEventSignalProducerError::CreateHostEventSignalFdFailed {
                        code: producer_api.last_error_code(),
                    },
                );
            }
            Err(error) => return Err(error),
        };
        Ok(NativeWindowHostLoopLinuxHostEventSignalProducer::new(
            producer_api,
            signal,
        ))
    }

    pub fn signal_host_event(
        &mut self,
    ) -> Result<(), NativeWindowHostLoopLinuxSelectorTimerFdBackendError> {
        let host_event = self.host_event.as_ref().ok_or(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidHostEventRawFd {
                raw_fd: -1,
            },
        )?;
        if !self.api.signal_host_event_fd_raw(host_event) {
            return Err(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::SignalHostEventFdFailed {
                    code: self.api.last_error_code(),
                },
            );
        }
        Ok(())
    }

    fn elapsed_nanos(&self) -> Result<u64, NativeWindowHostLoopLinuxSelectorTimerFdBackendError> {
        u64::try_from(self.origin.elapsed().as_nanos())
            .map_err(|_| NativeWindowHostLoopLinuxSelectorTimerFdBackendError::ElapsedNanosOverflow)
    }

    pub fn wait_for_host_event(
        &mut self,
        _window_size: NativeWindowSize,
        _size_changed: bool,
    ) -> Result<(), NativeWindowHostLoopLinuxSelectorTimerFdBackendError> {
        let selector = self.selector.as_ref().ok_or(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidSelectorRawFd {
                raw_fd: -1,
            },
        )?;
        let host_event = self.host_event.as_ref().ok_or(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidHostEventRawFd {
                raw_fd: -1,
            },
        )?;
        let status = self.api.selector_wait_for_event_raw(selector, host_event);
        let last_error_code = if status == NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED {
            self.api.last_error_code()
        } else {
            0
        };
        native_window_host_loop_linux_selector_timer_fd_host_event_from_status(
            status,
            last_error_code,
        )
    }

    pub fn wait_until_deadline_or_host_event(
        &mut self,
        deadline_nanos: u64,
        _window_size: NativeWindowSize,
        _size_changed: bool,
    ) -> Result<
        NativeWindowHostLoopLinuxSelectorTimerFdWake,
        NativeWindowHostLoopLinuxSelectorTimerFdBackendError,
    > {
        let now_nanos = self.elapsed_nanos()?;
        let plan = native_window_host_loop_linux_selector_timer_fd_deadline_plan(
            now_nanos,
            deadline_nanos,
        )?;
        let NativeWindowHostLoopLinuxSelectorTimerFdDeadlinePlan::RelativeTimespec(timespec) = plan
        else {
            return Ok(NativeWindowHostLoopLinuxSelectorTimerFdWake::TimerFired);
        };
        let selector = self.selector.as_ref().ok_or(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidSelectorRawFd {
                raw_fd: -1,
            },
        )?;
        let timer = self.timer.as_ref().ok_or(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidTimerRawFd { raw_fd: -1 },
        )?;
        let host_event = self.host_event.as_ref().ok_or(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidHostEventRawFd {
                raw_fd: -1,
            },
        )?;
        let api = &mut self.api;
        if !api.arm_timer_fd_relative_timespec(timer, timespec) {
            return Err(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::ArmTimerFdFailed {
                    code: api.last_error_code(),
                },
            );
        }
        let status = api.selector_wait_for_timer_or_event_raw(selector, timer, host_event);
        let last_error_code = if status == NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED {
            api.last_error_code()
        } else {
            0
        };
        native_window_host_loop_linux_selector_timer_fd_wake_from_status(status, last_error_code)
    }
}

#[derive(Debug)]
pub struct NativeWindowHostLoopLinuxHostEventSignalProducer<
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi,
> {
    api: Api,
    signal: Option<NativeWindowHostLoopLinuxHostEventSignalFd>,
}

impl<Api> NativeWindowHostLoopLinuxHostEventSignalProducer<Api>
where
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi,
{
    pub fn new(api: Api, signal: NativeWindowHostLoopLinuxHostEventSignalFd) -> Self {
        Self {
            api,
            signal: Some(signal),
        }
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn api_mut(&mut self) -> &mut Api {
        &mut self.api
    }

    pub fn are_handles_open(&self) -> bool {
        self.signal.is_some()
    }

    pub fn close_signal_handle_if_open(&mut self) -> bool {
        let signal = self.signal.take();
        if let Some(signal) = signal {
            let _ = self.api.close_host_event_signal_fd_raw(&signal);
            return true;
        }
        false
    }

    pub fn signal_host_event(
        &mut self,
    ) -> Result<(), NativeWindowHostLoopLinuxHostEventSignalProducerError> {
        let signal = self.signal.as_ref().ok_or(
            NativeWindowHostLoopLinuxHostEventSignalProducerError::InvalidHostEventSignalRawFd {
                raw_fd: -1,
            },
        )?;
        if !self.api.signal_host_event_signal_fd_raw(signal) {
            return Err(
                NativeWindowHostLoopLinuxHostEventSignalProducerError::SignalHostEventSignalFdFailed {
                    code: self.api.last_error_code(),
                },
            );
        }
        Ok(())
    }
}

impl<Api> Drop for NativeWindowHostLoopLinuxHostEventSignalProducer<Api>
where
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi,
{
    fn drop(&mut self) {
        self.close_signal_handle_if_open();
    }
}

impl<Api> Drop for NativeWindowHostLoopLinuxSelectorTimerFdBackend<Api>
where
    Api: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    fn drop(&mut self) {
        self.close_handles_if_open();
    }
}

impl<Api> NativeWindowHostLoopDeadlineTimerClock
    for NativeWindowHostLoopLinuxSelectorTimerFdBackend<Api>
where
    Api: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    type Error = NativeWindowHostLoopLinuxSelectorTimerFdBackendError;

    fn now_nanos(&mut self) -> Result<u64, Self::Error> {
        self.elapsed_nanos()
    }
}

impl<Api> NativeWindowHostLoopInterruptibleDeadlineWaiter
    for NativeWindowHostLoopLinuxSelectorTimerFdBackend<Api>
where
    Api: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    type Error = NativeWindowHostLoopLinuxSelectorTimerFdBackendError;

    fn wait_for_host_event(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<(), Self::Error> {
        NativeWindowHostLoopLinuxSelectorTimerFdBackend::wait_for_host_event(
            self,
            window_size,
            size_changed,
        )
    }

    fn wait_until_deadline_or_host_event(
        &mut self,
        deadline_nanos: u64,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<NativeWindowHostLoopInterruptibleDeadlineWake, Self::Error> {
        let wake =
            NativeWindowHostLoopLinuxSelectorTimerFdBackend::wait_until_deadline_or_host_event(
                self,
                deadline_nanos,
                window_size,
                size_changed,
            )?;
        Ok(match wake {
            NativeWindowHostLoopLinuxSelectorTimerFdWake::TimerFired => {
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached
            }
            NativeWindowHostLoopLinuxSelectorTimerFdWake::HostEventReady => {
                NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady
            }
        })
    }
}

pub fn build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection<Api>(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
    api: Api,
) -> Result<
    NativeWindowHostLoopLinuxSelectorTimerFdBackend<Api>,
    NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError,
>
where
    Api: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    let checked_selection =
        validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
            selection.platform(),
            selection.backend(),
        )
        .map_err(NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::BackendSupportFailed)?;
    if checked_selection.platform() != NativeWindowHostLoopPlatformKind::Linux
        || checked_selection.backend()
            != NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd
    {
        return Err(
            NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: checked_selection.platform(),
                    requested: checked_selection.backend(),
                },
            ),
        );
    }
    NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).map_err(
        NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::SelectorTimerFdBackendFailed,
    )
}

#[cfg(target_os = "linux")]
#[derive(Debug, Default)]
pub struct NativeWindowHostLoopLinuxSelectorTimerFdSysApi {
    last_error_code: u32,
}

#[cfg(target_os = "linux")]
impl NativeWindowHostLoopLinuxSelectorTimerFdSysApi {
    pub fn new() -> Self {
        Self::default()
    }

    fn clear_error(&mut self) {
        self.last_error_code = 0;
    }

    fn set_error_code(&mut self, code: i32) {
        self.last_error_code = u32::try_from(code).unwrap_or(u32::MAX);
    }

    fn set_last_os_error(&mut self) {
        self.last_error_code = std::io::Error::last_os_error()
            .raw_os_error()
            .and_then(|code| u32::try_from(code).ok())
            .unwrap_or(u32::MAX);
    }

    fn drain_u64_fd(&mut self, raw_fd: i32) -> bool {
        let mut counter = 0_u64;
        let read_result = unsafe {
            libc::read(
                raw_fd,
                (&mut counter as *mut u64).cast::<libc::c_void>(),
                std::mem::size_of::<u64>(),
            )
        };
        if read_result < 0 {
            self.set_last_os_error();
            return false;
        }
        if read_result != std::mem::size_of::<u64>() as libc::ssize_t {
            self.set_error_code(libc::EIO);
            return false;
        }
        if counter == 0 {
            self.set_error_code(libc::EIO);
            return false;
        }
        self.clear_error();
        true
    }

    fn drain_timer_fd(&mut self, timer: &NativeWindowHostLoopLinuxTimerFd) -> bool {
        self.drain_u64_fd(native_window_host_loop_linux_timer_fd_raw(timer))
    }

    fn drain_host_event_fd(&mut self, host_event: &NativeWindowHostLoopLinuxHostEventFd) -> bool {
        self.drain_u64_fd(native_window_host_loop_linux_host_event_fd_raw(host_event))
    }

    fn write_eventfd_counter_raw(&mut self, raw_fd: i32) -> bool {
        let counter = 1_u64;
        let write_result = unsafe {
            libc::write(
                raw_fd,
                (&counter as *const u64).cast::<libc::c_void>(),
                std::mem::size_of::<u64>(),
            )
        };
        if write_result < 0 {
            self.set_last_os_error();
            return false;
        }
        if write_result != std::mem::size_of::<u64>() as libc::ssize_t {
            self.set_error_code(libc::EIO);
            return false;
        }
        self.clear_error();
        true
    }
}

#[cfg(target_os = "linux")]
impl NativeWindowHostLoopLinuxSelectorTimerFdRawApi
    for NativeWindowHostLoopLinuxSelectorTimerFdSysApi
{
    fn create_selector_raw(&mut self) -> i32 {
        let raw_fd = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
        if raw_fd < 0 {
            self.set_last_os_error();
        } else {
            self.clear_error();
        }
        raw_fd
    }

    fn create_timer_fd_raw(&mut self) -> i32 {
        let raw_fd = unsafe {
            libc::timerfd_create(
                libc::CLOCK_MONOTONIC,
                libc::TFD_CLOEXEC | libc::TFD_NONBLOCK,
            )
        };
        if raw_fd < 0 {
            self.set_last_os_error();
        } else {
            self.clear_error();
        }
        raw_fd
    }

    fn create_host_event_fd_raw(&mut self) -> i32 {
        let raw_fd = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK) };
        if raw_fd < 0 {
            self.set_last_os_error();
        } else {
            self.clear_error();
        }
        raw_fd
    }

    fn register_timer_fd_raw(
        &mut self,
        selector: &NativeWindowHostLoopLinuxSelectorFd,
        timer: &NativeWindowHostLoopLinuxTimerFd,
    ) -> bool {
        let mut event = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: native_window_host_loop_linux_timer_fd_raw(timer) as u64,
        };
        let result = unsafe {
            libc::epoll_ctl(
                native_window_host_loop_linux_selector_fd_raw(selector),
                libc::EPOLL_CTL_ADD,
                native_window_host_loop_linux_timer_fd_raw(timer),
                &mut event,
            )
        };
        if result != 0 {
            self.set_last_os_error();
            return false;
        }
        self.clear_error();
        true
    }

    fn register_host_event_fd_raw(
        &mut self,
        selector: &NativeWindowHostLoopLinuxSelectorFd,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool {
        let mut event = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: native_window_host_loop_linux_host_event_fd_raw(host_event) as u64,
        };
        let result = unsafe {
            libc::epoll_ctl(
                native_window_host_loop_linux_selector_fd_raw(selector),
                libc::EPOLL_CTL_ADD,
                native_window_host_loop_linux_host_event_fd_raw(host_event),
                &mut event,
            )
        };
        if result != 0 {
            self.set_last_os_error();
            return false;
        }
        self.clear_error();
        true
    }

    fn signal_host_event_fd_raw(
        &mut self,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool {
        self.write_eventfd_counter_raw(native_window_host_loop_linux_host_event_fd_raw(host_event))
    }

    fn arm_timer_fd_relative_timespec(
        &mut self,
        timer: &NativeWindowHostLoopLinuxTimerFd,
        timespec: NativeWindowHostLoopLinuxTimerFdTimespec,
    ) -> bool {
        let seconds = match libc::time_t::try_from(timespec.seconds()) {
            Ok(seconds) => seconds,
            Err(_) => {
                self.set_error_code(libc::EOVERFLOW);
                return false;
            }
        };
        let nanoseconds = match libc::c_long::try_from(timespec.nanoseconds()) {
            Ok(nanoseconds) => nanoseconds,
            Err(_) => {
                self.set_error_code(libc::EOVERFLOW);
                return false;
            }
        };
        let interval = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        let value = libc::timespec {
            tv_sec: seconds,
            tv_nsec: nanoseconds,
        };
        let timer_spec = libc::itimerspec {
            it_interval: interval,
            it_value: value,
        };
        let result = unsafe {
            libc::timerfd_settime(
                native_window_host_loop_linux_timer_fd_raw(timer),
                0,
                &timer_spec,
                std::ptr::null_mut(),
            )
        };
        if result != 0 {
            self.set_last_os_error();
            return false;
        }
        self.clear_error();
        true
    }

    fn selector_wait_for_timer_or_event_raw(
        &mut self,
        selector: &NativeWindowHostLoopLinuxSelectorFd,
        timer: &NativeWindowHostLoopLinuxTimerFd,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> u32 {
        let mut event = libc::epoll_event { events: 0, u64: 0 };
        let wait_result = unsafe {
            libc::epoll_wait(
                native_window_host_loop_linux_selector_fd_raw(selector),
                &mut event,
                1,
                -1,
            )
        };
        if wait_result < 0 {
            self.set_last_os_error();
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
        }
        if wait_result == 0 {
            self.set_error_code(libc::ETIMEDOUT);
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
        }
        let terminal_events = (libc::EPOLLERR | libc::EPOLLHUP) as u32;
        if event.events & terminal_events != 0 || event.events & libc::EPOLLIN as u32 == 0 {
            self.set_error_code(libc::EIO);
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
        }
        if event.u64 == native_window_host_loop_linux_timer_fd_raw(timer) as u64 {
            if !self.drain_timer_fd(timer) {
                return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
            }
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED;
        }
        if event.u64 == native_window_host_loop_linux_host_event_fd_raw(host_event) as u64 {
            if !self.drain_host_event_fd(host_event) {
                return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
            }
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY;
        }
        self.set_error_code(libc::EINVAL);
        NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED
    }

    fn selector_wait_for_event_raw(
        &mut self,
        selector: &NativeWindowHostLoopLinuxSelectorFd,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> u32 {
        let mut event = libc::epoll_event { events: 0, u64: 0 };
        let wait_result = unsafe {
            libc::epoll_wait(
                native_window_host_loop_linux_selector_fd_raw(selector),
                &mut event,
                1,
                -1,
            )
        };
        if wait_result < 0 {
            self.set_last_os_error();
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
        }
        if wait_result == 0 {
            self.set_error_code(libc::ETIMEDOUT);
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
        }
        if event.u64 != native_window_host_loop_linux_host_event_fd_raw(host_event) as u64 {
            self.set_error_code(libc::EINVAL);
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
        }
        let terminal_events = (libc::EPOLLERR | libc::EPOLLHUP) as u32;
        if event.events & terminal_events != 0 || event.events & libc::EPOLLIN as u32 == 0 {
            self.set_error_code(libc::EIO);
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
        }
        if !self.drain_host_event_fd(host_event) {
            return NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED;
        }
        NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY
    }

    fn close_selector_raw(&mut self, selector: &NativeWindowHostLoopLinuxSelectorFd) -> bool {
        let result =
            unsafe { libc::close(native_window_host_loop_linux_selector_fd_raw(selector)) };
        if result != 0 {
            self.set_last_os_error();
            return false;
        }
        true
    }

    fn close_timer_fd_raw(&mut self, timer: &NativeWindowHostLoopLinuxTimerFd) -> bool {
        let result = unsafe { libc::close(native_window_host_loop_linux_timer_fd_raw(timer)) };
        if result != 0 {
            self.set_last_os_error();
            return false;
        }
        true
    }

    fn close_host_event_fd_raw(
        &mut self,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool {
        let result =
            unsafe { libc::close(native_window_host_loop_linux_host_event_fd_raw(host_event)) };
        if result != 0 {
            self.set_last_os_error();
            return false;
        }
        true
    }

    fn last_error_code(&mut self) -> u32 {
        self.last_error_code
    }
}

#[cfg(target_os = "linux")]
impl NativeWindowHostLoopLinuxHostEventSignalRawApi
    for NativeWindowHostLoopLinuxSelectorTimerFdSysApi
{
    fn clone_host_event_signal_fd_raw(
        &mut self,
        host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> i32 {
        let raw_fd = unsafe {
            libc::fcntl(
                native_window_host_loop_linux_host_event_fd_raw(host_event),
                libc::F_DUPFD_CLOEXEC,
                0,
            )
        };
        if raw_fd < 0 {
            self.set_last_os_error();
        } else {
            self.clear_error();
        }
        raw_fd
    }

    fn signal_host_event_signal_fd_raw(
        &mut self,
        signal: &NativeWindowHostLoopLinuxHostEventSignalFd,
    ) -> bool {
        self.write_eventfd_counter_raw(native_window_host_loop_linux_host_event_signal_fd_raw(
            signal,
        ))
    }

    fn close_host_event_signal_fd_raw(
        &mut self,
        signal: &NativeWindowHostLoopLinuxHostEventSignalFd,
    ) -> bool {
        unsafe {
            libc::close(native_window_host_loop_linux_host_event_signal_fd_raw(
                signal,
            )) == 0
        }
    }

    fn last_error_code(&mut self) -> u32 {
        self.last_error_code
    }
}

#[cfg(target_os = "linux")]
pub fn native_window_host_loop_linux_selector_timer_fd_backend_from_selection(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
) -> Result<
    NativeWindowHostLoopLinuxSelectorTimerFdBackend<NativeWindowHostLoopLinuxSelectorTimerFdSysApi>,
    NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError,
> {
    build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection(
        selection,
        NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new(),
    )
}

#[cfg(target_os = "linux")]
pub fn native_window_host_loop_platform_wait_backend_from_selection(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
) -> Result<
    NativeWindowHostLoopLinuxOnlyPlatformWaitBackend<
        NativeWindowHostLoopLinuxSelectorTimerFdSysApi,
    >,
    NativeWindowHostLoopPlatformWaitHostBuildError,
> {
    build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api(
        selection,
        NativeWindowHostLoopLinuxSelectorTimerFdSysApi::new(),
    )
}

#[derive(Debug)]
pub enum NativeWindowHostLoopNeverWindowsWaitRawApi {}

impl NativeWindowHostLoopWindowsWaitRawApi for NativeWindowHostLoopNeverWindowsWaitRawApi {
    fn create_waitable_timer_raw(&mut self) -> isize {
        match *self {}
    }

    fn set_waitable_timer_relative_100ns(
        &mut self,
        _handle: &NativeWindowHostLoopWindowsWaitHandle,
        _relative_due_time_100ns: i64,
    ) -> bool {
        match *self {}
    }

    fn msg_wait_for_timer_or_message_raw(
        &mut self,
        _handle: &NativeWindowHostLoopWindowsWaitHandle,
    ) -> u32 {
        match *self {}
    }

    fn msg_wait_for_message_raw(&mut self) -> u32 {
        match *self {}
    }

    fn close_handle_raw(&mut self, _handle: &NativeWindowHostLoopWindowsWaitHandle) -> bool {
        match *self {}
    }

    fn last_error_code(&mut self) -> u32 {
        match *self {}
    }
}

#[derive(Debug)]
pub enum NativeWindowHostLoopNeverMacosRunLoopTimerRawApi {}

impl NativeWindowHostLoopMacosRunLoopTimerRawApi
    for NativeWindowHostLoopNeverMacosRunLoopTimerRawApi
{
    fn create_run_loop_timer_raw(&mut self) -> isize {
        match *self {}
    }

    fn schedule_run_loop_timer_relative_nanos(
        &mut self,
        _handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
        _relative_nanos: u64,
    ) -> bool {
        match *self {}
    }

    fn run_loop_wait_for_timer_or_event_raw(
        &mut self,
        _handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
    ) -> u32 {
        match *self {}
    }

    fn run_loop_wait_for_event_raw(&mut self) -> u32 {
        match *self {}
    }

    fn invalidate_run_loop_timer_raw(
        &mut self,
        _handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
    ) -> bool {
        match *self {}
    }

    fn last_error_code(&mut self) -> u32 {
        match *self {}
    }
}

#[derive(Debug)]
pub enum NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi {}

impl NativeWindowHostLoopLinuxSelectorTimerFdRawApi
    for NativeWindowHostLoopNeverLinuxSelectorTimerFdRawApi
{
    fn create_selector_raw(&mut self) -> i32 {
        match *self {}
    }

    fn create_timer_fd_raw(&mut self) -> i32 {
        match *self {}
    }

    fn create_host_event_fd_raw(&mut self) -> i32 {
        match *self {}
    }

    fn register_timer_fd_raw(
        &mut self,
        _selector: &NativeWindowHostLoopLinuxSelectorFd,
        _timer: &NativeWindowHostLoopLinuxTimerFd,
    ) -> bool {
        match *self {}
    }

    fn register_host_event_fd_raw(
        &mut self,
        _selector: &NativeWindowHostLoopLinuxSelectorFd,
        _host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool {
        match *self {}
    }

    fn signal_host_event_fd_raw(
        &mut self,
        _host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool {
        match *self {}
    }

    fn arm_timer_fd_relative_timespec(
        &mut self,
        _timer: &NativeWindowHostLoopLinuxTimerFd,
        _relative_timespec: NativeWindowHostLoopLinuxTimerFdTimespec,
    ) -> bool {
        match *self {}
    }

    fn selector_wait_for_timer_or_event_raw(
        &mut self,
        _selector: &NativeWindowHostLoopLinuxSelectorFd,
        _timer: &NativeWindowHostLoopLinuxTimerFd,
        _host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> u32 {
        match *self {}
    }

    fn selector_wait_for_event_raw(
        &mut self,
        _selector: &NativeWindowHostLoopLinuxSelectorFd,
        _host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> u32 {
        match *self {}
    }

    fn close_selector_raw(&mut self, _selector: &NativeWindowHostLoopLinuxSelectorFd) -> bool {
        match *self {}
    }

    fn close_timer_fd_raw(&mut self, _timer: &NativeWindowHostLoopLinuxTimerFd) -> bool {
        match *self {}
    }

    fn close_host_event_fd_raw(
        &mut self,
        _host_event: &NativeWindowHostLoopLinuxHostEventFd,
    ) -> bool {
        match *self {}
    }

    fn last_error_code(&mut self) -> u32 {
        match *self {}
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopWindowsWaitHandle {
    raw_handle: isize,
}

fn native_window_host_loop_windows_wait_handle_raw(
    handle: &NativeWindowHostLoopWindowsWaitHandle,
) -> isize {
    handle.raw_handle
}

pub const NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMER_SIGNALED: u32 = 0;
pub const NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ONE_HANDLE: u32 = 1;
pub const NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ZERO_HANDLES: u32 = 0;
pub const NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMEOUT: u32 = 258;
pub const NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_FAILED: u32 = 0xFFFF_FFFF;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopWindowsDeadlinePlan {
    AlreadyReached,
    Relative100ns(i64),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopWindowsWaitBackendError {
    InvalidRawHandle { raw_handle: isize },
    CreateWaitableTimerFailed { code: u32 },
    SetWaitableTimerFailed { code: u32 },
    WaitFailed { code: u32 },
    UnexpectedWaitStatus { status: u32 },
    ElapsedNanosOverflow,
    DeadlineDeltaOverflow { now_nanos: u64, deadline_nanos: u64 },
    DeadlineDelta100nsOverflow { delta_nanos: u64 },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopWindowsWaitBackendBuildError {
    BackendSupportFailed(NativeWindowHostLoopPlatformWaitBackendSupportError),
    WaitBackendFailed(NativeWindowHostLoopWindowsWaitBackendError),
}

pub trait NativeWindowHostLoopWindowsWaitRawApi {
    fn create_waitable_timer_raw(&mut self) -> isize;

    fn set_waitable_timer_relative_100ns(
        &mut self,
        handle: &NativeWindowHostLoopWindowsWaitHandle,
        relative_due_time_100ns: i64,
    ) -> bool;

    fn msg_wait_for_timer_or_message_raw(
        &mut self,
        handle: &NativeWindowHostLoopWindowsWaitHandle,
    ) -> u32;

    fn msg_wait_for_message_raw(&mut self) -> u32;

    fn close_handle_raw(&mut self, handle: &NativeWindowHostLoopWindowsWaitHandle) -> bool;

    fn last_error_code(&mut self) -> u32;
}

pub fn native_window_host_loop_windows_wait_handle_from_raw(
    raw_handle: isize,
) -> Result<NativeWindowHostLoopWindowsWaitHandle, NativeWindowHostLoopWindowsWaitBackendError> {
    if raw_handle == 0 || raw_handle == -1 {
        return Err(NativeWindowHostLoopWindowsWaitBackendError::InvalidRawHandle { raw_handle });
    }
    Ok(NativeWindowHostLoopWindowsWaitHandle { raw_handle })
}

pub fn native_window_host_loop_windows_deadline_plan(
    now_nanos: u64,
    deadline_nanos: u64,
) -> Result<NativeWindowHostLoopWindowsDeadlinePlan, NativeWindowHostLoopWindowsWaitBackendError> {
    if deadline_nanos <= now_nanos {
        return Ok(NativeWindowHostLoopWindowsDeadlinePlan::AlreadyReached);
    }
    let delta_nanos = deadline_nanos.checked_sub(now_nanos).ok_or(
        NativeWindowHostLoopWindowsWaitBackendError::DeadlineDeltaOverflow {
            now_nanos,
            deadline_nanos,
        },
    )?;
    let rounded_delta = delta_nanos.checked_add(99).ok_or(
        NativeWindowHostLoopWindowsWaitBackendError::DeadlineDelta100nsOverflow { delta_nanos },
    )?;
    let relative_100ns_u64 = rounded_delta / 100;
    let relative_100ns_i64 = i64::try_from(relative_100ns_u64).map_err(|_| {
        NativeWindowHostLoopWindowsWaitBackendError::DeadlineDelta100nsOverflow { delta_nanos }
    })?;
    Ok(NativeWindowHostLoopWindowsDeadlinePlan::Relative100ns(
        -relative_100ns_i64,
    ))
}

pub fn native_window_host_loop_windows_wait_wake_from_timer_or_message_status(
    status: u32,
    last_error_code: u32,
) -> Result<
    NativeWindowHostLoopInterruptibleDeadlineWake,
    NativeWindowHostLoopWindowsWaitBackendError,
> {
    match status {
        NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMER_SIGNALED => {
            Ok(NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached)
        }
        NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ONE_HANDLE => {
            Ok(NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady)
        }
        NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_FAILED => {
            Err(NativeWindowHostLoopWindowsWaitBackendError::WaitFailed {
                code: last_error_code,
            })
        }
        status => Err(NativeWindowHostLoopWindowsWaitBackendError::UnexpectedWaitStatus { status }),
    }
}

pub fn native_window_host_loop_windows_host_event_from_message_status(
    status: u32,
    last_error_code: u32,
) -> Result<(), NativeWindowHostLoopWindowsWaitBackendError> {
    match status {
        NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ZERO_HANDLES => Ok(()),
        NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_FAILED => {
            Err(NativeWindowHostLoopWindowsWaitBackendError::WaitFailed {
                code: last_error_code,
            })
        }
        status => Err(NativeWindowHostLoopWindowsWaitBackendError::UnexpectedWaitStatus { status }),
    }
}

#[derive(Debug)]
pub struct NativeWindowHostLoopWindowsWaitBackend<Api: NativeWindowHostLoopWindowsWaitRawApi> {
    origin: std::time::Instant,
    api: Api,
    handle: Option<NativeWindowHostLoopWindowsWaitHandle>,
}

impl<Api> NativeWindowHostLoopWindowsWaitBackend<Api>
where
    Api: NativeWindowHostLoopWindowsWaitRawApi,
{
    pub fn new(mut api: Api) -> Result<Self, NativeWindowHostLoopWindowsWaitBackendError> {
        let raw_handle = api.create_waitable_timer_raw();
        let handle = match native_window_host_loop_windows_wait_handle_from_raw(raw_handle) {
            Ok(handle) => handle,
            Err(NativeWindowHostLoopWindowsWaitBackendError::InvalidRawHandle { .. }) => {
                return Err(
                    NativeWindowHostLoopWindowsWaitBackendError::CreateWaitableTimerFailed {
                        code: api.last_error_code(),
                    },
                );
            }
            Err(error) => return Err(error),
        };
        Ok(Self {
            origin: std::time::Instant::now(),
            api,
            handle: Some(handle),
        })
    }

    pub fn api(&self) -> &Api {
        &self.api
    }

    pub fn api_mut(&mut self) -> &mut Api {
        &mut self.api
    }

    pub fn is_handle_open(&self) -> bool {
        self.handle.is_some()
    }

    pub fn close_handle_if_open(&mut self) -> bool {
        let Some(handle) = self.handle.take() else {
            return false;
        };
        let _ = self.api.close_handle_raw(&handle);
        true
    }

    fn elapsed_nanos(&self) -> Result<u64, NativeWindowHostLoopWindowsWaitBackendError> {
        u64::try_from(self.origin.elapsed().as_nanos())
            .map_err(|_| NativeWindowHostLoopWindowsWaitBackendError::ElapsedNanosOverflow)
    }
}

impl<Api> Drop for NativeWindowHostLoopWindowsWaitBackend<Api>
where
    Api: NativeWindowHostLoopWindowsWaitRawApi,
{
    fn drop(&mut self) {
        self.close_handle_if_open();
    }
}

impl<Api> NativeWindowHostLoopDeadlineTimerClock for NativeWindowHostLoopWindowsWaitBackend<Api>
where
    Api: NativeWindowHostLoopWindowsWaitRawApi,
{
    type Error = NativeWindowHostLoopWindowsWaitBackendError;

    fn now_nanos(&mut self) -> Result<u64, Self::Error> {
        self.elapsed_nanos()
    }
}

impl<Api> NativeWindowHostLoopInterruptibleDeadlineWaiter
    for NativeWindowHostLoopWindowsWaitBackend<Api>
where
    Api: NativeWindowHostLoopWindowsWaitRawApi,
{
    type Error = NativeWindowHostLoopWindowsWaitBackendError;

    fn wait_for_host_event(
        &mut self,
        _window_size: NativeWindowSize,
        _size_changed: bool,
    ) -> Result<(), Self::Error> {
        let status = self.api.msg_wait_for_message_raw();
        let last_error_code = if status == NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_FAILED {
            self.api.last_error_code()
        } else {
            0
        };
        native_window_host_loop_windows_host_event_from_message_status(status, last_error_code)
    }

    fn wait_until_deadline_or_host_event(
        &mut self,
        deadline_nanos: u64,
        _window_size: NativeWindowSize,
        _size_changed: bool,
    ) -> Result<NativeWindowHostLoopInterruptibleDeadlineWake, Self::Error> {
        let now_nanos = self.elapsed_nanos()?;
        let plan = native_window_host_loop_windows_deadline_plan(now_nanos, deadline_nanos)?;
        let NativeWindowHostLoopWindowsDeadlinePlan::Relative100ns(relative_due_time_100ns) = plan
        else {
            return Ok(NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached);
        };
        let handle = self.handle.as_ref().ok_or(
            NativeWindowHostLoopWindowsWaitBackendError::InvalidRawHandle { raw_handle: 0 },
        )?;
        let api = &mut self.api;
        if !api.set_waitable_timer_relative_100ns(handle, relative_due_time_100ns) {
            return Err(
                NativeWindowHostLoopWindowsWaitBackendError::SetWaitableTimerFailed {
                    code: api.last_error_code(),
                },
            );
        }
        let status = api.msg_wait_for_timer_or_message_raw(handle);
        let last_error_code = if status == NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_FAILED {
            api.last_error_code()
        } else {
            0
        };
        native_window_host_loop_windows_wait_wake_from_timer_or_message_status(
            status,
            last_error_code,
        )
    }
}

pub fn build_native_window_host_loop_windows_wait_backend_from_selection<Api>(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
    api: Api,
) -> Result<
    NativeWindowHostLoopWindowsWaitBackend<Api>,
    NativeWindowHostLoopWindowsWaitBackendBuildError,
>
where
    Api: NativeWindowHostLoopWindowsWaitRawApi,
{
    let checked_selection =
        validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
            selection.platform(),
            selection.backend(),
        )
        .map_err(NativeWindowHostLoopWindowsWaitBackendBuildError::BackendSupportFailed)?;
    if checked_selection.platform() != NativeWindowHostLoopPlatformKind::Windows
        || checked_selection.backend()
            != NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait
    {
        return Err(
            NativeWindowHostLoopWindowsWaitBackendBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: checked_selection.platform(),
                    requested: checked_selection.backend(),
                },
            ),
        );
    }
    NativeWindowHostLoopWindowsWaitBackend::new(api)
        .map_err(NativeWindowHostLoopWindowsWaitBackendBuildError::WaitBackendFailed)
}

impl<WindowsApi, MacosApi, LinuxApi> NativeWindowHostLoopDeadlineTimerClock
    for NativeWindowHostLoopPlatformWaitBackend<WindowsApi, MacosApi, LinuxApi>
where
    WindowsApi: NativeWindowHostLoopWindowsWaitRawApi,
    MacosApi: NativeWindowHostLoopMacosRunLoopTimerRawApi,
    LinuxApi: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    type Error = NativeWindowHostLoopPlatformWaitBackendError<
        NativeWindowHostLoopWindowsWaitBackendError,
        NativeWindowHostLoopMacosRunLoopTimerBackendError,
        NativeWindowHostLoopLinuxSelectorTimerFdBackendError,
    >;

    fn now_nanos(&mut self) -> Result<u64, Self::Error> {
        match self {
            NativeWindowHostLoopPlatformWaitBackend::WindowsWaitableTimerMessageWait(backend) => {
                backend.now_nanos().map_err(
                    NativeWindowHostLoopPlatformWaitBackendError::WindowsWaitableTimerMessageWait,
                )
            }
            NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer(backend) => {
                NativeWindowHostLoopDeadlineTimerClock::now_nanos(backend)
                    .map_err(NativeWindowHostLoopPlatformWaitBackendError::MacosRunLoopTimer)
            }
            NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(backend) => {
                NativeWindowHostLoopDeadlineTimerClock::now_nanos(backend)
                    .map_err(NativeWindowHostLoopPlatformWaitBackendError::LinuxSelectorTimerFd)
            }
        }
    }
}

impl<WindowsApi, MacosApi, LinuxApi> NativeWindowHostLoopInterruptibleDeadlineWaiter
    for NativeWindowHostLoopPlatformWaitBackend<WindowsApi, MacosApi, LinuxApi>
where
    WindowsApi: NativeWindowHostLoopWindowsWaitRawApi,
    MacosApi: NativeWindowHostLoopMacosRunLoopTimerRawApi,
    LinuxApi: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    type Error = NativeWindowHostLoopPlatformWaitBackendError<
        NativeWindowHostLoopWindowsWaitBackendError,
        NativeWindowHostLoopMacosRunLoopTimerBackendError,
        NativeWindowHostLoopLinuxSelectorTimerFdBackendError,
    >;

    fn wait_for_host_event(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<(), Self::Error> {
        match self {
            NativeWindowHostLoopPlatformWaitBackend::WindowsWaitableTimerMessageWait(backend) => {
                backend
                    .wait_for_host_event(window_size, size_changed)
                    .map_err(
                    NativeWindowHostLoopPlatformWaitBackendError::WindowsWaitableTimerMessageWait,
                )
            }
            NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer(backend) => backend
                .wait_for_host_event(window_size, size_changed)
                .map_err(NativeWindowHostLoopPlatformWaitBackendError::MacosRunLoopTimer),
            NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(backend) => backend
                .wait_for_host_event(window_size, size_changed)
                .map_err(NativeWindowHostLoopPlatformWaitBackendError::LinuxSelectorTimerFd),
        }
    }

    fn wait_until_deadline_or_host_event(
        &mut self,
        deadline_nanos: u64,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<NativeWindowHostLoopInterruptibleDeadlineWake, Self::Error> {
        match self {
            NativeWindowHostLoopPlatformWaitBackend::WindowsWaitableTimerMessageWait(backend) => {
                backend
                    .wait_until_deadline_or_host_event(deadline_nanos, window_size, size_changed)
                    .map_err(
                        NativeWindowHostLoopPlatformWaitBackendError::WindowsWaitableTimerMessageWait,
                    )
            }
            NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer(backend) => {
                NativeWindowHostLoopInterruptibleDeadlineWaiter::wait_until_deadline_or_host_event(
                    backend,
                    deadline_nanos,
                    window_size,
                    size_changed,
                )
                .map_err(NativeWindowHostLoopPlatformWaitBackendError::MacosRunLoopTimer)
            }
            NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(backend) => {
                NativeWindowHostLoopInterruptibleDeadlineWaiter::wait_until_deadline_or_host_event(
                    backend,
                    deadline_nanos,
                    window_size,
                    size_changed,
                )
                .map_err(NativeWindowHostLoopPlatformWaitBackendError::LinuxSelectorTimerFd)
            }
        }
    }
}

pub fn build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis<
    WindowsApi,
    MacosApi,
    LinuxApi,
>(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
    windows_api: WindowsApi,
    macos_api: MacosApi,
    linux_api: LinuxApi,
) -> Result<
    NativeWindowHostLoopPlatformWaitBackend<WindowsApi, MacosApi, LinuxApi>,
    NativeWindowHostLoopPlatformWaitHostBuildError,
>
where
    WindowsApi: NativeWindowHostLoopWindowsWaitRawApi,
    MacosApi: NativeWindowHostLoopMacosRunLoopTimerRawApi,
    LinuxApi: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    let checked_selection =
        validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
            selection.platform(),
            selection.backend(),
        )
        .map_err(NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed)?;
    match (checked_selection.platform(), checked_selection.backend()) {
        (
            NativeWindowHostLoopPlatformKind::Windows,
            NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
        ) => build_native_window_host_loop_windows_wait_backend_from_selection(
            checked_selection,
            windows_api,
        )
        .map(NativeWindowHostLoopPlatformWaitBackend::WindowsWaitableTimerMessageWait)
        .map_err(|error| match error {
            NativeWindowHostLoopWindowsWaitBackendBuildError::BackendSupportFailed(error) => {
                NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(error)
            }
            NativeWindowHostLoopWindowsWaitBackendBuildError::WaitBackendFailed(error) => {
                NativeWindowHostLoopPlatformWaitHostBuildError::WindowsWaitBackendFailed(error)
            }
        }),
        (
            NativeWindowHostLoopPlatformKind::Macos,
            NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
        ) => build_native_window_host_loop_macos_run_loop_timer_backend_from_selection(
            checked_selection,
            macos_api,
        )
        .map(NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer)
        .map_err(|error| match error {
            NativeWindowHostLoopMacosRunLoopTimerBackendBuildError::BackendSupportFailed(error) => {
                NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(error)
            }
            NativeWindowHostLoopMacosRunLoopTimerBackendBuildError::RunLoopTimerBackendFailed(
                error,
            ) => NativeWindowHostLoopPlatformWaitHostBuildError::MacosRunLoopTimerBackendFailed(
                error,
            ),
        }),
        (
            NativeWindowHostLoopPlatformKind::Linux,
            NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
        ) => build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection(
            checked_selection,
            linux_api,
        )
        .map(NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd)
        .map_err(|error| {
            match error {
            NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::BackendSupportFailed(
                error,
            ) => NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(error),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::SelectorTimerFdBackendFailed(
                error,
            ) => {
                NativeWindowHostLoopPlatformWaitHostBuildError::LinuxSelectorTimerFdBackendFailed(
                    error,
                )
            }
        }
        }),
        (platform, backend) => Err(
            NativeWindowHostLoopPlatformWaitHostBuildError::BackendImplementationUnavailable {
                platform,
                backend,
            },
        ),
    }
}

pub fn build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api<
    WindowsApi,
>(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
    api: WindowsApi,
) -> Result<
    NativeWindowHostLoopWindowsOnlyPlatformWaitBackend<WindowsApi>,
    NativeWindowHostLoopPlatformWaitHostBuildError,
>
where
    WindowsApi: NativeWindowHostLoopWindowsWaitRawApi,
{
    let checked_selection =
        validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
            selection.platform(),
            selection.backend(),
        )
        .map_err(NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed)?;
    match (checked_selection.platform(), checked_selection.backend()) {
        (
            NativeWindowHostLoopPlatformKind::Windows,
            NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
        ) => build_native_window_host_loop_windows_wait_backend_from_selection(
            checked_selection,
            api,
        )
        .map(NativeWindowHostLoopPlatformWaitBackend::WindowsWaitableTimerMessageWait)
        .map_err(|error| match error {
            NativeWindowHostLoopWindowsWaitBackendBuildError::BackendSupportFailed(error) => {
                NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(error)
            }
            NativeWindowHostLoopWindowsWaitBackendBuildError::WaitBackendFailed(error) => {
                NativeWindowHostLoopPlatformWaitHostBuildError::WindowsWaitBackendFailed(error)
            }
        }),
        (platform, backend) => Err(
            NativeWindowHostLoopPlatformWaitHostBuildError::BackendImplementationUnavailable {
                platform,
                backend,
            },
        ),
    }
}

pub fn build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api<LinuxApi>(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
    api: LinuxApi,
) -> Result<
    NativeWindowHostLoopLinuxOnlyPlatformWaitBackend<LinuxApi>,
    NativeWindowHostLoopPlatformWaitHostBuildError,
>
where
    LinuxApi: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    let checked_selection =
        validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
            selection.platform(),
            selection.backend(),
        )
        .map_err(NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed)?;
    match (checked_selection.platform(), checked_selection.backend()) {
        (
            NativeWindowHostLoopPlatformKind::Linux,
            NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
        ) => build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection(
            checked_selection,
            api,
        )
        .map(NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd)
        .map_err(|error| {
            match error {
            NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::BackendSupportFailed(
                error,
            ) => NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(error),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::SelectorTimerFdBackendFailed(
                error,
            ) => {
                NativeWindowHostLoopPlatformWaitHostBuildError::LinuxSelectorTimerFdBackendFailed(
                    error,
                )
            }
        }
        }),
        (platform, backend) => Err(
            NativeWindowHostLoopPlatformWaitHostBuildError::BackendImplementationUnavailable {
                platform,
                backend,
            },
        ),
    }
}

pub type NativeWindowHostLoopPlatformWaitRunLoopHost<Host, WindowsApi, MacosApi, LinuxApi> =
    NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost<
        Host,
        NativeWindowHostLoopPlatformWaitBackend<WindowsApi, MacosApi, LinuxApi>,
    >;

pub fn native_window_host_loop_platform_wait_run_loop_host_from_backend<
    Host,
    WindowsApi,
    MacosApi,
    LinuxApi,
>(
    host: Host,
    backend: NativeWindowHostLoopPlatformWaitBackend<WindowsApi, MacosApi, LinuxApi>,
) -> NativeWindowHostLoopPlatformWaitRunLoopHost<Host, WindowsApi, MacosApi, LinuxApi>
where
    WindowsApi: NativeWindowHostLoopWindowsWaitRawApi,
    MacosApi: NativeWindowHostLoopMacosRunLoopTimerRawApi,
    LinuxApi: NativeWindowHostLoopLinuxSelectorTimerFdRawApi,
{
    let wait_adapter =
        NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);
    NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost::new(host, wait_adapter)
}

#[cfg(target_os = "windows")]
pub struct NativeWindowHostLoopWindowsWaitSysApi;

#[cfg(target_os = "windows")]
impl NativeWindowHostLoopWindowsWaitRawApi for NativeWindowHostLoopWindowsWaitSysApi {
    fn create_waitable_timer_raw(&mut self) -> isize {
        unsafe {
            windows_sys::Win32::System::Threading::CreateWaitableTimerW(
                std::ptr::null(),
                0,
                std::ptr::null(),
            ) as isize
        }
    }

    fn set_waitable_timer_relative_100ns(
        &mut self,
        handle: &NativeWindowHostLoopWindowsWaitHandle,
        relative_due_time_100ns: i64,
    ) -> bool {
        unsafe {
            windows_sys::Win32::System::Threading::SetWaitableTimer(
                native_window_host_loop_windows_wait_handle_raw(handle)
                    as windows_sys::Win32::Foundation::HANDLE,
                &relative_due_time_100ns,
                0,
                None,
                std::ptr::null(),
                0,
            ) != 0
        }
    }

    fn msg_wait_for_timer_or_message_raw(
        &mut self,
        handle: &NativeWindowHostLoopWindowsWaitHandle,
    ) -> u32 {
        let handles = [native_window_host_loop_windows_wait_handle_raw(handle)
            as windows_sys::Win32::Foundation::HANDLE];
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::MsgWaitForMultipleObjects(
                1,
                handles.as_ptr(),
                0,
                windows_sys::Win32::System::Threading::INFINITE,
                windows_sys::Win32::UI::WindowsAndMessaging::QS_ALLINPUT,
            )
        }
    }

    fn msg_wait_for_message_raw(&mut self) -> u32 {
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::MsgWaitForMultipleObjects(
                0,
                std::ptr::null(),
                0,
                windows_sys::Win32::System::Threading::INFINITE,
                windows_sys::Win32::UI::WindowsAndMessaging::QS_ALLINPUT,
            )
        }
    }

    fn close_handle_raw(&mut self, handle: &NativeWindowHostLoopWindowsWaitHandle) -> bool {
        unsafe {
            windows_sys::Win32::Foundation::CloseHandle(
                native_window_host_loop_windows_wait_handle_raw(handle)
                    as windows_sys::Win32::Foundation::HANDLE,
            ) != 0
        }
    }

    fn last_error_code(&mut self) -> u32 {
        unsafe { windows_sys::Win32::Foundation::GetLastError() }
    }
}

#[cfg(target_os = "windows")]
pub fn native_window_host_loop_windows_wait_backend_from_selection(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
) -> Result<
    NativeWindowHostLoopWindowsWaitBackend<NativeWindowHostLoopWindowsWaitSysApi>,
    NativeWindowHostLoopWindowsWaitBackendBuildError,
> {
    build_native_window_host_loop_windows_wait_backend_from_selection(
        selection,
        NativeWindowHostLoopWindowsWaitSysApi,
    )
}

#[cfg(target_os = "windows")]
pub fn native_window_host_loop_platform_wait_backend_from_selection(
    selection: NativeWindowHostLoopPlatformWaitBackendSelection,
) -> Result<
    NativeWindowHostLoopWindowsOnlyPlatformWaitBackend<NativeWindowHostLoopWindowsWaitSysApi>,
    NativeWindowHostLoopPlatformWaitHostBuildError,
> {
    build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api(
        selection,
        NativeWindowHostLoopWindowsWaitSysApi,
    )
}

#[cfg(target_os = "windows")]
pub fn native_window_run_loop_platform_wait_backend_from_config(
    config: NativeWindowRunLoopConfig,
) -> Result<
    NativeWindowHostLoopWindowsOnlyPlatformWaitBackend<NativeWindowHostLoopWindowsWaitSysApi>,
    NativeWindowRunLoopPlatformWaitBackendFromConfigError,
> {
    let selection = native_window_run_loop_platform_wait_backend_selection(config)
        .map_err(NativeWindowRunLoopPlatformWaitBackendFromConfigError::Config)?;
    native_window_host_loop_platform_wait_backend_from_selection(selection)
        .map_err(NativeWindowRunLoopPlatformWaitBackendFromConfigError::Build)
}

pub const NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY: u32 = 1;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopEventQueueStatusAdapterError<AdapterError> {
    InvalidRawStatus { raw_status: u32 },
    AdapterFailed(AdapterError),
}

pub trait NativeWindowHostLoopEventQueueStatusAdapter {
    type Error;

    fn wait_for_host_event_raw_status(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<u32, Self::Error>;
}

pub fn wait_native_window_host_loop_event_queue_raw_status_with_adapter<Adapter>(
    adapter: &mut Adapter,
    window_size: NativeWindowSize,
    size_changed: bool,
) -> Result<(), NativeWindowHostLoopEventQueueStatusAdapterError<Adapter::Error>>
where
    Adapter: NativeWindowHostLoopEventQueueStatusAdapter,
{
    let raw_status = adapter
        .wait_for_host_event_raw_status(window_size, size_changed)
        .map_err(NativeWindowHostLoopEventQueueStatusAdapterError::AdapterFailed)?;
    if raw_status != NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY {
        return Err(
            NativeWindowHostLoopEventQueueStatusAdapterError::InvalidRawStatus { raw_status },
        );
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct NativeWindowHostLoopEventQueueStatusWaiter<Adapter> {
    adapter: Adapter,
}

impl<Adapter> NativeWindowHostLoopEventQueueStatusWaiter<Adapter> {
    pub fn new(adapter: Adapter) -> Self {
        Self { adapter }
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn adapter_mut(&mut self) -> &mut Adapter {
        &mut self.adapter
    }

    pub fn into_inner(self) -> Adapter {
        self.adapter
    }
}

impl<Adapter> NativeWindowHostLoopEventQueueWaiter
    for NativeWindowHostLoopEventQueueStatusWaiter<Adapter>
where
    Adapter: NativeWindowHostLoopEventQueueStatusAdapter,
{
    type Error = NativeWindowHostLoopEventQueueStatusAdapterError<Adapter::Error>;

    fn wait_for_host_event(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<(), Self::Error> {
        wait_native_window_host_loop_event_queue_raw_status_with_adapter(
            &mut self.adapter,
            window_size,
            size_changed,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopMessagePumpStatusAdapterError<PumpError> {
    PumpFailed(PumpError),
}

pub trait NativeWindowHostLoopMessagePumpAdapter {
    type Error;

    fn pump_host_messages(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug)]
pub struct NativeWindowHostLoopMessagePumpStatusAdapter<Adapter> {
    adapter: Adapter,
}

impl<Adapter> NativeWindowHostLoopMessagePumpStatusAdapter<Adapter> {
    pub fn new(adapter: Adapter) -> Self {
        Self { adapter }
    }

    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    pub fn adapter_mut(&mut self) -> &mut Adapter {
        &mut self.adapter
    }

    pub fn into_inner(self) -> Adapter {
        self.adapter
    }
}

impl<Adapter> NativeWindowHostLoopEventQueueStatusAdapter
    for NativeWindowHostLoopMessagePumpStatusAdapter<Adapter>
where
    Adapter: NativeWindowHostLoopMessagePumpAdapter,
{
    type Error = NativeWindowHostLoopMessagePumpStatusAdapterError<Adapter::Error>;

    fn wait_for_host_event_raw_status(
        &mut self,
        window_size: NativeWindowSize,
        size_changed: bool,
    ) -> Result<u32, Self::Error> {
        self.adapter
            .pump_host_messages(window_size, size_changed)
            .map_err(NativeWindowHostLoopMessagePumpStatusAdapterError::PumpFailed)?;
        Ok(NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY)
    }
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
    wait_strategy_state: NativeWindowHostLoopWaitStrategyState,
}

impl NativeWindowHostLoopSchedulerState {
    pub fn new() -> Self {
        Self {
            runner_state: NativeWindowHostLoopRunnerState::new(),
            wait_strategy_state: NativeWindowHostLoopWaitStrategyState::new(),
        }
    }

    pub fn title_initialized(&self) -> bool {
        self.runner_state.title_initialized()
    }

    pub fn wait_strategy_state(&self) -> NativeWindowHostLoopWaitStrategyState {
        self.wait_strategy_state
    }
}

impl Default for NativeWindowHostLoopSchedulerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopWaitStrategyState {
    frame_pacing_target_fps: Option<NativeWindowTargetFps>,
    frame_pacing_remainder_nanos: u32,
}

impl NativeWindowHostLoopWaitStrategyState {
    pub fn new() -> Self {
        Self {
            frame_pacing_target_fps: None,
            frame_pacing_remainder_nanos: 0,
        }
    }

    pub fn frame_pacing_target_fps(self) -> Option<NativeWindowTargetFps> {
        self.frame_pacing_target_fps
    }

    pub fn frame_pacing_remainder_nanos(self) -> u32 {
        self.frame_pacing_remainder_nanos
    }
}

impl Default for NativeWindowHostLoopWaitStrategyState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeWindowHostLoopWaitInstructionPlan {
    pub next_strategy_state: NativeWindowHostLoopWaitStrategyState,
    pub instruction: NativeWindowHostLoopWaitInstruction,
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
        instruction: NativeWindowHostLoopWaitInstruction,
        outcome: NativeWindowHostLoopWaitOutcome,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopSchedulerResumeReady {
    HostEventPumped {
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    FramePresentPaced {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
    },
    FrameIntervalTimerFired {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        wait_nanos: u32,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeWindowHostLoopSchedulerResumeState {
    Ready(NativeWindowHostLoopSchedulerResumeReady),
    WaitingForFrameIntervalTimer {
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        wait_nanos: u32,
        timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
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
            wait_backend: NativeWindowRunLoopWaitBackend::default(),
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
            wait_backend: NativeWindowRunLoopWaitBackend::default(),
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
            wait_backend: NativeWindowRunLoopWaitBackend::default(),
        }
    }

    pub fn new_with_wait_backend(
        demo: GuiDemo,
        counter_value: i32,
        scale: usize,
        target_fps: NativeWindowTargetFps,
        host_loop_policy: NativeWindowHostLoopRunPolicy,
        wait_backend: NativeWindowRunLoopWaitBackend,
    ) -> Self {
        Self {
            demo,
            counter_value,
            scale,
            target_fps,
            host_loop_policy,
            wait_backend,
        }
    }

    pub fn new_with_frame_interval_wait_backend(
        demo: GuiDemo,
        counter_value: i32,
        scale: usize,
        target_fps: NativeWindowTargetFps,
        host_loop_policy: NativeWindowHostLoopRunPolicy,
        frame_interval_wait_backend: NativeWindowRunLoopFrameIntervalWaitBackend,
    ) -> Self {
        Self::new_with_wait_backend(
            demo,
            counter_value,
            scale,
            target_fps,
            host_loop_policy,
            NativeWindowRunLoopWaitBackend::from(frame_interval_wait_backend),
        )
    }

    pub fn new_with_platform_wait_backend_selection(
        demo: GuiDemo,
        counter_value: i32,
        scale: usize,
        target_fps: NativeWindowTargetFps,
        host_loop_policy: NativeWindowHostLoopRunPolicy,
        selection: NativeWindowHostLoopPlatformWaitBackendSelection,
    ) -> Self {
        Self {
            demo,
            counter_value,
            scale,
            target_fps,
            host_loop_policy,
            wait_backend: NativeWindowRunLoopWaitBackend::PlatformWait(selection),
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

pub fn validate_minifb_window_run_loop_wait_backend(
    requested: NativeWindowRunLoopWaitBackend,
    target_fps: NativeWindowTargetFps,
) -> Result<
    NativeWindowFrameIntervalWaitAuthorityMode,
    NativeWindowRunLoopFrameIntervalWaitBackendError,
> {
    let active_authority =
        NativeWindowRunLoopWaitBackend::MinifbInternalTargetFps.authority_mode(target_fps);
    let requested_authority = requested.authority_mode(target_fps);
    combine_native_window_frame_interval_wait_authority_mode(active_authority, requested_authority)
        .map_err(
            |reason| NativeWindowRunLoopFrameIntervalWaitBackendError::Unsupported {
                runner: NativeWindowRunLoopFrameIntervalWaitBackendRunner::Minifb,
                requested,
                reason,
            },
        )
}

pub fn validate_minifb_window_run_loop_frame_interval_wait_backend(
    requested: NativeWindowRunLoopFrameIntervalWaitBackend,
    target_fps: NativeWindowTargetFps,
) -> Result<
    NativeWindowFrameIntervalWaitAuthorityMode,
    NativeWindowRunLoopFrameIntervalWaitBackendError,
> {
    validate_minifb_window_run_loop_wait_backend(
        NativeWindowRunLoopWaitBackend::from(requested),
        target_fps,
    )
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

pub fn native_window_host_loop_wait_instruction_plan(
    strategy_state: NativeWindowHostLoopWaitStrategyState,
    request: NativeWindowHostLoopWaitRequest,
) -> NativeWindowHostLoopWaitInstructionPlan {
    match request {
        NativeWindowHostLoopWaitRequest::WaitForHostEvent {
            window_size,
            size_changed,
        } => NativeWindowHostLoopWaitInstructionPlan {
            next_strategy_state: strategy_state,
            instruction: NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
                window_size,
                size_changed,
            },
        },
        NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed,
            frame_interval,
        } => {
            let target_fps = frame_interval.target_fps();
            let target_fps_value = u32::from(target_fps.value());
            let carried_remainder = if strategy_state.frame_pacing_target_fps() == Some(target_fps)
            {
                strategy_state.frame_pacing_remainder_nanos()
            } else {
                0
            };
            let combined_remainder =
                carried_remainder + frame_interval.remainder_nanos_per_second();
            let (wait_nanos, next_remainder) = if combined_remainder >= target_fps_value {
                (
                    frame_interval.nanos_per_frame() + 1,
                    combined_remainder - target_fps_value,
                )
            } else {
                (frame_interval.nanos_per_frame(), combined_remainder)
            };
            NativeWindowHostLoopWaitInstructionPlan {
                next_strategy_state: NativeWindowHostLoopWaitStrategyState {
                    frame_pacing_target_fps: Some(target_fps),
                    frame_pacing_remainder_nanos: next_remainder,
                },
                instruction: NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                    presentation,
                    window_size,
                    size_changed,
                    frame_interval,
                    wait_nanos,
                },
            }
        }
    }
}

pub fn native_window_host_loop_scheduler_resume_state_from_wait_outcome(
    outcome: NativeWindowHostLoopWaitOutcome,
) -> NativeWindowHostLoopSchedulerResumeState {
    match outcome {
        NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
            window_size,
            size_changed,
        } => NativeWindowHostLoopSchedulerResumeState::Ready(
            NativeWindowHostLoopSchedulerResumeReady::HostEventPumped {
                window_size,
                size_changed,
            },
        ),
        NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
            presentation,
            window_size,
            size_changed,
        } => NativeWindowHostLoopSchedulerResumeState::Ready(
            NativeWindowHostLoopSchedulerResumeReady::FramePresentPaced {
                presentation,
                window_size,
                size_changed,
            },
        ),
        NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        } => NativeWindowHostLoopSchedulerResumeState::WaitingForFrameIntervalTimer {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        },
        NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        } => NativeWindowHostLoopSchedulerResumeState::Ready(
            NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed,
                wait_nanos,
                timer_registration_id,
            },
        ),
    }
}

pub fn native_window_host_loop_scheduler_resume_ready_from_timer_fire(
    outcome: NativeWindowHostLoopTimerFireOutcome,
) -> NativeWindowHostLoopSchedulerResumeReady {
    match outcome {
        NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
        } => NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired {
            presentation,
            window_size,
            size_changed,
            wait_nanos,
            timer_registration_id,
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
            let instruction_plan = native_window_host_loop_wait_instruction_plan(
                scheduler_state.wait_strategy_state,
                request.clone(),
            );
            let outcome = host
                .wait_after_budget_exhausted(instruction_plan.instruction.clone())
                .map_err(NativeWindowHostLoopError::HostWaitFailed)?;
            scheduler_state.wait_strategy_state = instruction_plan.next_strategy_state;
            Ok(NativeWindowHostLoopSchedulerSliceResult::Waited {
                completed_turns,
                decision,
                request,
                instruction: instruction_plan.instruction,
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
            NativeWindowHostLoopSchedulerSliceResult::Waited { outcome, .. } => {
                match native_window_host_loop_scheduler_resume_state_from_wait_outcome(outcome) {
                    NativeWindowHostLoopSchedulerResumeState::Ready(_) => {}
                    NativeWindowHostLoopSchedulerResumeState::WaitingForFrameIntervalTimer {
                        presentation,
                        window_size,
                        size_changed,
                        wait_nanos,
                        timer_registration_id,
                    } => {
                        return Err(NativeWindowHostLoopError::TimerFireResumeRequired {
                            presentation,
                            window_size,
                            size_changed,
                            wait_nanos,
                            timer_registration_id,
                        });
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowFrameIntervalWaitAuthorityMode {
    MinifbInternalTargetFps { target_fps: NativeWindowTargetFps },
    HostOwnedDeadlineTimer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowFrameIntervalWaitAuthorityModeError {
    ConflictingFrameIntervalAuthorities {
        active: NativeWindowFrameIntervalWaitAuthorityMode,
        requested: NativeWindowFrameIntervalWaitAuthorityMode,
    },
    TargetFpsMismatch {
        authority_target_fps: NativeWindowTargetFps,
        instruction_target_fps: NativeWindowTargetFps,
    },
}

pub fn native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(
    target_fps: NativeWindowTargetFps,
) -> NativeWindowFrameIntervalWaitAuthorityMode {
    NativeWindowFrameIntervalWaitAuthorityMode::MinifbInternalTargetFps { target_fps }
}

pub fn native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer(
) -> NativeWindowFrameIntervalWaitAuthorityMode {
    NativeWindowFrameIntervalWaitAuthorityMode::HostOwnedDeadlineTimer
}

pub fn combine_native_window_frame_interval_wait_authority_mode(
    active: NativeWindowFrameIntervalWaitAuthorityMode,
    requested: NativeWindowFrameIntervalWaitAuthorityMode,
) -> Result<
    NativeWindowFrameIntervalWaitAuthorityMode,
    NativeWindowFrameIntervalWaitAuthorityModeError,
> {
    match (active, requested) {
        (
            NativeWindowFrameIntervalWaitAuthorityMode::MinifbInternalTargetFps {
                target_fps: active_target_fps,
            },
            NativeWindowFrameIntervalWaitAuthorityMode::MinifbInternalTargetFps {
                target_fps: requested_target_fps,
            },
        ) if active_target_fps == requested_target_fps => Ok(active),
        (
            NativeWindowFrameIntervalWaitAuthorityMode::HostOwnedDeadlineTimer,
            NativeWindowFrameIntervalWaitAuthorityMode::HostOwnedDeadlineTimer,
        ) => Ok(active),
        _ => Err(
            NativeWindowFrameIntervalWaitAuthorityModeError::ConflictingFrameIntervalAuthorities {
                active,
                requested,
            },
        ),
    }
}

pub fn validate_native_window_frame_interval_wait_authority_mode(
    authority_mode: NativeWindowFrameIntervalWaitAuthorityMode,
    frame_interval: NativeWindowFrameIntervalRequest,
) -> Result<(), NativeWindowFrameIntervalWaitAuthorityModeError> {
    match authority_mode {
        NativeWindowFrameIntervalWaitAuthorityMode::MinifbInternalTargetFps { target_fps } => {
            let instruction_target_fps = frame_interval.target_fps();
            if instruction_target_fps != target_fps {
                return Err(
                    NativeWindowFrameIntervalWaitAuthorityModeError::TargetFpsMismatch {
                        authority_target_fps: target_fps,
                        instruction_target_fps,
                    },
                );
            }
            Ok(())
        }
        NativeWindowFrameIntervalWaitAuthorityMode::HostOwnedDeadlineTimer => Ok(()),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeWindowMinifbFramePacingAuthority {
    target_fps: NativeWindowTargetFps,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeWindowMinifbFramePacingAuthorityError {
    FrameIntervalAuthorityConflict {
        active: NativeWindowFrameIntervalWaitAuthorityMode,
        requested: NativeWindowFrameIntervalWaitAuthorityMode,
    },
    FrameIntervalTargetFpsMismatch {
        authority_target_fps: NativeWindowTargetFps,
        instruction_target_fps: NativeWindowTargetFps,
    },
    FrameIntervalWaitNanosMismatch {
        wait_nanos: u32,
        nanos_per_frame: u32,
    },
}

impl NativeWindowMinifbFramePacingAuthority {
    pub fn new(target_fps: NativeWindowTargetFps) -> Self {
        Self { target_fps }
    }

    pub fn target_fps(self) -> NativeWindowTargetFps {
        self.target_fps
    }

    pub fn target_fps_usize(self) -> usize {
        self.target_fps.as_usize()
    }

    pub fn frame_interval_wait_authority_mode(self) -> NativeWindowFrameIntervalWaitAuthorityMode {
        native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(self.target_fps)
    }

    pub fn frame_interval_wait_outcome(
        self,
        presentation: NativeWindowBackendLoopPresentation,
        window_size: NativeWindowSize,
        size_changed: bool,
        frame_interval: NativeWindowFrameIntervalRequest,
        wait_nanos: u32,
    ) -> Result<NativeWindowHostLoopWaitOutcome, NativeWindowMinifbFramePacingAuthorityError> {
        if let Err(error) = validate_native_window_frame_interval_wait_authority_mode(
            self.frame_interval_wait_authority_mode(),
            frame_interval,
        ) {
            return Err(match error {
                NativeWindowFrameIntervalWaitAuthorityModeError::ConflictingFrameIntervalAuthorities {
                    active,
                    requested,
                } => NativeWindowMinifbFramePacingAuthorityError::FrameIntervalAuthorityConflict {
                    active,
                    requested,
                },
                NativeWindowFrameIntervalWaitAuthorityModeError::TargetFpsMismatch {
                    authority_target_fps,
                    instruction_target_fps,
                } => NativeWindowMinifbFramePacingAuthorityError::FrameIntervalTargetFpsMismatch {
                    authority_target_fps,
                    instruction_target_fps,
                },
            });
        }
        let nanos_per_frame = frame_interval.nanos_per_frame();
        if wait_nanos != nanos_per_frame && wait_nanos != nanos_per_frame + 1 {
            return Err(
                NativeWindowMinifbFramePacingAuthorityError::FrameIntervalWaitNanosMismatch {
                    wait_nanos,
                    nanos_per_frame,
                },
            );
        }
        Ok(NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
            presentation,
            window_size,
            size_changed,
        })
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
#[derive(Debug)]
struct MinifbNativeWindowHostLoopMessagePumpAdapter<'window> {
    window: &'window mut minifb::Window,
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
impl NativeWindowHostLoopMessagePumpAdapter for MinifbNativeWindowHostLoopMessagePumpAdapter<'_> {
    type Error = std::convert::Infallible;

    fn pump_host_messages(
        &mut self,
        _window_size: NativeWindowSize,
        _size_changed: bool,
    ) -> Result<(), Self::Error> {
        self.window.update();
        Ok(())
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
type MinifbNativeWindowHostLoopMessagePumpWaitError = NativeWindowHostLoopEventQueueWaitError<
    NativeWindowHostLoopEventQueueStatusAdapterError<
        NativeWindowHostLoopMessagePumpStatusAdapterError<std::convert::Infallible>,
    >,
>;

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
#[derive(Debug)]
enum MinifbNativeWindowHostLoopWaitError {
    EventQueueWaitFailed(MinifbNativeWindowHostLoopMessagePumpWaitError),
    FramePacingAuthorityFailed(NativeWindowMinifbFramePacingAuthorityError),
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
#[derive(Clone, Debug, Eq, PartialEq)]
enum MinifbNativeWindowVisualHostWaitError {
    VisualHostWaitUnsupported {
        instruction: NativeWindowHostLoopWaitInstruction,
    },
}

#[cfg(all(feature = "window", target_os = "linux", not(target_arch = "wasm32")))]
#[derive(Debug)]
/// Stores the Linux host-event signal producer used by minifb observed input callbacks.
pub struct MinifbNativeWindowLinuxHostEventSignalCallbackState<
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi,
> {
    producer: NativeWindowHostLoopLinuxHostEventSignalProducer<Api>,
    first_error: Option<NativeWindowHostLoopLinuxHostEventSignalProducerError>,
}

#[cfg(all(feature = "window", target_os = "linux", not(target_arch = "wasm32")))]
impl<Api> MinifbNativeWindowLinuxHostEventSignalCallbackState<Api>
where
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi,
{
    pub fn new(producer: NativeWindowHostLoopLinuxHostEventSignalProducer<Api>) -> Self {
        Self {
            producer,
            first_error: None,
        }
    }

    #[cfg(test)]
    fn producer(&self) -> &NativeWindowHostLoopLinuxHostEventSignalProducer<Api> {
        &self.producer
    }

    pub fn signal_observed_input(&mut self) {
        if self.first_error.is_some() {
            return;
        }
        if let Err(error) = self.producer.signal_host_event() {
            self.first_error = Some(error);
        }
    }

    pub fn take_first_error(
        &mut self,
    ) -> Option<NativeWindowHostLoopLinuxHostEventSignalProducerError> {
        self.first_error.take()
    }
}

#[cfg(all(feature = "window", target_os = "linux", not(target_arch = "wasm32")))]
impl<Api> NativeWindowHostEventSignalErrorState
    for std::rc::Rc<std::cell::RefCell<MinifbNativeWindowLinuxHostEventSignalCallbackState<Api>>>
where
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi,
{
    fn take_host_event_signal_error(
        &mut self,
    ) -> Option<NativeWindowHostLoopLinuxHostEventSignalProducerError> {
        self.borrow_mut().take_first_error()
    }
}

#[cfg(all(feature = "window", target_os = "linux", not(target_arch = "wasm32")))]
/// Signals a Linux host-event fd when minifb reports already observed keyboard or text input.
pub struct MinifbNativeWindowLinuxHostEventSignalInputCallback<
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi,
> {
    state:
        std::rc::Rc<std::cell::RefCell<MinifbNativeWindowLinuxHostEventSignalCallbackState<Api>>>,
}

#[cfg(all(feature = "window", target_os = "linux", not(target_arch = "wasm32")))]
impl<Api> MinifbNativeWindowLinuxHostEventSignalInputCallback<Api>
where
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi,
{
    pub fn new(
        state: std::rc::Rc<
            std::cell::RefCell<MinifbNativeWindowLinuxHostEventSignalCallbackState<Api>>,
        >,
    ) -> Self {
        Self { state }
    }
}

#[cfg(all(feature = "window", target_os = "linux", not(target_arch = "wasm32")))]
impl<Api> minifb::InputCallback for MinifbNativeWindowLinuxHostEventSignalInputCallback<Api>
where
    Api: NativeWindowHostLoopLinuxHostEventSignalRawApi + 'static,
{
    fn add_char(&mut self, _uni_char: u32) {
        self.state.borrow_mut().signal_observed_input();
    }

    fn set_key_state(&mut self, _key: minifb::Key, _state: bool) {
        self.state.borrow_mut().signal_observed_input();
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn wait_minifb_window_host_event_message_pump(
    window: &mut minifb::Window,
    window_size: NativeWindowSize,
    size_changed: bool,
) -> Result<NativeWindowHostLoopWaitOutcome, MinifbNativeWindowHostLoopMessagePumpWaitError> {
    let adapter = NativeWindowHostLoopMessagePumpStatusAdapter::new(
        MinifbNativeWindowHostLoopMessagePumpAdapter { window },
    );
    let mut waiter = NativeWindowHostLoopEventQueueStatusWaiter::new(adapter);
    let event_queue_outcome = execute_native_window_host_loop_event_queue_wait_with_waiter(
        NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed,
        },
        &mut waiter,
    )?;
    match event_queue_outcome {
        NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady {
            window_size,
            size_changed,
        } => Ok(NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
            window_size,
            size_changed,
        }),
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
struct MinifbNativeWindowVisualRunLoopHost<'window> {
    window: &'window mut minifb::Window,
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
impl NativeWindowRunLoopHost for MinifbNativeWindowVisualRunLoopHost<'_> {
    type EventError = NativeWindowEventPumpError;
    type PresentError = String;
    type WaitError = MinifbNativeWindowVisualHostWaitError;

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
        instruction: NativeWindowHostLoopWaitInstruction,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
        Err(MinifbNativeWindowVisualHostWaitError::VisualHostWaitUnsupported { instruction })
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
struct MinifbNativeWindowRunLoopHost<'window> {
    window: &'window mut minifb::Window,
    frame_pacing_authority: NativeWindowMinifbFramePacingAuthority,
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
impl NativeWindowRunLoopHost for MinifbNativeWindowRunLoopHost<'_> {
    type EventError = NativeWindowEventPumpError;
    type PresentError = String;
    type WaitError = MinifbNativeWindowHostLoopWaitError;

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
        instruction: NativeWindowHostLoopWaitInstruction,
    ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
        Ok(match instruction {
            NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
                window_size,
                size_changed,
            } => wait_minifb_window_host_event_message_pump(self.window, window_size, size_changed)
                .map_err(MinifbNativeWindowHostLoopWaitError::EventQueueWaitFailed)?,
            NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size,
                size_changed,
                frame_interval,
                wait_nanos,
            } => self
                .frame_pacing_authority
                .frame_interval_wait_outcome(
                    presentation,
                    window_size,
                    size_changed,
                    frame_interval,
                    wait_nanos,
                )
                .map_err(MinifbNativeWindowHostLoopWaitError::FramePacingAuthorityFailed)?,
        })
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn configure_minifb_window_frame_pacing(
    window: &mut minifb::Window,
    authority: NativeWindowMinifbFramePacingAuthority,
) {
    let target_fps = authority.target_fps_usize();
    window.set_target_fps(target_fps);
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn minifb_native_window_frame_pacing_authority(
    target_fps: NativeWindowTargetFps,
) -> NativeWindowMinifbFramePacingAuthority {
    NativeWindowMinifbFramePacingAuthority::new(target_fps)
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn minifb_native_window_host_loop_wait_error_message(
    error: MinifbNativeWindowHostLoopWaitError,
) -> String {
    match error {
        MinifbNativeWindowHostLoopWaitError::EventQueueWaitFailed(error) => {
            format!("EventQueueWaitFailed({error:?})")
        }
        MinifbNativeWindowHostLoopWaitError::FramePacingAuthorityFailed(error) => {
            format!("FramePacingAuthorityFailed({error:?})")
        }
    }
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
pub fn run_minifb_window_loop(
    config: NativeWindowRunLoopConfig,
) -> Result<NativeWindowRunLoopExit, NativeWindowRunLoopError> {
    use minifb::{ScaleMode, Window, WindowOptions};

    validate_minifb_window_run_loop_wait_backend(config.wait_backend, config.target_fps)
        .map_err(NativeWindowRunLoopError::FrameIntervalWaitBackendUnsupported)?;
    let frame_pacing_authority = minifb_native_window_frame_pacing_authority(config.target_fps);
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
    configure_minifb_window_frame_pacing(&mut window, frame_pacing_authority);
    window.set_background_color(9, 13, 18);

    let mut host = MinifbNativeWindowRunLoopHost {
        window: &mut window,
        frame_pacing_authority,
    };
    run_native_window_host_loop_with_policy_and_target_fps(
        &mut backend_loop,
        &mut host,
        config.host_loop_policy,
        config.target_fps,
    )
    .map_err(native_window_run_loop_error_from_host_loop)
}

#[cfg(all(feature = "window", target_os = "windows", not(target_arch = "wasm32")))]
pub fn run_windows_platform_wait_window_loop(
    config: NativeWindowRunLoopConfig,
) -> Result<NativeWindowRunLoopExit, NativeWindowRunLoopError> {
    use minifb::{ScaleMode, Window, WindowOptions};

    let platform_wait_backend = native_window_run_loop_platform_wait_backend_from_config(config)
        .map_err(NativeWindowRunLoopError::PlatformWaitBackendFromConfigFailed)?;
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
    window.set_background_color(9, 13, 18);

    let visual_host = MinifbNativeWindowVisualRunLoopHost {
        window: &mut window,
    };
    let mut host = native_window_host_loop_platform_wait_run_loop_host_from_backend(
        visual_host,
        platform_wait_backend,
    );
    run_native_window_host_loop_with_policy_and_target_fps(
        &mut backend_loop,
        &mut host,
        config.host_loop_policy,
        config.target_fps,
    )
    .map_err(NativeWindowRunLoopError::WindowsPlatformWaitHostLoopFailed)
}

#[cfg(all(feature = "window", not(target_arch = "wasm32")))]
fn native_window_run_loop_error_from_host_loop(
    error: NativeWindowHostLoopError<
        NativeWindowEventPumpError,
        String,
        MinifbNativeWindowHostLoopWaitError,
    >,
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
        NativeWindowHostLoopError::HostWaitFailed(error) => {
            NativeWindowRunLoopError::HostWaitFailed {
                message: minifb_native_window_host_loop_wait_error_message(error),
            }
        }
        NativeWindowHostLoopError::TimerFireResumeRequired {
            timer_registration_id,
            ..
        } => NativeWindowRunLoopError::TimerFireResumeRequired {
            timer_registration_id: timer_registration_id.raw_id(),
        },
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
    fn native_window_frame_interval_wait_authority_combines_same_minifb_target() {
        let target_fps = NativeWindowTargetFps::default();
        let active =
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps);
        let requested =
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps);

        assert_eq!(
            combine_native_window_frame_interval_wait_authority_mode(active, requested).unwrap(),
            active
        );
    }

    #[test]
    fn native_window_frame_interval_wait_authority_rejects_minifb_and_deadline_conflict() {
        let target_fps = NativeWindowTargetFps::default();
        let minifb =
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps);
        let deadline = native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer();

        assert_eq!(
            combine_native_window_frame_interval_wait_authority_mode(minifb, deadline).unwrap_err(),
            NativeWindowFrameIntervalWaitAuthorityModeError::ConflictingFrameIntervalAuthorities {
                active: minifb,
                requested: deadline,
            }
        );
        assert_eq!(
            combine_native_window_frame_interval_wait_authority_mode(deadline, minifb).unwrap_err(),
            NativeWindowFrameIntervalWaitAuthorityModeError::ConflictingFrameIntervalAuthorities {
                active: deadline,
                requested: minifb,
            }
        );
    }

    #[test]
    fn native_window_frame_interval_wait_authority_rejects_minifb_target_mismatch() {
        let active_target_fps = NativeWindowTargetFps::default();
        let requested_target_fps = NativeWindowTargetFps::new(120).unwrap();
        let active = native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(
            active_target_fps,
        );
        let requested = native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(
            requested_target_fps,
        );

        assert_eq!(
            combine_native_window_frame_interval_wait_authority_mode(active, requested)
                .unwrap_err(),
            NativeWindowFrameIntervalWaitAuthorityModeError::ConflictingFrameIntervalAuthorities {
                active,
                requested,
            }
        );
    }

    #[test]
    fn native_window_frame_interval_wait_authority_validates_minifb_instruction_target_fps() {
        let authority_target_fps = NativeWindowTargetFps::default();
        let instruction_target_fps = NativeWindowTargetFps::new(120).unwrap();
        let authority_mode =
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(
                authority_target_fps,
            );
        let frame_interval = native_window_frame_interval_request(instruction_target_fps);

        assert_eq!(
            validate_native_window_frame_interval_wait_authority_mode(
                authority_mode,
                frame_interval
            )
            .unwrap_err(),
            NativeWindowFrameIntervalWaitAuthorityModeError::TargetFpsMismatch {
                authority_target_fps,
                instruction_target_fps,
            }
        );
    }

    #[test]
    fn native_window_frame_interval_wait_authority_validates_host_owned_deadline_timer() {
        let authority_mode =
            native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer();
        let frame_interval =
            native_window_frame_interval_request(NativeWindowTargetFps::new(120).unwrap());

        assert_eq!(
            validate_native_window_frame_interval_wait_authority_mode(
                authority_mode,
                frame_interval
            )
            .unwrap(),
            ()
        );
    }

    #[test]
    fn native_window_minifb_frame_pacing_authority_accepts_matching_frame_interval() {
        let target_fps = NativeWindowTargetFps::default();
        let authority = NativeWindowMinifbFramePacingAuthority::new(target_fps);
        let frame_interval = native_window_frame_interval_request(target_fps);
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 4,
            width: window_size.width,
            height: window_size.height,
        };

        assert_eq!(authority.target_fps(), target_fps);
        assert_eq!(authority.target_fps_usize(), target_fps.as_usize());
        assert_eq!(
            authority.frame_interval_wait_authority_mode(),
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps)
        );
        assert_eq!(
            authority
                .frame_interval_wait_outcome(
                    presentation,
                    window_size,
                    true,
                    frame_interval,
                    frame_interval.nanos_per_frame(),
                )
                .unwrap(),
            NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
                presentation,
                window_size,
                size_changed: true,
            }
        );
    }

    #[test]
    fn native_window_minifb_frame_pacing_authority_accepts_remainder_carry_wait_nanos() {
        let target_fps = NativeWindowTargetFps::default();
        let authority = NativeWindowMinifbFramePacingAuthority::new(target_fps);
        let frame_interval = native_window_frame_interval_request(target_fps);
        let window_size = NativeWindowSize::new(640, 480);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 5,
            width: window_size.width,
            height: window_size.height,
        };

        assert_eq!(
            authority
                .frame_interval_wait_outcome(
                    presentation,
                    window_size,
                    false,
                    frame_interval,
                    frame_interval.nanos_per_frame() + 1,
                )
                .unwrap(),
            NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
                presentation,
                window_size,
                size_changed: false,
            }
        );
    }

    #[test]
    fn native_window_minifb_frame_pacing_authority_rejects_target_fps_mismatch() {
        let authority_target_fps = NativeWindowTargetFps::default();
        let instruction_target_fps = NativeWindowTargetFps::new(120).unwrap();
        let authority = NativeWindowMinifbFramePacingAuthority::new(authority_target_fps);
        let frame_interval = native_window_frame_interval_request(instruction_target_fps);
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 6,
            width: window_size.width,
            height: window_size.height,
        };

        assert_eq!(
            authority
                .frame_interval_wait_outcome(
                    presentation,
                    window_size,
                    false,
                    frame_interval,
                    frame_interval.nanos_per_frame(),
                )
                .unwrap_err(),
            NativeWindowMinifbFramePacingAuthorityError::FrameIntervalTargetFpsMismatch {
                authority_target_fps,
                instruction_target_fps,
            }
        );
    }

    #[test]
    fn native_window_minifb_frame_pacing_authority_rejects_invalid_wait_nanos() {
        let target_fps = NativeWindowTargetFps::default();
        let authority = NativeWindowMinifbFramePacingAuthority::new(target_fps);
        let frame_interval = native_window_frame_interval_request(target_fps);
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 7,
            width: window_size.width,
            height: window_size.height,
        };
        let wait_nanos = frame_interval.nanos_per_frame() + 2;

        assert_eq!(
            authority
                .frame_interval_wait_outcome(
                    presentation,
                    window_size,
                    false,
                    frame_interval,
                    wait_nanos,
                )
                .unwrap_err(),
            NativeWindowMinifbFramePacingAuthorityError::FrameIntervalWaitNanosMismatch {
                wait_nanos,
                nanos_per_frame: frame_interval.nanos_per_frame(),
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
                wait_backend: NativeWindowRunLoopWaitBackend::MinifbInternalTargetFps,
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
                wait_backend: NativeWindowRunLoopWaitBackend::MinifbInternalTargetFps,
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
                wait_backend: NativeWindowRunLoopWaitBackend::MinifbInternalTargetFps,
            }
        );
        assert_eq!(
            NativeWindowRunLoopConfig::new_with_frame_interval_wait_backend(
                GuiDemo::Mandelbrot,
                0,
                1,
                custom_fps,
                custom_policy,
                NativeWindowRunLoopFrameIntervalWaitBackend::HostOwnedDeadlineTimer,
            ),
            NativeWindowRunLoopConfig {
                demo: GuiDemo::Mandelbrot,
                counter_value: 0,
                scale: 1,
                target_fps: custom_fps,
                host_loop_policy: custom_policy,
                wait_backend: NativeWindowRunLoopWaitBackend::HostOwnedDeadlineTimer,
            }
        );
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        assert_eq!(
            NativeWindowRunLoopConfig::new_with_platform_wait_backend_selection(
                GuiDemo::Mandelbrot,
                0,
                1,
                custom_fps,
                custom_policy,
                selection,
            ),
            NativeWindowRunLoopConfig {
                demo: GuiDemo::Mandelbrot,
                counter_value: 0,
                scale: 1,
                target_fps: custom_fps,
                host_loop_policy: custom_policy,
                wait_backend: NativeWindowRunLoopWaitBackend::PlatformWait(selection),
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
    fn native_window_run_loop_frame_interval_backend_maps_to_authority_mode() {
        let target_fps = NativeWindowTargetFps::new(120).unwrap();
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();

        assert_eq!(
            NativeWindowRunLoopFrameIntervalWaitBackend::MinifbInternalTargetFps
                .authority_mode(target_fps),
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps)
        );
        assert_eq!(
            NativeWindowRunLoopFrameIntervalWaitBackend::HostOwnedDeadlineTimer
                .authority_mode(target_fps),
            native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer()
        );
        assert_eq!(
            NativeWindowRunLoopWaitBackend::from(
                NativeWindowRunLoopFrameIntervalWaitBackend::HostOwnedDeadlineTimer
            ),
            NativeWindowRunLoopWaitBackend::HostOwnedDeadlineTimer
        );
        assert_eq!(
            NativeWindowRunLoopWaitBackend::PlatformWait(selection).authority_mode(target_fps),
            native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer()
        );
    }

    #[test]
    fn native_window_run_loop_platform_wait_config_extracts_only_platform_selection() {
        let target_fps = NativeWindowTargetFps::default();
        let policy = NativeWindowHostLoopRunPolicy::default();
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        let config = NativeWindowRunLoopConfig::new_with_platform_wait_backend_selection(
            GuiDemo::Counter,
            0,
            1,
            target_fps,
            policy,
            selection,
        );

        assert_eq!(
            native_window_run_loop_platform_wait_backend_selection(config).unwrap(),
            selection
        );
        assert_eq!(
            native_window_run_loop_platform_wait_backend_selection(NativeWindowRunLoopConfig::new(
                GuiDemo::Counter,
                0,
                1,
            ))
            .unwrap_err(),
            NativeWindowRunLoopPlatformWaitBackendConfigError::NotPlatformWaitBackend {
                requested: NativeWindowRunLoopWaitBackend::MinifbInternalTargetFps,
            }
        );
    }

    #[test]
    fn native_window_minifb_run_loop_backend_validation_rejects_host_owned_deadline_timer() {
        let target_fps = NativeWindowTargetFps::default();
        let requested = NativeWindowRunLoopWaitBackend::HostOwnedDeadlineTimer;
        let active_authority =
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps);
        let requested_authority =
            native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer();

        assert_eq!(
            validate_minifb_window_run_loop_wait_backend(requested, target_fps).unwrap_err(),
            NativeWindowRunLoopFrameIntervalWaitBackendError::Unsupported {
                runner: NativeWindowRunLoopFrameIntervalWaitBackendRunner::Minifb,
                requested,
                reason:
                    NativeWindowFrameIntervalWaitAuthorityModeError::ConflictingFrameIntervalAuthorities {
                        active: active_authority,
                        requested: requested_authority,
                    },
            }
        );
    }

    #[test]
    fn native_window_minifb_run_loop_backend_validation_rejects_platform_wait_backend() {
        let target_fps = NativeWindowTargetFps::default();
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        let requested = NativeWindowRunLoopWaitBackend::PlatformWait(selection);
        let active_authority =
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps);
        let requested_authority =
            native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer();

        assert_eq!(
            validate_minifb_window_run_loop_wait_backend(requested, target_fps).unwrap_err(),
            NativeWindowRunLoopFrameIntervalWaitBackendError::Unsupported {
                runner: NativeWindowRunLoopFrameIntervalWaitBackendRunner::Minifb,
                requested,
                reason:
                    NativeWindowFrameIntervalWaitAuthorityModeError::ConflictingFrameIntervalAuthorities {
                        active: active_authority,
                        requested: requested_authority,
                    },
            }
        );
    }

    #[test]
    fn native_window_minifb_run_loop_backend_validation_accepts_minifb_internal_pacing() {
        let target_fps = NativeWindowTargetFps::default();
        let requested = NativeWindowRunLoopWaitBackend::MinifbInternalTargetFps;

        assert_eq!(
            validate_minifb_window_run_loop_wait_backend(requested, target_fps).unwrap(),
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps)
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
    fn native_window_host_loop_wait_instruction_distributes_frame_remainder() {
        let target_fps = NativeWindowTargetFps::default();
        let window_size = NativeWindowSize::new(640, 480);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 3,
            width: window_size.width,
            height: window_size.height,
        };
        let request = NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(target_fps),
        };

        let plan1 = native_window_host_loop_wait_instruction_plan(
            NativeWindowHostLoopWaitStrategyState::new(),
            request.clone(),
        );
        assert_eq!(
            plan1.next_strategy_state.frame_pacing_target_fps(),
            Some(target_fps)
        );
        assert_eq!(plan1.next_strategy_state.frame_pacing_remainder_nanos(), 40);
        assert_eq!(
            plan1.instruction,
            NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(target_fps),
                wait_nanos: 16_666_666,
            }
        );

        let plan2 = native_window_host_loop_wait_instruction_plan(
            plan1.next_strategy_state,
            request.clone(),
        );
        assert_eq!(plan2.next_strategy_state.frame_pacing_remainder_nanos(), 20);
        assert_eq!(
            plan2.instruction,
            NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(target_fps),
                wait_nanos: 16_666_667,
            }
        );

        let plan3 =
            native_window_host_loop_wait_instruction_plan(plan2.next_strategy_state, request);
        assert_eq!(plan3.next_strategy_state.frame_pacing_remainder_nanos(), 0);
        assert_eq!(
            plan3.instruction,
            NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(target_fps),
                wait_nanos: 16_666_667,
            }
        );
    }

    #[test]
    fn native_window_host_loop_wait_instruction_resets_remainder_on_target_fps_change() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 4,
            width: window_size.width,
            height: window_size.height,
        };
        let first_target = NativeWindowTargetFps::default();
        let first_request = NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(first_target),
        };
        let first_plan = native_window_host_loop_wait_instruction_plan(
            NativeWindowHostLoopWaitStrategyState::new(),
            first_request,
        );
        assert_eq!(
            first_plan
                .next_strategy_state
                .frame_pacing_remainder_nanos(),
            40
        );

        let second_target = NativeWindowTargetFps::new(120).unwrap();
        let second_request = NativeWindowHostLoopWaitRequest::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(second_target),
        };
        let second_plan = native_window_host_loop_wait_instruction_plan(
            first_plan.next_strategy_state,
            second_request,
        );
        assert_eq!(
            second_plan.next_strategy_state.frame_pacing_target_fps(),
            Some(second_target)
        );
        assert_eq!(
            second_plan
                .next_strategy_state
                .frame_pacing_remainder_nanos(),
            40
        );
        assert_eq!(
            second_plan.instruction,
            NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(second_target),
                wait_nanos: 8_333_333,
            }
        );
    }

    #[test]
    fn native_window_host_loop_scheduler_resume_state_accepts_already_paced_waits() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 8,
            width: window_size.width,
            height: window_size.height,
        };

        assert_eq!(
            native_window_host_loop_scheduler_resume_state_from_wait_outcome(
                NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                    window_size,
                    size_changed: true,
                }
            ),
            NativeWindowHostLoopSchedulerResumeState::Ready(
                NativeWindowHostLoopSchedulerResumeReady::HostEventPumped {
                    window_size,
                    size_changed: true,
                }
            )
        );
        assert_eq!(
            native_window_host_loop_scheduler_resume_state_from_wait_outcome(
                NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
                    presentation,
                    window_size,
                    size_changed: false,
                }
            ),
            NativeWindowHostLoopSchedulerResumeState::Ready(
                NativeWindowHostLoopSchedulerResumeReady::FramePresentPaced {
                    presentation,
                    window_size,
                    size_changed: false,
                }
            )
        );
    }

    #[test]
    fn native_window_host_loop_scheduler_resume_state_requires_timer_fire() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 9,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 101 };

        assert_eq!(
            native_window_host_loop_scheduler_resume_state_from_wait_outcome(
                NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
                    presentation,
                    window_size,
                    size_changed: false,
                    wait_nanos: 16_666_666,
                    timer_registration_id,
                }
            ),
            NativeWindowHostLoopSchedulerResumeState::WaitingForFrameIntervalTimer {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id,
            }
        );
    }

    #[test]
    fn native_window_host_loop_scheduler_resume_ready_accepts_timer_fire_evidence() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 10,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 102 };

        assert_eq!(
            native_window_host_loop_scheduler_resume_ready_from_timer_fire(
                NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
                    presentation,
                    window_size,
                    size_changed: true,
                    wait_nanos: 16_666_667,
                    timer_registration_id,
                }
            ),
            NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_667,
                timer_registration_id,
            }
        );
    }

    #[test]
    fn native_window_host_loop_wait_outcome_preserves_timer_fire_evidence() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 11,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 104 };

        assert_eq!(
            native_window_host_loop_wait_outcome_from_timer_fire(
                NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
                    presentation,
                    window_size,
                    size_changed: true,
                    wait_nanos: 16_666_667,
                    timer_registration_id,
                }
            ),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_667,
                timer_registration_id,
            }
        );
    }

    #[test]
    fn native_window_host_loop_scheduler_resume_state_accepts_timer_fire_wait_outcome() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 12,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 105 };

        assert_eq!(
            native_window_host_loop_scheduler_resume_state_from_wait_outcome(
                NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                    presentation,
                    window_size,
                    size_changed: false,
                    wait_nanos: 16_666_666,
                    timer_registration_id,
                }
            ),
            NativeWindowHostLoopSchedulerResumeState::Ready(
                NativeWindowHostLoopSchedulerResumeReady::FrameIntervalTimerFired {
                    presentation,
                    window_size,
                    size_changed: false,
                    wait_nanos: 16_666_666,
                    timer_registration_id,
                }
            )
        );
    }

    #[test]
    fn native_window_thread_wait_sleeps_for_frame_interval_instruction() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 8,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut sleeper = ScriptedNativeWindowHostLoopThreadSleeper::new();

        assert_eq!(
            execute_native_window_host_loop_thread_wait_with_sleeper(instruction, &mut sleeper)
                .unwrap(),
            NativeWindowHostLoopThreadWaitOutcome::FrameIntervalSlept {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
            }
        );
        assert_eq!(sleeper.sleep_calls, vec![16_666_666]);
    }

    #[test]
    fn native_window_thread_wait_rejects_host_event_without_queue_backend() {
        let window_size = NativeWindowSize::new(0, 240);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let mut sleeper = ScriptedNativeWindowHostLoopThreadSleeper::new();

        assert_eq!(
            execute_native_window_host_loop_thread_wait_with_sleeper(instruction, &mut sleeper)
                .unwrap_err(),
            NativeWindowHostLoopThreadWaitError::HostEventWaitUnsupported {
                window_size,
                size_changed: true,
            }
        );
        assert!(sleeper.sleep_calls.is_empty());
    }

    #[test]
    fn native_window_thread_wait_rejects_invalid_wait_nanos_without_sleep() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 9,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 1,
        };
        let mut sleeper = ScriptedNativeWindowHostLoopThreadSleeper::new();

        assert_eq!(
            execute_native_window_host_loop_thread_wait_with_sleeper(instruction, &mut sleeper)
                .unwrap_err(),
            NativeWindowHostLoopThreadWaitError::FrameIntervalWaitNanosMismatch {
                wait_nanos: 1,
                nanos_per_frame: 16_666_666,
            }
        );
        assert!(sleeper.sleep_calls.is_empty());
    }

    #[test]
    fn native_window_thread_wait_preserves_sleeper_error() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 10,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let mut sleeper =
            ScriptedNativeWindowHostLoopThreadSleeper::new().with_error("sleep failed");

        assert_eq!(
            execute_native_window_host_loop_thread_wait_with_sleeper(instruction, &mut sleeper)
                .unwrap_err(),
            NativeWindowHostLoopThreadWaitError::SleeperFailed("sleep failed")
        );
        assert_eq!(sleeper.sleep_calls, vec![16_666_667]);
    }

    #[test]
    fn native_window_timer_registration_registers_frame_interval_instruction() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 11,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(42);

        assert_eq!(
            execute_native_window_host_loop_timer_registration_with_registrar(
                instruction,
                &mut registrar
            )
            .unwrap(),
            NativeWindowHostLoopTimerRegistrationOutcome::FrameIntervalTimerRegistered {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_667,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 42 },
            }
        );
        assert_eq!(registrar.registration_calls, vec![16_666_667]);
    }

    #[test]
    fn native_window_timer_registration_rejects_host_event_without_queue_backend() {
        let window_size = NativeWindowSize::new(640, 0);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(9);

        assert_eq!(
            execute_native_window_host_loop_timer_registration_with_registrar(
                instruction,
                &mut registrar
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerRegistrationError::HostEventTimerRegistrationUnsupported {
                window_size,
                size_changed: true,
            }
        );
        assert!(registrar.registration_calls.is_empty());
    }

    #[test]
    fn native_window_timer_registration_rejects_invalid_wait_nanos_without_registration() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 12,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 2,
        };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(9);

        assert_eq!(
            execute_native_window_host_loop_timer_registration_with_registrar(
                instruction,
                &mut registrar
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerRegistrationError::FrameIntervalWaitNanosMismatch {
                wait_nanos: 2,
                nanos_per_frame: 16_666_666,
            }
        );
        assert!(registrar.registration_calls.is_empty());
    }

    #[test]
    fn native_window_timer_registration_rejects_invalid_raw_timer_id() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 13,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(0);

        assert_eq!(
            execute_native_window_host_loop_timer_registration_with_registrar(
                instruction,
                &mut registrar
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerRegistrationError::InvalidTimerRegistrationId { raw_id: 0 }
        );
        assert_eq!(registrar.registration_calls, vec![16_666_666]);
    }

    #[test]
    fn native_window_timer_registration_preserves_registrar_error() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 14,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut registrar =
            ScriptedNativeWindowHostLoopTimerRegistrar::new(7).with_error("timer failed");

        assert_eq!(
            execute_native_window_host_loop_timer_registration_with_registrar(
                instruction,
                &mut registrar
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerRegistrationError::RegistrarFailed("timer failed")
        );
        assert_eq!(registrar.registration_calls, vec![16_666_666]);
    }

    #[test]
    fn native_window_timer_registration_wait_returns_timer_registered_outcome() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 15,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(77);

        assert_eq!(
            execute_native_window_host_loop_timer_registration_wait_with_registrar(
                instruction,
                &mut registrar
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_667,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 77 },
            }
        );
        assert_eq!(registrar.registration_calls, vec![16_666_667]);
    }

    #[test]
    fn native_window_timer_registration_wait_rejects_host_event_without_registration() {
        let window_size = NativeWindowSize::new(640, 0);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(9);

        assert_eq!(
            execute_native_window_host_loop_timer_registration_wait_with_registrar(
                instruction,
                &mut registrar
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerRegistrationError::HostEventTimerRegistrationUnsupported {
                window_size,
                size_changed: false,
            }
        );
        assert!(registrar.registration_calls.is_empty());
    }

    #[test]
    fn native_window_timer_registration_wait_preserves_registrar_error() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 16,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut registrar =
            ScriptedNativeWindowHostLoopTimerRegistrar::new(7).with_error("timer failed");

        assert_eq!(
            execute_native_window_host_loop_timer_registration_wait_with_registrar(
                instruction,
                &mut registrar
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerRegistrationError::RegistrarFailed("timer failed")
        );
        assert_eq!(registrar.registration_calls, vec![16_666_666]);
    }

    #[test]
    fn native_window_timer_fire_wait_accepts_matching_registered_timer() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 17,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 88 };
        let outcome = NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
            presentation,
            window_size,
            size_changed: true,
            wait_nanos: 16_666_667,
            timer_registration_id,
        };
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(88);

        assert_eq!(
            execute_native_window_host_loop_timer_fire_wait_with_waiter(outcome, &mut waiter)
                .unwrap(),
            NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_667,
                timer_registration_id,
            }
        );
        assert_eq!(waiter.wait_calls, vec![timer_registration_id]);
    }

    #[test]
    fn native_window_timer_fire_wait_preserves_already_fired_evidence_without_waiter_call() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 18,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 106 };
        let outcome = NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
            presentation,
            window_size,
            size_changed: true,
            wait_nanos: 16_666_667,
            timer_registration_id,
        };
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(106);

        assert_eq!(
            execute_native_window_host_loop_timer_fire_wait_with_waiter(outcome, &mut waiter)
                .unwrap(),
            NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_667,
                timer_registration_id,
            }
        );
        assert!(waiter.wait_calls.is_empty());
    }

    #[test]
    fn native_window_timer_fire_wait_rejects_host_event_outcome_without_waiter_call() {
        let window_size = NativeWindowSize::new(640, 0);
        let outcome = NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
            window_size,
            size_changed: false,
        };
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(88);

        assert_eq!(
            execute_native_window_host_loop_timer_fire_wait_with_waiter(outcome, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopTimerFireError::HostEventPumpOutcomeUnsupported {
                window_size,
                size_changed: false,
            }
        );
        assert!(waiter.wait_calls.is_empty());
    }

    #[test]
    fn native_window_timer_fire_wait_rejects_already_paced_present_without_waiter_call() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 18,
            width: window_size.width,
            height: window_size.height,
        };
        let outcome = NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
            presentation,
            window_size,
            size_changed: false,
        };
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(88);

        assert_eq!(
            execute_native_window_host_loop_timer_fire_wait_with_waiter(outcome, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopTimerFireError::FramePresentOutcomeUnsupported {
                presentation,
                window_size,
                size_changed: false,
            }
        );
        assert!(waiter.wait_calls.is_empty());
    }

    #[test]
    fn native_window_timer_fire_wait_preserves_waiter_error() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 19,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 89 };
        let outcome = NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
            presentation,
            window_size,
            size_changed: false,
            wait_nanos: 16_666_666,
            timer_registration_id,
        };
        let mut waiter =
            ScriptedNativeWindowHostLoopTimerFireWaiter::new(89).with_error("timer fire failed");

        assert_eq!(
            execute_native_window_host_loop_timer_fire_wait_with_waiter(outcome, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopTimerFireError::WaiterFailed("timer fire failed")
        );
        assert_eq!(waiter.wait_calls, vec![timer_registration_id]);
    }

    #[test]
    fn native_window_timer_fire_wait_rejects_invalid_fired_timer_id() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 20,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 90 };
        let outcome = NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
            presentation,
            window_size,
            size_changed: false,
            wait_nanos: 16_666_666,
            timer_registration_id,
        };
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(0);

        assert_eq!(
            execute_native_window_host_loop_timer_fire_wait_with_waiter(outcome, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopTimerFireError::InvalidFiredTimerRegistrationId { raw_id: 0 }
        );
        assert_eq!(waiter.wait_calls, vec![timer_registration_id]);
    }

    #[test]
    fn native_window_timer_fire_wait_rejects_mismatched_fired_timer_id() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 21,
            width: window_size.width,
            height: window_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 91 };
        let outcome = NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
            presentation,
            window_size,
            size_changed: false,
            wait_nanos: 16_666_666,
            timer_registration_id,
        };
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(92);

        assert_eq!(
            execute_native_window_host_loop_timer_fire_wait_with_waiter(outcome, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopTimerFireError::FiredTimerRegistrationMismatch {
                expected_raw_id: 91,
                actual_raw_id: 92,
            }
        );
        assert_eq!(waiter.wait_calls, vec![timer_registration_id]);
    }

    #[test]
    fn native_window_timer_wakeup_registers_then_waits_for_matching_timer() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 22,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 93 };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(93);
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(93);

        assert_eq!(
            execute_native_window_host_loop_timer_wakeup_with_backend(
                instruction,
                &mut registrar,
                &mut waiter
            )
            .unwrap(),
            NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_667,
                timer_registration_id,
            }
        );
        assert_eq!(registrar.registration_calls, vec![16_666_667]);
        assert_eq!(waiter.wait_calls, vec![timer_registration_id]);
    }

    #[test]
    fn native_window_timer_wakeup_wait_returns_timer_fired_wait_outcome() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 23,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 107 };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(107);
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(107);

        assert_eq!(
            execute_native_window_host_loop_timer_wakeup_wait_with_backend(
                instruction,
                &mut registrar,
                &mut waiter
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_667,
                timer_registration_id,
            }
        );
        assert_eq!(registrar.registration_calls, vec![16_666_667]);
        assert_eq!(waiter.wait_calls, vec![timer_registration_id]);
    }

    #[test]
    fn native_window_timer_wakeup_rejects_host_event_before_waiter_call() {
        let window_size = NativeWindowSize::new(640, 0);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(93);
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(93);

        assert_eq!(
            execute_native_window_host_loop_timer_wakeup_with_backend(
                instruction,
                &mut registrar,
                &mut waiter
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerWakeError::RegistrationFailed(
                NativeWindowHostLoopTimerRegistrationError::HostEventTimerRegistrationUnsupported {
                    window_size,
                    size_changed: true,
                }
            )
        );
        assert!(registrar.registration_calls.is_empty());
        assert!(waiter.wait_calls.is_empty());
    }

    #[test]
    fn native_window_timer_wakeup_preserves_registration_error_without_waiter_call() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 23,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut registrar =
            ScriptedNativeWindowHostLoopTimerRegistrar::new(93).with_error("timer failed");
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(93);

        assert_eq!(
            execute_native_window_host_loop_timer_wakeup_with_backend(
                instruction,
                &mut registrar,
                &mut waiter
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerWakeError::RegistrationFailed(
                NativeWindowHostLoopTimerRegistrationError::RegistrarFailed("timer failed")
            )
        );
        assert_eq!(registrar.registration_calls, vec![16_666_666]);
        assert!(waiter.wait_calls.is_empty());
    }

    #[test]
    fn native_window_timer_wakeup_rejects_invalid_registration_id_without_waiter_call() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 24,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(0);
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(93);

        assert_eq!(
            execute_native_window_host_loop_timer_wakeup_with_backend(
                instruction,
                &mut registrar,
                &mut waiter
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerWakeError::RegistrationFailed(
                NativeWindowHostLoopTimerRegistrationError::InvalidTimerRegistrationId {
                    raw_id: 0
                }
            )
        );
        assert_eq!(registrar.registration_calls, vec![16_666_666]);
        assert!(waiter.wait_calls.is_empty());
    }

    #[test]
    fn native_window_timer_wakeup_preserves_fire_waiter_error_after_registration() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 25,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 94 };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(94);
        let mut waiter =
            ScriptedNativeWindowHostLoopTimerFireWaiter::new(94).with_error("timer fire failed");

        assert_eq!(
            execute_native_window_host_loop_timer_wakeup_with_backend(
                instruction,
                &mut registrar,
                &mut waiter
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerWakeError::FireFailed(
                NativeWindowHostLoopTimerFireError::WaiterFailed("timer fire failed")
            )
        );
        assert_eq!(registrar.registration_calls, vec![16_666_666]);
        assert_eq!(waiter.wait_calls, vec![timer_registration_id]);
    }

    #[test]
    fn native_window_timer_wakeup_rejects_mismatched_fired_timer_after_registration() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 26,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 95 };
        let mut registrar = ScriptedNativeWindowHostLoopTimerRegistrar::new(95);
        let mut waiter = ScriptedNativeWindowHostLoopTimerFireWaiter::new(96);

        assert_eq!(
            execute_native_window_host_loop_timer_wakeup_with_backend(
                instruction,
                &mut registrar,
                &mut waiter
            )
            .unwrap_err(),
            NativeWindowHostLoopTimerWakeError::FireFailed(
                NativeWindowHostLoopTimerFireError::FiredTimerRegistrationMismatch {
                    expected_raw_id: 95,
                    actual_raw_id: 96,
                }
            )
        );
        assert_eq!(registrar.registration_calls, vec![16_666_666]);
        assert_eq!(waiter.wait_calls, vec![timer_registration_id]);
    }

    #[test]
    fn native_window_deadline_timer_adapter_registers_and_fires_frame_interval() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 27,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 1 };
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );

        assert_eq!(
            execute_native_window_host_loop_deadline_timer_wakeup_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap(),
            NativeWindowHostLoopTimerFireOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: true,
                wait_nanos: 16_666_666,
                timer_registration_id,
            }
        );
        assert_eq!(adapter.active_timer(), None);
        assert_eq!(adapter.next_raw_id(), 2);
        assert_eq!(adapter.clock.now_calls, 1);
        assert_eq!(adapter.sleeper.sleep_until_calls, vec![16_667_666]);
    }

    #[test]
    fn native_window_deadline_timer_adapter_returns_timer_fired_wait_outcome() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 28,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 1 };
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(2_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );

        assert_eq!(
            execute_native_window_host_loop_deadline_timer_wakeup_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id,
            }
        );
        assert_eq!(adapter.active_timer(), None);
        assert_eq!(adapter.next_raw_id(), 2);
        assert_eq!(adapter.clock.now_calls, 1);
        assert_eq!(adapter.sleeper.sleep_until_calls, vec![16_668_666]);
    }

    #[test]
    fn native_window_deadline_timer_adapter_rejects_active_overlap() {
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );

        assert_eq!(adapter.register_timer_nanos(10).unwrap(), 1);
        assert_eq!(
            adapter.register_timer_nanos(20).unwrap_err(),
            NativeWindowHostLoopDeadlineTimerAdapterError::ActiveTimerAlreadyRegistered {
                active_raw_id: 1,
            }
        );
        assert_eq!(
            adapter.active_timer(),
            Some(NativeWindowHostLoopDeadlineTimerRecord {
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
                deadline_nanos: 1_010,
            })
        );
    }

    #[test]
    fn native_window_deadline_timer_adapter_rejects_missing_active_timer() {
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );

        assert_eq!(
            adapter
                .wait_for_timer_fire(NativeWindowHostLoopTimerRegistrationId { raw_id: 1 })
                .unwrap_err(),
            NativeWindowHostLoopDeadlineTimerAdapterError::NoActiveTimer {
                requested_raw_id: 1,
            }
        );
        assert!(adapter.sleeper.sleep_until_calls.is_empty());
    }

    #[test]
    fn native_window_deadline_timer_adapter_rejects_mismatched_fire_id() {
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );

        assert_eq!(adapter.register_timer_nanos(10).unwrap(), 1);
        assert_eq!(
            adapter
                .wait_for_timer_fire(NativeWindowHostLoopTimerRegistrationId { raw_id: 2 })
                .unwrap_err(),
            NativeWindowHostLoopDeadlineTimerAdapterError::FiredTimerRegistrationMismatch {
                expected_raw_id: 1,
                actual_raw_id: 2,
            }
        );
        assert_eq!(
            adapter.active_timer(),
            Some(NativeWindowHostLoopDeadlineTimerRecord {
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
                deadline_nanos: 1_010,
            })
        );
        assert!(adapter.sleeper.sleep_until_calls.is_empty());
    }

    #[test]
    fn native_window_deadline_timer_adapter_rejects_registration_id_overflow() {
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        adapter.next_raw_id = u32::MAX;

        assert_eq!(
            adapter.register_timer_nanos(10).unwrap_err(),
            NativeWindowHostLoopDeadlineTimerAdapterError::TimerRegistrationIdOverflow {
                last_raw_id: u32::MAX,
            }
        );
        assert_eq!(adapter.active_timer(), None);
        assert_eq!(adapter.clock.now_calls, 0);
    }

    #[test]
    fn native_window_deadline_timer_adapter_rejects_deadline_overflow() {
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(u64::MAX),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );

        assert_eq!(
            adapter.register_timer_nanos(1).unwrap_err(),
            NativeWindowHostLoopDeadlineTimerAdapterError::DeadlineNanosOverflow {
                now_nanos: u64::MAX,
                wait_nanos: 1,
            }
        );
        assert_eq!(adapter.active_timer(), None);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_deadline_timer_adapter_preserves_clock_error() {
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000).with_error("clock failed"),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );

        assert_eq!(
            adapter.register_timer_nanos(10).unwrap_err(),
            NativeWindowHostLoopDeadlineTimerAdapterError::ClockFailed("clock failed")
        );
        assert_eq!(adapter.active_timer(), None);
        assert_eq!(adapter.next_raw_id(), 1);
        assert_eq!(adapter.clock.now_calls, 1);
    }

    #[test]
    fn native_window_deadline_timer_adapter_preserves_active_timer_on_sleep_error() {
        let mut adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new().with_error("sleep failed"),
        );

        assert_eq!(adapter.register_timer_nanos(10).unwrap(), 1);
        assert_eq!(
            adapter
                .wait_for_timer_fire(NativeWindowHostLoopTimerRegistrationId { raw_id: 1 })
                .unwrap_err(),
            NativeWindowHostLoopDeadlineTimerAdapterError::SleeperFailed("sleep failed")
        );
        assert_eq!(
            adapter.active_timer(),
            Some(NativeWindowHostLoopDeadlineTimerRecord {
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
                deadline_nanos: 1_010,
            })
        );
        assert_eq!(adapter.sleeper.sleep_until_calls, vec![1_010]);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_waits_for_host_event_only() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        );

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert_eq!(adapter.waiter.host_event_calls, vec![(window_size, true)]);
        assert!(adapter.waiter.frame_interval_calls.is_empty());
        assert_eq!(adapter.clock.now_calls, 0);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_returns_timer_fired_on_deadline() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 31,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(2_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        );

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
            }
        );
        assert!(adapter.waiter.host_event_calls.is_empty());
        assert_eq!(
            adapter.waiter.frame_interval_calls,
            vec![(16_668_666, window_size, false)]
        );
        assert_eq!(adapter.clock.now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 2);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_returns_host_ready_without_timer_fire() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 32,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(3_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady,
            ),
        );

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(adapter.waiter.host_event_calls.is_empty());
        assert_eq!(
            adapter.waiter.frame_interval_calls,
            vec![(16_669_667, window_size, true)]
        );
        assert_eq!(adapter.clock.now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 2);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_rejects_invalid_wait_before_side_effects() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 33,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 1,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(4_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        );

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitNanosMismatch {
                wait_nanos: 1,
                nanos_per_frame: 16_666_666,
            }
        );
        assert!(adapter.waiter.host_event_calls.is_empty());
        assert!(adapter.waiter.frame_interval_calls.is_empty());
        assert_eq!(adapter.clock.now_calls, 0);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_preserves_host_event_wait_error() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            )
            .with_host_event_error("host event wait failed"),
        );

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::HostEventWaitFailed(
                "host event wait failed"
            )
        );
        assert_eq!(adapter.waiter.host_event_calls, vec![(window_size, false)]);
        assert!(adapter.waiter.frame_interval_calls.is_empty());
        assert_eq!(adapter.clock.now_calls, 0);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_preserves_clock_error() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 34,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(5_000).with_error("clock failed"),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        );

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::ClockFailed("clock failed")
        );
        assert!(adapter.waiter.frame_interval_calls.is_empty());
        assert_eq!(adapter.clock.now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_rejects_deadline_overflow() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 35,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(u64::MAX),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        );

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::DeadlineNanosOverflow {
                now_nanos: u64::MAX,
                wait_nanos: 16_666_666,
            }
        );
        assert!(adapter.waiter.frame_interval_calls.is_empty());
        assert_eq!(adapter.clock.now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_rejects_timer_id_overflow() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 36,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter {
            next_raw_id: u32::MAX,
            clock: ScriptedNativeWindowHostLoopDeadlineTimerClock::new(6_000),
            waiter: ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        };

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::TimerRegistrationIdOverflow {
                last_raw_id: u32::MAX,
            }
        );
        assert!(adapter.waiter.frame_interval_calls.is_empty());
        assert_eq!(adapter.clock.now_calls, 0);
        assert_eq!(adapter.next_raw_id(), u32::MAX);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_preserves_frame_wait_error() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 37,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let mut adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(7_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            )
            .with_frame_interval_error("frame wait failed"),
        );

        assert_eq!(
            execute_native_window_host_loop_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitFailed(
                "frame wait failed"
            )
        );
        assert_eq!(
            adapter.waiter.frame_interval_calls,
            vec![(16_673_667, window_size, true)]
        );
        assert_eq!(adapter.clock.now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 2);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_waits_for_host_event_only() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            1_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        );
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert_eq!(
            adapter.backend().host_event_calls,
            vec![(window_size, true)]
        );
        assert!(adapter.backend().frame_interval_calls.is_empty());
        assert_eq!(adapter.backend().now_calls, 0);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_returns_timer_fired_on_deadline() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 38,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            4_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        );
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
            }
        );
        assert!(adapter.backend().host_event_calls.is_empty());
        assert_eq!(
            adapter.backend().frame_interval_calls,
            vec![(16_670_666, window_size, false)]
        );
        assert_eq!(adapter.backend().now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 2);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_returns_host_ready_without_timer_fire(
    ) {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 39,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            5_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady,
        );
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(adapter.backend().host_event_calls.is_empty());
        assert_eq!(
            adapter.backend().frame_interval_calls,
            vec![(16_671_667, window_size, true)]
        );
        assert_eq!(adapter.backend().now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 2);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_rejects_invalid_wait_before_side_effects(
    ) {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 40,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 1,
        };
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            6_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        );
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitNanosMismatch {
                wait_nanos: 1,
                nanos_per_frame: 16_666_666,
            }
        );
        assert!(adapter.backend().host_event_calls.is_empty());
        assert!(adapter.backend().frame_interval_calls.is_empty());
        assert_eq!(adapter.backend().now_calls, 0);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_rejects_timer_id_overflow_before_side_effects(
    ) {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 41,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            7_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        );
        let mut adapter = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter {
            next_raw_id: u32::MAX,
            backend,
        };

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::TimerRegistrationIdOverflow {
                last_raw_id: u32::MAX,
            }
        );
        assert!(adapter.backend().host_event_calls.is_empty());
        assert!(adapter.backend().frame_interval_calls.is_empty());
        assert_eq!(adapter.backend().now_calls, 0);
        assert_eq!(adapter.next_raw_id(), u32::MAX);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_preserves_clock_error() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 42,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            8_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        )
        .with_clock_error("clock failed");
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::ClockFailed(
                "clock failed"
            )
        );
        assert!(adapter.backend().frame_interval_calls.is_empty());
        assert_eq!(adapter.backend().now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_rejects_deadline_overflow() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 43,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            u64::MAX,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        );
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::DeadlineNanosOverflow {
                now_nanos: u64::MAX,
                wait_nanos: 16_666_666,
            }
        );
        assert!(adapter.backend().frame_interval_calls.is_empty());
        assert_eq!(adapter.backend().now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 1);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_preserves_frame_wait_error() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 44,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            9_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        )
        .with_frame_interval_error("frame wait failed");
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitFailed(
                "frame wait failed"
            )
        );
        assert_eq!(
            adapter.backend().frame_interval_calls,
            vec![(16_675_667, window_size, true)]
        );
        assert_eq!(adapter.backend().now_calls, 1);
        assert_eq!(adapter.next_raw_id(), 2);
    }

    #[test]
    fn native_window_event_queue_wait_waits_for_host_event_instruction() {
        let window_size = NativeWindowSize::new(800, 600);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let mut waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();

        assert_eq!(
            execute_native_window_host_loop_event_queue_wait_with_waiter(instruction, &mut waiter)
                .unwrap(),
            NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady {
                window_size,
                size_changed: true,
            }
        );
        assert_eq!(waiter.wait_calls, vec![(window_size, true)]);
    }

    #[test]
    fn native_window_event_queue_wait_rejects_frame_interval_without_timer_backend() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 15,
            width: window_size.width,
            height: window_size.height,
        };
        let frame_interval = native_window_frame_interval_request(NativeWindowTargetFps::default());
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval,
            wait_nanos: 16_666_666,
        };
        let mut waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();

        assert_eq!(
            execute_native_window_host_loop_event_queue_wait_with_waiter(instruction, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopEventQueueWaitError::FrameIntervalEventQueueWaitUnsupported {
                presentation,
                window_size,
                size_changed: false,
                frame_interval,
                wait_nanos: 16_666_666,
            }
        );
        assert!(waiter.wait_calls.is_empty());
    }

    #[test]
    fn native_window_event_queue_wait_preserves_waiter_error() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let mut waiter =
            ScriptedNativeWindowHostLoopEventQueueWaiter::new().with_error("event queue failed");

        assert_eq!(
            execute_native_window_host_loop_event_queue_wait_with_waiter(instruction, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopEventQueueWaitError::WaiterFailed("event queue failed")
        );
        assert_eq!(waiter.wait_calls, vec![(window_size, false)]);
    }

    #[test]
    fn native_window_wait_owner_dispatches_host_event_to_event_queue_only() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let event_waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let mut owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);

        assert_eq!(
            execute_native_window_host_loop_wait_with_owner(instruction, &mut owner).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert_eq!(
            owner.event_queue_waiter().wait_calls,
            vec![(window_size, true)]
        );
        assert_eq!(owner.frame_interval_timer().clock.now_calls, 0);
        assert!(owner
            .frame_interval_timer()
            .sleeper
            .sleep_until_calls
            .is_empty());
    }

    #[test]
    fn native_window_wait_owner_dispatches_frame_interval_to_timer_only() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 29,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let event_waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(2_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 1 };
        let mut owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);

        assert_eq!(
            owner.frame_interval_wait_authority_mode(),
            native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer()
        );
        assert_eq!(
            execute_native_window_host_loop_wait_with_owner(instruction, &mut owner).unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id,
            }
        );
        assert!(owner.event_queue_waiter().wait_calls.is_empty());
        assert_eq!(owner.frame_interval_timer().active_timer(), None);
        assert_eq!(owner.frame_interval_timer().next_raw_id(), 2);
        assert_eq!(owner.frame_interval_timer().clock.now_calls, 1);
        assert_eq!(
            owner.frame_interval_timer().sleeper.sleep_until_calls,
            vec![16_668_666]
        );
    }

    #[test]
    fn native_window_wait_owner_ignores_frame_authority_for_host_event_wait() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let event_waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let requested_authority_mode =
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(
                NativeWindowTargetFps::default(),
            );
        let mut owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);

        assert_eq!(
            execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode(
                instruction,
                &mut owner,
                requested_authority_mode,
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert_eq!(
            owner.event_queue_waiter().wait_calls,
            vec![(window_size, true)]
        );
        assert_eq!(owner.frame_interval_timer().clock.now_calls, 0);
        assert_eq!(owner.frame_interval_timer().next_raw_id(), 1);
        assert_eq!(owner.frame_interval_timer().active_timer(), None);
        assert!(owner
            .frame_interval_timer()
            .sleeper
            .sleep_until_calls
            .is_empty());
    }

    #[test]
    fn native_window_wait_owner_rejects_minifb_frame_authority_before_timer_mutation() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 31,
            width: window_size.width,
            height: window_size.height,
        };
        let target_fps = NativeWindowTargetFps::default();
        let frame_interval = native_window_frame_interval_request(target_fps);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval,
            wait_nanos: frame_interval.nanos_per_frame(),
        };
        let event_waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(2_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let requested_authority_mode =
            native_window_frame_interval_wait_authority_mode_minifb_internal_target_fps(target_fps);
        let active_authority_mode =
            native_window_frame_interval_wait_authority_mode_host_owned_deadline_timer();
        let mut owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);

        assert_eq!(
            execute_native_window_host_loop_wait_with_owner_and_frame_interval_authority_mode(
                instruction,
                &mut owner,
                requested_authority_mode,
            )
            .unwrap_err(),
            NativeWindowHostLoopWaitOwnerError::FrameIntervalAuthorityFailed(
                NativeWindowFrameIntervalWaitAuthorityModeError::ConflictingFrameIntervalAuthorities {
                    active: active_authority_mode,
                    requested: requested_authority_mode,
                }
            )
        );
        assert!(owner.event_queue_waiter().wait_calls.is_empty());
        assert_eq!(owner.frame_interval_timer().clock.now_calls, 0);
        assert_eq!(owner.frame_interval_timer().next_raw_id(), 1);
        assert_eq!(owner.frame_interval_timer().active_timer(), None);
        assert!(owner
            .frame_interval_timer()
            .sleeper
            .sleep_until_calls
            .is_empty());
    }

    #[test]
    fn native_window_wait_owner_preserves_event_queue_error_stage() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let event_waiter =
            ScriptedNativeWindowHostLoopEventQueueWaiter::new().with_error("event queue failed");
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let mut owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);

        assert_eq!(
            execute_native_window_host_loop_wait_with_owner(instruction, &mut owner).unwrap_err(),
            NativeWindowHostLoopWaitOwnerError::EventQueueWaitFailed(
                NativeWindowHostLoopEventQueueWaitError::WaiterFailed("event queue failed")
            )
        );
        assert_eq!(
            owner.event_queue_waiter().wait_calls,
            vec![(window_size, false)]
        );
        assert_eq!(owner.frame_interval_timer().clock.now_calls, 0);
        assert!(owner
            .frame_interval_timer()
            .sleeper
            .sleep_until_calls
            .is_empty());
    }

    #[test]
    fn native_window_wait_owner_preserves_frame_interval_timer_error_stage() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 30,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let event_waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(2_000).with_error("clock failed"),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let mut owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);

        assert_eq!(
            execute_native_window_host_loop_wait_with_owner(instruction, &mut owner).unwrap_err(),
            NativeWindowHostLoopWaitOwnerError::FrameIntervalTimerWakeFailed(
                NativeWindowHostLoopTimerWakeError::RegistrationFailed(
                    NativeWindowHostLoopTimerRegistrationError::RegistrarFailed(
                        NativeWindowHostLoopDeadlineTimerAdapterError::ClockFailed("clock failed")
                    )
                )
            )
        );
        assert!(owner.event_queue_waiter().wait_calls.is_empty());
        assert_eq!(owner.frame_interval_timer().clock.now_calls, 1);
        assert!(owner
            .frame_interval_timer()
            .sleeper
            .sleep_until_calls
            .is_empty());
    }

    #[test]
    fn native_window_host_owned_deadline_wait_host_delegates_non_wait_operations() {
        let loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let inner_host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);
        let event_waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let wait_owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);
        let mut host = NativeWindowHostOwnedDeadlineWaitRunLoopHost::new(inner_host, wait_owner);
        let present_buffer =
            NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(1, 1, vec![0x00112233])
                .unwrap();
        let present_frame =
            NativePresenterFrame::from_rgb0_present_buffer(&present_buffer).unwrap();

        assert_eq!(
            host.poll_event_snapshot(loop_state.event_pump_input())
                .unwrap(),
            snapshot
        );
        host.set_window_title("delegated title");
        host.pump_events_only();
        host.present_frame(present_frame).unwrap();

        assert_eq!(host.host().cursor, 1);
        assert_eq!(host.host().titles, vec!["delegated title".to_string()]);
        assert_eq!(host.host().pump_count, 1);
        assert_eq!(host.host().present_frames, vec![(1, 1)]);
        assert!(host.host().wait_instructions.is_empty());
        assert!(host.wait_owner().event_queue_waiter().wait_calls.is_empty());
        assert_eq!(host.wait_owner().frame_interval_timer().clock.now_calls, 0);
    }

    #[test]
    fn native_window_host_owned_deadline_wait_host_uses_owner_for_host_event_wait() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let event_waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let wait_owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);
        let mut host = NativeWindowHostOwnedDeadlineWaitRunLoopHost::new(inner_host, wait_owner);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(
            host.wait_owner().event_queue_waiter().wait_calls,
            vec![(window_size, true)]
        );
        assert_eq!(host.wait_owner().frame_interval_timer().clock.now_calls, 0);
    }

    #[test]
    fn native_window_host_owned_deadline_wait_host_uses_owner_for_frame_interval_wait() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 41,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let event_waiter = ScriptedNativeWindowHostLoopEventQueueWaiter::new();
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(2_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let wait_owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);
        let mut host = NativeWindowHostOwnedDeadlineWaitRunLoopHost::new(inner_host, wait_owner);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert!(host.wait_owner().event_queue_waiter().wait_calls.is_empty());
        assert_eq!(
            host.wait_owner().frame_interval_timer().active_timer(),
            None
        );
        assert_eq!(host.wait_owner().frame_interval_timer().clock.now_calls, 1);
        assert_eq!(
            host.wait_owner()
                .frame_interval_timer()
                .sleeper
                .sleep_until_calls,
            vec![16_668_666]
        );
    }

    #[test]
    fn native_window_host_owned_deadline_wait_host_preserves_owner_wait_error() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let event_waiter =
            ScriptedNativeWindowHostLoopEventQueueWaiter::new().with_error("queue failed");
        let timer_adapter = NativeWindowHostLoopDeadlineTimerAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopDeadlineTimerSleeper::new(),
        );
        let wait_owner = NativeWindowHostLoopWaitOwner::new(event_waiter, timer_adapter);
        let mut host = NativeWindowHostOwnedDeadlineWaitRunLoopHost::new(inner_host, wait_owner);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap_err(),
            NativeWindowHostLoopWaitOwnerError::EventQueueWaitFailed(
                NativeWindowHostLoopEventQueueWaitError::WaiterFailed("queue failed")
            )
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(
            host.wait_owner().event_queue_waiter().wait_calls,
            vec![(window_size, false)]
        );
        assert_eq!(host.wait_owner().frame_interval_timer().clock.now_calls, 0);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_host_delegates_non_wait_operations() {
        let loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let inner_host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);
        let wait_adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        );
        let mut host =
            NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost::new(inner_host, wait_adapter);
        let present_buffer =
            NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(1, 1, vec![0x00112233])
                .unwrap();
        let present_frame =
            NativePresenterFrame::from_rgb0_present_buffer(&present_buffer).unwrap();

        assert_eq!(
            host.poll_event_snapshot(loop_state.event_pump_input())
                .unwrap(),
            snapshot
        );
        host.set_window_title("delegated title");
        host.pump_events_only();
        host.present_frame(present_frame).unwrap();

        assert_eq!(host.host().cursor, 1);
        assert_eq!(host.host().titles, vec!["delegated title".to_string()]);
        assert_eq!(host.host().pump_count, 1);
        assert_eq!(host.host().present_frames, vec![(1, 1)]);
        assert!(host.host().wait_instructions.is_empty());
        assert!(host.wait_adapter().waiter().host_event_calls.is_empty());
        assert!(host.wait_adapter().waiter().frame_interval_calls.is_empty());
        assert_eq!(host.wait_adapter().clock().now_calls, 0);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_host_uses_adapter_for_host_event_wait() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let wait_adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        );
        let mut host =
            NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost::new(inner_host, wait_adapter);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(
            host.wait_adapter().waiter().host_event_calls,
            vec![(window_size, true)]
        );
        assert!(host.wait_adapter().waiter().frame_interval_calls.is_empty());
        assert_eq!(host.wait_adapter().clock().now_calls, 0);
        assert_eq!(host.wait_adapter().next_raw_id(), 1);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_host_returns_timer_fired_on_deadline() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 42,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let wait_adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(2_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            ),
        );
        let mut host =
            NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost::new(inner_host, wait_adapter);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert!(host.wait_adapter().waiter().host_event_calls.is_empty());
        assert_eq!(
            host.wait_adapter().waiter().frame_interval_calls,
            vec![(16_668_666, window_size, false)]
        );
        assert_eq!(host.wait_adapter().clock().now_calls, 1);
        assert_eq!(host.wait_adapter().next_raw_id(), 2);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_host_returns_host_ready_without_timer_fire() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 43,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_667,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let wait_adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(3_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady,
            ),
        );
        let mut host =
            NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost::new(inner_host, wait_adapter);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert!(host.wait_adapter().waiter().host_event_calls.is_empty());
        assert_eq!(
            host.wait_adapter().waiter().frame_interval_calls,
            vec![(16_669_667, window_size, true)]
        );
        assert_eq!(host.wait_adapter().clock().now_calls, 1);
        assert_eq!(host.wait_adapter().next_raw_id(), 2);
    }

    #[test]
    fn native_window_interruptible_deadline_wait_host_preserves_adapter_wait_error() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let wait_adapter = NativeWindowHostLoopInterruptibleDeadlineWaitAdapter::new(
            ScriptedNativeWindowHostLoopDeadlineTimerClock::new(1_000),
            ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter::new(
                NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
            )
            .with_host_event_error("host event wait failed"),
        );
        let mut host =
            NativeWindowHostLoopInterruptibleDeadlineWaitRunLoopHost::new(inner_host, wait_adapter);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap_err(),
            NativeWindowHostLoopInterruptibleDeadlineWaitAdapterError::HostEventWaitFailed(
                "host event wait failed"
            )
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(
            host.wait_adapter().waiter().host_event_calls,
            vec![(window_size, false)]
        );
        assert!(host.wait_adapter().waiter().frame_interval_calls.is_empty());
        assert_eq!(host.wait_adapter().clock().now_calls, 0);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_host_delegates_non_wait_operations() {
        let loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let snapshot = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let inner_host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(snapshot)]);
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            1_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        );
        let wait_adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);
        let mut host = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost::new(
            inner_host,
            wait_adapter,
        );
        let present_buffer =
            NativeRgb0PresentBuffer::from_rgb0_pixels_for_smoke_demo(1, 1, vec![0x00112233])
                .unwrap();
        let present_frame =
            NativePresenterFrame::from_rgb0_present_buffer(&present_buffer).unwrap();

        assert_eq!(
            host.poll_event_snapshot(loop_state.event_pump_input())
                .unwrap(),
            snapshot
        );
        host.set_window_title("single owner delegated title");
        host.pump_events_only();
        host.present_frame(present_frame).unwrap();

        assert_eq!(host.host().cursor, 1);
        assert_eq!(
            host.host().titles,
            vec!["single owner delegated title".to_string()]
        );
        assert_eq!(host.host().pump_count, 1);
        assert_eq!(host.host().present_frames, vec![(1, 1)]);
        assert!(host.host().wait_instructions.is_empty());
        assert!(host.wait_adapter().backend().host_event_calls.is_empty());
        assert!(host
            .wait_adapter()
            .backend()
            .frame_interval_calls
            .is_empty());
        assert_eq!(host.wait_adapter().backend().now_calls, 0);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_host_uses_adapter_for_host_event_wait(
    ) {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            1_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        );
        let wait_adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);
        let mut host = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost::new(
            inner_host,
            wait_adapter,
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(
            host.wait_adapter().backend().host_event_calls,
            vec![(window_size, true)]
        );
        assert!(host
            .wait_adapter()
            .backend()
            .frame_interval_calls
            .is_empty());
        assert_eq!(host.wait_adapter().backend().now_calls, 0);
        assert_eq!(host.wait_adapter().next_raw_id(), 1);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_host_returns_timer_fired_on_deadline()
    {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 45,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            2_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        );
        let wait_adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);
        let mut host = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost::new(
            inner_host,
            wait_adapter,
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert!(host.wait_adapter().backend().host_event_calls.is_empty());
        assert_eq!(
            host.wait_adapter().backend().frame_interval_calls,
            vec![(16_668_666, window_size, false)]
        );
        assert_eq!(host.wait_adapter().backend().now_calls, 1);
        assert_eq!(host.wait_adapter().next_raw_id(), 2);
    }

    #[test]
    fn native_window_single_owner_interruptible_deadline_wait_host_preserves_adapter_wait_error() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let inner_host =
            ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("inner wait used");
        let backend = ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend::new(
            1_000,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached,
        )
        .with_host_event_error("host event wait failed");
        let wait_adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);
        let mut host = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost::new(
            inner_host,
            wait_adapter,
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::HostEventWaitFailed(
                "host event wait failed"
            )
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(
            host.wait_adapter().backend().host_event_calls,
            vec![(window_size, false)]
        );
        assert!(host
            .wait_adapter()
            .backend()
            .frame_interval_calls
            .is_empty());
        assert_eq!(host.wait_adapter().backend().now_calls, 0);
    }

    #[test]
    fn native_window_platform_wait_backend_validation_accepts_matching_backend() {
        let cases = [
            (
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            ),
            (
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            ),
            (
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            ),
        ];

        for (current, requested) in cases {
            assert_eq!(
                validate_native_window_host_loop_platform_wait_backend_kind_for_platform(
                    current, requested,
                )
                .unwrap(),
                requested
            );
        }
    }

    #[test]
    fn native_window_platform_wait_backend_validation_rejects_all_real_platform_mismatches() {
        let cases = [
            (
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            ),
            (
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            ),
            (
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::HeadlessScripted,
            ),
            (
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            ),
            (
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            ),
            (
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::HeadlessScripted,
            ),
            (
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            ),
            (
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            ),
            (
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::HeadlessScripted,
            ),
        ];

        for (current, requested) in cases {
            assert_eq!(
                validate_native_window_host_loop_platform_wait_backend_kind_for_platform(
                    current, requested,
                )
                .unwrap_err(),
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current,
                    requested,
                }
            );
        }
    }

    #[test]
    fn native_window_platform_wait_backend_validation_rejects_unsupported_platform() {
        let requested =
            NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait;

        assert_eq!(
            validate_native_window_host_loop_platform_wait_backend_kind_for_platform(
                NativeWindowHostLoopPlatformKind::Unsupported,
                requested,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitBackendSupportError::RequestedBackendUnsupportedPlatform {
                current: NativeWindowHostLoopPlatformKind::Unsupported,
                requested,
            }
        );
    }

    #[test]
    fn native_window_platform_wait_backend_default_maps_real_platforms_without_headless_fallback() {
        let cases = [
            (
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            ),
            (
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            ),
            (
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            ),
        ];

        for (current, expected) in cases {
            let actual =
                native_window_host_loop_default_platform_wait_backend_kind_for_platform(current)
                    .unwrap();
            assert_eq!(actual, expected);
            assert_ne!(
                actual,
                NativeWindowHostLoopPlatformWaitBackendKind::HeadlessScripted
            );
        }
    }

    #[test]
    fn native_window_platform_wait_backend_default_rejects_unsupported_platform() {
        assert_eq!(
            native_window_host_loop_default_platform_wait_backend_kind_for_platform(
                NativeWindowHostLoopPlatformKind::Unsupported,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitBackendSupportError::DefaultBackendUnsupportedPlatform {
                current: NativeWindowHostLoopPlatformKind::Unsupported,
            }
        );
    }

    #[test]
    fn native_window_current_platform_wait_backend_default_matches_cfg_platform() {
        let current = native_window_host_loop_current_platform_kind();

        #[cfg(target_os = "macos")]
        assert_eq!(current, NativeWindowHostLoopPlatformKind::Macos);
        #[cfg(target_os = "windows")]
        assert_eq!(current, NativeWindowHostLoopPlatformKind::Windows);
        #[cfg(target_os = "linux")]
        assert_eq!(current, NativeWindowHostLoopPlatformKind::Linux);
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        assert_eq!(current, NativeWindowHostLoopPlatformKind::Unsupported);

        match native_window_host_loop_default_platform_wait_backend_kind() {
            Ok(backend) => assert_ne!(
                backend,
                NativeWindowHostLoopPlatformWaitBackendKind::HeadlessScripted
            ),
            Err(error) => assert_eq!(
                error,
                NativeWindowHostLoopPlatformWaitBackendSupportError::DefaultBackendUnsupportedPlatform {
                    current: NativeWindowHostLoopPlatformKind::Unsupported,
                }
            ),
        }
    }

    #[test]
    fn native_window_platform_wait_backend_selection_carries_validated_platform_and_backend() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap();

        assert_eq!(
            selection.platform(),
            NativeWindowHostLoopPlatformKind::Linux
        );
        assert_eq!(
            selection.backend(),
            NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd
        );
    }

    #[test]
    fn native_window_platform_wait_backend_selection_rejects_headless_scripted_for_native() {
        assert_eq!(
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::HeadlessScripted,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                current: NativeWindowHostLoopPlatformKind::Linux,
                requested: NativeWindowHostLoopPlatformWaitBackendKind::HeadlessScripted,
            }
        );
    }

    #[test]
    fn native_window_platform_wait_backend_selection_rejects_unsupported_platform() {
        assert_eq!(
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Unsupported,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitBackendSupportError::RequestedBackendUnsupportedPlatform {
                current: NativeWindowHostLoopPlatformKind::Unsupported,
                requested: NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            }
        );
    }

    #[test]
    fn native_window_platform_wait_backend_default_selection_matches_supported_platforms() {
        let cases = [
            (
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            ),
            (
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            ),
            (
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            ),
        ];

        for (platform, backend) in cases {
            let selection =
                native_window_host_loop_default_platform_wait_backend_selection_for_platform(
                    platform,
                )
                .unwrap();
            assert_eq!(selection.platform(), platform);
            assert_eq!(selection.backend(), backend);
        }
    }

    #[test]
    fn native_window_platform_wait_backend_builder_preserves_selection_as_unavailable() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();

        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_from_selection(selection)
                .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::BackendImplementationUnavailable {
                platform: NativeWindowHostLoopPlatformKind::Windows,
                backend:
                    NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            }
        );
        assert_eq!(
            selection.platform(),
            NativeWindowHostLoopPlatformKind::Windows
        );
        assert_eq!(
            selection.backend(),
            NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait
        );
    }

    #[test]
    fn native_window_platform_wait_backend_builder_returns_support_failure_before_unavailable() {
        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_for_platform(
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: NativeWindowHostLoopPlatformKind::Macos,
                    requested: NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
                }
            )
        );
    }

    #[test]
    fn native_window_platform_wait_backend_with_windows_api_builds_windows_backend() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(81);

        let backend =
            build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api(
                selection, api,
            )
            .unwrap();

        match backend {
            NativeWindowHostLoopPlatformWaitBackend::WindowsWaitableTimerMessageWait(backend) => {
                assert!(backend.is_handle_open());
                assert_eq!(backend.api().create_calls, 1);
            }
        }
    }

    #[test]
    fn native_window_platform_wait_backend_with_windows_api_preserves_unavailable_real_backends() {
        let cases = [
            (
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            ),
            (
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            ),
        ];

        for (platform, backend) in cases {
            let selection =
                validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                    platform, backend,
                )
                .unwrap();
            let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(82);

            assert_eq!(
                build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api(
                    selection,
                    api,
                )
                .unwrap_err(),
                NativeWindowHostLoopPlatformWaitHostBuildError::BackendImplementationUnavailable {
                    platform,
                    backend,
                }
            );
        }
    }

    #[test]
    fn native_window_platform_wait_backend_with_windows_api_preserves_support_failure() {
        let selection = NativeWindowHostLoopPlatformWaitBackendSelection {
            platform: NativeWindowHostLoopPlatformKind::Macos,
            backend: NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
        };
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(83);

        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api(
                selection, api,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: NativeWindowHostLoopPlatformKind::Macos,
                    requested:
                        NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
                }
            )
        );
    }

    #[test]
    fn native_window_platform_wait_backend_with_windows_api_preserves_windows_failure() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(0).with_last_error_code(84);

        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api(
                selection, api,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::WindowsWaitBackendFailed(
                NativeWindowHostLoopWindowsWaitBackendError::CreateWaitableTimerFailed { code: 84 }
            )
        );
    }

    #[test]
    fn native_window_platform_wait_backend_with_linux_api_builds_linux_backend() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(101, 102);

        let backend =
            build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api(
                selection, api,
            )
            .unwrap();

        match backend {
            NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(backend) => {
                assert!(backend.are_handles_open());
                assert_eq!(backend.api().selector_create_calls, 1);
                assert_eq!(backend.api().timer_create_calls, 1);
                assert_eq!(backend.api().host_event_create_calls, 1);
                assert_eq!(backend.api().register_calls, vec![(101, 102)]);
                assert_eq!(backend.api().register_host_event_calls, vec![(101, 103)]);
            }
        }
    }

    #[test]
    fn native_window_platform_wait_backend_with_linux_api_preserves_unavailable_real_backends() {
        let cases = [
            (
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            ),
            (
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            ),
        ];

        for (platform, backend) in cases {
            let selection =
                validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                    platform, backend,
                )
                .unwrap();
            let raw_method_calls = std::rc::Rc::new(std::cell::Cell::new(0));
            let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(104, 105)
                .with_raw_method_call_counter(std::rc::Rc::clone(&raw_method_calls));

            assert_eq!(
                build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api(
                    selection, api,
                )
                .unwrap_err(),
                NativeWindowHostLoopPlatformWaitHostBuildError::BackendImplementationUnavailable {
                    platform,
                    backend,
                }
            );
            assert_eq!(raw_method_calls.get(), 0);
        }
    }

    #[test]
    fn native_window_platform_wait_backend_with_linux_api_preserves_support_failure_before_raw_calls(
    ) {
        let selection = NativeWindowHostLoopPlatformWaitBackendSelection {
            platform: NativeWindowHostLoopPlatformKind::Windows,
            backend: NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
        };
        let raw_method_calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(106, 107)
            .with_raw_method_call_counter(std::rc::Rc::clone(&raw_method_calls));

        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api(
                selection, api,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: NativeWindowHostLoopPlatformKind::Windows,
                    requested: NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
                }
            )
        );
        assert_eq!(raw_method_calls.get(), 0);
    }

    #[test]
    fn native_window_platform_wait_backend_with_linux_api_preserves_linux_failure() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(-1, 109)
            .with_last_error_code(110);

        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_from_selection_with_linux_api(
                selection, api,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::LinuxSelectorTimerFdBackendFailed(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateSelectorFailed {
                    code: 110,
                }
            )
        );
    }

    #[test]
    fn native_window_platform_wait_backend_with_raw_apis_builds_selected_macos_backend() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            )
            .unwrap();
        let windows_api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(0);
        let macos_api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(87);
        let linux_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(-1, -1);

        let backend =
            build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis(
                selection,
                windows_api,
                macos_api,
                linux_api,
            )
            .unwrap();

        match backend {
            NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer(backend) => {
                assert!(backend.is_handle_open());
                assert_eq!(backend.api().create_calls, 1);
            }
            _ => panic!("unexpected platform backend"),
        }
    }

    #[test]
    fn native_window_platform_wait_backend_with_raw_apis_builds_selected_linux_backend() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap();
        let windows_api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(0);
        let macos_api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(0);
        let linux_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(88, 89);

        let backend =
            build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis(
                selection,
                windows_api,
                macos_api,
                linux_api,
            )
            .unwrap();

        match backend {
            NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(backend) => {
                assert!(backend.are_handles_open());
                assert_eq!(backend.api().selector_create_calls, 1);
                assert_eq!(backend.api().timer_create_calls, 1);
                assert_eq!(backend.api().host_event_create_calls, 1);
                assert_eq!(backend.api().register_host_event_calls, vec![(88, 90)]);
            }
            _ => panic!("unexpected platform backend"),
        }
    }

    #[test]
    fn native_window_platform_wait_backend_with_raw_apis_preserves_macos_failure() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            )
            .unwrap();
        let windows_api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(90);
        let macos_api =
            ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(0).with_last_error_code(91);
        let linux_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(92, 93);

        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis(
                selection,
                windows_api,
                macos_api,
                linux_api,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::MacosRunLoopTimerBackendFailed(
                NativeWindowHostLoopMacosRunLoopTimerBackendError::CreateRunLoopTimerFailed {
                    code: 91,
                }
            )
        );
    }

    #[test]
    fn native_window_platform_wait_backend_with_raw_apis_preserves_linux_failure() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap();
        let windows_api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(94);
        let macos_api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(95);
        let linux_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(-1, 96)
            .with_last_error_code(97);

        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis(
                selection,
                windows_api,
                macos_api,
                linux_api,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::LinuxSelectorTimerFdBackendFailed(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateSelectorFailed {
                    code: 97,
                }
            )
        );
    }

    #[test]
    fn native_window_platform_wait_backend_with_raw_apis_support_failure_precedes_raw_create() {
        let selection = NativeWindowHostLoopPlatformWaitBackendSelection {
            platform: NativeWindowHostLoopPlatformKind::Macos,
            backend: NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
        };
        let windows_api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(0);
        let macos_api =
            ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(0).with_last_error_code(98);
        let linux_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(-1, -1)
            .with_last_error_code(99);

        assert_eq!(
            build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis(
                selection,
                windows_api,
                macos_api,
                linux_api,
            )
            .unwrap_err(),
            NativeWindowHostLoopPlatformWaitHostBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: NativeWindowHostLoopPlatformKind::Macos,
                    requested: NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
                }
            )
        );
    }

    #[test]
    fn native_window_platform_wait_run_loop_host_wraps_existing_backend_infallibly() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        let api =
            ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(85).with_message_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ZERO_HANDLES,
            ]);
        let backend =
            build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api(
                selection, api,
            )
            .unwrap();
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let mut host = native_window_host_loop_platform_wait_run_loop_host_from_backend(
            ScriptedNativeWindowRunLoopHost::new(Vec::new()),
            backend,
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        match host.wait_adapter().backend() {
            NativeWindowHostLoopPlatformWaitBackend::WindowsWaitableTimerMessageWait(backend) => {
                assert_eq!(backend.api().message_wait_calls, 1);
                assert!(backend.api().timer_wait_calls.is_empty());
            }
            NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer(backend) => {
                match *backend.api() {}
            }
            NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(backend) => {
                match *backend.api() {}
            }
        }
    }

    #[test]
    fn native_window_platform_wait_run_loop_host_keeps_host_ready_outcome_non_timer() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(86)
            .with_timer_or_message_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ONE_HANDLE,
            ]);
        let backend =
            build_native_window_host_loop_platform_wait_backend_from_selection_with_windows_api(
                selection, api,
            )
            .unwrap();
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 86,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut host = native_window_host_loop_platform_wait_run_loop_host_from_backend(
            ScriptedNativeWindowRunLoopHost::new(Vec::new()),
            backend,
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: false,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        match host.wait_adapter().backend() {
            NativeWindowHostLoopPlatformWaitBackend::WindowsWaitableTimerMessageWait(backend) => {
                assert_eq!(backend.api().timer_wait_calls, vec![86]);
            }
            NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer(backend) => {
                match *backend.api() {}
            }
            NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(backend) => {
                match *backend.api() {}
            }
        }
    }

    #[test]
    fn native_window_platform_wait_run_loop_host_wraps_macos_backend() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            )
            .unwrap();
        let backend =
            build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis(
                selection,
                ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(0),
                ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(98)
                    .with_timer_or_event_statuses(vec![
                        NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED,
                    ]),
                ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(-1, -1),
            )
            .unwrap();
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 98,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut host = native_window_host_loop_platform_wait_run_loop_host_from_backend(
            ScriptedNativeWindowRunLoopHost::new(Vec::new()),
            backend,
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        match host.wait_adapter().backend() {
            NativeWindowHostLoopPlatformWaitBackend::MacosRunLoopTimer(backend) => {
                assert_eq!(backend.api().timer_wait_calls, vec![98]);
            }
            _ => panic!("unexpected platform backend"),
        }
    }

    #[test]
    fn native_window_platform_wait_run_loop_host_wraps_linux_backend() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap();
        let backend =
            build_native_window_host_loop_platform_wait_backend_from_selection_with_raw_apis(
                selection,
                ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(0),
                ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(0),
                ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(99, 100)
                    .with_timer_or_event_statuses(vec![
                        NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY,
                    ]),
            )
            .unwrap();
        let window_size = NativeWindowSize::new(640, 480);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 99,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let mut host = native_window_host_loop_platform_wait_run_loop_host_from_backend(
            ScriptedNativeWindowRunLoopHost::new(Vec::new()),
            backend,
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        match host.wait_adapter().backend() {
            NativeWindowHostLoopPlatformWaitBackend::LinuxSelectorTimerFd(backend) => {
                assert_eq!(backend.api().timer_wait_calls, vec![(99, 100, 101)]);
            }
            _ => panic!("unexpected platform backend"),
        }
    }

    #[test]
    fn native_window_macos_run_loop_timer_handle_rejects_null_and_invalid_raw_handles() {
        assert_eq!(
            native_window_host_loop_macos_run_loop_timer_handle_from_raw(0).unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendError::InvalidRawHandle { raw_handle: 0 }
        );
        assert_eq!(
            native_window_host_loop_macos_run_loop_timer_handle_from_raw(-1).unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendError::InvalidRawHandle { raw_handle: -1 }
        );
        let handle = native_window_host_loop_macos_run_loop_timer_handle_from_raw(41).unwrap();
        assert_eq!(
            native_window_host_loop_macos_run_loop_timer_handle_raw(&handle),
            41
        );
    }

    #[test]
    fn native_window_macos_run_loop_deadline_plan_uses_checked_relative_nanos() {
        assert_eq!(
            native_window_host_loop_macos_run_loop_deadline_plan(10, 10).unwrap(),
            NativeWindowHostLoopMacosRunLoopDeadlinePlan::AlreadyReached
        );
        assert_eq!(
            native_window_host_loop_macos_run_loop_deadline_plan(11, 10).unwrap(),
            NativeWindowHostLoopMacosRunLoopDeadlinePlan::AlreadyReached
        );
        assert_eq!(
            native_window_host_loop_macos_run_loop_deadline_plan(10, 11).unwrap(),
            NativeWindowHostLoopMacosRunLoopDeadlinePlan::RelativeNanos(1)
        );
        assert_eq!(
            native_window_host_loop_macos_run_loop_deadline_plan(10, 25).unwrap(),
            NativeWindowHostLoopMacosRunLoopDeadlinePlan::RelativeNanos(15)
        );
    }

    #[test]
    fn native_window_macos_run_loop_status_maps_timer_event_and_failures() {
        assert_eq!(
            native_window_host_loop_macos_run_loop_wake_from_timer_or_event_status(
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED,
                0,
            )
            .unwrap(),
            NativeWindowHostLoopMacosRunLoopWake::TimerFired
        );
        assert_eq!(
            native_window_host_loop_macos_run_loop_wake_from_timer_or_event_status(
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_HOST_EVENT_READY,
                0,
            )
            .unwrap(),
            NativeWindowHostLoopMacosRunLoopWake::HostEventReady
        );
        assert_eq!(
            native_window_host_loop_macos_run_loop_wake_from_timer_or_event_status(
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_FAILED,
                77,
            )
            .unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendError::RunLoopWaitFailed { code: 77 }
        );
        assert_eq!(
            native_window_host_loop_macos_run_loop_wake_from_timer_or_event_status(99, 0)
                .unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendError::UnexpectedRunLoopStatus {
                status: 99,
            }
        );
        assert_eq!(
            native_window_host_loop_macos_run_loop_host_event_from_status(
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED,
                0,
            )
            .unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendError::UnexpectedRunLoopStatus {
                status: NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED,
            }
        );
    }

    #[test]
    fn native_window_macos_run_loop_backend_rejects_timer_creation_failure() {
        let api =
            ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(0).with_last_error_code(5);

        assert_eq!(
            NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendError::CreateRunLoopTimerFailed { code: 5 }
        );
    }

    #[test]
    fn native_window_macos_run_loop_backend_wait_for_host_event_uses_event_only_wait() {
        let window_size = NativeWindowSize::new(640, 480);
        let api =
            ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(81).with_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_HOST_EVENT_READY,
            ]);
        let mut backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();

        assert_eq!(backend.wait_for_host_event(window_size, true).unwrap(), ());
        assert_eq!(backend.api().create_calls, 1);
        assert_eq!(backend.api().event_wait_calls, 1);
        assert!(backend.api().timer_wait_calls.is_empty());
        assert!(backend.api().schedule_calls.is_empty());
    }

    #[test]
    fn native_window_macos_run_loop_backend_wait_until_deadline_schedules_relative_timer() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(82)
            .with_timer_or_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED,
            ]);
        let mut backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();

        let wake = backend
            .wait_until_deadline_or_host_event(10_000_000_000, window_size, false)
            .unwrap();

        assert_eq!(wake, NativeWindowHostLoopMacosRunLoopWake::TimerFired);
        assert_eq!(backend.api().schedule_calls.len(), 1);
        assert_eq!(backend.api().schedule_calls[0].0, 82);
        assert!(backend.api().schedule_calls[0].1 > 0);
        assert_eq!(backend.api().timer_wait_calls, vec![82]);
        assert_eq!(backend.api().event_wait_calls, 0);
    }

    #[test]
    fn native_window_macos_run_loop_backend_wait_until_deadline_maps_host_ready() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(83)
            .with_timer_or_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_HOST_EVENT_READY,
            ]);
        let mut backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(10_000_000_000, window_size, true)
                .unwrap(),
            NativeWindowHostLoopMacosRunLoopWake::HostEventReady
        );
        assert_eq!(backend.api().timer_wait_calls, vec![83]);
    }

    #[test]
    fn native_window_macos_run_loop_backend_wait_until_deadline_rejects_schedule_failure() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(84)
            .with_last_error_code(1001)
            .with_schedule_result(false);
        let mut backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(10_000_000_000, window_size, false)
                .unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendError::ScheduleRunLoopTimerFailed {
                code: 1001,
            }
        );
        assert!(backend.api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_macos_run_loop_backend_wait_until_deadline_already_reached_avoids_raw_wait() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(85);
        let mut backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(0, window_size, false)
                .unwrap(),
            NativeWindowHostLoopMacosRunLoopWake::TimerFired
        );
        assert!(backend.api().schedule_calls.is_empty());
        assert!(backend.api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_macos_run_loop_backend_invalidates_handle_once() {
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(86);
        let mut backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();

        assert_eq!(backend.invalidate_handle_if_open(), true);
        assert_eq!(backend.invalidate_handle_if_open(), false);
        assert_eq!(backend.api().invalidate_calls, vec![86]);
        assert_eq!(backend.is_handle_open(), false);
    }

    #[test]
    fn native_window_macos_run_loop_backend_builder_requires_validated_macos_selection() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(87);

        assert_eq!(
            build_native_window_host_loop_macos_run_loop_timer_backend_from_selection(
                selection, api
            )
            .unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: NativeWindowHostLoopPlatformKind::Windows,
                    requested:
                        NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
                }
            )
        );
    }

    #[test]
    fn native_window_macos_run_loop_backend_builder_preserves_raw_api_failure() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            )
            .unwrap();
        let api =
            ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(0).with_last_error_code(8);

        assert_eq!(
            build_native_window_host_loop_macos_run_loop_timer_backend_from_selection(
                selection, api
            )
            .unwrap_err(),
            NativeWindowHostLoopMacosRunLoopTimerBackendBuildError::RunLoopTimerBackendFailed(
                NativeWindowHostLoopMacosRunLoopTimerBackendError::CreateRunLoopTimerFailed {
                    code: 8,
                }
            )
        );
    }

    #[test]
    fn native_window_macos_run_loop_wait_trait_maps_timer_to_deadline_reached() {
        let window_size = NativeWindowSize::new(320, 200);
        let frame_interval = native_window_frame_interval_request(NativeWindowTargetFps::default());
        let wait_nanos = frame_interval.nanos_per_frame();
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 24,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval,
            wait_nanos,
        };
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(88)
            .with_timer_or_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED,
            ]);
        let backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter,
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
            }
        );
        assert_eq!(adapter.backend().api().timer_wait_calls, vec![88]);
        assert_eq!(adapter.backend().api().event_wait_calls, 0);
    }

    #[test]
    fn native_window_macos_run_loop_wait_trait_keeps_host_ready_non_timer() {
        let window_size = NativeWindowSize::new(640, 480);
        let frame_interval = native_window_frame_interval_request(NativeWindowTargetFps::default());
        let wait_nanos = frame_interval.nanos_per_frame();
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 25,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval,
            wait_nanos,
        };
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(89)
            .with_timer_or_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_HOST_EVENT_READY,
            ]);
        let backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();
        let mut host = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost::new(
            ScriptedNativeWindowRunLoopHost::new(Vec::new()),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend),
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(
            host.wait_adapter().backend().api().timer_wait_calls,
            vec![89]
        );
    }

    #[test]
    fn native_window_macos_run_loop_wait_trait_rejects_timer_status_for_event_wait() {
        let window_size = NativeWindowSize::new(800, 600);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let api =
            ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(90).with_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED,
            ]);
        let backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter,
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::HostEventWaitFailed(
                NativeWindowHostLoopMacosRunLoopTimerBackendError::UnexpectedRunLoopStatus {
                    status: NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_TIMER_FIRED,
                }
            )
        );
        assert_eq!(adapter.backend().api().event_wait_calls, 1);
        assert!(adapter.backend().api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_macos_run_loop_wait_trait_preserves_schedule_error() {
        let window_size = NativeWindowSize::new(640, 360);
        let frame_interval = native_window_frame_interval_request(NativeWindowTargetFps::default());
        let wait_nanos = frame_interval.nanos_per_frame();
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 26,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval,
            wait_nanos,
        };
        let api = ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi::new(91)
            .with_last_error_code(92)
            .with_schedule_result(false);
        let backend = NativeWindowHostLoopMacosRunLoopTimerBackend::new(api).unwrap();
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter,
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitFailed(
                NativeWindowHostLoopMacosRunLoopTimerBackendError::ScheduleRunLoopTimerFailed {
                    code: 92,
                }
            )
        );
        assert!(adapter.backend().api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_handles_accept_zero_and_reject_negative_raw_fds() {
        assert_eq!(
            native_window_host_loop_linux_selector_fd_from_raw(-1).unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidSelectorRawFd {
                raw_fd: -1,
            }
        );
        assert_eq!(
            native_window_host_loop_linux_timer_fd_from_raw(-1).unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidTimerRawFd { raw_fd: -1 }
        );
        assert_eq!(
            native_window_host_loop_linux_host_event_fd_from_raw(-1).unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidHostEventRawFd {
                raw_fd: -1,
            }
        );
        assert_eq!(
            native_window_host_loop_linux_host_event_signal_fd_from_raw(-1).unwrap_err(),
            NativeWindowHostLoopLinuxHostEventSignalProducerError::InvalidHostEventSignalRawFd {
                raw_fd: -1,
            }
        );

        let selector = native_window_host_loop_linux_selector_fd_from_raw(0).unwrap();
        let timer = native_window_host_loop_linux_timer_fd_from_raw(0).unwrap();
        let host_event = native_window_host_loop_linux_host_event_fd_from_raw(0).unwrap();
        let signal = native_window_host_loop_linux_host_event_signal_fd_from_raw(0).unwrap();
        assert_eq!(native_window_host_loop_linux_selector_fd_raw(&selector), 0);
        assert_eq!(native_window_host_loop_linux_timer_fd_raw(&timer), 0);
        assert_eq!(
            native_window_host_loop_linux_host_event_fd_raw(&host_event),
            0
        );
        assert_eq!(
            native_window_host_loop_linux_host_event_signal_fd_raw(&signal),
            0
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_timespec_uses_checked_seconds_and_nanoseconds() {
        assert_eq!(
            native_window_host_loop_linux_timer_fd_timespec_from_nanos(0).unwrap(),
            NativeWindowHostLoopLinuxTimerFdTimespec {
                seconds: 0,
                nanoseconds: 0,
            }
        );
        assert_eq!(
            native_window_host_loop_linux_timer_fd_timespec_from_nanos(1_000_000_001).unwrap(),
            NativeWindowHostLoopLinuxTimerFdTimespec {
                seconds: 1,
                nanoseconds: 1,
            }
        );
        assert_eq!(
            native_window_host_loop_linux_timer_fd_timespec_from_nanos(u64::MAX).unwrap(),
            NativeWindowHostLoopLinuxTimerFdTimespec {
                seconds: 18_446_744_073,
                nanoseconds: 709_551_615,
            }
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_deadline_plan_uses_already_reached_or_timespec() {
        assert_eq!(
            native_window_host_loop_linux_selector_timer_fd_deadline_plan(1_000, 1_000).unwrap(),
            NativeWindowHostLoopLinuxSelectorTimerFdDeadlinePlan::AlreadyReached
        );
        assert_eq!(
            native_window_host_loop_linux_selector_timer_fd_deadline_plan(1_000, 1_000_001_001)
                .unwrap(),
            NativeWindowHostLoopLinuxSelectorTimerFdDeadlinePlan::RelativeTimespec(
                NativeWindowHostLoopLinuxTimerFdTimespec {
                    seconds: 1,
                    nanoseconds: 1,
                }
            )
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_status_maps_timer_host_event_and_failures() {
        assert_eq!(
            native_window_host_loop_linux_selector_timer_fd_wake_from_status(
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED,
                0,
            )
            .unwrap(),
            NativeWindowHostLoopLinuxSelectorTimerFdWake::TimerFired
        );
        assert_eq!(
            native_window_host_loop_linux_selector_timer_fd_wake_from_status(
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY,
                0,
            )
            .unwrap(),
            NativeWindowHostLoopLinuxSelectorTimerFdWake::HostEventReady
        );
        assert_eq!(
            native_window_host_loop_linux_selector_timer_fd_wake_from_status(
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED,
                98,
            )
            .unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::SelectorWaitFailed { code: 98 }
        );
        assert_eq!(
            native_window_host_loop_linux_selector_timer_fd_wake_from_status(7, 0).unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::UnexpectedSelectorStatus {
                status: 7,
            }
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_host_event_status_rejects_timer_fired() {
        assert_eq!(
            native_window_host_loop_linux_selector_timer_fd_host_event_from_status(
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY,
                0,
            )
            .unwrap(),
            ()
        );
        assert_eq!(
            native_window_host_loop_linux_selector_timer_fd_host_event_from_status(
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED,
                0,
            )
            .unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::UnexpectedSelectorStatus {
                status: NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED,
            }
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_rejects_selector_creation_failure() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(-1, 5)
            .with_last_error_code(11);

        assert_eq!(
            NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateSelectorFailed { code: 11 }
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_rejects_timer_fd_creation_failure() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, -1)
            .with_last_error_code(12);

        let error = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap_err();
        assert_eq!(
            error,
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateTimerFdFailed { code: 12 }
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_rejects_register_failure() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5)
            .with_last_error_code(13)
            .with_register_result(false);

        assert_eq!(
            NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::RegisterTimerFdFailed {
                code: 13,
            }
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_rejects_host_event_creation_failure() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5)
            .with_host_event_raw_fd(-1)
            .with_last_error_code(14);

        assert_eq!(
            NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateHostEventFdFailed {
                code: 14,
            }
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_rejects_host_event_register_failure() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5)
            .with_last_error_code(15)
            .with_register_host_event_result(false);

        assert_eq!(
            NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::RegisterHostEventFdFailed {
                code: 15,
            }
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_signal_host_event_writes_event_fd() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();

        assert_eq!(backend.signal_host_event().unwrap(), ());
        assert_eq!(backend.api().signal_host_event_calls, vec![6]);
        assert_eq!(backend.api().last_error_calls, 0);
        assert!(backend.are_handles_open());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_signal_host_event_preserves_raw_failure() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5)
            .with_last_error_code(16)
            .with_signal_host_event_result(false);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();

        assert_eq!(
            backend.signal_host_event().unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::SignalHostEventFdFailed {
                code: 16,
            }
        );
        assert_eq!(backend.api().signal_host_event_calls, vec![6]);
        assert_eq!(backend.api().last_error_calls, 1);
        assert!(backend.are_handles_open());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_signal_host_event_rejects_closed_backend() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();
        assert_eq!(backend.close_handles_if_open(), true);

        assert_eq!(
            backend.signal_host_event().unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::InvalidHostEventRawFd {
                raw_fd: -1,
            }
        );
        assert!(backend.api().signal_host_event_calls.is_empty());
    }

    #[test]
    fn native_window_linux_host_event_signal_producer_duplicates_and_signals_handle() {
        let backend_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(backend_api).unwrap();
        let producer_api = ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi::new(20);
        let mut producer = backend
            .create_host_event_signal_producer(producer_api)
            .unwrap();

        assert!(producer.are_handles_open());
        assert_eq!(producer.api().clone_calls, vec![6]);
        assert_eq!(producer.signal_host_event().unwrap(), ());
        assert_eq!(producer.api().signal_calls, vec![20]);
        assert_eq!(producer.api().last_error_calls, 0);
    }

    #[test]
    fn native_window_linux_host_event_signal_producer_preserves_clone_failure() {
        let backend_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(backend_api).unwrap();
        let producer_api = ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi::new(-1)
            .with_last_error_code(21);

        assert_eq!(
            backend
                .create_host_event_signal_producer(producer_api)
                .unwrap_err(),
            NativeWindowHostLoopLinuxHostEventSignalProducerError::CreateHostEventSignalFdFailed {
                code: 21,
            }
        );
    }

    #[test]
    fn native_window_linux_host_event_signal_producer_rejects_closed_backend() {
        let backend_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let mut backend =
            NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(backend_api).unwrap();
        assert_eq!(backend.close_handles_if_open(), true);
        let producer_api = ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi::new(20);

        assert_eq!(
            backend
                .create_host_event_signal_producer(producer_api)
                .unwrap_err(),
            NativeWindowHostLoopLinuxHostEventSignalProducerError::InvalidHostEventRawFd {
                raw_fd: -1,
            }
        );
    }

    #[test]
    fn native_window_linux_host_event_signal_producer_preserves_signal_failure() {
        let backend_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(backend_api).unwrap();
        let producer_api = ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi::new(20)
            .with_last_error_code(22)
            .with_signal_result(false);
        let mut producer = backend
            .create_host_event_signal_producer(producer_api)
            .unwrap();

        assert_eq!(
            producer.signal_host_event().unwrap_err(),
            NativeWindowHostLoopLinuxHostEventSignalProducerError::SignalHostEventSignalFdFailed {
                code: 22,
            }
        );
        assert_eq!(producer.api().signal_calls, vec![20]);
        assert_eq!(producer.api().last_error_calls, 1);
        assert!(producer.are_handles_open());
    }

    #[test]
    fn native_window_linux_host_event_signal_producer_closes_signal_handle_once() {
        let backend_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(backend_api).unwrap();
        let producer_api = ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi::new(20);
        let mut producer = backend
            .create_host_event_signal_producer(producer_api)
            .unwrap();

        assert_eq!(producer.close_signal_handle_if_open(), true);
        assert_eq!(producer.close_signal_handle_if_open(), false);
        assert_eq!(producer.api().close_calls, vec![20]);
        assert_eq!(
            producer.signal_host_event().unwrap_err(),
            NativeWindowHostLoopLinuxHostEventSignalProducerError::InvalidHostEventSignalRawFd {
                raw_fd: -1,
            }
        );
    }

    #[cfg(all(feature = "window", target_os = "linux", not(target_arch = "wasm32")))]
    #[test]
    fn native_window_linux_minifb_input_callback_signals_observed_input() {
        let backend_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(backend_api).unwrap();
        let producer_api = ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi::new(92);
        let producer = backend
            .create_host_event_signal_producer(producer_api)
            .unwrap();
        let state = std::rc::Rc::new(std::cell::RefCell::new(
            MinifbNativeWindowLinuxHostEventSignalCallbackState::new(producer),
        ));
        let mut callback = MinifbNativeWindowLinuxHostEventSignalInputCallback::new(state.clone());

        minifb::InputCallback::add_char(&mut callback, 0x3042);
        minifb::InputCallback::set_key_state(&mut callback, minifb::Key::A, true);

        let state = state.borrow();
        assert_eq!(state.producer().api().signal_calls, vec![92, 92]);
        assert_eq!(state.first_error, None);
    }

    #[cfg(all(feature = "window", target_os = "linux", not(target_arch = "wasm32")))]
    #[test]
    fn native_window_linux_minifb_input_callback_records_first_signal_error() {
        let backend_api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(4, 5);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(backend_api).unwrap();
        let producer_api = ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi::new(93)
            .with_signal_result(false)
            .with_last_error_code(77);
        let producer = backend
            .create_host_event_signal_producer(producer_api)
            .unwrap();
        let state = std::rc::Rc::new(std::cell::RefCell::new(
            MinifbNativeWindowLinuxHostEventSignalCallbackState::new(producer),
        ));
        let mut callback = MinifbNativeWindowLinuxHostEventSignalInputCallback::new(state.clone());

        minifb::InputCallback::set_key_state(&mut callback, minifb::Key::A, true);
        minifb::InputCallback::add_char(&mut callback, 0x3042);

        let mut state = state.borrow_mut();
        assert_eq!(state.producer().api().signal_calls, vec![93]);
        assert_eq!(
            state.take_first_error(),
            Some(
                NativeWindowHostLoopLinuxHostEventSignalProducerError::SignalHostEventSignalFdFailed {
                    code: 77,
                }
            )
        );
        assert_eq!(state.take_first_error(), None);
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_wait_for_host_event_uses_event_only_wait() {
        let window_size = NativeWindowSize::new(640, 480);
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(6, 7)
            .with_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY,
            ]);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();

        assert_eq!(backend.wait_for_host_event(window_size, true).unwrap(), ());
        assert_eq!(backend.api().selector_create_calls, 1);
        assert_eq!(backend.api().timer_create_calls, 1);
        assert_eq!(backend.api().host_event_create_calls, 1);
        assert_eq!(backend.api().register_calls, vec![(6, 7)]);
        assert_eq!(backend.api().register_host_event_calls, vec![(6, 8)]);
        assert_eq!(backend.api().event_wait_calls, vec![(6, 8)]);
        assert!(backend.api().timer_wait_calls.is_empty());
        assert!(backend.api().arm_calls.is_empty());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_wait_until_deadline_arms_timespec_and_maps_timer(
    ) {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(8, 9)
            .with_timer_or_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED,
            ]);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();

        let wake = backend
            .wait_until_deadline_or_host_event(10_000_000_000, window_size, false)
            .unwrap();

        assert_eq!(
            wake,
            NativeWindowHostLoopLinuxSelectorTimerFdWake::TimerFired
        );
        assert_eq!(backend.api().arm_calls.len(), 1);
        assert_eq!(backend.api().arm_calls[0].0, 9);
        assert!(backend.api().arm_calls[0].1.seconds() >= 0);
        assert_eq!(backend.api().timer_wait_calls, vec![(8, 9, 10)]);
        assert!(backend.api().event_wait_calls.is_empty());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_wait_until_deadline_maps_host_ready() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(10, 11)
            .with_timer_or_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY,
            ]);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(10_000_000_000, window_size, true)
                .unwrap(),
            NativeWindowHostLoopLinuxSelectorTimerFdWake::HostEventReady
        );
        assert_eq!(backend.api().timer_wait_calls, vec![(10, 11, 12)]);
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_wait_until_deadline_rejects_arm_failure() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(12, 13)
            .with_last_error_code(14)
            .with_arm_result(false);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(10_000_000_000, window_size, false)
                .unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendError::ArmTimerFdFailed { code: 14 }
        );
        assert!(backend.api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_wait_until_deadline_already_reached_avoids_raw_wait(
    ) {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(14, 15);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(0, window_size, false)
                .unwrap(),
            NativeWindowHostLoopLinuxSelectorTimerFdWake::TimerFired
        );
        assert!(backend.api().arm_calls.is_empty());
        assert!(backend.api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_closes_selector_timer_and_host_event_once() {
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(16, 17);
        let mut backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();

        assert_eq!(backend.close_handles_if_open(), true);
        assert_eq!(backend.close_handles_if_open(), false);
        assert_eq!(backend.api().close_host_event_calls, vec![18]);
        assert_eq!(backend.api().close_timer_calls, vec![17]);
        assert_eq!(backend.api().close_selector_calls, vec![16]);
        assert_eq!(backend.are_handles_open(), false);
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_builder_requires_validated_linux_selection() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Macos,
                NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(18, 19);

        assert_eq!(
            build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection(
                selection, api
            )
            .unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: NativeWindowHostLoopPlatformKind::Macos,
                    requested: NativeWindowHostLoopPlatformWaitBackendKind::MacosRunLoopTimer,
                }
            )
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_backend_builder_preserves_raw_api_failure() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(-1, 19)
            .with_last_error_code(20);

        assert_eq!(
            build_native_window_host_loop_linux_selector_timer_fd_backend_from_selection(
                selection, api
            )
            .unwrap_err(),
            NativeWindowHostLoopLinuxSelectorTimerFdBackendBuildError::SelectorTimerFdBackendFailed(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::CreateSelectorFailed {
                    code: 20,
                }
            )
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_wait_trait_maps_timer_to_deadline_reached() {
        let window_size = NativeWindowSize::new(320, 200);
        let frame_interval = native_window_frame_interval_request(NativeWindowTargetFps::default());
        let wait_nanos = frame_interval.nanos_per_frame();
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 21,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval,
            wait_nanos,
        };
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(21, 22)
            .with_timer_or_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED,
            ]);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter,
            )
            .unwrap(),
            NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size,
                size_changed: false,
                wait_nanos,
                timer_registration_id: NativeWindowHostLoopTimerRegistrationId { raw_id: 1 },
            }
        );
        assert_eq!(adapter.backend().api().timer_wait_calls, vec![(21, 22, 23)]);
        assert!(adapter.backend().api().event_wait_calls.is_empty());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_wait_trait_keeps_host_ready_non_timer() {
        let window_size = NativeWindowSize::new(640, 480);
        let frame_interval = native_window_frame_interval_request(NativeWindowTargetFps::default());
        let wait_nanos = frame_interval.nanos_per_frame();
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 22,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval,
            wait_nanos,
        };
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(23, 24)
            .with_timer_or_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_HOST_EVENT_READY,
            ]);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();
        let mut host = NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitRunLoopHost::new(
            ScriptedNativeWindowRunLoopHost::new(Vec::new()),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend),
        );

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap(),
            NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                window_size,
                size_changed: true,
            }
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(
            host.wait_adapter().backend().api().timer_wait_calls,
            vec![(23, 24, 25)]
        );
    }

    #[test]
    fn native_window_linux_selector_timer_fd_wait_trait_rejects_timer_status_for_event_wait() {
        let window_size = NativeWindowSize::new(800, 600);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(25, 26)
            .with_event_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED,
            ]);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter,
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::HostEventWaitFailed(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::UnexpectedSelectorStatus {
                    status: NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_TIMER_FIRED,
                }
            )
        );
        assert_eq!(adapter.backend().api().event_wait_calls, vec![(25, 27)]);
        assert!(adapter.backend().api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_linux_selector_timer_fd_wait_trait_preserves_arm_error() {
        let window_size = NativeWindowSize::new(640, 360);
        let frame_interval = native_window_frame_interval_request(NativeWindowTargetFps::default());
        let wait_nanos = frame_interval.nanos_per_frame();
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 23,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval,
            wait_nanos,
        };
        let api = ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi::new(27, 28)
            .with_last_error_code(29)
            .with_arm_result(false);
        let backend = NativeWindowHostLoopLinuxSelectorTimerFdBackend::new(api).unwrap();
        let mut adapter =
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapter::new(backend);

        assert_eq!(
            execute_native_window_host_loop_single_owner_interruptible_deadline_wait_with_adapter(
                instruction,
                &mut adapter,
            )
            .unwrap_err(),
            NativeWindowHostLoopSingleOwnerInterruptibleDeadlineWaitAdapterError::FrameIntervalWaitFailed(
                NativeWindowHostLoopLinuxSelectorTimerFdBackendError::ArmTimerFdFailed {
                    code: 29,
                }
            )
        );
        assert!(adapter.backend().api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_windows_wait_handle_rejects_null_and_invalid_raw_handles() {
        assert_eq!(
            native_window_host_loop_windows_wait_handle_from_raw(0).unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::InvalidRawHandle { raw_handle: 0 }
        );
        assert_eq!(
            native_window_host_loop_windows_wait_handle_from_raw(-1).unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::InvalidRawHandle { raw_handle: -1 }
        );

        assert!(native_window_host_loop_windows_wait_handle_from_raw(41).is_ok());
    }

    #[test]
    fn native_window_windows_deadline_plan_uses_already_reached_or_rounded_relative_100ns() {
        assert_eq!(
            native_window_host_loop_windows_deadline_plan(1_000, 1_000).unwrap(),
            NativeWindowHostLoopWindowsDeadlinePlan::AlreadyReached
        );
        assert_eq!(
            native_window_host_loop_windows_deadline_plan(1_000, 1_001).unwrap(),
            NativeWindowHostLoopWindowsDeadlinePlan::Relative100ns(-1)
        );
        assert_eq!(
            native_window_host_loop_windows_deadline_plan(1_000, 1_100).unwrap(),
            NativeWindowHostLoopWindowsDeadlinePlan::Relative100ns(-1)
        );
        assert_eq!(
            native_window_host_loop_windows_deadline_plan(1_000, 1_101).unwrap(),
            NativeWindowHostLoopWindowsDeadlinePlan::Relative100ns(-2)
        );
    }

    #[test]
    fn native_window_windows_deadline_plan_rejects_100ns_overflow() {
        assert_eq!(
            native_window_host_loop_windows_deadline_plan(0, u64::MAX).unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::DeadlineDelta100nsOverflow {
                delta_nanos: u64::MAX,
            }
        );
    }

    #[test]
    fn native_window_windows_wait_status_maps_timer_message_and_failures() {
        assert_eq!(
            native_window_host_loop_windows_wait_wake_from_timer_or_message_status(
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMER_SIGNALED,
                0,
            )
            .unwrap(),
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached
        );
        assert_eq!(
            native_window_host_loop_windows_wait_wake_from_timer_or_message_status(
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ONE_HANDLE,
                0,
            )
            .unwrap(),
            NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady
        );
        assert_eq!(
            native_window_host_loop_windows_wait_wake_from_timer_or_message_status(
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_FAILED,
                87,
            )
            .unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::WaitFailed { code: 87 }
        );
        assert_eq!(
            native_window_host_loop_windows_wait_wake_from_timer_or_message_status(
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMEOUT,
                0,
            )
            .unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::UnexpectedWaitStatus {
                status: NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMEOUT,
            }
        );
    }

    #[test]
    fn native_window_windows_message_status_maps_zero_handle_message_ready() {
        assert_eq!(
            native_window_host_loop_windows_host_event_from_message_status(
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ZERO_HANDLES,
                0,
            )
            .unwrap(),
            ()
        );
        assert_eq!(
            native_window_host_loop_windows_host_event_from_message_status(
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_FAILED,
                1234,
            )
            .unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::WaitFailed { code: 1234 }
        );
        assert_eq!(
            native_window_host_loop_windows_host_event_from_message_status(
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMEOUT,
                0,
            )
            .unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::UnexpectedWaitStatus {
                status: NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMEOUT,
            }
        );
    }

    #[test]
    fn native_window_windows_backend_rejects_timer_creation_failure() {
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(0).with_last_error_code(5);

        assert_eq!(
            NativeWindowHostLoopWindowsWaitBackend::new(api).unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::CreateWaitableTimerFailed { code: 5 }
        );
    }

    #[test]
    fn native_window_windows_backend_wait_for_host_event_uses_message_only_wait() {
        let window_size = NativeWindowSize::new(640, 480);
        let api =
            ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(55).with_message_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ZERO_HANDLES,
            ]);
        let mut backend = NativeWindowHostLoopWindowsWaitBackend::new(api).unwrap();

        assert_eq!(backend.wait_for_host_event(window_size, true).unwrap(), ());
        assert_eq!(backend.api().create_calls, 1);
        assert_eq!(backend.api().message_wait_calls, 1);
        assert!(backend.api().timer_wait_calls.is_empty());
        assert!(backend.api().set_calls.is_empty());
    }

    #[test]
    fn native_window_windows_backend_wait_until_deadline_sets_timer_and_maps_deadline() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(61)
            .with_timer_or_message_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMER_SIGNALED,
            ]);
        let mut backend = NativeWindowHostLoopWindowsWaitBackend::new(api).unwrap();

        let wake = backend
            .wait_until_deadline_or_host_event(100_000_000, window_size, false)
            .unwrap();

        assert_eq!(
            wake,
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached
        );
        assert_eq!(backend.api().set_calls.len(), 1);
        assert_eq!(backend.api().set_calls[0].0, 61);
        assert!(backend.api().set_calls[0].1 < 0);
        assert_eq!(backend.api().timer_wait_calls, vec![61]);
        assert_eq!(backend.api().message_wait_calls, 0);
    }

    #[test]
    fn native_window_windows_backend_wait_until_deadline_maps_host_ready() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(62)
            .with_timer_or_message_statuses(vec![
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_MESSAGE_READY_ONE_HANDLE,
            ]);
        let mut backend = NativeWindowHostLoopWindowsWaitBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(100_000_000, window_size, true)
                .unwrap(),
            NativeWindowHostLoopInterruptibleDeadlineWake::HostEventReady
        );
        assert_eq!(backend.api().timer_wait_calls, vec![62]);
    }

    #[test]
    fn native_window_windows_backend_wait_until_deadline_rejects_set_timer_failure() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(63)
            .with_last_error_code(1460)
            .with_set_result(false);
        let mut backend = NativeWindowHostLoopWindowsWaitBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(100_000_000, window_size, false)
                .unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendError::SetWaitableTimerFailed { code: 1460 }
        );
        assert!(backend.api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_windows_backend_wait_until_deadline_already_reached_avoids_raw_wait() {
        let window_size = NativeWindowSize::new(320, 200);
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(64);
        let mut backend = NativeWindowHostLoopWindowsWaitBackend::new(api).unwrap();

        assert_eq!(
            backend
                .wait_until_deadline_or_host_event(0, window_size, false)
                .unwrap(),
            NativeWindowHostLoopInterruptibleDeadlineWake::DeadlineReached
        );
        assert!(backend.api().set_calls.is_empty());
        assert!(backend.api().timer_wait_calls.is_empty());
    }

    #[test]
    fn native_window_windows_backend_close_handle_once() {
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(71);
        let mut backend = NativeWindowHostLoopWindowsWaitBackend::new(api).unwrap();

        assert_eq!(backend.close_handle_if_open(), true);
        assert_eq!(backend.close_handle_if_open(), false);
        assert_eq!(backend.api().close_calls, vec![71]);
        assert_eq!(backend.is_handle_open(), false);
    }

    #[test]
    fn native_window_windows_backend_builder_requires_validated_windows_selection() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Linux,
                NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(72);

        assert_eq!(
            build_native_window_host_loop_windows_wait_backend_from_selection(selection, api)
                .unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendBuildError::BackendSupportFailed(
                NativeWindowHostLoopPlatformWaitBackendSupportError::BackendPlatformMismatch {
                    current: NativeWindowHostLoopPlatformKind::Linux,
                    requested: NativeWindowHostLoopPlatformWaitBackendKind::LinuxSelectorTimerFd,
                }
            )
        );
    }

    #[test]
    fn native_window_windows_backend_builder_preserves_raw_api_failure() {
        let selection =
            validate_native_window_host_loop_platform_wait_backend_selection_for_platform(
                NativeWindowHostLoopPlatformKind::Windows,
                NativeWindowHostLoopPlatformWaitBackendKind::WindowsWaitableTimerMessageWait,
            )
            .unwrap();
        let api = ScriptedNativeWindowHostLoopWindowsWaitRawApi::new(0).with_last_error_code(8);

        assert_eq!(
            build_native_window_host_loop_windows_wait_backend_from_selection(selection, api)
                .unwrap_err(),
            NativeWindowHostLoopWindowsWaitBackendBuildError::WaitBackendFailed(
                NativeWindowHostLoopWindowsWaitBackendError::CreateWaitableTimerFailed { code: 8 }
            )
        );
    }

    #[test]
    fn native_window_event_queue_status_waiter_accepts_ready_status_through_wait_boundary() {
        let window_size = NativeWindowSize::new(900, 700);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let adapter = ScriptedNativeWindowHostLoopEventQueueStatusAdapter::new(
            NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY,
        );
        let mut waiter = NativeWindowHostLoopEventQueueStatusWaiter::new(adapter);

        assert_eq!(
            execute_native_window_host_loop_event_queue_wait_with_waiter(instruction, &mut waiter)
                .unwrap(),
            NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady {
                window_size,
                size_changed: true,
            }
        );
        assert_eq!(waiter.adapter().status_calls, vec![(window_size, true)]);
    }

    #[test]
    fn native_window_event_queue_status_waiter_rejects_invalid_raw_status() {
        let window_size = NativeWindowSize::new(900, 700);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let adapter = ScriptedNativeWindowHostLoopEventQueueStatusAdapter::new(0);
        let mut waiter = NativeWindowHostLoopEventQueueStatusWaiter::new(adapter);

        assert_eq!(
            execute_native_window_host_loop_event_queue_wait_with_waiter(instruction, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopEventQueueWaitError::WaiterFailed(
                NativeWindowHostLoopEventQueueStatusAdapterError::InvalidRawStatus {
                    raw_status: 0,
                }
            )
        );
        assert_eq!(waiter.adapter().status_calls, vec![(window_size, false)]);
    }

    #[test]
    fn native_window_event_queue_status_waiter_preserves_adapter_error() {
        let window_size = NativeWindowSize::new(900, 700);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let adapter = ScriptedNativeWindowHostLoopEventQueueStatusAdapter::new(
            NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY,
        )
        .with_error("queue adapter failed");
        let mut waiter = NativeWindowHostLoopEventQueueStatusWaiter::new(adapter);

        assert_eq!(
            execute_native_window_host_loop_event_queue_wait_with_waiter(instruction, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopEventQueueWaitError::WaiterFailed(
                NativeWindowHostLoopEventQueueStatusAdapterError::AdapterFailed(
                    "queue adapter failed"
                )
            )
        );
        assert_eq!(waiter.adapter().status_calls, vec![(window_size, false)]);
    }

    #[test]
    fn native_window_event_queue_status_waiter_is_not_called_for_frame_interval_instruction() {
        let window_size = NativeWindowSize::new(320, 200);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 16,
            width: window_size.width,
            height: window_size.height,
        };
        let frame_interval = native_window_frame_interval_request(NativeWindowTargetFps::default());
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: false,
            frame_interval,
            wait_nanos: 16_666_666,
        };
        let adapter = ScriptedNativeWindowHostLoopEventQueueStatusAdapter::new(
            NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY,
        );
        let mut waiter = NativeWindowHostLoopEventQueueStatusWaiter::new(adapter);

        assert_eq!(
            execute_native_window_host_loop_event_queue_wait_with_waiter(instruction, &mut waiter)
                .unwrap_err(),
            NativeWindowHostLoopEventQueueWaitError::FrameIntervalEventQueueWaitUnsupported {
                presentation,
                window_size,
                size_changed: false,
                frame_interval,
                wait_nanos: 16_666_666,
            }
        );
        assert!(waiter.adapter().status_calls.is_empty());
    }

    #[test]
    fn native_window_message_pump_status_adapter_maps_success_to_ready_status() {
        let window_size = NativeWindowSize::new(1024, 768);
        let pump_adapter = ScriptedNativeWindowHostLoopMessagePumpAdapter::new();
        let mut status_adapter = NativeWindowHostLoopMessagePumpStatusAdapter::new(pump_adapter);

        assert_eq!(
            status_adapter
                .wait_for_host_event_raw_status(window_size, true)
                .unwrap(),
            NATIVE_WINDOW_HOST_EVENT_QUEUE_NORMALIZED_STATUS_READY
        );
        assert_eq!(
            status_adapter.adapter().pump_calls,
            vec![(window_size, true)]
        );
    }

    #[test]
    fn native_window_message_pump_status_adapter_preserves_pump_error() {
        let window_size = NativeWindowSize::new(1024, 768);
        let pump_adapter =
            ScriptedNativeWindowHostLoopMessagePumpAdapter::new().with_error("pump failed");
        let mut status_adapter = NativeWindowHostLoopMessagePumpStatusAdapter::new(pump_adapter);

        assert_eq!(
            status_adapter
                .wait_for_host_event_raw_status(window_size, false)
                .unwrap_err(),
            NativeWindowHostLoopMessagePumpStatusAdapterError::PumpFailed("pump failed")
        );
        assert_eq!(
            status_adapter.adapter().pump_calls,
            vec![(window_size, false)]
        );
    }

    #[test]
    fn native_window_message_pump_waiter_reaches_event_queue_wait_boundary() {
        let window_size = NativeWindowSize::new(1024, 768);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: true,
        };
        let pump_adapter = ScriptedNativeWindowHostLoopMessagePumpAdapter::new();
        let status_adapter = NativeWindowHostLoopMessagePumpStatusAdapter::new(pump_adapter);
        let mut waiter = NativeWindowHostLoopEventQueueStatusWaiter::new(status_adapter);

        assert_eq!(
            execute_native_window_host_loop_event_queue_wait_with_waiter(instruction, &mut waiter)
                .unwrap(),
            NativeWindowHostLoopEventQueueWaitOutcome::HostEventReady {
                window_size,
                size_changed: true,
            }
        );
        assert_eq!(
            waiter.adapter().adapter().pump_calls,
            vec![(window_size, true)]
        );
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
                instruction: NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
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
            host.wait_instructions,
            vec![NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
                window_size: unavailable_size,
                size_changed: true,
            }]
        );
    }

    #[test]
    fn native_window_host_loop_scheduler_slice_accumulates_frame_interval_remainder() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let first_drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let second_drawable = first_drawable;
        let mut host =
            ScriptedNativeWindowRunLoopHost::new(vec![Ok(first_drawable), Ok(second_drawable)]);
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
            scheduler_state
                .wait_strategy_state()
                .frame_pacing_remainder_nanos(),
            40
        );

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
            scheduler_state
                .wait_strategy_state()
                .frame_pacing_remainder_nanos(),
            20
        );

        assert_eq!(host.wait_instructions.len(), 2);
        match &host.wait_instructions[0] {
            NativeWindowHostLoopWaitInstruction::WaitForFrameInterval { wait_nanos, .. } => {
                assert_eq!(*wait_nanos, 16_666_666);
            }
            NativeWindowHostLoopWaitInstruction::WaitForHostEvent { .. } => {
                panic!("frame interval instruction expected")
            }
        }
        match &host.wait_instructions[1] {
            NativeWindowHostLoopWaitInstruction::WaitForFrameInterval { wait_nanos, .. } => {
                assert_eq!(*wait_nanos, 16_666_667);
            }
            NativeWindowHostLoopWaitInstruction::WaitForHostEvent { .. } => {
                panic!("frame interval instruction expected")
            }
        }
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
            host.wait_instructions,
            vec![NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
                window_size: unavailable_size,
                size_changed: true,
            }]
        );
        assert_eq!(
            scheduler_state
                .wait_strategy_state()
                .frame_pacing_remainder_nanos(),
            0
        );
        assert!(host.wait_outcomes.is_empty());
    }

    #[test]
    fn native_window_host_loop_scheduler_slice_keeps_wait_strategy_state_on_wait_error() {
        let mut loop_state = native_window_backend_loop_counter();
        let initial_size = loop_state.initial_size();
        let drawable = native_window_backend_loop_snapshot(
            &loop_state,
            NativeWindowEventPumpCloseState::Open,
            initial_size,
            false,
            NativeWindowPointerSample::Unavailable,
        );
        let mut host =
            ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable)]).with_wait_error("wait failed");
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
        assert_eq!(
            scheduler_state
                .wait_strategy_state()
                .frame_pacing_remainder_nanos(),
            0
        );
        assert_eq!(host.wait_instructions.len(), 1);
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
        assert!(host.wait_instructions.is_empty());
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
            host.wait_instructions,
            vec![NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
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
            host.wait_instructions,
            vec![NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size: initial_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(
                    NativeWindowTargetFps::default()
                ),
                wait_nanos: 16_666_666,
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
    fn native_window_host_loop_with_policy_requires_timer_fire_before_resume() {
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
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 1,
            width: initial_size.width,
            height: initial_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 103 };
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable), Ok(close)])
            .with_wait_outcome(
                NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
                    presentation,
                    window_size: initial_size,
                    size_changed: false,
                    wait_nanos: 16_666_666,
                    timer_registration_id,
                },
            );

        assert_eq!(
            run_native_window_host_loop_with_policy(
                &mut loop_state,
                &mut host,
                NativeWindowHostLoopRunPolicy::default()
            )
            .unwrap_err(),
            NativeWindowHostLoopError::TimerFireResumeRequired {
                presentation,
                window_size: initial_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id,
            }
        );
        assert_eq!(host.cursor, 1);
        assert_eq!(
            host.wait_instructions,
            vec![NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size: initial_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(
                    NativeWindowTargetFps::default()
                ),
                wait_nanos: 16_666_666,
            }]
        );
        assert_eq!(
            host.wait_outcomes,
            vec![
                NativeWindowHostLoopWaitOutcome::FrameIntervalTimerRegistered {
                    presentation,
                    window_size: initial_size,
                    size_changed: false,
                    wait_nanos: 16_666_666,
                    timer_registration_id,
                }
            ]
        );
    }

    #[test]
    fn native_window_host_loop_with_policy_resumes_after_timer_fire_wait_outcome() {
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
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 1,
            width: initial_size.width,
            height: initial_size.height,
        };
        let timer_registration_id = NativeWindowHostLoopTimerRegistrationId { raw_id: 104 };
        let mut host = ScriptedNativeWindowRunLoopHost::new(vec![Ok(drawable), Ok(close)])
            .with_wait_outcome(NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size: initial_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id,
            });

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
        assert_eq!(
            host.wait_instructions,
            vec![NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size: initial_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(
                    NativeWindowTargetFps::default()
                ),
                wait_nanos: 16_666_666,
            }]
        );
        assert_eq!(
            host.wait_outcomes,
            vec![NativeWindowHostLoopWaitOutcome::FrameIntervalTimerFired {
                presentation,
                window_size: initial_size,
                size_changed: false,
                wait_nanos: 16_666_666,
                timer_registration_id,
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
            host.wait_instructions,
            vec![NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                presentation,
                window_size: initial_size,
                size_changed: false,
                frame_interval: native_window_frame_interval_request(target_fps),
                wait_nanos: 8_333_333,
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
            host.wait_instructions,
            vec![NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
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
        wait_instructions: Vec<NativeWindowHostLoopWaitInstruction>,
        wait_outcomes: Vec<NativeWindowHostLoopWaitOutcome>,
        present_error: Option<&'static str>,
        wait_error: Option<&'static str>,
        wait_outcome_override: Option<NativeWindowHostLoopWaitOutcome>,
    }

    impl ScriptedNativeWindowRunLoopHost {
        fn new(snapshots: Vec<Result<NativeWindowEventPumpSnapshot, &'static str>>) -> Self {
            Self {
                snapshots,
                cursor: 0,
                titles: Vec::new(),
                pump_count: 0,
                present_frames: Vec::new(),
                wait_instructions: Vec::new(),
                wait_outcomes: Vec::new(),
                present_error: None,
                wait_error: None,
                wait_outcome_override: None,
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

        fn with_wait_outcome(mut self, outcome: NativeWindowHostLoopWaitOutcome) -> Self {
            self.wait_outcome_override = Some(outcome);
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
            instruction: NativeWindowHostLoopWaitInstruction,
        ) -> Result<NativeWindowHostLoopWaitOutcome, Self::WaitError> {
            self.wait_instructions.push(instruction.clone());
            if let Some(error) = self.wait_error {
                return Err(error);
            }
            let outcome = if let Some(outcome) = self.wait_outcome_override.clone() {
                outcome
            } else {
                match instruction {
                    NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
                        window_size,
                        size_changed,
                    } => NativeWindowHostLoopWaitOutcome::HostEventPumpAlreadyPaced {
                        window_size,
                        size_changed,
                    },
                    NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
                        presentation,
                        window_size,
                        size_changed,
                        frame_interval: _,
                        wait_nanos: _,
                    } => NativeWindowHostLoopWaitOutcome::FramePresentAlreadyPaced {
                        presentation,
                        window_size,
                        size_changed,
                    },
                }
            };
            self.wait_outcomes.push(outcome.clone());
            Ok(outcome)
        }
    }

    #[derive(Debug)]
    struct ScriptedNativeWindowHostEventSignalErrorState {
        error: Option<NativeWindowHostLoopLinuxHostEventSignalProducerError>,
        take_calls: usize,
    }

    impl ScriptedNativeWindowHostEventSignalErrorState {
        fn clean() -> Self {
            Self {
                error: None,
                take_calls: 0,
            }
        }

        fn with_error(error: NativeWindowHostLoopLinuxHostEventSignalProducerError) -> Self {
            Self {
                error: Some(error),
                take_calls: 0,
            }
        }
    }

    impl NativeWindowHostEventSignalErrorState for ScriptedNativeWindowHostEventSignalErrorState {
        fn take_host_event_signal_error(
            &mut self,
        ) -> Option<NativeWindowHostLoopLinuxHostEventSignalProducerError> {
            self.take_calls += 1;
            self.error.take()
        }
    }

    #[test]
    fn native_window_host_event_signal_wait_guard_returns_signal_error_before_delegate_wait() {
        let window_size = NativeWindowSize::new(640, 480);
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForHostEvent {
            window_size,
            size_changed: false,
        };
        let delegate = ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("wait");
        let state = ScriptedNativeWindowHostEventSignalErrorState::with_error(
            NativeWindowHostLoopLinuxHostEventSignalProducerError::SignalHostEventSignalFdFailed {
                code: 11,
            },
        );
        let mut host = NativeWindowHostEventSignalWaitGuardRunLoopHost::new(delegate, state);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction).unwrap_err(),
            NativeWindowHostEventSignalWaitError::HostEventSignalFailed(
                NativeWindowHostLoopLinuxHostEventSignalProducerError::SignalHostEventSignalFdFailed {
                    code: 11,
                },
            )
        );
        assert!(host.host().wait_instructions.is_empty());
        assert_eq!(host.signal_state().take_calls, 1);
    }

    #[test]
    fn native_window_host_event_signal_wait_guard_delegates_without_synthetic_outcome() {
        let window_size = NativeWindowSize::new(320, 240);
        let presentation = NativeWindowBackendLoopPresentation {
            frame_id: 42,
            width: window_size.width,
            height: window_size.height,
        };
        let instruction = NativeWindowHostLoopWaitInstruction::WaitForFrameInterval {
            presentation,
            window_size,
            size_changed: true,
            frame_interval: native_window_frame_interval_request(NativeWindowTargetFps::default()),
            wait_nanos: 16_666_666,
        };
        let delegate = ScriptedNativeWindowRunLoopHost::new(Vec::new()).with_wait_error("wait");
        let state = ScriptedNativeWindowHostEventSignalErrorState::clean();
        let mut host = NativeWindowHostEventSignalWaitGuardRunLoopHost::new(delegate, state);

        assert_eq!(
            host.wait_after_budget_exhausted(instruction.clone())
                .unwrap_err(),
            NativeWindowHostEventSignalWaitError::DelegateWaitFailed("wait")
        );
        assert_eq!(host.host().wait_instructions, vec![instruction]);
        assert_eq!(host.signal_state().take_calls, 1);
    }

    struct ScriptedNativeWindowHostLoopThreadSleeper {
        sleep_calls: Vec<u32>,
        error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopThreadSleeper {
        fn new() -> Self {
            Self {
                sleep_calls: Vec::new(),
                error: None,
            }
        }

        fn with_error(mut self, error: &'static str) -> Self {
            self.error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopThreadSleeper for ScriptedNativeWindowHostLoopThreadSleeper {
        type Error = &'static str;

        fn sleep_for_nanos(&mut self, wait_nanos: u32) -> Result<(), Self::Error> {
            self.sleep_calls.push(wait_nanos);
            if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(())
            }
        }
    }

    struct ScriptedNativeWindowHostLoopTimerRegistrar {
        raw_id: u32,
        registration_calls: Vec<u32>,
        error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopTimerRegistrar {
        fn new(raw_id: u32) -> Self {
            Self {
                raw_id,
                registration_calls: Vec::new(),
                error: None,
            }
        }

        fn with_error(mut self, error: &'static str) -> Self {
            self.error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopTimerRegistrar for ScriptedNativeWindowHostLoopTimerRegistrar {
        type Error = &'static str;

        fn register_timer_nanos(&mut self, wait_nanos: u32) -> Result<u32, Self::Error> {
            self.registration_calls.push(wait_nanos);
            if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(self.raw_id)
            }
        }
    }

    struct ScriptedNativeWindowHostLoopTimerFireWaiter {
        raw_id: u32,
        wait_calls: Vec<NativeWindowHostLoopTimerRegistrationId>,
        error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopTimerFireWaiter {
        fn new(raw_id: u32) -> Self {
            Self {
                raw_id,
                wait_calls: Vec::new(),
                error: None,
            }
        }

        fn with_error(mut self, error: &'static str) -> Self {
            self.error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopTimerFireWaiter for ScriptedNativeWindowHostLoopTimerFireWaiter {
        type Error = &'static str;

        fn wait_for_timer_fire(
            &mut self,
            timer_registration_id: NativeWindowHostLoopTimerRegistrationId,
        ) -> Result<u32, Self::Error> {
            self.wait_calls.push(timer_registration_id);
            if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(self.raw_id)
            }
        }
    }

    struct ScriptedNativeWindowHostLoopDeadlineTimerClock {
        now_nanos: u64,
        now_calls: usize,
        error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopDeadlineTimerClock {
        fn new(now_nanos: u64) -> Self {
            Self {
                now_nanos,
                now_calls: 0,
                error: None,
            }
        }

        fn with_error(mut self, error: &'static str) -> Self {
            self.error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopDeadlineTimerClock for ScriptedNativeWindowHostLoopDeadlineTimerClock {
        type Error = &'static str;

        fn now_nanos(&mut self) -> Result<u64, Self::Error> {
            self.now_calls += 1;
            if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(self.now_nanos)
            }
        }
    }

    struct ScriptedNativeWindowHostLoopDeadlineTimerSleeper {
        sleep_until_calls: Vec<u64>,
        error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopDeadlineTimerSleeper {
        fn new() -> Self {
            Self {
                sleep_until_calls: Vec::new(),
                error: None,
            }
        }

        fn with_error(mut self, error: &'static str) -> Self {
            self.error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopDeadlineTimerSleeper for ScriptedNativeWindowHostLoopDeadlineTimerSleeper {
        type Error = &'static str;

        fn sleep_until_nanos(&mut self, deadline_nanos: u64) -> Result<(), Self::Error> {
            self.sleep_until_calls.push(deadline_nanos);
            if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(())
            }
        }
    }

    struct ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter {
        wake: NativeWindowHostLoopInterruptibleDeadlineWake,
        host_event_calls: Vec<(NativeWindowSize, bool)>,
        frame_interval_calls: Vec<(u64, NativeWindowSize, bool)>,
        host_event_error: Option<&'static str>,
        frame_interval_error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter {
        fn new(wake: NativeWindowHostLoopInterruptibleDeadlineWake) -> Self {
            Self {
                wake,
                host_event_calls: Vec::new(),
                frame_interval_calls: Vec::new(),
                host_event_error: None,
                frame_interval_error: None,
            }
        }

        fn with_host_event_error(mut self, error: &'static str) -> Self {
            self.host_event_error = Some(error);
            self
        }

        fn with_frame_interval_error(mut self, error: &'static str) -> Self {
            self.frame_interval_error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopInterruptibleDeadlineWaiter
        for ScriptedNativeWindowHostLoopInterruptibleDeadlineWaiter
    {
        type Error = &'static str;

        fn wait_for_host_event(
            &mut self,
            window_size: NativeWindowSize,
            size_changed: bool,
        ) -> Result<(), Self::Error> {
            self.host_event_calls.push((window_size, size_changed));
            if let Some(error) = self.host_event_error {
                Err(error)
            } else {
                Ok(())
            }
        }

        fn wait_until_deadline_or_host_event(
            &mut self,
            deadline_nanos: u64,
            window_size: NativeWindowSize,
            size_changed: bool,
        ) -> Result<NativeWindowHostLoopInterruptibleDeadlineWake, Self::Error> {
            self.frame_interval_calls
                .push((deadline_nanos, window_size, size_changed));
            if let Some(error) = self.frame_interval_error {
                Err(error)
            } else {
                Ok(self.wake)
            }
        }
    }

    struct ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend {
        now_nanos: u64,
        wake: NativeWindowHostLoopInterruptibleDeadlineWake,
        now_calls: usize,
        host_event_calls: Vec<(NativeWindowSize, bool)>,
        frame_interval_calls: Vec<(u64, NativeWindowSize, bool)>,
        clock_error: Option<&'static str>,
        host_event_error: Option<&'static str>,
        frame_interval_error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend {
        fn new(now_nanos: u64, wake: NativeWindowHostLoopInterruptibleDeadlineWake) -> Self {
            Self {
                now_nanos,
                wake,
                now_calls: 0,
                host_event_calls: Vec::new(),
                frame_interval_calls: Vec::new(),
                clock_error: None,
                host_event_error: None,
                frame_interval_error: None,
            }
        }

        fn with_clock_error(mut self, error: &'static str) -> Self {
            self.clock_error = Some(error);
            self
        }

        fn with_host_event_error(mut self, error: &'static str) -> Self {
            self.host_event_error = Some(error);
            self
        }

        fn with_frame_interval_error(mut self, error: &'static str) -> Self {
            self.frame_interval_error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopDeadlineTimerClock
        for ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend
    {
        type Error = &'static str;

        fn now_nanos(&mut self) -> Result<u64, Self::Error> {
            self.now_calls += 1;
            if let Some(error) = self.clock_error {
                Err(error)
            } else {
                Ok(self.now_nanos)
            }
        }
    }

    impl NativeWindowHostLoopInterruptibleDeadlineWaiter
        for ScriptedNativeWindowHostLoopSingleOwnerInterruptibleDeadlineBackend
    {
        type Error = &'static str;

        fn wait_for_host_event(
            &mut self,
            window_size: NativeWindowSize,
            size_changed: bool,
        ) -> Result<(), Self::Error> {
            self.host_event_calls.push((window_size, size_changed));
            if let Some(error) = self.host_event_error {
                Err(error)
            } else {
                Ok(())
            }
        }

        fn wait_until_deadline_or_host_event(
            &mut self,
            deadline_nanos: u64,
            window_size: NativeWindowSize,
            size_changed: bool,
        ) -> Result<NativeWindowHostLoopInterruptibleDeadlineWake, Self::Error> {
            self.frame_interval_calls
                .push((deadline_nanos, window_size, size_changed));
            if let Some(error) = self.frame_interval_error {
                Err(error)
            } else {
                Ok(self.wake)
            }
        }
    }

    #[derive(Debug)]
    struct ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi {
        create_raw_handle: isize,
        schedule_result: bool,
        timer_or_event_statuses: Vec<u32>,
        event_statuses: Vec<u32>,
        last_error_code: u32,
        create_calls: usize,
        schedule_calls: Vec<(isize, u64)>,
        timer_wait_calls: Vec<isize>,
        event_wait_calls: usize,
        invalidate_calls: Vec<isize>,
        last_error_calls: usize,
    }

    impl ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi {
        fn new(create_raw_handle: isize) -> Self {
            Self {
                create_raw_handle,
                schedule_result: true,
                timer_or_event_statuses: Vec::new(),
                event_statuses: Vec::new(),
                last_error_code: 0,
                create_calls: 0,
                schedule_calls: Vec::new(),
                timer_wait_calls: Vec::new(),
                event_wait_calls: 0,
                invalidate_calls: Vec::new(),
                last_error_calls: 0,
            }
        }

        fn with_last_error_code(mut self, code: u32) -> Self {
            self.last_error_code = code;
            self
        }

        fn with_schedule_result(mut self, result: bool) -> Self {
            self.schedule_result = result;
            self
        }

        fn with_timer_or_event_statuses(mut self, statuses: Vec<u32>) -> Self {
            self.timer_or_event_statuses = statuses;
            self
        }

        fn with_event_statuses(mut self, statuses: Vec<u32>) -> Self {
            self.event_statuses = statuses;
            self
        }

        fn next_status(statuses: &mut Vec<u32>) -> u32 {
            if statuses.is_empty() {
                NATIVE_WINDOW_HOST_LOOP_MACOS_RUN_LOOP_STATUS_FAILED
            } else {
                statuses.remove(0)
            }
        }
    }

    impl NativeWindowHostLoopMacosRunLoopTimerRawApi
        for ScriptedNativeWindowHostLoopMacosRunLoopTimerRawApi
    {
        fn create_run_loop_timer_raw(&mut self) -> isize {
            self.create_calls += 1;
            self.create_raw_handle
        }

        fn schedule_run_loop_timer_relative_nanos(
            &mut self,
            handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
            relative_nanos: u64,
        ) -> bool {
            self.schedule_calls.push((
                native_window_host_loop_macos_run_loop_timer_handle_raw(handle),
                relative_nanos,
            ));
            self.schedule_result
        }

        fn run_loop_wait_for_timer_or_event_raw(
            &mut self,
            handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
        ) -> u32 {
            self.timer_wait_calls
                .push(native_window_host_loop_macos_run_loop_timer_handle_raw(
                    handle,
                ));
            Self::next_status(&mut self.timer_or_event_statuses)
        }

        fn run_loop_wait_for_event_raw(&mut self) -> u32 {
            self.event_wait_calls += 1;
            Self::next_status(&mut self.event_statuses)
        }

        fn invalidate_run_loop_timer_raw(
            &mut self,
            handle: &NativeWindowHostLoopMacosRunLoopTimerHandle,
        ) -> bool {
            self.invalidate_calls
                .push(native_window_host_loop_macos_run_loop_timer_handle_raw(
                    handle,
                ));
            true
        }

        fn last_error_code(&mut self) -> u32 {
            self.last_error_calls += 1;
            self.last_error_code
        }
    }

    #[derive(Debug)]
    struct ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi {
        selector_raw_fd: i32,
        timer_raw_fd: i32,
        host_event_raw_fd: i32,
        register_result: bool,
        register_host_event_result: bool,
        signal_host_event_result: bool,
        arm_result: bool,
        timer_or_event_statuses: Vec<u32>,
        event_statuses: Vec<u32>,
        last_error_code: u32,
        selector_create_calls: usize,
        timer_create_calls: usize,
        host_event_create_calls: usize,
        register_calls: Vec<(i32, i32)>,
        register_host_event_calls: Vec<(i32, i32)>,
        signal_host_event_calls: Vec<i32>,
        arm_calls: Vec<(i32, NativeWindowHostLoopLinuxTimerFdTimespec)>,
        timer_wait_calls: Vec<(i32, i32, i32)>,
        event_wait_calls: Vec<(i32, i32)>,
        close_selector_calls: Vec<i32>,
        close_timer_calls: Vec<i32>,
        close_host_event_calls: Vec<i32>,
        last_error_calls: usize,
        raw_method_call_counter: Option<std::rc::Rc<std::cell::Cell<usize>>>,
    }

    impl ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi {
        fn new(selector_raw_fd: i32, timer_raw_fd: i32) -> Self {
            let host_event_raw_fd = timer_raw_fd + 1;
            Self {
                selector_raw_fd,
                timer_raw_fd,
                host_event_raw_fd,
                register_result: true,
                register_host_event_result: true,
                signal_host_event_result: true,
                arm_result: true,
                timer_or_event_statuses: Vec::new(),
                event_statuses: Vec::new(),
                last_error_code: 0,
                selector_create_calls: 0,
                timer_create_calls: 0,
                host_event_create_calls: 0,
                register_calls: Vec::new(),
                register_host_event_calls: Vec::new(),
                signal_host_event_calls: Vec::new(),
                arm_calls: Vec::new(),
                timer_wait_calls: Vec::new(),
                event_wait_calls: Vec::new(),
                close_selector_calls: Vec::new(),
                close_timer_calls: Vec::new(),
                close_host_event_calls: Vec::new(),
                last_error_calls: 0,
                raw_method_call_counter: None,
            }
        }

        fn count_raw_method_call(&self) {
            if let Some(counter) = &self.raw_method_call_counter {
                counter.set(counter.get() + 1);
            }
        }

        fn with_raw_method_call_counter(
            mut self,
            counter: std::rc::Rc<std::cell::Cell<usize>>,
        ) -> Self {
            self.raw_method_call_counter = Some(counter);
            self
        }

        fn with_host_event_raw_fd(mut self, raw_fd: i32) -> Self {
            self.host_event_raw_fd = raw_fd;
            self
        }

        fn with_last_error_code(mut self, code: u32) -> Self {
            self.last_error_code = code;
            self
        }

        fn with_register_result(mut self, result: bool) -> Self {
            self.register_result = result;
            self
        }

        fn with_register_host_event_result(mut self, result: bool) -> Self {
            self.register_host_event_result = result;
            self
        }

        fn with_signal_host_event_result(mut self, result: bool) -> Self {
            self.signal_host_event_result = result;
            self
        }

        fn with_arm_result(mut self, result: bool) -> Self {
            self.arm_result = result;
            self
        }

        fn with_timer_or_event_statuses(mut self, statuses: Vec<u32>) -> Self {
            self.timer_or_event_statuses = statuses;
            self
        }

        fn with_event_statuses(mut self, statuses: Vec<u32>) -> Self {
            self.event_statuses = statuses;
            self
        }

        fn next_status(statuses: &mut Vec<u32>) -> u32 {
            if statuses.is_empty() {
                NATIVE_WINDOW_HOST_LOOP_LINUX_SELECTOR_STATUS_FAILED
            } else {
                statuses.remove(0)
            }
        }
    }

    impl NativeWindowHostLoopLinuxSelectorTimerFdRawApi
        for ScriptedNativeWindowHostLoopLinuxSelectorTimerFdRawApi
    {
        fn create_selector_raw(&mut self) -> i32 {
            self.count_raw_method_call();
            self.selector_create_calls += 1;
            self.selector_raw_fd
        }

        fn create_timer_fd_raw(&mut self) -> i32 {
            self.count_raw_method_call();
            self.timer_create_calls += 1;
            self.timer_raw_fd
        }

        fn create_host_event_fd_raw(&mut self) -> i32 {
            self.count_raw_method_call();
            self.host_event_create_calls += 1;
            self.host_event_raw_fd
        }

        fn register_timer_fd_raw(
            &mut self,
            selector: &NativeWindowHostLoopLinuxSelectorFd,
            timer: &NativeWindowHostLoopLinuxTimerFd,
        ) -> bool {
            self.count_raw_method_call();
            self.register_calls.push((
                native_window_host_loop_linux_selector_fd_raw(selector),
                native_window_host_loop_linux_timer_fd_raw(timer),
            ));
            self.register_result
        }

        fn register_host_event_fd_raw(
            &mut self,
            selector: &NativeWindowHostLoopLinuxSelectorFd,
            host_event: &NativeWindowHostLoopLinuxHostEventFd,
        ) -> bool {
            self.count_raw_method_call();
            self.register_host_event_calls.push((
                native_window_host_loop_linux_selector_fd_raw(selector),
                native_window_host_loop_linux_host_event_fd_raw(host_event),
            ));
            self.register_host_event_result
        }

        fn signal_host_event_fd_raw(
            &mut self,
            host_event: &NativeWindowHostLoopLinuxHostEventFd,
        ) -> bool {
            self.count_raw_method_call();
            self.signal_host_event_calls
                .push(native_window_host_loop_linux_host_event_fd_raw(host_event));
            self.signal_host_event_result
        }

        fn arm_timer_fd_relative_timespec(
            &mut self,
            timer: &NativeWindowHostLoopLinuxTimerFd,
            timespec: NativeWindowHostLoopLinuxTimerFdTimespec,
        ) -> bool {
            self.count_raw_method_call();
            self.arm_calls
                .push((native_window_host_loop_linux_timer_fd_raw(timer), timespec));
            self.arm_result
        }

        fn selector_wait_for_timer_or_event_raw(
            &mut self,
            selector: &NativeWindowHostLoopLinuxSelectorFd,
            timer: &NativeWindowHostLoopLinuxTimerFd,
            host_event: &NativeWindowHostLoopLinuxHostEventFd,
        ) -> u32 {
            self.count_raw_method_call();
            self.timer_wait_calls.push((
                native_window_host_loop_linux_selector_fd_raw(selector),
                native_window_host_loop_linux_timer_fd_raw(timer),
                native_window_host_loop_linux_host_event_fd_raw(host_event),
            ));
            Self::next_status(&mut self.timer_or_event_statuses)
        }

        fn selector_wait_for_event_raw(
            &mut self,
            selector: &NativeWindowHostLoopLinuxSelectorFd,
            host_event: &NativeWindowHostLoopLinuxHostEventFd,
        ) -> u32 {
            self.count_raw_method_call();
            self.event_wait_calls.push((
                native_window_host_loop_linux_selector_fd_raw(selector),
                native_window_host_loop_linux_host_event_fd_raw(host_event),
            ));
            Self::next_status(&mut self.event_statuses)
        }

        fn close_selector_raw(&mut self, selector: &NativeWindowHostLoopLinuxSelectorFd) -> bool {
            self.count_raw_method_call();
            self.close_selector_calls
                .push(native_window_host_loop_linux_selector_fd_raw(selector));
            true
        }

        fn close_timer_fd_raw(&mut self, timer: &NativeWindowHostLoopLinuxTimerFd) -> bool {
            self.count_raw_method_call();
            self.close_timer_calls
                .push(native_window_host_loop_linux_timer_fd_raw(timer));
            true
        }

        fn close_host_event_fd_raw(
            &mut self,
            host_event: &NativeWindowHostLoopLinuxHostEventFd,
        ) -> bool {
            self.count_raw_method_call();
            self.close_host_event_calls
                .push(native_window_host_loop_linux_host_event_fd_raw(host_event));
            true
        }

        fn last_error_code(&mut self) -> u32 {
            self.count_raw_method_call();
            self.last_error_calls += 1;
            self.last_error_code
        }
    }

    #[derive(Debug)]
    struct ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi {
        clone_raw_fd: i32,
        signal_result: bool,
        last_error_code: u32,
        clone_calls: Vec<i32>,
        signal_calls: Vec<i32>,
        close_calls: Vec<i32>,
        last_error_calls: usize,
    }

    impl ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi {
        fn new(clone_raw_fd: i32) -> Self {
            Self {
                clone_raw_fd,
                signal_result: true,
                last_error_code: 0,
                clone_calls: Vec::new(),
                signal_calls: Vec::new(),
                close_calls: Vec::new(),
                last_error_calls: 0,
            }
        }

        fn with_last_error_code(mut self, code: u32) -> Self {
            self.last_error_code = code;
            self
        }

        fn with_signal_result(mut self, result: bool) -> Self {
            self.signal_result = result;
            self
        }
    }

    impl NativeWindowHostLoopLinuxHostEventSignalRawApi
        for ScriptedNativeWindowHostLoopLinuxHostEventSignalRawApi
    {
        fn clone_host_event_signal_fd_raw(
            &mut self,
            host_event: &NativeWindowHostLoopLinuxHostEventFd,
        ) -> i32 {
            self.clone_calls
                .push(native_window_host_loop_linux_host_event_fd_raw(host_event));
            self.clone_raw_fd
        }

        fn signal_host_event_signal_fd_raw(
            &mut self,
            signal: &NativeWindowHostLoopLinuxHostEventSignalFd,
        ) -> bool {
            self.signal_calls
                .push(native_window_host_loop_linux_host_event_signal_fd_raw(
                    signal,
                ));
            self.signal_result
        }

        fn close_host_event_signal_fd_raw(
            &mut self,
            signal: &NativeWindowHostLoopLinuxHostEventSignalFd,
        ) -> bool {
            self.close_calls
                .push(native_window_host_loop_linux_host_event_signal_fd_raw(
                    signal,
                ));
            true
        }

        fn last_error_code(&mut self) -> u32 {
            self.last_error_calls += 1;
            self.last_error_code
        }
    }

    #[derive(Debug)]
    struct ScriptedNativeWindowHostLoopWindowsWaitRawApi {
        create_raw_handle: isize,
        set_result: bool,
        timer_or_message_statuses: Vec<u32>,
        message_statuses: Vec<u32>,
        last_error_code: u32,
        create_calls: usize,
        set_calls: Vec<(isize, i64)>,
        timer_wait_calls: Vec<isize>,
        message_wait_calls: usize,
        close_calls: Vec<isize>,
        last_error_calls: usize,
    }

    impl ScriptedNativeWindowHostLoopWindowsWaitRawApi {
        fn new(create_raw_handle: isize) -> Self {
            Self {
                create_raw_handle,
                set_result: true,
                timer_or_message_statuses: Vec::new(),
                message_statuses: Vec::new(),
                last_error_code: 0,
                create_calls: 0,
                set_calls: Vec::new(),
                timer_wait_calls: Vec::new(),
                message_wait_calls: 0,
                close_calls: Vec::new(),
                last_error_calls: 0,
            }
        }

        fn with_last_error_code(mut self, code: u32) -> Self {
            self.last_error_code = code;
            self
        }

        fn with_set_result(mut self, result: bool) -> Self {
            self.set_result = result;
            self
        }

        fn with_timer_or_message_statuses(mut self, statuses: Vec<u32>) -> Self {
            self.timer_or_message_statuses = statuses;
            self
        }

        fn with_message_statuses(mut self, statuses: Vec<u32>) -> Self {
            self.message_statuses = statuses;
            self
        }

        fn next_status(statuses: &mut Vec<u32>) -> u32 {
            if statuses.is_empty() {
                NATIVE_WINDOW_HOST_LOOP_WINDOWS_WAIT_STATUS_TIMEOUT
            } else {
                statuses.remove(0)
            }
        }
    }

    impl NativeWindowHostLoopWindowsWaitRawApi for ScriptedNativeWindowHostLoopWindowsWaitRawApi {
        fn create_waitable_timer_raw(&mut self) -> isize {
            self.create_calls += 1;
            self.create_raw_handle
        }

        fn set_waitable_timer_relative_100ns(
            &mut self,
            handle: &NativeWindowHostLoopWindowsWaitHandle,
            relative_due_time_100ns: i64,
        ) -> bool {
            self.set_calls.push((
                native_window_host_loop_windows_wait_handle_raw(handle),
                relative_due_time_100ns,
            ));
            self.set_result
        }

        fn msg_wait_for_timer_or_message_raw(
            &mut self,
            handle: &NativeWindowHostLoopWindowsWaitHandle,
        ) -> u32 {
            self.timer_wait_calls
                .push(native_window_host_loop_windows_wait_handle_raw(handle));
            Self::next_status(&mut self.timer_or_message_statuses)
        }

        fn msg_wait_for_message_raw(&mut self) -> u32 {
            self.message_wait_calls += 1;
            Self::next_status(&mut self.message_statuses)
        }

        fn close_handle_raw(&mut self, handle: &NativeWindowHostLoopWindowsWaitHandle) -> bool {
            self.close_calls
                .push(native_window_host_loop_windows_wait_handle_raw(handle));
            true
        }

        fn last_error_code(&mut self) -> u32 {
            self.last_error_calls += 1;
            self.last_error_code
        }
    }

    struct ScriptedNativeWindowHostLoopEventQueueWaiter {
        wait_calls: Vec<(NativeWindowSize, bool)>,
        error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopEventQueueWaiter {
        fn new() -> Self {
            Self {
                wait_calls: Vec::new(),
                error: None,
            }
        }

        fn with_error(mut self, error: &'static str) -> Self {
            self.error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopEventQueueWaiter for ScriptedNativeWindowHostLoopEventQueueWaiter {
        type Error = &'static str;

        fn wait_for_host_event(
            &mut self,
            window_size: NativeWindowSize,
            size_changed: bool,
        ) -> Result<(), Self::Error> {
            self.wait_calls.push((window_size, size_changed));
            if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(())
            }
        }
    }

    struct ScriptedNativeWindowHostLoopEventQueueStatusAdapter {
        raw_status: u32,
        status_calls: Vec<(NativeWindowSize, bool)>,
        error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopEventQueueStatusAdapter {
        fn new(raw_status: u32) -> Self {
            Self {
                raw_status,
                status_calls: Vec::new(),
                error: None,
            }
        }

        fn with_error(mut self, error: &'static str) -> Self {
            self.error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopEventQueueStatusAdapter
        for ScriptedNativeWindowHostLoopEventQueueStatusAdapter
    {
        type Error = &'static str;

        fn wait_for_host_event_raw_status(
            &mut self,
            window_size: NativeWindowSize,
            size_changed: bool,
        ) -> Result<u32, Self::Error> {
            self.status_calls.push((window_size, size_changed));
            if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(self.raw_status)
            }
        }
    }

    struct ScriptedNativeWindowHostLoopMessagePumpAdapter {
        pump_calls: Vec<(NativeWindowSize, bool)>,
        error: Option<&'static str>,
    }

    impl ScriptedNativeWindowHostLoopMessagePumpAdapter {
        fn new() -> Self {
            Self {
                pump_calls: Vec::new(),
                error: None,
            }
        }

        fn with_error(mut self, error: &'static str) -> Self {
            self.error = Some(error);
            self
        }
    }

    impl NativeWindowHostLoopMessagePumpAdapter for ScriptedNativeWindowHostLoopMessagePumpAdapter {
        type Error = &'static str;

        fn pump_host_messages(
            &mut self,
            window_size: NativeWindowSize,
            size_changed: bool,
        ) -> Result<(), Self::Error> {
            self.pump_calls.push((window_size, size_changed));
            if let Some(error) = self.error {
                Err(error)
            } else {
                Ok(())
            }
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
