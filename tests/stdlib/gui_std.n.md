# std/gui host contract

このファイルは `std/gui` が platform 実装ではなく、`alloc/gui` の `GuiEffect` を runtime が解釈する host/effect contract であることを確認します。

## gui_runtime_interprets_effect_as_command

[目的/もくてき]:
- application は host を直接呼ばず、`GuiEffect` を返します。
- runtime helper は redraw request を `GuiRuntimeCommand` data に変換します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok]\n[0] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let host %GuiHost gui_host_new gui_capabilities_text_grid window_id 9
    let command_result %Result GuiRuntimeCommand GuiError gui_runtime_interpret_effect &host request_redraw 12
    let checks match command_result:
        Result::Ok command:
            match command:
                GuiRuntimeCommand::RequestRedraw window:
                    checks_push checks_new assert_eq_i32 12 window_id_raw &window
                GuiRuntimeCommand::Noop:
                    checks_push checks_new assert false
                GuiRuntimeCommand::SetTitle _title:
                    checks_push checks_new assert false
        Result::Err _error:
            checks_push checks_new assert false
    let shown checks_print_report checks
    checks_exit_code shown
```

## gui_std_contract_values_have_no_platform_handle

[目的/もくてき]:
- window、timer、IME、text measure、accessibility が raw platform handle ではなく typed value で表されることを固定します。
- text metrics と accessibility snapshot は host update の data contract として扱います。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok,ok,ok]\n[0] ok\n[1] ok\n[2] ok\n[3] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/gui" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let win %WindowId window_id 4
    let timer %TimerRequest timer_request win timer_id 2 1000 true
    let metrics %TextMetrics text_metrics 80 16 12
    let ime %ImeStateRequest ime_state_request win ImeState::Enabled
    let root %AccessibilityNodeSnapshot accessibility_node_snapshot accessibility_node_id 1 AccessibilityRole::Button "Run" true
    let check0 assert_eq_i32 1000 timer_request_interval_ms &timer
    let check1 assert_eq_i32 80 text_metrics_width &metrics
    let check2 assert accessibility_node_is_focused &root
    let check3 match ime_state_request_state &ime:
        ImeState::Enabled:
            assert true
        ImeState::Disabled:
            assert false
    let checks0 checks_new
    let checks1 checks_push checks0 check0
    let checks2 checks_push checks1 check1
    let checks3 checks_push checks2 check2
    let checks checks_push checks3 check3
    let shown checks_print_report checks
    checks_exit_code shown
```

## gui_error_display_keeps_typed_error

[目的/もくてき]:
- unsupported を silent no-op にせず `GuiError` value として扱い、display helper は表示 label だけを担当します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "std/gui" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let check0 assert gui_error_is_unsupported GuiError::Unsupported
    let check1 assert_str_eq "unsupported" gui_error_label GuiError::Unsupported
    let checks0 checks_new
    let checks1 checks_push checks0 check0
    let checks checks_push checks1 check1
    let shown checks_print_report checks
    checks_exit_code shown
```
