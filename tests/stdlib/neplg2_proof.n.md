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

fn check_span_proven %fn Result unit SelfhostProofRefutation Result unit str \result:
    match result:
        Result::Ok _:
            Result::Ok unit
        Result::Err _refutation:
            Result::Err "expected span proof"

fn check_span_invalid %fn Result unit SelfhostProofRefutation Result unit str \result:
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result::Ok unit
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::UnexpectedEvidence _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::TypeKindMismatch _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::OwnerTransitionInvalid _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::BorrowAccessInvalid _issue:
                    Result::Err "expected invalid span refutation"
                SelfhostProofRefutation::EffectBoundaryInvalid _issue:
                    Result::Err "expected invalid span refutation"
        Result::Ok _:
            Result::Err "invalid span was accepted"

fn main %impure fn void i32 \void:
    let valid %SelfhostSourceSpan source_span_new_unchecked 0 0 4
    let invalid %SelfhostSourceSpan source_span_new_unchecked 0 5 2
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

fn check_domain_mismatch %fn SelfhostProofRefutation Result unit str \refutation:
    match refutation:
        SelfhostProofRefutation::FactObligationMismatch mismatch:
            match mismatch.fact_domain:
                SelfhostProofDomain::Module:
                    match mismatch.obligation_domain:
                        SelfhostProofDomain::Source:
                            Result::Ok unit
                        SelfhostProofDomain::Module:
                            Result::Err "expected source obligation domain"
                        SelfhostProofDomain::Type:
                            Result::Err "expected source obligation domain"
                        SelfhostProofDomain::Trait:
                            Result::Err "expected source obligation domain"
                        SelfhostProofDomain::Lifetime:
                            Result::Err "expected source obligation domain"
                        SelfhostProofDomain::Owner:
                            Result::Err "expected source obligation domain"
                        SelfhostProofDomain::Effect:
                            Result::Err "expected source obligation domain"
                        SelfhostProofDomain::Resource:
                            Result::Err "expected source obligation domain"
                SelfhostProofDomain::Source:
                    Result::Err "expected module fact domain"
                SelfhostProofDomain::Type:
                    Result::Err "expected module fact domain"
                SelfhostProofDomain::Trait:
                    Result::Err "expected module fact domain"
                SelfhostProofDomain::Lifetime:
                    Result::Err "expected module fact domain"
                SelfhostProofDomain::Owner:
                    Result::Err "expected module fact domain"
                SelfhostProofDomain::Effect:
                    Result::Err "expected module fact domain"
                SelfhostProofDomain::Resource:
                    Result::Err "expected module fact domain"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result::Err "expected fact/obligation mismatch"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result::Err "expected fact/obligation mismatch"

fn main %impure fn void i32 \void:
    let span %SelfhostSourceSpan source_span_new_unchecked 0 0 5
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
            let checks1 checks_push checks0 Result::Err "mismatched proof query was accepted"
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

fn check_initialized %fn SelfhostResourceCellState Result unit str \state:
    match state:
        SelfhostResourceCellState::Initialized:
            Result::Ok unit
        SelfhostResourceCellState::Uninitialized:
            Result::Err "expected initialized cell"
        SelfhostResourceCellState::Moved:
            Result::Err "expected initialized cell"
        SelfhostResourceCellState::Dropped:
            Result::Err "expected initialized cell"

fn check_moved %fn SelfhostResourceCellState Result unit str \state:
    match state:
        SelfhostResourceCellState::Moved:
            Result::Ok unit
        SelfhostResourceCellState::Uninitialized:
            Result::Err "expected moved cell"
        SelfhostResourceCellState::Initialized:
            Result::Err "expected moved cell"
        SelfhostResourceCellState::Dropped:
            Result::Err "expected moved cell"

