# NEPLg2 self-host effect proof

## effect_boundary_uses_generic_proof

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "neplg2/core/ty/effect" as *
#import "std/test" as *

fn check_pure_context <(Result<SelfhostEffectContext,SelfhostProofRefutation>)->Result<(),str>> (result):
    match result:
        Result::Ok context:
            match context:
                SelfhostEffectContext::PureContext:
                    Result<(),str>::Ok ()
                SelfhostEffectContext::ImpureContext:
                    Result<(),str>::Err "expected pure effect context"
                SelfhostEffectContext::UnsafeBoundary:
                    Result<(),str>::Err "expected pure effect context"
        Result::Err _refutation:
            Result<(),str>::Err "expected effect boundary proof"

fn check_unsafe_boundary <(Result<SelfhostEffectContext,SelfhostProofRefutation>)->Result<(),str>> (result):
    match result:
        Result::Ok context:
            match context:
                SelfhostEffectContext::UnsafeBoundary:
                    Result<(),str>::Ok ()
                SelfhostEffectContext::PureContext:
                    Result<(),str>::Err "expected unsafe boundary context"
                SelfhostEffectContext::ImpureContext:
                    Result<(),str>::Err "expected unsafe boundary context"
        Result::Err _refutation:
            Result<(),str>::Err "expected unsafe boundary proof"

fn check_impure_effect_rejected <(Result<SelfhostEffectContext,SelfhostProofRefutation>)->Result<(),str>> (result):
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::EffectBoundaryInvalid issue:
                    match issue.reason:
                        SelfhostEffectBoundaryError::ImpureEffectInPureContext:
                            Result<(),str>::Ok ()
                        SelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary:
                            Result<(),str>::Err "expected impure effect rejection"
                        SelfhostEffectBoundaryError::InternalAllocEscapeNotProven:
                            Result<(),str>::Err "expected impure effect rejection"
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::TypeKindMismatch _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
        Result::Ok _context:
            Result<(),str>::Err "impure effect was accepted in pure context"

fn check_escaping_alloc_rejected <(Result<SelfhostEffectContext,SelfhostProofRefutation>)->Result<(),str>> (result):
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::EffectBoundaryInvalid issue:
                    match issue.reason:
                        SelfhostEffectBoundaryError::InternalAllocEscapeNotProven:
                            Result<(),str>::Ok ()
                        SelfhostEffectBoundaryError::ImpureEffectInPureContext:
                            Result<(),str>::Err "expected escaping allocation rejection"
                        SelfhostEffectBoundaryError::UnsafeMemoryOutsideBoundary:
                            Result<(),str>::Err "expected escaping allocation rejection"
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::TypeKindMismatch _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result<(),str>::Err "expected effect boundary refutation"
        Result::Ok _context:
            Result<(),str>::Err "escaping allocation was accepted in pure context"

fn main <()*>i32> ():
    let span <SelfhostSourceSpan> source_span_new 0 0 4
    let pure_fact <SelfhostEffectObservationFact> selfhost_effect_observation_fact_new SelfhostEffectKind::Pure SelfhostEffectEscapeState::NotApplicable span
    let no_escape_alloc <SelfhostEffectObservationFact> selfhost_effect_observation_fact_new SelfhostEffectKind::InternalAlloc SelfhostEffectEscapeState::NoEscapeProven span
    let escaping_alloc <SelfhostEffectObservationFact> selfhost_effect_observation_fact_new SelfhostEffectKind::InternalAlloc SelfhostEffectEscapeState::MayEscape span
    let io_fact <SelfhostEffectObservationFact> selfhost_effect_observation_fact_new SelfhostEffectKind::ExternalIo SelfhostEffectEscapeState::NotApplicable span
    let unsafe_fact <SelfhostEffectObservationFact> selfhost_effect_observation_fact_new SelfhostEffectKind::UnsafeMemory SelfhostEffectEscapeState::NotApplicable span
    let checks0 checks_new
    let checks1 checks_push checks0 check_pure_context selfhost_proof_effect_allowed SelfhostEffectContext::PureContext pure_fact
    let checks2 checks_push checks1 check_pure_context selfhost_proof_effect_allowed SelfhostEffectContext::PureContext no_escape_alloc
    let checks3 checks_push checks2 check_impure_effect_rejected selfhost_proof_effect_allowed SelfhostEffectContext::PureContext io_fact
    let checks4 checks_push checks3 check_escaping_alloc_rejected selfhost_proof_effect_allowed SelfhostEffectContext::PureContext escaping_alloc
    let checks5 checks_push checks4 check_unsafe_boundary selfhost_proof_effect_allowed SelfhostEffectContext::UnsafeBoundary unsafe_fact
    let shown checks_print_report checks5
    checks_exit_code shown
```
