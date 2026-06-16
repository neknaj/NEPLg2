# GUI render2d row batch drain

このファイルは、F5bx の RGBA8888 row batch drain が row batch cursor を scheduler budget 内で進め、byte payload / host present / fallback に進まないことを固定する。

source policy coverage labels:

- render2d_row_batch_drain_facade_ok
- render2d_row_batch_drain_complete_before_budget_ok
- render2d_row_batch_drain_negative_budget_error_ok
- render2d_row_batch_drain_zero_budget_exhausted_ok
- render2d_row_batch_drain_partial_budget_progress_ok
- render2d_row_batch_drain_completion_count_ok
- render2d_row_batch_drain_progress_invariant_checked_ok
- render2d_row_batch_drain_no_platform_no_fallback

## complete before budget

complete cursor は budget より先に判定される。負 budget でも `Completed` になり、`StepBudgetExhausted` に隠れない。

neplg2:test[stdio, normalize_newlines]
stdout: ""
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_drain" as *
#import "core/math" as *
#import "core/result" as *

// render2d_row_batch_drain_complete_before_budget_ok
// render2d_row_batch_drain_no_platform_no_fallback

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            1
        Result::Ok surface:
            let dirty_owner %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 301
            match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                Result::Err _:
                    2
                Result::Ok frame:
                    let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 2
                    match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                        Result::Err _:
                            3
                        Result::Ok plan:
                            match gui_rgba8888_row_batch_cursor_start plan:
                                Result::Err _:
                                    4
                                Result::Ok cursor:
                                    match gui_rgba8888_row_batch_drain_budget cursor sub 0 1:
                                        Result::Err error:
                                            match gui_rgba8888_row_batch_drain_error_free error:
                                                Result::Ok _:
                                                    5
                                                Result::Err _:
                                                    5
                                        Result::Ok terminal:
                                            let status %GuiRgba8888RowBatchDrainStatus gui_rgba8888_row_batch_drain_terminal_status &terminal
                                            let count_ok %bool eq gui_rgba8888_row_batch_drain_terminal_emitted_count &terminal 0
                                            let next_cursor %GuiRgba8888RowBatchCursorOwner gui_rgba8888_row_batch_drain_terminal_finish_cursor terminal
                                            let index_ok %bool eq gui_rgba8888_row_batch_cursor_batch_index &next_cursor 0
                                            let status_ok %bool match status:
                                                GuiRgba8888RowBatchDrainStatus::Completed:
                                                    true
                                                _:
                                                    false
                                            match gui_rgba8888_row_batch_cursor_free next_cursor:
                                                Result::Ok _:
                                                    if and status_ok and count_ok index_ok 0 6
                                                Result::Err _:
                                                    6
```

## negative budget error

ready cursor に負 budget を渡すと `InvalidBudget` の owner-bearing error になる。

neplg2:test[stdio, normalize_newlines]
stdout: ""
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_drain" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *

// render2d_row_batch_drain_negative_budget_error_ok

fn is_invalid_budget %fn GuiRgba8888RowBatchDrainErrorKind bool \kind:
    match kind:
        GuiRgba8888RowBatchDrainErrorKind::InvalidBudget:
            true
        _:
            false

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            1
        Result::Ok surface:
            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                Result::Err _:
                    2
                Result::Ok dirty_owner:
                    let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 302
                    match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                        Result::Err _:
                            3
                        Result::Ok frame:
                            let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
                            match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                                Result::Err _:
                                    4
                                Result::Ok plan:
                                    match gui_rgba8888_row_batch_cursor_start plan:
                                        Result::Err _:
                                            5
                                        Result::Ok cursor:
                                            match gui_rgba8888_row_batch_drain_budget cursor sub 0 1:
                                                Result::Ok terminal:
                                                    match gui_rgba8888_row_batch_drain_terminal_free terminal:
                                                        Result::Ok _:
                                                            6
                                                        Result::Err _:
                                                            6
                                                Result::Err error:
                                                    let kind_ok %bool is_invalid_budget gui_rgba8888_row_batch_drain_error_kind &error
                                                    let recovered %GuiRgba8888RowBatchCursorOwner gui_rgba8888_row_batch_drain_error_cursor error
                                                    let index_ok %bool eq gui_rgba8888_row_batch_cursor_batch_index &recovered 0
                                                    match gui_rgba8888_row_batch_cursor_free recovered:
                                                        Result::Ok _:
                                                            if and kind_ok index_ok 0 7
                                                        Result::Err _:
                                                            7
```