fn check_drop_after_move %fn SelfhostProofRefutation Result unit str \refutation:
    match refutation:
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result::Err "expected proof refutation"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result::Err "expected proof refutation"
        SelfhostProofRefutation::ResourceCellTransitionInvalid issue:
            match issue.reason:
                SelfhostResourceCellTransitionError::DropAfterMove:
                    Result::Ok unit
                SelfhostResourceCellTransitionError::InitializeAlreadyInitialized:
                    Result::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::InitializeAfterDrop:
                    Result::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::MoveUninitialized:
                    Result::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::MoveAfterMove:
                    Result::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::MoveAfterDrop:
                    Result::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::DropUninitialized:
                    Result::Err "expected drop-after-move"
                SelfhostResourceCellTransitionError::DoubleDrop:
                    Result::Err "expected drop-after-move"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result::Err "expected resource transition refutation"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result::Err "expected resource transition refutation"

fn main %impure fn void i32 \void:
    let span %SelfhostSourceSpan source_span_new_unchecked 0 0 4
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
                            let checks3 checks_push checks2 Result::Err "drop after move was accepted"
                            let shown checks_print_report checks3
                            checks_exit_code shown
                Result::Err _refutation:
                    let checks2 checks_push checks1 Result::Err "move transition failed"
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _refutation:
            let checks1 checks_push checks0 Result::Err "initialize transition failed"
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

fn check_none_seen %fn SelfhostModuleDirectiveState Result unit str \state:
    match state:
        SelfhostModuleDirectiveState::NoneSeen:
            Result::Ok unit
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result::Err "expected no singleton directive"
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result::Err "expected no singleton directive"
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result::Err "expected no singleton directive"

fn check_target_seen %fn SelfhostModuleDirectiveState Result unit str \state:
    match state:
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result::Ok unit
        SelfhostModuleDirectiveState::NoneSeen:
            Result::Err "expected target directive"
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result::Err "expected target directive"
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result::Err "expected target directive"

fn check_both_seen %fn SelfhostModuleDirectiveState Result unit str \state:
    match state:
        SelfhostModuleDirectiveState::EntryAndTargetSeen _seen:
            Result::Ok unit
        SelfhostModuleDirectiveState::NoneSeen:
            Result::Err "expected entry and target directives"
        SelfhostModuleDirectiveState::EntrySeen _entry_span:
            Result::Err "expected entry and target directives"
        SelfhostModuleDirectiveState::TargetSeen _target_span:
            Result::Err "expected entry and target directives"

fn check_duplicate_target %fn SelfhostProofRefutation Result unit str \refutation:
    match refutation:
        SelfhostProofRefutation::ModuleDirectiveDuplicate duplicate:
            match duplicate.kind:
                SelfhostModuleDirectiveKind::Target:
                    Result::Ok unit
                SelfhostModuleDirectiveKind::Entry:
                    Result::Err "expected duplicate target"
                SelfhostModuleDirectiveKind::Other:
                    Result::Err "expected duplicate target"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result::Err "expected duplicate target"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result::Err "expected duplicate target"

fn main %impure fn void i32 \void:
    let span1 %SelfhostSourceSpan source_span_new_unchecked 0 0 7
    let span2 %SelfhostSourceSpan source_span_new_unchecked 0 8 15
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
                                    let checks4 checks_push checks3 Result::Err "duplicate target was accepted"
                                    let shown checks_print_report checks4
                                    checks_exit_code shown
                        Result::Err _refutation:
                            let checks3 checks_push checks2 Result::Err "entry transition failed"
                            let shown checks_print_report checks3
                            checks_exit_code shown
                Result::Err _refutation:
                    let checks2 checks_push checks1 Result::Err "target transition failed"
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _refutation:
            let checks1 checks_push checks0 Result::Err "non-singleton transition failed"
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

fn check_open_empty_wasm %fn SelfhostRawBackendState Result unit str \state:
    match state:
        SelfhostRawBackendState::OpenEmpty open_block:
            match open_block.kind:
                SelfhostRawBackendKind::Wasm:
                    Result::Ok unit
                SelfhostRawBackendKind::LlvmIr:
                    Result::Err "expected wasm empty block"
        SelfhostRawBackendState::Normal:
            Result::Err "expected empty block"
        SelfhostRawBackendState::OpenReady _kind:
            Result::Err "expected empty block"

