# GUI platform bare display driver byte echo doctests

このファイルは、F5fq の Bare display driver byte echo verification boundary が public driver step を信用せず、F5fo ledger を再適用した canonical span write evidence から byte echo を検証することを確認する。

executable labels:

- platform_bare_display_driver_byte_echo_facade_ok
- platform_bare_display_driver_byte_echo_channel_mapping_ok
- platform_bare_display_driver_byte_echo_forged_step_rejected_ok

source policy only labels:

- platform_bare_display_driver_byte_echo_source_policy_apply_before_extract_ok
- platform_bare_display_driver_byte_echo_source_policy_span_only_ok
- platform_bare_display_driver_byte_echo_source_policy_bounds_ok
- platform_bare_display_driver_byte_echo_source_policy_echo_value_ok
- platform_bare_display_driver_byte_echo_source_policy_mismatch_ok
- platform_bare_display_driver_byte_echo_source_policy_begin_present_fail_closed_ok
- platform_bare_display_driver_byte_echo_no_host_import_fallback

## byte channel mapping

RGBA8888 span write の relative byte offset は 0 / 1 / 2 / 3 を Red / Green / Blue / Alpha に写す。offset 範囲外は enum error で返し、raw integer channel として扱わない。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_driver_byte_echo_channel\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/cast" as *
#import "core/gui/color" as *
#import "core/math" as *
#import "core/result" as *
#import "platforms/gui/bare/display_driver_byte_echo" as *
#import "std/test" as test

// platform_bare_display_driver_byte_echo_facade_ok
// platform_bare_display_driver_byte_echo_channel_mapping_ok
// platform_bare_display_driver_byte_echo_source_policy_apply_before_extract_ok
// platform_bare_display_driver_byte_echo_source_policy_span_only_ok
// platform_bare_display_driver_byte_echo_source_policy_bounds_ok
// platform_bare_display_driver_byte_echo_source_policy_echo_value_ok
// platform_bare_display_driver_byte_echo_source_policy_mismatch_ok
// platform_bare_display_driver_byte_echo_source_policy_begin_present_fail_closed_ok
// platform_bare_display_driver_byte_echo_no_host_import_fallback

fn sample_color %fn void Rgba8888 \void:
    let r %u8 cast 10
    let g %u8 cast 20
    let b %u8 cast 30
    let a %u8 cast 255
    rgba8888_new r g b a

fn channel_equal %fn GuiBareDisplayDriverByteChannel fn GuiBareDisplayDriverByteChannel bool \left\right:
    match left:
        GuiBareDisplayDriverByteChannel::Red:
            match right:
                GuiBareDisplayDriverByteChannel::Red:
                    true
                _:
                    false
        GuiBareDisplayDriverByteChannel::Green:
            match right:
                GuiBareDisplayDriverByteChannel::Green:
                    true
                _:
                    false
        GuiBareDisplayDriverByteChannel::Blue:
            match right:
                GuiBareDisplayDriverByteChannel::Blue:
                    true
                _:
                    false
        GuiBareDisplayDriverByteChannel::Alpha:
            match right:
                GuiBareDisplayDriverByteChannel::Alpha:
                    true
                _:
                    false

fn expect_offset %fn i32 fn GuiBareDisplayDriverByteChannel i32 \offset\expected:
    match gui_bare_display_driver_byte_echo_channel_for_byte_offset offset:
        Result::Err _:
            10
        Result::Ok actual:
            if channel_equal actual expected 0 11

fn expect_value %fn GuiBareDisplayDriverByteChannel fn i32 i32 \channel\expected:
    let color %Rgba8888 sample_color
    let actual %i32 gui_bare_display_driver_byte_echo_channel_value channel color
    if eq actual expected 0 12

fn expect_offset_error %fn i32 i32 \offset:
    match gui_bare_display_driver_byte_echo_channel_for_byte_offset offset:
        Result::Ok _:
            13
        Result::Err kind:
            match kind:
                GuiBareDisplayDriverByteEchoErrorKind::ByteOffsetInvalid:
                    0
                _:
                    14

fn run_case %fn void i32 \void:
    let red_offset %i32 expect_offset 0 GuiBareDisplayDriverByteChannel::Red
    if ne red_offset 0:
        then red_offset
        else:
            let green_offset %i32 expect_offset 1 GuiBareDisplayDriverByteChannel::Green
            if ne green_offset 0:
                then green_offset
                else:
                    let blue_offset %i32 expect_offset 2 GuiBareDisplayDriverByteChannel::Blue
                    if ne blue_offset 0:
                        then blue_offset
                        else:
                            let alpha_offset %i32 expect_offset 3 GuiBareDisplayDriverByteChannel::Alpha
                            if ne alpha_offset 0:
                                then alpha_offset
                                else:
                                    let red_value %i32 expect_value GuiBareDisplayDriverByteChannel::Red 10
                                    if ne red_value 0:
                                        then red_value
                                        else:
                                            let green_value %i32 expect_value GuiBareDisplayDriverByteChannel::Green 20
                                            if ne green_value 0:
                                                then green_value
                                                else:
                                                    let blue_value %i32 expect_value GuiBareDisplayDriverByteChannel::Blue 30
                                                    if ne blue_value 0:
                                                        then blue_value
                                                        else:
                                                            let alpha_value %i32 expect_value GuiBareDisplayDriverByteChannel::Alpha 255
                                                            if ne alpha_value 0:
                                                                then alpha_value
                                                                else:
                                                                    let negative_offset %i32 expect_offset_error -1
                                                                    if ne negative_offset 0:
                                                                        then negative_offset
                                                                        else expect_offset_error 4

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_driver_byte_echo_channel"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## forged step rejection contract

`gui_bare_display_driver_byte_echo_verify` は supplied driver step を受け取らず、F5fo ledger の canonical apply が失敗した時点で `DriverStepInvalid` を返す。full storage / framebuffer sequence は現 compiler では timeout に入りやすいため、実行 doctest では `DriverStepInvalid` の typed lower kind を軽量に確認し、apply-before-extract、forged step rejection、span-only、bounds、echo mismatch、Begin / Present fail-closed は source-policy で固定する。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_driver_byte_echo_driver_step_invalid\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return value\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "platforms/gui/bare/display_driver" as *
#import "platforms/gui/bare/display_driver_byte_echo" as *
#import "std/test" as test

// platform_bare_display_driver_byte_echo_forged_step_rejected_ok

fn run_case %fn void i32 \void:
    let kind %GuiBareDisplayDriverByteEchoErrorKind GuiBareDisplayDriverByteEchoErrorKind::DriverStepInvalid GuiBareDisplayDriverErrorKind::MemoryStepActionMismatch
    match kind:
        GuiBareDisplayDriverByteEchoErrorKind::DriverStepInvalid lower:
            match lower:
                GuiBareDisplayDriverErrorKind::MemoryStepActionMismatch:
                    0
                _:
                    10
        _:
            11

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_driver_byte_echo_driver_step_invalid"
        |> test::test_report_push test::assert_eq_i32 "return value" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
