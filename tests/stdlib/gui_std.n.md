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
#import "core/result" as *
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
#import "core/result" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let win %WindowId window_id 4
    let timer %TimerRequest timer_request win timer_id 2 1000 true
    let measurer %HostTextMeasurer host_text_measurer_fixed gui_capabilities_text_grid 8 16 12
    let metrics %TextMeasureResult unwrap_ok measure_text &measurer (text_measure_request_new (text_run_id_new 1) (font_id_new 1) 80 10)
    let ime %ImeStateRequest ime_state_request win ImeState::Enabled
    let root %AccessibilityNodeSnapshot accessibility_node_snapshot accessibility_node_id 1 AccessibilityRole::Button "Run" true
    let check0 assert_eq_i32 1000 timer_request_interval_ms &timer
    let check1 assert_eq_i32 80 text_measure_result_width &metrics
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

## gui_runtime_interprets_update_effect_batch

[目的/もくてき]:
- `Update.effects` の 2 個目以降を runtime が落とさず、command batch として保持することを確認します。

neplg2:test[stdio, normalize_newlines]
stdout: "Checked [ok,ok]\n[0] ok\n[1] ok\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "alloc/gui" as *
#import "core/option" as *
#import "core/result" as *
#import "std/gui" as *
#import "std/test" as *

fn main %impure fn unit i32 \unit:
    let host %GuiHost gui_host_new gui_capabilities_text_grid window_id 9
    let batch0 %GuiEffectBatch gui_effect_batch_empty
    let batch1 %GuiEffectBatch unwrap_ok gui_effect_batch_push batch0 request_redraw 1
    let batch2 %GuiEffectBatch unwrap_ok gui_effect_batch_push batch1 request_redraw 2
    let update %Update i32 update_result_batch 0 batch2
    let commands %GuiRuntimeCommandBatch unwrap_ok gui_runtime_interpret_update &host update
    let count_check assert_eq_i32 2 gui_runtime_command_batch_count &commands
    let second_check match gui_runtime_command_batch_second &commands:
        Option::Some command:
            match command:
                GuiRuntimeCommand::RequestRedraw window:
                    assert_eq_i32 2 window_id_raw &window
                _:
                    assert false
        Option::None:
            assert false
    let checks checks_push (checks_push checks_new count_check) second_check
    let shown checks_print_report checks
    checks_exit_code shown
```

## gui_runtime_rejects_unsupported_capability

[目的/もくてき]:
- surface を持たない headless host に対する redraw request が silent no-op ではなく `GuiError::Unsupported` になることを確認します。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/gui" as *

fn main %fn unit i32 \unit:
    let host %GuiHost gui_host_headless
    match gui_runtime_interpret_effect &host request_redraw 1:
        Result::Ok _command:
            1
        Result::Err error:
            match error:
                GuiError::Unsupported:
                    0
                _:
                    2
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
