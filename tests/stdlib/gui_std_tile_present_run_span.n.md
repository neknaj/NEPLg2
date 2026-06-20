# GUI std row tile RLE present run span doctests

このファイルは、F5df の std layer RGBA8888 row tile RLE present run-span boundary が、F5cq run record の tile-local linear run を 1 行以内の span stream に分解する contract を固定する。

source policy labels:

- std_row_tile_rle_present_run_span_facade_ok
- std_row_tile_rle_present_run_span_row_span_shape_ok
- std_row_tile_rle_present_run_span_start_validates_descriptor_and_run_ok
- std_row_tile_rle_present_run_span_cross_row_split_ok
- std_row_tile_rle_present_run_span_explicit_completed_ok
- std_row_tile_rle_present_run_span_no_raw_no_platform_no_fallback

## row crossing run split

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_run_span\" count=5 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"first span code\" expected=\"222\" actual=\"222\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"second span code\" expected=\"33\" actual=\"33\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"completed code\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"row span height\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"run end error\" expected=\"77\" actual=\"77\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/row_tile_rle" as *
#import "alloc/gui/render2d/row_tile_rle_packet" as *
#import "core/cast" as *
#import "core/gui/color" as *
#import "core/math" as *
#import "core/result" as *
#import "std/gui/tile_present" as *
#import "std/gui/tile_present_host_command" as *
#import "std/gui/tile_present_run_span" as *
#import "std/gui/window" as *
#import "std/test" as test

// std_row_tile_rle_present_run_span_facade_ok
// std_row_tile_rle_present_run_span_row_span_shape_ok
// std_row_tile_rle_present_run_span_start_validates_descriptor_and_run_ok
// std_row_tile_rle_present_run_span_cross_row_split_ok
// std_row_tile_rle_present_run_span_explicit_completed_ok
// std_row_tile_rle_present_run_span_no_raw_no_platform_no_fallback

fn sample_descriptor %fn void GuiRgba8888RowTileRlePresentDescriptor \void:
    let surface %SurfaceId unwrap_ok surface_id_result 2
    let frame %FrameId unwrap_ok frame_id_result 4
    let packet %GuiRgba8888RowTileRlePacketDescriptor GuiRgba8888RowTileRlePacketDescriptor 4 0 0 0 2 2 2 4 8 16 2 1 8 2 24
    GuiRgba8888RowTileRlePresentDescriptor surface frame packet

fn sample_color %fn void Rgba8888 \void:
    let r %u8 cast 11
    let g %u8 cast 12
    let b %u8 cast 13
    let a %u8 cast 255
    rgba8888_new r g b a

fn sample_record %fn i32 fn i32 GuiRgba8888RowTileRlePresentHostCommandRunRecord \offset\count:
    let descriptor %GuiRgba8888RowTileRlePresentDescriptor sample_descriptor
    let color %Rgba8888 sample_color
    let run %GuiRgba8888RowTileRleRun GuiRgba8888RowTileRleRun offset count color
    gui_rgba8888_row_tile_rle_present_host_command_run_record descriptor run

fn span_code %fn &GuiRgba8888RowTileRlePresentRunRowSpan i32 \span:
    add:
        add:
            mul gui_rgba8888_row_tile_rle_present_run_row_span_x span 100
            mul gui_rgba8888_row_tile_rle_present_run_row_span_y span 10
        gui_rgba8888_row_tile_rle_present_run_row_span_width span

fn first_span_code %fn void i32 \void:
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord sample_record 2 5
    match gui_rgba8888_row_tile_rle_present_run_span_start record:
        Result::Err _error:
            -1
        Result::Ok cursor:
            match gui_rgba8888_row_tile_rle_present_run_span_step cursor:
                Result::Err _error2:
                    -2
                Result::Ok result:
                    match result:
                        GuiRgba8888RowTileRlePresentRunSpanStepResult::SpanReady ready:
                            let span %GuiRgba8888RowTileRlePresentRunRowSpan gui_rgba8888_row_tile_rle_present_run_span_ready_span &ready
                            span_code &span
                        GuiRgba8888RowTileRlePresentRunSpanStepResult::Completed:
                            -3

