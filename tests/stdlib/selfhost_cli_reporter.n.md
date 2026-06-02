# selfhost CLI reporter

## selfhost_cli_reporter_renders_single_human_and_json

neplg2:test[stdio, normalize_newlines]
stdout: mlstr:
##: Checked [ok,ok]
##: [0] ok
##: [1] ok
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/reporter/render/single" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/span" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let span %SelfhostSourceSpan source_span_new 1 10 15
    let label %SelfhostDiagnosticLabel selfhost_diag_label_new span "token"
    let diag %SelfhostDiagnostic selfhost_diag_with_note selfhost_diag_with_primary_label selfhost_diag_error SelfhostDiagnosticCode::Parser SelfhostParserDiagnosticCode::TokenIndex "second" label "fix it"
    let human %str selfhost_cli_render_diagnostic_human &diag
    let json %str selfhost_cli_render_diagnostic_json &diag
    let checks:
        checks_new
        |> checks_push assert_str_eq "error[parser.token.index_unavailable]: second\n  --> file 1:10..15\n  = label: token\n  = note: fix it\n" human
        |> checks_push assert_str_eq "{\"severity\":\"error\",\"code\":\"parser.token.index_unavailable\",\"message\":\"second\",\"primary_label\":{\"file_id\":1,\"start\":10,\"end\":15,\"message\":\"token\"},\"note\":\"fix it\"}" json
    let shown checks_print_report checks;
    checks_exit_code shown
```

## selfhost_cli_reporter_writes_json_stdout_and_human_stderr

neplg2:test[stdio, normalize_newlines]
stdout: "{\"severity\":\"error\",\"code\":\"parser.token.index_unavailable\",\"message\":\"bad input\",\"primary_label\":null,\"note\":null}"
stderr: "error[parser.token.index_unavailable]: bad input\n"
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/reporter/write" as *
#import "neplg2/core/infra/diag" as *
#import "core/result" as *

fn main %impure fn void i32 \void:
    let diag %SelfhostDiagnostic selfhost_diag_error SelfhostDiagnosticCode::Parser SelfhostParserDiagnosticCode::TokenIndex "bad input"
    match selfhost_cli_write_json_diagnostic_stdout diag:
        Result::Err _e:
            1
        Result::Ok _:
            match selfhost_cli_write_human_diagnostic_stderr diag:
                Result::Err _e:
                    2
                Result::Ok _:
                    0
```

## selfhost_cli_reporter_renders_collection_human_and_json

neplg2:test[stdio, normalize_newlines]
stdout: mlstr:
##: Checked [ok,ok]
##: [0] ok
##: [1] ok
exit_code: 0
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/cli/reporter/render/collection" as *
#import "neplg2/core/infra/diag" as *
#import "neplg2/core/infra/span" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let span %SelfhostSourceSpan source_span_new 1 10 15
    let label %SelfhostDiagnosticLabel selfhost_diag_label_new span "token"
    let diag0 %SelfhostDiagnostic selfhost_diag_warning SelfhostDiagnosticCode::Cli SelfhostCliDiagnosticCode::MissingInput "first"
    let diag1 %SelfhostDiagnostic selfhost_diag_with_note selfhost_diag_with_primary_label selfhost_diag_error SelfhostDiagnosticCode::Parser SelfhostParserDiagnosticCode::TokenIndex "second" label "fix it"
    let diagnostics0 %SelfhostDiagnostics unwrap_ok selfhost_diagnostics_new
    let diagnostics1 %SelfhostDiagnostics unwrap_ok selfhost_diagnostics_push diagnostics0 diag0
    let diagnostics2 %SelfhostDiagnostics unwrap_ok selfhost_diagnostics_push diagnostics1 diag1
    let human %str selfhost_cli_render_diagnostics_human &diagnostics2
    let json %str selfhost_cli_render_diagnostics_json &diagnostics2
    selfhost_diagnostics_free diagnostics2
    let checks:
        checks_new
        |> checks_push assert_str_eq "warning[cli.input.missing]: first\nerror[parser.token.index_unavailable]: second\n  --> file 1:10..15\n  = label: token\n  = note: fix it\n" human
        |> checks_push assert_str_eq "[{\"severity\":\"warning\",\"code\":\"cli.input.missing\",\"message\":\"first\",\"primary_label\":null,\"note\":null},{\"severity\":\"error\",\"code\":\"parser.token.index_unavailable\",\"message\":\"second\",\"primary_label\":{\"file_id\":1,\"start\":10,\"end\":15,\"message\":\"token\"},\"note\":\"fix it\"}]" json
    let shown checks_print_report checks;
    checks_exit_code shown
```
