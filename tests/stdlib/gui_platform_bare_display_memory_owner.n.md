# GUI platform bare display memory owner doctests

このファイルは、F5fr の Bare raw display memory ownership boundary が raw byte memory を owner に閉じ込め、F5fq を owner の canonical driver state から再実行した 1 byte だけを反映することを確認する。

executable labels:

- platform_bare_display_memory_owner_facade_ok
- platform_bare_display_memory_owner_create_free_ok
- platform_bare_display_memory_owner_read_bounds_ok

source policy only labels:

- platform_bare_display_memory_owner_source_policy_owner_embeds_driver_state_ok
- platform_bare_display_memory_owner_source_policy_no_copy_owner_ok
- platform_bare_display_memory_owner_source_policy_reverify_before_write_ok
- platform_bare_display_memory_owner_source_policy_write_success_before_advance_ok
- platform_bare_display_memory_owner_source_policy_exact_one_byte_ok
- platform_bare_display_memory_owner_source_policy_owner_recovery_ok
- platform_bare_display_memory_owner_no_host_import_fallback

## create and read bounds

`gui_bare_display_memory_owner_create` は bare framebuffer config から canonical driver state と exact-size `RegionToken u8` を持つ owner を作る。raw storage は公開せず、範囲外 read は enum error として返す。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_memory_owner_create\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "platforms/gui/bare/display_memory_owner" as *
#import "platforms/gui/bare/framebuffer" as *
#import "std/gui/window" as *
#import "std/test" as test

// platform_bare_display_memory_owner_facade_ok
// platform_bare_display_memory_owner_create_free_ok
// platform_bare_display_memory_owner_read_bounds_ok
// platform_bare_display_memory_owner_source_policy_owner_embeds_driver_state_ok
// platform_bare_display_memory_owner_source_policy_no_copy_owner_ok
// platform_bare_display_memory_owner_source_policy_reverify_before_write_ok
// platform_bare_display_memory_owner_source_policy_write_success_before_advance_ok
// platform_bare_display_memory_owner_source_policy_exact_one_byte_ok
// platform_bare_display_memory_owner_source_policy_owner_recovery_ok
// platform_bare_display_memory_owner_no_host_import_fallback

fn finish_case %impure fn GuiBareDisplayMemoryOwner impure fn i32 i32 \owner\code:
    match gui_bare_display_memory_owner_free owner:
        Result::Err _:
            90
        Result::Ok _:
            code

fn read_oob_ok %fn &GuiBareDisplayMemoryOwner i32 \owner:
    match gui_bare_display_memory_owner_read_byte owner 16:
        Result::Ok _:
            20
        Result::Err kind:
            match kind:
                GuiBareDisplayMemoryOwnerReadErrorKind::ByteIndexOutOfBounds:
                    0
                _:
                    21

fn last_verified_empty_ok %fn &GuiBareDisplayMemoryOwner i32 \owner:
    match gui_bare_display_memory_owner_last_verified owner:
        Option::None:
            0
        Option::Some _:
            30

fn check_owner %impure fn GuiBareDisplayMemoryOwner i32 \owner:
    let byte_count %i32 gui_bare_display_memory_owner_surface_byte_count &owner
    if ne byte_count 16:
        then finish_case owner 10
        else:
            let verified_count %i32 gui_bare_display_memory_owner_verified_byte_count &owner
            if ne verified_count 0:
                then finish_case owner 11
                else:
                    let last_ok %i32 last_verified_empty_ok &owner
                    if ne last_ok 0:
                        then finish_case owner last_ok
                        else:
                            let read_ok %i32 read_oob_ok &owner
                            finish_case owner read_ok

fn run_case %impure fn void i32 \void:
    match surface_id_result 91:
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
                            check_owner owner

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_memory_owner_create"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
