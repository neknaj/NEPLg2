# GUI platform bare display driver host import doctests

このファイルは、F5fp の Bare display driver host import boundary が F5fo の pure ledger preflight を host import の前に通し、host status を typed outcome へ変換してから同じ ledger へ再適用する設計であることを確認する。

executable labels:

- platform_bare_display_driver_host_import_facade_ok
- platform_bare_display_driver_host_import_status_mapping_ok

source policy only labels:

- platform_bare_display_driver_host_import_preflight_before_host_ok
- platform_bare_display_driver_host_import_default_unsupported_ok
- platform_bare_display_driver_host_import_span_byte_evidence_ok
- platform_bare_display_driver_host_import_no_loop_queue_fallback

## host import status mapping

status `0` 以外は fail-closed に扱う。`-1` は unsupported host、既知の負 status は typed error、未知の負 status と正の non-zero status は backend failure に写す。

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_platform_bare_display_driver_host_import\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"status mapping\" expected=\"0\" actual=\"0\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/gui/error" as *
#import "core/math" as *
#import "platforms/gui/bare/display_driver_host_import" as *
#import "std/test" as test

// platform_bare_display_driver_host_import_facade_ok
// platform_bare_display_driver_host_import_status_mapping_ok
// platform_bare_display_driver_host_import_preflight_before_host_ok
// platform_bare_display_driver_host_import_default_unsupported_ok
// platform_bare_display_driver_host_import_span_byte_evidence_ok
// platform_bare_display_driver_host_import_no_loop_queue_fallback

fn check_error %fn i32 fn GuiError i32 \status\expected:
    match gui_bare_display_driver_host_import_status_error status:
        GuiError::Unsupported:
            match expected:
                GuiError::Unsupported:
                    0
                _:
                    10
        GuiError::InvalidCommand:
            match expected:
                GuiError::InvalidCommand:
                    0
                _:
                    11
        GuiError::ResourceExhausted:
            match expected:
                GuiError::ResourceExhausted:
                    0
                _:
                    12
        GuiError::BackendFailure:
            match expected:
                GuiError::BackendFailure:
                    0
                _:
                    13
        GuiError::InvalidGeometry:
            14
        GuiError::InvalidColor:
            15
        GuiError::MissingCapability:
            16

fn run_case %fn void i32 \void:
    let unsupported %i32 check_error -1 GuiError::Unsupported
    if ne unsupported 0:
        then unsupported
        else:
            let invalid %i32 check_error -2 GuiError::InvalidCommand
            if ne invalid 0:
                then invalid
                else:
                    let exhausted %i32 check_error -3 GuiError::ResourceExhausted
                    if ne exhausted 0:
                        then exhausted
                        else:
                            let unknown_negative %i32 check_error -99 GuiError::BackendFailure
                            if ne unknown_negative 0:
                                then unknown_negative
                                else check_error 7 GuiError::BackendFailure

fn main %impure fn void i32 \void:
    let actual %i32 run_case
    let report:
        test::test_report_new "gui_platform_bare_display_driver_host_import"
        |> test::test_report_push test::assert_eq_i32 "status mapping" 0 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```
