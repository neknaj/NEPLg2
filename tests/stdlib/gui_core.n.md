# core/gui no_alloc substrate

`core/gui` の Phase 1 型が std/alloc/platform なしで使えることを確認する。

## geometry arithmetic

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/gui" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let p %GuiPoint gui_point_new 2 3
    let delta %GuiPoint gui_point_new 5 -1
    let moved %GuiPoint gui_point_add p delta
    let back %GuiPoint gui_point_sub moved delta
    assert_eq_i32 7 gui_point_x &moved
    assert_eq_i32 2 gui_point_y &moved
    assert_eq_i32 2 gui_point_x &back
    assert_eq_i32 3 gui_point_y &back
    let rect %GuiRect gui_rect_new 10 20 30 40
    let shifted %GuiRect gui_rect_translate rect delta
    assert_eq_i32 15 gui_rect_x &shifted
    assert_eq_i32 19 gui_rect_y &shifted
    assert_eq_i32 45 gui_rect_right &shifted
    assert gui_rect_contains_point &shifted gui_point_new 20 20
    0
```

## color constructor

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/cast" as *
#import "core/gui" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let r %u8 cast 12
    let g %u8 cast 34
    let b %u8 cast 56
    let a %u8 cast 200
    let rgb %Rgb888 rgb888_new r g b
    let rgba %Rgba8888 rgba8888_from_rgb rgb a
    assert_eq_i32 12 cast rgba8888_r &rgba
    assert_eq_i32 34 cast rgba8888_g &rgba
    assert_eq_i32 56 cast rgba8888_b &rgba
    assert_eq_i32 200 cast rgba8888_a &rgba
    0
```

## text grid capability and lifecycle event smoke

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/gui" as *
#import "core/test" as *

fn lifecycle_code %fn GuiEvent i32 \event:
    match event:
        GuiEvent::Lifecycle lifecycle:
            match lifecycle:
                LifecycleEvent::Started:
                    0
                _:
                    1
        _:
            2

fn main %fn unit i32 \unit:
    let caps %GuiCapabilities gui_capabilities_text_grid
    assert surface_kind_is_text_grid gui_capabilities_surface_kind &caps
    assert gui_capabilities_requires_flush &caps
    let event %GuiEvent gui_event_lifecycle LifecycleEvent::Started
    assert_eq_i32 0 lifecycle_code event
    0
```

## event constructors keep platform data typed

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/gui" as *
#import "core/test" as *

fn pointer_code %fn GuiEvent i32 \event:
    match event:
        GuiEvent::Pointer pointer:
            match pointer_event_kind &pointer:
                PointerEventKind::Down:
                    pointer_event_pointer_id &pointer
                _:
                    1
        _:
            2

fn main %fn unit i32 \unit:
    let point %GuiPoint gui_point_new 1200 3400
    let pointer %PointerEvent pointer_event_new PointerEventKind::Down 7 point PointerButton::Primary
    let position %GuiPoint pointer_event_position &pointer
    assert_eq_i32 1200 gui_point_x &position
    assert_eq_i32 3400 gui_point_y &position
    match pointer_event_button &pointer:
        PointerButton::Primary:
            unit
        _:
            test_fail "unexpected pointer button"
    assert_eq_i32 7 pointer_code gui_event_pointer pointer
    0
```

## text measurement mock

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/gui" as *
#import "core/result" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let measurer %MockTextMeasurer mock_text_measurer_new 8 16 12
    let request %TextMeasureRequest text_measure_request_new text_run_id_new 4 font_id_new 1 200 5
    match measure_text &measurer request:
        Result::Ok metrics:
            assert_eq_i32 40 text_measure_result_width &metrics
            assert_eq_i32 16 text_measure_result_height &metrics
            assert_eq_i32 12 text_measure_result_baseline &metrics
            0
        Result::Err _e:
            1
```

## draw target mock and flush separation

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/cast" as *
#import "core/gui" as *
#import "core/result" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let r %u8 cast 1
    let g %u8 cast 2
    let b %u8 cast 3
    let a %u8 cast 255
    let color %Rgba8888 rgba8888_new r g b a
    let target0 %MockDrawTarget mock_draw_target_new 4 4
    let inside_point %GuiPoint gui_point_new 1 1
    let outside_point %GuiPoint gui_point_new 10 10
    let inside %Pixel Rgba8888 pixel_new inside_point color
    let outside %Pixel Rgba8888 pixel_new outside_point color
    match draw_pixel target0 inside:
        Result::Ok target1:
            match draw_pixel target1 outside:
                Result::Ok target2:
                    match fill_solid target2 gui_rect_new 0 0 2 2 color:
                        Result::Ok target3:
                            match clear target3 color:
                                Result::Ok target4:
                                    match flush target4:
                                        Result::Ok target5:
                                            assert_eq_i32 1 mock_draw_target_pixel_count &target5
                                            assert_eq_i32 1 mock_draw_target_fill_count &target5
                                            assert_eq_i32 1 mock_draw_target_clear_count &target5
                                            assert_eq_i32 1 mock_draw_target_flush_count &target5
                                            0
                                        Result::Err _e:
                                            1
                                Result::Err _e:
                                    1
                        Result::Err _e:
                            1
                Result::Err _e:
                    1
        Result::Err _e:
            1
```

## render target mock command stream

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/cast" as *
#import "core/gui" as *
#import "core/result" as *
#import "core/test" as *

fn main %fn unit i32 \unit:
    let zero %u8 cast 0
    let full %u8 cast 255
    let fg %Rgba8888 rgba8888_new full full full full
    let bg %Rgba8888 rgba8888_new zero zero zero full
    let style %TextCellStyle text_cell_style_new fg bg
    let point %TextGridPoint text_grid_point_new 2 3
    let run_id %TextRunId text_run_id_new 7
    let run %TextCellRun text_cell_run_new point run_id 5 style
    let command %RenderCommand render_command_text_cell_run run
    match render_one mock_render_target_new command:
        Result::Ok target:
            assert_eq_i32 1 mock_render_target_command_count &target
            assert_eq_i32 1 mock_render_target_text_cell_count &target
            0
        Result::Err _e:
            1
```
