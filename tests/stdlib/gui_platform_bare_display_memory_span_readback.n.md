# GUI platform bare display memory span readback doctests

このファイルは、F5fs の Bare display memory span write/readback boundary が owner の canonical driver state から span を確定し、owner 内 raw memory だけを使って span byte range の store / readback evidence を作ることを確認する。

executable labels:

- platform_bare_display_memory_span_readback_facade_ok
- platform_bare_display_memory_span_readback_import_create_free_ok

source policy only labels:

- platform_bare_display_memory_span_readback_source_policy_driver_apply_authority_ok
- platform_bare_display_memory_span_readback_source_policy_no_copy_owner_payload_ok
- platform_bare_display_memory_span_readback_source_policy_no_forgeable_public_authority_ok
- platform_bare_display_memory_span_readback_source_policy_store_before_readback_ok
- platform_bare_display_memory_span_readback_source_policy_state_advance_after_readback_ok
- platform_bare_display_memory_span_readback_source_policy_clears_single_byte_evidence_ok
- platform_bare_display_memory_span_readback_source_policy_span_not_frame_ready_ok
- platform_bare_display_memory_span_readback_no_host_import_fallback

## import and owner lifecycle

`display_memory_span_readback` は bare facade から import できる。span write/readback success path は canonical span sequence を必要とするため source-policy で構造を固定し、この実行テストでは owner lifecycle が壊れていないことを確認する。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_memory_span_readback_import\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "platforms/gui/bare" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_display_memory_span_readback_facade_ok
// platform_bare_display_memory_span_readback_import_create_free_ok
// platform_bare_display_memory_span_readback_source_policy_driver_apply_authority_ok
// platform_bare_display_memory_span_readback_source_policy_no_copy_owner_payload_ok
// platform_bare_display_memory_span_readback_source_policy_no_forgeable_public_authority_ok
// platform_bare_display_memory_span_readback_source_policy_store_before_readback_ok
// platform_bare_display_memory_span_readback_source_policy_state_advance_after_readback_ok
// platform_bare_display_memory_span_readback_source_policy_clears_single_byte_evidence_ok
// platform_bare_display_memory_span_readback_source_policy_span_not_frame_ready_ok
// platform_bare_display_memory_span_readback_no_host_import_fallback

fn finish_case %impure fn GuiBareDisplayMemoryOwner impure fn i32 i32 \owner\code:
    match gui_bare_display_memory_owner_free owner:
        Result::Err _:
            90
        Result::Ok _:
            code

fn run_case %impure fn void i32 \void:
    match surface_id_result 92:
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
        test::test_report_new "gui_platform_bare_display_memory_span_readback_import"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