fn check_open_ready_wasm %fn SelfhostRawBackendState Result unit str \state:
    match state:
        SelfhostRawBackendState::OpenReady kind:
            match kind:
                SelfhostRawBackendKind::Wasm:
                    Result::Ok unit
                SelfhostRawBackendKind::LlvmIr:
                    Result::Err "expected wasm ready block"
        SelfhostRawBackendState::Normal:
            Result::Err "expected ready block"
        SelfhostRawBackendState::OpenEmpty _open_block:
            Result::Err "expected ready block"

fn check_normal %fn SelfhostRawBackendState Result unit str \state:
    match state:
        SelfhostRawBackendState::Normal:
            Result::Ok unit
        SelfhostRawBackendState::OpenEmpty _open_block:
            Result::Err "expected normal state"
        SelfhostRawBackendState::OpenReady _kind:
            Result::Err "expected normal state"

fn check_raw_text_refutation %fn SelfhostProofRefutation Result unit str \refutation:
    match refutation:
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result::Ok unit
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result::Err "expected text-without-block refutation"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result::Err "expected text-without-block refutation"

fn main %impure fn void i32 \void:
    let span %SelfhostSourceSpan source_span_new_unchecked 0 0 5
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
                                    let checks4 checks_push checks3 Result::Err "orphan raw text was accepted"
                                    let shown checks_print_report checks4
                                    checks_exit_code shown
                        Result::Err _refutation:
                            let checks3 checks_push checks2 Result::Err "stream end transition failed"
                            let shown checks_print_report checks3
                            checks_exit_code shown
                Result::Err _refutation:
                    let checks2 checks_push checks1 Result::Err "raw text transition failed"
                    let shown checks_print_report checks2
                    checks_exit_code shown
        Result::Err _refutation:
            let checks1 checks_push checks0 Result::Err "raw block transition failed"
            let shown checks_print_report checks1
            checks_exit_code shown
