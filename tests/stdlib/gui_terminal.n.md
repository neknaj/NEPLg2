# GUI terminal TextGrid bridge

このファイルは `features/gui` から terminal backend の TextGrid surface 型を使えることを確認します。
DOM、OS、ANSI escape、TTY raw mode には触れず、GUI substrate の backend 境界としての型と helper だけを固定します。

## gui_terminal_text_grid_capability_surface

[目的/もくてき]:
- terminal backend が `TextGrid` surface capability を返すことを[確/たし]かめます。
- サイズ、入力 capability、flush 要否が raw terminal state ではなく struct field と helper で扱えることを固定します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"gui_terminal_text_grid_capability_surface\" count=8 failed=0\nassertion index=0 status=ok kind=bool label=\"surface is TextGrid\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"cols\" expected=\"80\" actual=\"80\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"rows\" expected=\"24\" actual=\"24\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"keyboard capability\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"text input capability\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=bool label=\"non-negative size\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"rejects non-TextGrid profile\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=bool label=\"rejects negative size\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "features/gui" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let cap %TextGridCapability unwrap_ok text_grid_capability 80 24
    let kind %SurfaceKind text_grid_capability_kind &cap
    let cols %i32 text_grid_capability_cols &cap
    let rows %i32 text_grid_capability_rows &cap
    let surface_ok %bool terminal_surface_is_text_grid kind
    let keyboard_ok %bool text_grid_capability_has_keyboard &cap
    let text_input_ok %bool text_grid_capability_has_text_input &cap
    let size_ok %bool text_grid_capability_has_non_negative_size &cap
    let invalid_caps %GuiCapabilities gui_capabilities_headless
    let rejected %bool:
        match terminal_profile_full 10 5 invalid_caps:
            Result::Err _err:
                true
            _:
                false
    let invalid_cols %i32 sub 0 1
    let invalid_size_rejected %bool:
        match text_grid_capability invalid_cols 24:
            Result::Err error:
                match error:
                    GuiError::InvalidGeometry:
                        true
                    _:
                        false
            Result::Ok _cap:
                false
    let checks:
        test_report_new "gui_terminal_text_grid_capability_surface"
        |> test_report_push assert "surface is TextGrid" surface_ok
        |> test_report_push assert_eq_i32 "cols" 80 cols
        |> test_report_push assert_eq_i32 "rows" 24 rows
        |> test_report_push assert "keyboard capability" keyboard_ok
        |> test_report_push assert "text input capability" text_input_ok
        |> test_report_push assert "non-negative size" size_ok
        |> test_report_push assert "rejects non-TextGrid profile" rejected
        |> test_report_push assert "rejects negative size" invalid_size_rejected
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```

## gui_terminal_text_cell_run_and_frame_helpers

[目的/もくてき]:
- `TextGridPoint`、`TextCellRun`、`TerminalFrame` が facade 経由で使えることを[確/たし]かめます。
- terminal frame が ANSI 文字列ではなく、core の `TextRunId`、cell 座標、style、capability を束ねる値として扱えることを固定します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"gui_terminal_text_cell_run_and_frame_helpers\" count=7 failed=0\nassertion index=0 status=ok kind=bool label=\"frame surface\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"frame cols\" expected=\"40\" actual=\"40\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"frame rows\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"run col\" expected=\"7\" actual=\"7\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"run row\" expected=\"10\" actual=\"10\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"cell count\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"run id\" expected=\"9\" actual=\"9\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std

#import "features/gui" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let profile %TerminalProfile unwrap_ok terminal_profile 40 10
    let base %TextGridPoint terminal_text_grid_point 2 3
    let point %TextGridPoint terminal_text_grid_point_translate base 5 7
    let run_id %TextRunId text_run_id_new 9
    let run %TextCellRun terminal_text_cell_default_run point run_id 5
    let frame %TerminalFrame terminal_frame profile run
    let frame_run %TextCellRun terminal_frame_run &frame
    let surface_ok %bool terminal_frame_surface_is_text_grid &frame
    let frame_cols %i32 terminal_frame_cols &frame
    let frame_rows %i32 terminal_frame_rows &frame
    let frame_point %TextGridPoint text_cell_run_point &frame_run
    let frame_run_id %TextRunId text_cell_run_id &frame_run
    let run_col %i32 text_grid_point_column &frame_point
    let run_row %i32 text_grid_point_row &frame_point
    let cell_count %i32 text_cell_run_cell_count &frame_run
    let run_raw %i32 text_run_id_raw &frame_run_id
    let checks:
        test_report_new "gui_terminal_text_cell_run_and_frame_helpers"
        |> test_report_push assert "frame surface" surface_ok
        |> test_report_push assert_eq_i32 "frame cols" 40 frame_cols
        |> test_report_push assert_eq_i32 "frame rows" 10 frame_rows
        |> test_report_push assert_eq_i32 "run col" 7 run_col
        |> test_report_push assert_eq_i32 "run row" 10 run_row
        |> test_report_push assert_eq_i32 "cell count" 5 cell_count
        |> test_report_push assert_eq_i32 "run id" 9 run_raw
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```
