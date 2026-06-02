# GUI terminal TextGrid bridge

このファイルは `features/gui` から terminal backend の TextGrid surface 型を使えることを確認します。
DOM、OS、ANSI escape、TTY raw mode には触れず、GUI substrate の backend 境界としての型と helper だけを固定します。

## gui_terminal_text_grid_capability_surface

[目的/もくてき]:
- terminal backend が `TextGrid` surface capability を返すことを[確/たし]かめます。
- サイズ、入力 capability、flush 要否が raw terminal state ではなく struct field と helper で扱えることを固定します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "Checked [ok,ok,ok,ok,ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n[4] ok\n[5] ok\n[6] ok\n[7] ok\n"
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
        checks_new
        |> checks_push assert "surface is TextGrid" surface_ok
        |> checks_push assert_eq_i32 "cols" 80 cols
        |> checks_push assert_eq_i32 "rows" 24 rows
        |> checks_push assert "keyboard capability" keyboard_ok
        |> checks_push assert "text input capability" text_input_ok
        |> checks_push assert "non-negative size" size_ok
        |> checks_push assert "rejects non-TextGrid profile" rejected
        |> checks_push assert "rejects negative size" invalid_size_rejected
    let shown checks_print_report checks
    checks_exit_code shown
```

## gui_terminal_text_cell_run_and_frame_helpers

[目的/もくてき]:
- `TextGridPoint`、`TextCellRun`、`TerminalFrame` が facade 経由で使えることを[確/たし]かめます。
- terminal frame が ANSI 文字列ではなく、core の `TextRunId`、cell 座標、style、capability を束ねる値として扱えることを固定します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "Checked [ok,ok,ok,ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n[4] ok\n[5] ok\n[6] ok\n"
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
        checks_new
        |> checks_push assert "frame surface" surface_ok
        |> checks_push assert_eq_i32 "frame cols" 40 frame_cols
        |> checks_push assert_eq_i32 "frame rows" 10 frame_rows
        |> checks_push assert_eq_i32 "run col" 7 run_col
        |> checks_push assert_eq_i32 "run row" 10 run_row
        |> checks_push assert_eq_i32 "cell count" 5 cell_count
        |> checks_push assert_eq_i32 "run id" 9 run_raw
    let shown checks_print_report checks
    checks_exit_code shown
```
