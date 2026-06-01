# GUI terminal TextGrid bridge

このファイルは `features/gui` から terminal backend の TextGrid surface 型を使えることを確認します。
DOM、OS、ANSI escape、TTY raw mode には触れず、GUI substrate の backend 境界としての型と helper だけを固定します。

## gui_terminal_text_grid_capability_surface

[目的/もくてき]:
- terminal backend が `TextGrid` surface capability を返すことを[確/たし]かめます。
- サイズ、入力 capability、flush 要否が raw terminal state ではなく struct field と helper で扱えることを固定します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "Checked [ok,ok,ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n[4] ok\n[5] ok\n"
```neplg2
#entry main
#indent 4
#target std

#import "features/gui" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let cap %TextGridCapability text_grid_capability 80 24
    let kind %TerminalSurfaceKind text_grid_capability_kind &cap
    let cols %i32 text_grid_capability_cols &cap
    let rows %i32 text_grid_capability_rows &cap
    let surface_ok %bool terminal_surface_is_text_grid kind
    let keyboard_ok %bool text_grid_capability_has_keyboard &cap
    let text_input_ok %bool text_grid_capability_has_text_input &cap
    let size_ok %bool text_grid_capability_has_non_negative_size &cap
    let checks:
        checks_new
        |> checks_push assert "surface is TextGrid" surface_ok
        |> checks_push assert_eq_i32 "cols" 80 cols
        |> checks_push assert_eq_i32 "rows" 24 rows
        |> checks_push assert "keyboard capability" keyboard_ok
        |> checks_push assert "text input capability" text_input_ok
        |> checks_push assert "non-negative size" size_ok
    let shown checks_print_report checks
    checks_exit_code shown
```

## gui_terminal_text_cell_run_and_frame_helpers

[目的/もくてき]:
- `TextGridPoint`、`TextCellRun`、`TerminalFrame` が facade 経由で使えることを[確/たし]かめます。
- terminal frame が ANSI 文字列ではなく、cell 座標、plain text、style、capability を束ねる値として扱えることを固定します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "Checked [ok,ok,ok,ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n[4] ok\n[5] ok\n[6] ok\n"
```neplg2
#entry main
#indent 4
#target std

#import "features/gui" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let cap %TextGridCapability text_grid_capability 40 10
    let point %TextGridPoint text_grid_point_translate text_grid_point 2 3 5 7
    let run %TextCellRun text_cell_run point "hello" text_cell_bold_style
    let frame %TerminalFrame terminal_frame cap run
    let frame_run %TextCellRun terminal_frame_run &frame
    let style %TextCellStyle text_cell_run_style &frame_run
    let surface_ok %bool terminal_frame_surface_is_text_grid &frame
    let frame_cols %i32 terminal_frame_cols &frame
    let frame_rows %i32 terminal_frame_rows &frame
    let run_col %i32 text_cell_run_col &frame_run
    let run_row %i32 text_cell_run_row &frame_run
    let run_text %str text_cell_run_text &frame_run
    let weight %TextCellWeight text_cell_style_weight &style
    let bold_ok %bool text_cell_weight_is_bold weight
    let checks:
        checks_new
        |> checks_push assert "frame surface" surface_ok
        |> checks_push assert_eq_i32 "frame cols" 40 frame_cols
        |> checks_push assert_eq_i32 "frame rows" 10 frame_rows
        |> checks_push assert_eq_i32 "run col" 7 run_col
        |> checks_push assert_eq_i32 "run row" 10 run_row
        |> checks_push assert_str_eq "run text" "hello" run_text
        |> checks_push assert "bold style" bold_ok
    let shown checks_print_report checks
    checks_exit_code shown
```
