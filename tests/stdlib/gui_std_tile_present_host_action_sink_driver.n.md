# GUI std row tile RLE present host action sink driver doctests

このファイルは、F5dd の std layer RGBA8888 row tile RLE present host action sink driver boundary が公開 facade から import できることと、source policy で固定する実装契約の coverage label を保持する。

behavior order は `nodesrc/test_web_gui_font_rendering_contract.js` が `stdlib/std/gui/tile_present_host_action_sink_driver.nepl` の source policy として検査する。ここでは heavy dispatch-loop scenario を再構築せず、doctest timeout を避けるため import smoke だけを実行する。

source policy labels:

- std_row_tile_rle_present_host_action_sink_driver_facade_ok
- std_row_tile_rle_present_host_action_sink_driver_owner_recovery_ok
- std_row_tile_rle_present_host_action_sink_driver_sink_before_completion_ok
- std_row_tile_rle_present_host_action_sink_driver_no_manufactured_outcome_ok
- std_row_tile_rle_present_host_action_sink_driver_no_direct_bridge_no_platform_no_fallback

## host action sink driver import smoke

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_tile_present_host_action_sink_driver\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"std tile present host action sink driver import\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "std/gui/tile_present_host_action_sink_driver" as *
#import "std/test" as test

// std_row_tile_rle_present_host_action_sink_driver_facade_ok
// std_row_tile_rle_present_host_action_sink_driver_owner_recovery_ok
// std_row_tile_rle_present_host_action_sink_driver_sink_before_completion_ok
// std_row_tile_rle_present_host_action_sink_driver_no_manufactured_outcome_ok
// std_row_tile_rle_present_host_action_sink_driver_no_direct_bridge_no_platform_no_fallback

fn main %impure fn void i32 \void:
    let report:
        test::test_report_new "gui_std_tile_present_host_action_sink_driver"
        |> test::test_report_push test::assert_eq_i32 "std tile present host action sink driver import" 0 0
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
