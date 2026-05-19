# NEPLg2 self-host proof

## source_span_validity_uses_typed_fact_and_obligation

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
#entry main
#target std
#indent 4

#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "std/test" as *

fn main <()*>i32> ():
    let valid <SelfhostSourceSpan> source_span_new 0 0 4
    let invalid <SelfhostSourceSpan> source_span_new 0 5 2
    let checks0 checks_new
    let checks1 checks_push checks0 check selfhost_proof_source_span_valid valid
    let checks2 checks_push checks1 check_ne true selfhost_proof_source_span_valid invalid
    let shown checks_print_report checks2
    checks_exit_code shown
```
