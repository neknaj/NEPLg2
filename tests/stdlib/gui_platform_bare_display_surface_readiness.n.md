# GUI platform bare display surface readiness doctests

このファイルは、F5fv の Bare whole-surface packet-readiness aggregation boundary が F5fu の owner-bearing packet readiness を順序付きに集約し、hardware flush や scheduler completion と混同しないことを確認する。

executable labels:

- platform_bare_display_surface_readiness_facade_ok
- platform_bare_display_surface_readiness_import_create_free_ok

source policy only labels:

- platform_bare_display_surface_readiness_source_policy_ready_value_authority_ok
- platform_bare_display_surface_readiness_source_policy_cursor_seal_ok
- platform_bare_display_surface_readiness_source_policy_continue_handoff_ok
- platform_bare_display_surface_readiness_source_policy_advance_error_recovery_ok
- platform_bare_display_surface_readiness_source_policy_completed_owner_ok
- platform_bare_display_surface_readiness_source_policy_ordered_tile_coverage_ok
- platform_bare_display_surface_readiness_source_policy_duplicate_gap_errors_ok
- platform_bare_display_surface_readiness_source_policy_no_copy_owner_payload_ok
- platform_bare_display_surface_readiness_no_loop_queue_fallback

## import and owner lifecycle

`display_surface_readiness` は bare facade から import できる。positive aggregation path は F5fu の packet readiness sequence を必要とするため source-policy で構造を固定し、この実行テストでは facade import と owner lifecycle が壊れていないことを確認する。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_surface_readiness_import\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "core/math" as *
#import "platforms/gui/bare" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_display_surface_readiness_facade_ok
// platform_bare_display_surface_readiness_import_create_free_ok
// platform_bare_display_surface_readiness_source_policy_ready_value_authority_ok
// platform_bare_display_surface_readiness_source_policy_cursor_seal_ok
// platform_bare_display_surface_readiness_source_policy_continue_handoff_ok
// platform_bare_display_surface_readiness_source_policy_advance_error_recovery_ok
// platform_bare_display_surface_readiness_source_policy_completed_owner_ok
// platform_bare_display_surface_readiness_source_policy_ordered_tile_coverage_ok
// platform_bare_display_surface_readiness_source_policy_duplicate_gap_errors_ok
// platform_bare_display_surface_readiness_source_policy_no_copy_owner_payload_ok
// platform_bare_display_surface_readiness_no_loop_queue_fallback

fn finish_case %impure fn GuiBareDisplayMemoryOwner impure fn i32 i32 \owner\code:
    match gui_bare_display_memory_owner_free owner:
        Result::Err _:
            90
        Result::Ok _:
            code

fn run_case %impure fn void i32 \void:
    match surface_id_result 95:
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
        test::test_report_new "gui_platform_bare_display_surface_readiness_import"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