```

## module_declaration_header_requires_parser_evidence

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
    ##: [7] ok
    ##: [8] ok
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

fn check_header_proven %fn Result SelfhostModuleDeclarationHeader SelfhostProofRefutation Result unit str \result:
    match result:
        Result::Ok header:
            match header.kind:
                SelfhostModuleDeclarationKind::Function:
                    Result::Ok unit
                SelfhostModuleDeclarationKind::Struct:
                    Result::Err "expected function header proof"
                SelfhostModuleDeclarationKind::Enum:
                    Result::Err "expected function header proof"
                SelfhostModuleDeclarationKind::Trait:
                    Result::Err "expected function header proof"
                SelfhostModuleDeclarationKind::Impl:
                    Result::Err "expected function header proof"
        Result::Err _refutation:
            Result::Err "expected declaration header proof"

fn check_header_missing %fn Result SelfhostModuleDeclarationHeader SelfhostProofRefutation Result unit str \result:
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result::Ok unit
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::UnexpectedEvidence _issue:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::TypeKindMismatch _issue:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::OwnerTransitionInvalid _issue:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::BorrowAccessInvalid _issue:
                    Result::Err "expected missing declaration header"
                SelfhostProofRefutation::EffectBoundaryInvalid _issue:
                    Result::Err "expected missing declaration header"
        Result::Ok _header:
            Result::Err "missing declaration header was accepted"

fn check_header_invalid %fn Result SelfhostModuleDeclarationHeader SelfhostProofRefutation Result unit str \result:
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result::Ok unit
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::UnexpectedEvidence _issue:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::TypeKindMismatch _issue:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::OwnerTransitionInvalid _issue:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::BorrowAccessInvalid _issue:
                    Result::Err "expected invalid declaration header"
                SelfhostProofRefutation::EffectBoundaryInvalid _issue:
                    Result::Err "expected invalid declaration header"
        Result::Ok _header:
            Result::Err "invalid declaration header was accepted"

fn main %impure fn void i32 \void:
    let header_span %SelfhostSourceSpan source_span_new_unchecked 0 0 24
    let keyword_span %SelfhostSourceSpan source_span_new_unchecked 0 0 2
    let head_span %SelfhostSourceSpan source_span_new_unchecked 0 3 7
    let head %SelfhostModuleDeclarationHead selfhost_module_declaration_head_new SelfhostModuleDeclarationHeadKind::Name head_span
    let type_range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 2 4 source_span_new_unchecked 0 8 20
    let lambda_range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 6 2 source_span_new_unchecked 0 21 24
    let header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Function SelfhostModuleDeclarationVisibility::Private header_span keyword_span some head type_range lambda_range
    let valid_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl some header header_span
    let missing_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl none header_span
    let invalid_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Struct SelfhostModuleDeclarationVisibility::Private header_span keyword_span some head type_range lambda_range
    let invalid_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl some invalid_header header_span
    let missing_type_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Function SelfhostModuleDeclarationVisibility::Private header_span keyword_span some head selfhost_syntax_range_empty lambda_range
    let missing_type_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl some missing_type_header header_span
    let missing_lambda_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Function SelfhostModuleDeclarationVisibility::Private header_span keyword_span some head type_range selfhost_syntax_range_empty
    let missing_lambda_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl some missing_lambda_header header_span
    let outside_type_range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 20 1 source_span_new_unchecked 0 25 28
    let outside_type_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Function SelfhostModuleDeclarationVisibility::Private header_span keyword_span some head outside_type_range lambda_range
    let outside_type_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::FunctionDecl some outside_type_header header_span
    let struct_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Struct SelfhostModuleDeclarationVisibility::Private header_span keyword_span some head type_range lambda_range
    let struct_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::StructDecl some struct_header header_span
    let impl_header_span %SelfhostSourceSpan source_span_new_unchecked 0 0 22
    let impl_keyword_span %SelfhostSourceSpan source_span_new_unchecked 0 4 8
    let impl_head_span %SelfhostSourceSpan source_span_new_unchecked 0 9 13
    let impl_head %SelfhostModuleDeclarationHead selfhost_module_declaration_head_new SelfhostModuleDeclarationHeadKind::Name impl_head_span
    let public_impl_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Impl SelfhostModuleDeclarationVisibility::Public impl_header_span impl_keyword_span some impl_head selfhost_syntax_range_empty selfhost_syntax_range_empty
    let public_impl_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::ImplDecl some public_impl_header impl_header_span
    let impl_type_range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 3 2 source_span_new_unchecked 0 14 19
    let impl_lambda_range %SelfhostSyntaxRange selfhost_syntax_range_new_unchecked 5 1 source_span_new_unchecked 0 20 22
    let impl_range_header %SelfhostModuleDeclarationHeader selfhost_module_declaration_header_new SelfhostModuleDeclarationKind::Impl SelfhostModuleDeclarationVisibility::Private impl_header_span impl_keyword_span some impl_head impl_type_range impl_lambda_range
    let impl_range_fact %SelfhostModuleDeclarationFact selfhost_module_declaration_fact_new SelfhostModuleItemKind::ImplDecl some impl_range_header impl_header_span
    let checks0 checks_new
    let checks1 checks_push checks0 check_header_proven selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function valid_fact
    let checks2 checks_push checks1 check_header_missing selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function missing_fact
    let checks3 checks_push checks2 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function invalid_fact
    let checks4 checks_push checks3 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Impl public_impl_fact
    let checks5 checks_push checks4 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function missing_type_fact
    let checks6 checks_push checks5 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function missing_lambda_fact
    let checks7 checks_push checks6 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Function outside_type_fact
    let checks8 checks_push checks7 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Struct struct_fact
    let checks9 checks_push checks8 check_header_invalid selfhost_proof_module_declaration_header SelfhostModuleDeclarationKind::Impl impl_range_fact
    let shown checks_print_report checks9
    checks_exit_code shown
```
