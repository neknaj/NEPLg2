# GUI std compositor tile RLE present host-command doctests

このファイルは、F5mu の std layer compositor tile RLE present host-command record boundary が F5mt step の public accessor だけから metadata 付き record を作り、lower F5cq / host import / packet record / platform / fallback へ進まないことを固定する。

source policy labels:

- std_compositor_tile_rle_present_host_command_facade_ok
- std_compositor_tile_rle_present_host_command_record_enum_ok
- std_compositor_tile_rle_present_host_command_step_mapping_ok
- std_compositor_tile_rle_present_host_command_uses_f5mt_accessor_ok
- std_compositor_tile_rle_present_host_command_no_lower_host_import_platform_fallback

## facade and record shape smoke

neplg2:test
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/compositor_tile_present_host_command" as *
#import "core/math" as *

// std_compositor_tile_rle_present_host_command_facade_ok
// std_compositor_tile_rle_present_host_command_record_enum_ok

fn step_result_code %fn GuiRgba8888CompositorTileRlePresentHostCommandStepResult i32 \result:
    match result:
        GuiRgba8888CompositorTileRlePresentHostCommandStepResult::Record record:
            match record:
                GuiRgba8888CompositorTileRlePresentHostCommandRecord::BeginFrame _descriptor:
                    1
                GuiRgba8888CompositorTileRlePresentHostCommandRecord::RunRecord _record:
                    2
                GuiRgba8888CompositorTileRlePresentHostCommandRecord::EndFrame _descriptor:
                    3
        GuiRgba8888CompositorTileRlePresentHostCommandStepResult::Completed:
            4

fn main %impure fn void i32 \void:
    let result %GuiRgba8888CompositorTileRlePresentHostCommandStepResult GuiRgba8888CompositorTileRlePresentHostCommandStepResult::Completed
    if eq step_result_code result 4:
        then 0
        else 1
```

## source-policy fixtures

neplg2:test[skip]
```neplg2
#indent 4

#import "std/gui/compositor_tile_present_host_command" as *

// std_compositor_tile_rle_present_host_command_step_mapping_ok
// std_compositor_tile_rle_present_host_command_uses_f5mt_accessor_ok
// std_compositor_tile_rle_present_host_command_no_lower_host_import_platform_fallback
```