## zero budget exhausted

ready cursor に zero budget を渡すと step を実行せず、cursor index 0 のまま `StepBudgetExhausted` になる。

neplg2:test[stdio, normalize_newlines]
stdout: ""
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_drain" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *

// render2d_row_batch_drain_zero_budget_exhausted_ok

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 2 2:
        Result::Err _:
            1
        Result::Ok surface:
            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                Result::Err _:
                    2
                Result::Ok dirty_owner:
                    let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 303
                    match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                        Result::Err _:
                            3
                        Result::Ok frame:
                            let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 1
                            match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                                Result::Err _:
                                    4
                                Result::Ok plan:
                                    match gui_rgba8888_row_batch_cursor_start plan:
                                        Result::Err _:
                                            5
                                        Result::Ok cursor:
                                            match gui_rgba8888_row_batch_drain_budget cursor 0:
                                                Result::Err error:
                                                    match gui_rgba8888_row_batch_drain_error_free error:
                                                        Result::Ok _:
                                                            6
                                                        Result::Err _:
                                                            6
                                                Result::Ok terminal:
                                                    let status %GuiRgba8888RowBatchDrainStatus gui_rgba8888_row_batch_drain_terminal_status &terminal
                                                    let count_ok %bool eq gui_rgba8888_row_batch_drain_terminal_emitted_count &terminal 0
                                                    let recovered %GuiRgba8888RowBatchCursorOwner gui_rgba8888_row_batch_drain_terminal_finish_cursor terminal
                                                    let index_ok %bool eq gui_rgba8888_row_batch_cursor_batch_index &recovered 0
                                                    let status_ok %bool match status:
                                                        GuiRgba8888RowBatchDrainStatus::StepBudgetExhausted:
                                                            true
                                                        _:
                                                            false
                                                    match gui_rgba8888_row_batch_cursor_free recovered:
                                                        Result::Ok _:
                                                            if and status_ok and count_ok index_ok 0 8
                                                        Result::Err _:
                                                            8
```

## partial budget progress

budget 2 で 3 batch の stream を進めると、2 batch だけ進み、cursor index 2 の `StepBudgetExhausted` になる。

neplg2:test[stdio, normalize_newlines]
stdout: ""
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_drain" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *

// render2d_row_batch_drain_partial_budget_progress_ok
// render2d_row_batch_drain_progress_invariant_checked_ok

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 4 5:
        Result::Err _:
            1
        Result::Ok surface:
            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                Result::Err _:
                    2
                Result::Ok dirty_owner:
                    let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 304
                    match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                        Result::Err _:
                            3
                        Result::Ok frame:
                            let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 2
                            match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                                Result::Err _:
                                    4
                                Result::Ok plan:
                                    match gui_rgba8888_row_batch_cursor_start plan:
                                        Result::Err _:
                                            5
                                        Result::Ok cursor:
                                            match gui_rgba8888_row_batch_drain_budget cursor 2:
                                                Result::Err error:
                                                    match gui_rgba8888_row_batch_drain_error_free error:
                                                        Result::Ok _:
                                                            6
                                                        Result::Err _:
                                                            6
                                                Result::Ok terminal:
                                                    let status %GuiRgba8888RowBatchDrainStatus gui_rgba8888_row_batch_drain_terminal_status &terminal
                                                    let count_ok %bool eq gui_rgba8888_row_batch_drain_terminal_emitted_count &terminal 2
                                                    let recovered %GuiRgba8888RowBatchCursorOwner gui_rgba8888_row_batch_drain_terminal_finish_cursor terminal
                                                    let index_ok %bool eq gui_rgba8888_row_batch_cursor_batch_index &recovered 2
                                                    let status_ok %bool match status:
                                                        GuiRgba8888RowBatchDrainStatus::StepBudgetExhausted:
                                                            true
                                                        _:
                                                            false
                                                    match gui_rgba8888_row_batch_cursor_free recovered:
                                                        Result::Ok _:
                                                            if and status_ok and count_ok index_ok 0 8
                                                        Result::Err _:
                                                            8
```