fn second_span_code %fn void i32 \void:
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord sample_record 2 5
    match gui_rgba8888_row_tile_rle_present_run_span_start record:
        Result::Err _error:
            -1
        Result::Ok cursor:
            match gui_rgba8888_row_tile_rle_present_run_span_step cursor:
                Result::Err _error2:
                    -2
                Result::Ok first_result:
                    match first_result:
                        GuiRgba8888RowTileRlePresentRunSpanStepResult::Completed:
                            -3
                        GuiRgba8888RowTileRlePresentRunSpanStepResult::SpanReady first_ready:
                            let second_cursor %GuiRgba8888RowTileRlePresentRunSpanCursor gui_rgba8888_row_tile_rle_present_run_span_ready_cursor &first_ready
                            match gui_rgba8888_row_tile_rle_present_run_span_step second_cursor:
                                Result::Err _error3:
                                    -4
                                Result::Ok second_result:
                                    match second_result:
                                        GuiRgba8888RowTileRlePresentRunSpanStepResult::Completed:
                                            -5
                                        GuiRgba8888RowTileRlePresentRunSpanStepResult::SpanReady second_ready:
                                            let span %GuiRgba8888RowTileRlePresentRunRowSpan gui_rgba8888_row_tile_rle_present_run_span_ready_span &second_ready
                                            span_code &span

fn completed_code %fn void i32 \void:
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord sample_record 2 5
    match gui_rgba8888_row_tile_rle_present_run_span_start record:
        Result::Err _error:
            -1
        Result::Ok cursor:
            match gui_rgba8888_row_tile_rle_present_run_span_step cursor:
                Result::Err _error2:
                    -2
                Result::Ok first_result:
                    match first_result:
                        GuiRgba8888RowTileRlePresentRunSpanStepResult::Completed:
                            -3
                        GuiRgba8888RowTileRlePresentRunSpanStepResult::SpanReady first_ready:
                            let second_cursor %GuiRgba8888RowTileRlePresentRunSpanCursor gui_rgba8888_row_tile_rle_present_run_span_ready_cursor &first_ready
                            match gui_rgba8888_row_tile_rle_present_run_span_step second_cursor:
                                Result::Err _error3:
                                    -4
                                Result::Ok second_result:
                                    match second_result:
                                        GuiRgba8888RowTileRlePresentRunSpanStepResult::Completed:
                                            -5
                                        GuiRgba8888RowTileRlePresentRunSpanStepResult::SpanReady second_ready:
                                            let third_cursor %GuiRgba8888RowTileRlePresentRunSpanCursor gui_rgba8888_row_tile_rle_present_run_span_ready_cursor &second_ready
                                            match gui_rgba8888_row_tile_rle_present_run_span_step third_cursor:
                                                Result::Err _error4:
                                                    -6
                                                Result::Ok third_result:
                                                    match third_result:
                                                        GuiRgba8888RowTileRlePresentRunSpanStepResult::Completed:
                                                            1
                                                        GuiRgba8888RowTileRlePresentRunSpanStepResult::SpanReady _ready:
                                                            -7

fn height_code %fn void i32 \void:
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord sample_record 2 5
    match gui_rgba8888_row_tile_rle_present_run_span_start record:
        Result::Err _error:
            -1
        Result::Ok cursor:
            match gui_rgba8888_row_tile_rle_present_run_span_step cursor:
                Result::Err _error2:
                    -2
                Result::Ok result:
                    match result:
                        GuiRgba8888RowTileRlePresentRunSpanStepResult::Completed:
                            -3
                        GuiRgba8888RowTileRlePresentRunSpanStepResult::SpanReady ready:
                            let span %GuiRgba8888RowTileRlePresentRunRowSpan gui_rgba8888_row_tile_rle_present_run_span_ready_span &ready
                            gui_rgba8888_row_tile_rle_present_run_row_span_height &span

fn run_end_error_code %fn void i32 \void:
    let record %GuiRgba8888RowTileRlePresentHostCommandRunRecord sample_record 7 2
    match gui_rgba8888_row_tile_rle_present_run_span_start record:
        Result::Ok _cursor:
            -1
        Result::Err error:
            let kind %GuiRgba8888RowTileRlePresentRunSpanStartErrorKind gui_rgba8888_row_tile_rle_present_run_span_start_error_kind &error
            match kind:
                GuiRgba8888RowTileRlePresentRunSpanStartErrorKind::RunEndOutOfBounds:
                    77
                _:
                    -2

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_run_span"
        |> test::test_report_push test::assert_eq_i32 "first span code" 222 first_span_code
        |> test::test_report_push test::assert_eq_i32 "second span code" 33 second_span_code
        |> test::test_report_push test::assert_eq_i32 "completed code" 1 completed_code
        |> test::test_report_push test::assert_eq_i32 "row span height" 1 height_code
        |> test::test_report_push test::assert_eq_i32 "run end error" 77 run_end_error_code
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
