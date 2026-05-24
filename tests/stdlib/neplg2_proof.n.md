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

#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "std/test" as *

fn check_span_proven %fn Result () SelfhostProofRefutation Result () str \result:
    match result:
        Result::Ok _:
            Result<(),str>::Ok ()
        Result::Err _refutation:
            Result<(),str>::Err "expected span proof"

fn check_span_invalid %fn Result () SelfhostProofRefutation Result () str \result:
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result<(),str>::Ok ()
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::UnexpectedEvidence _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::TypeKindMismatch _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::OwnerTransitionInvalid _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::BorrowAccessInvalid _issue:
                    Result<(),str>::Err "expected invalid span refutation"
                SelfhostProofRefutation::EffectBoundaryInvalid _issue:
                    Result<(),str>::Err "expected invalid span refutation"
        Result::Ok _:
            Result<(),str>::Err "invalid span was accepted"

fn main %impure fn () i32 \():
    let valid %SelfhostSourceSpan source_span_new 0 0 4
    let invalid %SelfhostSourceSpan source_span_new 0 5 2
    let checks0 checks_new
    let checks1 checks_push checks0 check_span_proven selfhost_proof_source_span_valid valid
    let checks2 checks_push checks1 check_span_invalid selfhost_proof_source_span_valid invalid
    let shown checks_print_report checks2
    checks_exit_code shown
