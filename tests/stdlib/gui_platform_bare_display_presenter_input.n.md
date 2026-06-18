# GUI platform bare display presenter input doctests

このファイルは、F5fy の Bare display presenter input boundary が typed `ExecuteHostAction` と `GuiBareDisplayMemoryOwner` だけを authority とし、F5fx bridge の結果を scheduler executor input へ戻すことを確認する。

executable labels:

- platform_bare_display_presenter_input_facade_ok
- platform_bare_display_presenter_input_import_create_free_ok

source policy only labels:

- platform_bare_display_presenter_input_execute_only_ok
- platform_bare_display_presenter_input_owner_action_authority_ok
- platform_bare_display_presenter_input_borrowed_operation_before_action_consumption_ok
- platform_bare_display_presenter_input_calls_f5fx_once_ok
- platform_bare_display_presenter_input_reuses_scheduler_input_ok
- platform_bare_display_presenter_input_category_missing_preserves_action_ok
- platform_bare_display_presenter_input_no_raw_state_public_api_ok
- platform_bare_display_presenter_input_no_direct_host_import_ok
- platform_bare_display_presenter_input_no_loop_queue_fallback

## import and owner lifecycle

`display_presenter_input` は bare facade から import できる。positive execution path は actual bare host import を必要とするため source-policy で固定し、この実行テストでは facade import と owner lifecycle が壊れていないことを確認する。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_presenter_input_import\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "platforms/gui/bare" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_display_presenter_input_facade_ok
// platform_bare_display_presenter_input_import_create_free_ok
// platform_bare_display_presenter_input_execute_only_ok
// platform_bare_display_presenter_input_owner_action_authority_ok
// platform_bare_display_presenter_input_borrowed_operation_before_action_consumption_ok
// platform_bare_display_presenter_input_calls_f5fx_once_ok
// platform_bare_display_presenter_input_reuses_scheduler_input_ok
// platform_bare_display_presenter_input_category_missing_preserves_action_ok
// platform_bare_display_presenter_input_no_raw_state_public_api_ok
// platform_bare_display_presenter_input_no_direct_host_import_ok
// platform_bare_display_presenter_input_no_loop_queue_fallback

fn finish_case %impure fn GuiBareDisplayMemoryOwner impure fn i32 i32 \owner\code:
    match gui_bare_display_memory_owner_free owner:
        Result::Err _:
            90
        Result::Ok _:
            code

fn run_case %impure fn void i32 \void:
    match surface_id_result 98:
        Result::Err _:
            1
        Result::Ok surface:
            match gui_bare_framebuffer_config_checked surface 2 2:
                Result::Err _:
                    2
                Result::Ok config:
                    match gui_bare_display_memory_owner_create config:
                        Result::Err _:
                            3
                        Result::Ok owner:
                            finish_case owner 0

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_presenter_input_import"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