## completion count

十分な budget では全 batch を進め、emitted count 3 の `Completed` になる。

neplg2:test[stdio, normalize_newlines]
stdout: ""
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "alloc/gui/render2d/software_surface" as *
#import "alloc/gui/render2d/dirty_surface" as *
#import "alloc/gui/render2d/bitmap_frame" as *
#import "alloc/gui/render2d/row_batch_plan" as *
#import "alloc/gui/render2d/row_batch_cursor" as *
#import "alloc/gui/render2d/row_batch_drain" as *
#import "core/gui/dirty_region" as *
#import "core/math" as *
#import "core/result" as *

// render2d_row_batch_drain_completion_count_ok

fn main %impure fn void i32 \void:
    match gui_rgba8888_software_surface_create 4 5:
        Result::Err _:
            1
        Result::Ok surface:
            let dirty_owner0 %GuiRgba8888SoftwareSurfaceDirtyOwner gui_rgba8888_software_surface_dirty_owner_from_surface surface
            match gui_rgba8888_software_surface_dirty_owner_push_region_checked dirty_owner0 dirty_region_full:
                Result::Err _:
                    2
                Result::Ok dirty_owner:
                    let frame_config %GuiRgba8888BitmapFrameConfig GuiRgba8888BitmapFrameConfig 305
                    match gui_rgba8888_bitmap_frame_prepare dirty_owner frame_config:
                        Result::Err _:
                            3
                        Result::Ok frame:
                            let plan_config %GuiRgba8888RowBatchPlanConfig GuiRgba8888RowBatchPlanConfig 2
                            match gui_rgba8888_row_batch_plan_prepare frame plan_config:
                                Result::Err _:
                                    4
                                Result::Ok plan:
                                    match gui_rgba8888_row_batch_cursor_start plan:
                                        Result::Err _:
                                            5
                                        Result::Ok cursor:
                                            match gui_rgba8888_row_batch_drain_budget cursor 8:
                                                Result::Err error:
                                                    match gui_rgba8888_row_batch_drain_error_free error:
                                                        Result::Ok _:
                                                            6
                                                        Result::Err _:
                                                            6
                                                Result::Ok terminal:
                                                    let status %GuiRgba8888RowBatchDrainStatus gui_rgba8888_row_batch_drain_terminal_status &terminal
                                                    let count_ok %bool eq gui_rgba8888_row_batch_drain_terminal_emitted_count &terminal 3
                                                    let recovered %GuiRgba8888RowBatchCursorOwner gui_rgba8888_row_batch_drain_terminal_finish_cursor terminal
                                                    let index_ok %bool eq gui_rgba8888_row_batch_cursor_batch_index &recovered 3
                                                    let status_ok %bool match status:
                                                        GuiRgba8888RowBatchDrainStatus::Completed:
                                                            true
                                                        _:
                                                            false
                                                    match gui_rgba8888_row_batch_cursor_free recovered:
                                                        Result::Ok _:
                                                            if and status_ok and count_ok index_ok 0 7
                                                        Result::Err _:
                                                            7
```
