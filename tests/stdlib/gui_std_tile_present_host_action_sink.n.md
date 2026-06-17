# GUI std row tile RLE present host action sink doctests

このファイルは、F5dc の std layer RGBA8888 row tile RLE present host action sink boundary が公開 facade から import できることと、source policy で固定する実装契約の coverage label を保持する。

behavior order は `nodesrc/test_web_gui_font_rendering_contract.js` が `stdlib/std/gui/tile_present_host_action_sink.nepl` の source policy として検査する。ここでは heavy platform executor scenario を再構築せず、doctest timeout を避けるため import smoke だけを実行する。

source policy labels:

- std_row_tile_rle_present_host_action_sink_facade_ok
- std_row_tile_rle_present_host_action_sink_executor_outcome_ok
- std_row_tile_rle_present_host_action_sink_support_preflight_ok
- std_row_tile_rle_present_host_action_sink_no_manufactured_success_ok
- std_row_tile_rle_present_host_action_sink_no_driver_no_platform_no_fallback

## host action sink import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_action_sink\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present host action sink import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_host_action_sink" as *
#import "std/test" as test

// std_row_tile_rle_present_host_action_sink_facade_ok
// std_row_tile_rle_present_host_action_sink_executor_outcome_ok
// std_row_tile_rle_present_host_action_sink_support_preflight_ok
// std_row_tile_rle_present_host_action_sink_no_manufactured_success_ok
// std_row_tile_rle_present_host_action_sink_no_driver_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_host_action_sink"
        |> test::test_report_push test::assert_eq_i32 "std tile present host action sink import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
