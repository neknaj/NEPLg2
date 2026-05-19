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

## module_directive_transition_rejects_duplicate_singletons

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "std/test" as *

fn check_none_seen <(SelfhostModuleDirectiveState)->Result<(),str>> (state):
    match state:
        SelfhostModuleDirectiveState::NoneSeen:
            Result<(),str>::Ok ()
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result<(),str>::Err "expected no singleton directive"
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result<(),str>::Err "expected no singleton directive"
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result<(),str>::Err "expected no singleton directive"

fn check_target_seen <(SelfhostModuleDirectiveState)->Result<(),str>> (state):
    match state:
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result<(),str>::Ok ()
        SelfhostModuleDirectiveState::NoneSeen:
            Result<(),str>::Err "expected target directive"
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result<(),str>::Err "expected target directive"
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result<(),str>::Err "expected target directive"

fn check_both_seen <(SelfhostModuleDirectiveState)->Result<(),str>> (state):
    match state:
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result<(),str>::Ok ()
        SelfhostModuleDirectiveState::NoneSeen:
            Result<(),str>::Err "expected entry and target directives"
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result<(),str>::Err "expected entry and target directives"
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result<(),str>::Err "expected entry and target directives"

fn check_duplicate_target <(SelfhostProofRefutation)->Result<(),str>> (refutation):
    match refutation:
        SelfhostProofRefutation::ModuleDirectiveDuplicate duplicate:
            match duplicate.kind:
                SelfhostModuleDirectiveKind::Target:
                    Result<(),str>::Ok ()
                SelfhostModuleDirectiveKind::Entry:
                    Result<(),str>::Err "expected duplicate target"
                SelfhostModuleDirectiveKind::Other:
                    Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::FactObligationMismatch:
            Result<(),str>::Err "expected duplicate target"

fn main <()*>i32> ():
    let span1 <SelfhostSourceSpan> source_span_new 0 0 7
    let span2 <SelfhostSourceSpan> source_span_new 0 8 15
    let checks0 checks_new
    let other_item <SelfhostModuleDirectiveFact> selfhost_module_directive_fact_new SelfhostModuleDirectiveKind::Other span1
    match selfhost_proof_module_directive_transition SelfhostModuleDirectiveState::NoneSeen other_item:
        Result::Ok state0:
            let checks1 checks_push checks0 check_none_seen state0
            let target_item <SelfhostModuleDirectiveFact> selfhost_module_directive_fact_new SelfhostModuleDirectiveKind::Target span1
            match selfhost_proof_module_directive_transition state0 target_item:
                Result::Ok state1:
                    let checks2 checks_push checks1 check_target_seen state1
                    let entry_item <SelfhostModuleDirectiveFact> selfhost_module_directive_fact_new SelfhostModuleDirectiveKind::Entry span2
                    match selfhost_proof_module_directive_transition state1 entry_item:
                        Result::Ok state2:
                            let checks3 checks_push checks2 check_both_seen state2
                            match selfhost_proof_module_directive_transition state2 target_item:
                                Result::Err refutation:
                                    let checks4 checks_push checks3 check_duplicate_target refutation
                                    let shown checks_print_report checks4
                                    checks_exit_code shown
                                Result::Ok _state:
                                    let checks4 checks_push checks3 Result<(),str>::Err "duplicate target was accepted"
                                    let shown checks_print_report checks4
                                    checks_exit_code shown
                        Result::Err _refutation:
                            let checks3 checks_push checks2 Result<(),str>::Err "entry transition failed"
                            let shown checks_print_report checks3
                            checks_exit_code shown
                Result::Err _refutation:
                    let checks2 checks_push checks1 Result<(),str>::Err "target transition failed"
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _refutation:
            let checks1 checks_push checks0 Result<(),str>::Err "non-singleton transition failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## raw_backend_transition_returns_typed_evidence_and_refutation

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "std/test" as *

fn check_open_empty_wasm <(SelfhostRawBackendState)->Result<(),str>> (state):
    match state:
        SelfhostRawBackendState::OpenEmpty open_block:
            match open_block.kind:
                SelfhostRawBackendKind::Wasm:
                    Result<(),str>::Ok ()
                SelfhostRawBackendKind::LlvmIr:
                    Result<(),str>::Err "expected wasm empty block"
        SelfhostRawBackendState::Normal:
            Result<(),str>::Err "expected empty block"
        SelfhostRawBackendState::OpenReady _kind:
            Result<(),str>::Err "expected empty block"

fn check_open_ready_wasm <(SelfhostRawBackendState)->Result<(),str>> (state):
    match state:
        SelfhostRawBackendState::OpenReady kind:
            match kind:
                SelfhostRawBackendKind::Wasm:
                    Result<(),str>::Ok ()
                SelfhostRawBackendKind::LlvmIr:
                    Result<(),str>::Err "expected wasm ready block"
        SelfhostRawBackendState::Normal:
            Result<(),str>::Err "expected ready block"
        SelfhostRawBackendState::OpenEmpty _open_block:
            Result<(),str>::Err "expected ready block"

fn check_normal <(SelfhostRawBackendState)->Result<(),str>> (state):
    match state:
        SelfhostRawBackendState::Normal:
            Result<(),str>::Ok ()
        SelfhostRawBackendState::OpenEmpty _open_block:
            Result<(),str>::Err "expected normal state"
        SelfhostRawBackendState::OpenReady _kind:
            Result<(),str>::Err "expected normal state"

fn check_raw_text_refutation <(SelfhostProofRefutation)->Result<(),str>> (refutation):
    match refutation:
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result<(),str>::Ok ()
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::FactObligationMismatch:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result<(),str>::Err "expected text-without-block refutation"

fn main <()*>i32> ():
    let span <SelfhostSourceSpan> source_span_new 0 0 5
    let checks0 checks_new
    let block_item <SelfhostRawBackendItemFact> selfhost_raw_backend_item_fact_new SelfhostRawBackendItemKind::WasmBlock span
    match selfhost_proof_raw_backend_transition SelfhostRawBackendState::Normal block_item:
        Result::Ok state1:
            let checks1 checks_push checks0 check_open_empty_wasm state1
            let text_item <SelfhostRawBackendItemFact> selfhost_raw_backend_item_fact_new SelfhostRawBackendItemKind::WasmText span
            match selfhost_proof_raw_backend_transition state1 text_item:
                Result::Ok state2:
                    let checks2 checks_push checks1 check_open_ready_wasm state2
                    let end_item <SelfhostRawBackendItemFact> selfhost_raw_backend_item_fact_new SelfhostRawBackendItemKind::StreamEnd span
                    match selfhost_proof_raw_backend_transition state2 end_item:
                        Result::Ok state3:
                            let checks3 checks_push checks2 check_normal state3
                            match selfhost_proof_raw_backend_transition SelfhostRawBackendState::Normal text_item:
                                Result::Err refutation:
                                    let checks4 checks_push checks3 check_raw_text_refutation refutation
                                    let shown checks_print_report checks4
                                    checks_exit_code shown
                                Result::Ok _state:
                                    let checks4 checks_push checks3 Result<(),str>::Err "orphan raw text was accepted"
                                    let shown checks_print_report checks4
                                    checks_exit_code shown
                        Result::Err _refutation:
                            let checks3 checks_push checks2 Result<(),str>::Err "stream end transition failed"
                            let shown checks_print_report checks3
                            checks_exit_code shown
                Result::Err _refutation:
                    let checks2 checks_push checks1 Result<(),str>::Err "raw text transition failed"
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _refutation:
            let checks1 checks_push checks0 Result<(),str>::Err "raw block transition failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```
