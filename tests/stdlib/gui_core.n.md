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