```

## fact_obligation_mismatch_preserves_domains

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "std/test" as *

fn check_domain_mismatch %fn SelfhostProofRefutation Result () str \refutation:
    match refutation:
        SelfhostProofRefutation::FactObligationMismatch mismatch:
            match mismatch.fact_domain:
                SelfhostProofDomain::Module:
                    match mismatch.obligation_domain:
                        SelfhostProofDomain::Source:
                            Result<(),str>::Ok ()
                        SelfhostProofDomain::Module:
                            Result<(),str>::Err "expected source obligation domain"
                        SelfhostProofDomain::Type:
                            Result<(),str>::Err "expected source obligation domain"
                        SelfhostProofDomain::Trait:
                            Result<(),str>::Err "expected source obligation domain"
                        SelfhostProofDomain::Lifetime:
                            Result<(),str>::Err "expected source obligation domain"
                        SelfhostProofDomain::Owner:
                            Result<(),str>::Err "expected source obligation domain"
                        SelfhostProofDomain::Effect:
                            Result<(),str>::Err "expected source obligation domain"
                        SelfhostProofDomain::Resource:
                            Result<(),str>::Err "expected source obligation domain"
                SelfhostProofDomain::Source:
                    Result<(),str>::Err "expected module fact domain"
                SelfhostProofDomain::Type:
                    Result<(),str>::Err "expected module fact domain"
                SelfhostProofDomain::Trait:
                    Result<(),str>::Err "expected module fact domain"
                SelfhostProofDomain::Lifetime:
                    Result<(),str>::Err "expected module fact domain"
                SelfhostProofDomain::Owner:
                    Result<(),str>::Err "expected module fact domain"
                SelfhostProofDomain::Effect:
                    Result<(),str>::Err "expected module fact domain"
                SelfhostProofDomain::Resource:
                    Result<(),str>::Err "expected module fact domain"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result<(),str>::Err "expected fact/obligation mismatch"

fn main %impure fn () i32 \():
    let span %SelfhostSourceSpan source_span_new 0 0 5
    let raw_item %SelfhostRawBackendItemFact selfhost_raw_backend_item_fact_new SelfhostRawBackendItemKind::WasmBlock span
    let fact %SelfhostProofFact SelfhostProofFact::RawBackendItemObserved raw_item
    let obligation %SelfhostProofObligation SelfhostProofObligation::SourceSpanValid span
    let checks0 checks_new
    match selfhost_proof_solve selfhost_proof_query_new fact obligation:
        SelfhostProofResult::Refuted refutation:
            let checks1 checks_push checks0 check_domain_mismatch refutation
            let shown checks_print_report checks1
            checks_exit_code shown
        SelfhostProofResult::Proven _evidence:
            let checks1 checks_push checks0 Result<(),str>::Err "mismatched proof query was accepted"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## resource_cell_transition_uses_generic_proof

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "neplg2/core/resource/move_state" as *
#import "std/test" as *

fn check_initialized %fn SelfhostResourceCellState Result () str \state:
    match state:
        SelfhostResourceCellState::Initialized:
            Result<(),str>::Ok ()
        SelfhostResourceCellState::Uninitialized:
            Result<(),str>::Err "expected initialized cell"
        SelfhostResourceCellState::Moved:
            Result<(),str>::Err "expected initialized cell"
        SelfhostResourceCellState::Dropped:
            Result<(),str>::Err "expected initialized cell"

fn check_moved %fn SelfhostResourceCellState Result () str \state:
    match state:
        SelfhostResourceCellState::Moved:
            Result<(),str>::Ok ()
        SelfhostResourceCellState::Uninitialized:
            Result<(),str>::Err "expected moved cell"
        SelfhostResourceCellState::Initialized:
            Result<(),str>::Err "expected moved cell"
        SelfhostResourceCellState::Dropped:
            Result<(),str>::Err "expected moved cell"

fn check_drop_after_move %fn SelfhostProofRefutation Result () str \refutation:
    match refutation:
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result<(),str>::Err "expected proof refutation"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result<(),str>::Err "expected proof refutation"
        SelfhostProofRefutation::ResourceCellTransitionInvalid issue:
            match issue.reason:
                SelfhostResourceCellTransitionError::DropAfterMove:
                    Result<(),str>::Ok ()
                SelfhostResourceCellTransitionError::InitializeAlreadyInitialized:
                    Result<(),str>::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::InitializeAfterDrop:
                    Result<(),str>::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::MoveUninitialized:
                    Result<(),str>::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::MoveAfterMove:
                    Result<(),str>::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::MoveAfterDrop:
                    Result<(),str>::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::DropUninitialized:
                    Result<(),str>::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::DoubleDrop:
                    Result<(),str>::Err "expected drop-after-move"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result<(),str>::Err "expected resource transition refutation"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result<(),str>::Err "expected resource transition refutation"

fn main %impure fn () i32 \():
    let span %SelfhostSourceSpan source_span_new 0 0 4
    let init_event %SelfhostResourceCellEventFact selfhost_resource_cell_event_fact_new SelfhostResourceCellEventKind::Initialize span
    let move_event %SelfhostResourceCellEventFact selfhost_resource_cell_event_fact_new SelfhostResourceCellEventKind::MoveOut span
    let drop_event %SelfhostResourceCellEventFact selfhost_resource_cell_event_fact_new SelfhostResourceCellEventKind::Drop span
    let checks0 checks_new
    match selfhost_proof_resource_cell_transition selfhost_resource_cell_state_initial init_event:
        Result::Ok state1:
            let checks1 checks_push checks0 check_initialized state1
            match selfhost_proof_resource_cell_transition state1 move_event:
                Result::Ok state2:
                    let checks2 checks_push checks1 check_moved state2
                    match selfhost_proof_resource_cell_transition state2 drop_event:
                        Result::Err refutation:
                            let checks3 checks_push checks2 check_drop_after_move refutation
                            let shown checks_print_report checks3
                            checks_exit_code shown
                        Result::Ok _state:
                            let checks3 checks_push checks2 Result<(),str>::Err "drop after move was accepted"
                            let shown checks_print_report checks3
                            checks_exit_code shown
                Result::Err _refutation:
                    let checks2 checks_push checks1 Result<(),str>::Err "move transition failed"
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _refutation:
            let checks1 checks_push checks0 Result<(),str>::Err "initialize transition failed"
            let shown checks_print_report checks1
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

fn check_none_seen %fn SelfhostModuleDirectiveState Result () str \state:
    match state:
        SelfhostModuleDirectiveState::NoneSeen:
            Result<(),str>::Ok ()
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result<(),str>::Err "expected no singleton directive"
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result<(),str>::Err "expected no singleton directive"
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result<(),str>::Err "expected no singleton directive"

fn check_target_seen %fn SelfhostModuleDirectiveState Result () str \state:
    match state:
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result<(),str>::Ok ()
        SelfhostModuleDirectiveState::NoneSeen:
            Result<(),str>::Err "expected target directive"
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result<(),str>::Err "expected target directive"
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result<(),str>::Err "expected target directive"

fn check_both_seen %fn SelfhostModuleDirectiveState Result () str \state:
    match state:
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result<(),str>::Ok ()
        SelfhostModuleDirectiveState::NoneSeen:
            Result<(),str>::Err "expected entry and target directives"
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result<(),str>::Err "expected entry and target directives"
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result<(),str>::Err "expected entry and target directives"

fn check_duplicate_target %fn SelfhostProofRefutation Result () str \refutation:
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
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result<(),str>::Err "expected duplicate target"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result<(),str>::Err "expected duplicate target"

fn main %impure fn () i32 \():
    let span1 %SelfhostSourceSpan source_span_new 0 0 7
    let span2 %SelfhostSourceSpan source_span_new 0 8 15
    let checks0 checks_new
    let other_item %SelfhostModuleDirectiveFact selfhost_module_directive_fact_new SelfhostModuleDirectiveKind::Other span1
    match selfhost_proof_module_directive_transition SelfhostModuleDirectiveState::NoneSeen other_item:
        Result::Ok state0:
            let checks1 checks_push checks0 check_none_seen state0
            let target_item %SelfhostModuleDirectiveFact selfhost_module_directive_fact_new SelfhostModuleDirectiveKind::Target span1
            match selfhost_proof_module_directive_transition state0 target_item:
                Result::Ok state1:
                    let checks2 checks_push checks1 check_target_seen state1
                    let entry_item %SelfhostModuleDirectiveFact selfhost_module_directive_fact_new SelfhostModuleDirectiveKind::Entry span2
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

fn check_open_empty_wasm %fn SelfhostRawBackendState Result () str \state:
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

fn check_open_ready_wasm %fn SelfhostRawBackendState Result () str \state:
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

fn check_normal %fn SelfhostRawBackendState Result () str \state:
    match state:
        SelfhostRawBackendState::Normal:
            Result<(),str>::Ok ()
        SelfhostRawBackendState::OpenEmpty _open_block:
            Result<(),str>::Err "expected normal state"
        SelfhostRawBackendState::OpenReady _kind:
            Result<(),str>::Err "expected normal state"

fn check_raw_text_refutation %fn SelfhostProofRefutation Result () str \refutation:
    match refutation:
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result<(),str>::Ok ()
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result<(),str>::Err "expected text-without-block refutation"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result<(),str>::Err "expected text-without-block refutation"

fn main %impure fn () i32 \():
    let span %SelfhostSourceSpan source_span_new 0 0 5
    let checks0 checks_new
    let block_item %SelfhostRawBackendItemFact selfhost_raw_backend_item_fact_new SelfhostRawBackendItemKind::WasmBlock span
    match selfhost_proof_raw_backend_transition SelfhostRawBackendState::Normal block_item:
        Result::Ok state1:
            let checks1 checks_push checks0 check_open_empty_wasm state1
            let text_item %SelfhostRawBackendItemFact selfhost_raw_backend_item_fact_new SelfhostRawBackendItemKind::WasmText span
            match selfhost_proof_raw_backend_transition state1 text_item:
                Result::Ok state2:
                    let checks2 checks_push checks1 check_open_ready_wasm state2
                    let end_item %SelfhostRawBackendItemFact selfhost_raw_backend_item_fact_new SelfhostRawBackendItemKind::StreamEnd span
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

## module_declaration_header_requires_parser_evidence

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

#import "core/option" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "neplg2/core/syntax/ast/module_ast" as *
#import "std/test" as *

fn check_header_proven %fn Result SelfhostModuleDeclarationHeader SelfhostProofRefutation Result () str \result:
    match result:
        Result::Ok header:
            match header.kind:
                SelfhostModuleDeclarationKind::Function:
                    Result<(),str>::Ok ()
                SelfhostModuleDeclarationKind::Struct:
                    Result<(),str>::Err "expected function header proof"
                SelfhostModuleDeclarationKind::Enum:
                    Result<(),str>::Err "expected function header proof"
                SelfhostModuleDeclarationKind::Trait:
                    Result<(),str>::Err "expected function header proof"
                SelfhostModuleDeclarationKind::Impl:
                    Result<(),str>::Err "expected function header proof"
        Result::Err _refutation:
            Result<(),str>::Err "expected declaration header proof"

fn check_header_missing %fn Result SelfhostModuleDeclarationHeader SelfhostProofRefutation Result () str \result:
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result<(),str>::Ok ()
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::UnexpectedEvidence _issue:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::TypeKindMismatch _issue:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::OwnerTransitionInvalid _issue:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::BorrowAccessInvalid _issue:
                    Result<(),str>::Err "expected missing declaration header"
                SelfhostProofRefutation::EffectBoundaryInvalid _issue:
                    Result<(),str>::Err "expected missing declaration header"
        Result::Ok _header:
            Result<(),str>::Err "missing declaration header was accepted"

fn check_header_invalid %fn Result SelfhostModuleDeclarationHeader SelfhostProofRefutation Result () str \result:
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result<(),str>::Ok ()
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::UnexpectedEvidence _issue:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::TypeKindMismatch _issue:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::OwnerTransitionInvalid _issue:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::BorrowAccessInvalid _issue:
                    Result<(),str>::Err "expected invalid declaration header"
                SelfhostProofRefutation::EffectBoundaryInvalid _issue:
                    Result<(),str>::Err "expected invalid declaration header"
        Result::Ok _header:
            Result<(),str>::Err "invalid declaration header was accepted"

fn main %impure fn () i32 \():
    let header_span %SelfhostSourceSpan source_span_new 0 0 24
    let keyword_span %SelfhostSourceSpan source_span_new 0 0 2
    let head_span %SelfhostSourceSpan source_span_new 0 3 7
    let head %SelfhostModuleDeclarationHead selfhost_module_declaration_head_new SelfhostModuleDeclarationHeadKind::Name head_span
    let header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Function SelfhostModuleDeclarationVisibility::Private header_span keyword_span some<SelfhostModuleDeclarationHead> head
    let valid_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl some<SelfhostModuleDeclarationHeader> header header_span
    let missing_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl none<SelfhostModuleDeclarationHeader> header_span
    let invalid_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Struct SelfhostModuleDeclarationVisibility::Private header_span keyword_span some<SelfhostModuleDeclarationHead> head
    let invalid_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl some<SelfhostModuleDeclarationHeader> invalid_header header_span
    let impl_header_span %SelfhostSourceSpan source_span_new 0 0 22
    let impl_keyword_span %SelfhostSourceSpan source_span_new 0 4 8
    let impl_head_span %SelfhostSourceSpan source_span_new 0 9 13
    let impl_head %SelfhostModuleDeclarationHead selfhost_module_declaration_head_new SelfhostModuleDeclarationHeadKind::Name impl_head_span
    let public_impl_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Impl SelfhostModuleDeclarationVisibility::Public impl_header_span impl_keyword_span some<SelfhostModuleDeclarationHead> impl_head
    let public_impl_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::ImplDecl some<SelfhostModuleDeclarationHeader> public_impl_header impl_header_span
    let checks0 checks_new
    let checks1 checks_push checks0 check_header_proven selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function valid_fact
    let checks2 checks_push checks1 check_header_missing selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function missing_fact
    let checks3 checks_push checks2 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function invalid_fact
    let checks4 checks_push checks3 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Impl public_impl_fact
    let shown checks_print_report checks4
    checks_exit_code shown
```
