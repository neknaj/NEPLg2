# GUI platform bare display operation driver bridge doctests

このファイルは、F5fx の Bare display operation-to-driver-adapter bridge が owner 内 canonical state から framebuffer / storage / memory / driver adapter の順に進み、public step value や raw storage を authority として受け取らないことを確認する。

executable labels:

- platform_bare_display_operation_driver_bridge_facade_ok
- platform_bare_display_operation_driver_bridge_import_create_free_ok

source policy only labels:

- platform_bare_display_operation_driver_bridge_source_policy_owner_operation_authority_ok
- platform_bare_display_operation_driver_bridge_source_policy_validation_before_host_ok
- platform_bare_display_operation_driver_bridge_source_policy_owner_error_recovery_ok
- platform_bare_display_operation_driver_bridge_source_policy_adapter_error_recovery_ok
- platform_bare_display_operation_driver_bridge_source_policy_completed_seal_ok
- platform_bare_display_operation_driver_bridge_source_policy_no_copy_owner_payload_ok
- platform_bare_display_operation_driver_bridge_no_loop_queue_fallback

## import and owner lifecycle

`display_operation_driver_bridge` は bare facade から import できる。positive bridge path は actual bare host import を必要とするため source-policy で構造を固定し、この実行テストでは facade import と owner lifecycle が壊れていないことを確認する。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_operation_driver_bridge_import\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "platforms/gui/bare" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_display_operation_driver_bridge_facade_ok
// platform_bare_display_operation_driver_bridge_import_create_free_ok
// platform_bare_display_operation_driver_bridge_source_policy_owner_operation_authority_ok
// platform_bare_display_operation_driver_bridge_source_policy_validation_before_host_ok
// platform_bare_display_operation_driver_bridge_source_policy_owner_error_recovery_ok
// platform_bare_display_operation_driver_bridge_source_policy_adapter_error_recovery_ok
// platform_bare_display_operation_driver_bridge_source_policy_completed_seal_ok
// platform_bare_display_operation_driver_bridge_source_policy_no_copy_owner_payload_ok
// platform_bare_display_operation_driver_bridge_no_loop_queue_fallback

fn finish_case %impure fn GuiBareDisplayMemoryOwner impure fn i32 i32 \owner\code:
    match gui_bare_display_memory_owner_free owner:
        Result::Err _:
            90
        Result::Ok _:
            code

fn run_case %impure fn void i32 \void:
    match surface_id_result 97:
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
        test::test_report_new "gui_platform_bare_display_operation_driver_bridge_import"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
