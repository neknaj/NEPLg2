# NEPLg2 self-host type proof

## type_kind_compatibility_uses_generic_proof

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
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn check_i32_kind %fn Result SelfhostTypeKind SelfhostProofRefutation Result unit str \result:
    match result:
        Result::Ok kind:
            match kind:
                SelfhostTypeKind::I32:
                    Result<unit,str>::Ok unit
                SelfhostTypeKind::Error:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Unit:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Bool:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::I64:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::U8:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Char:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Str:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::F32:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::F64:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Never:
                    Result<unit,str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Function:
                    Result<unit,str>::Err "expected i32 kind proof"
        Result::Err _refutation:
            Result<unit,str>::Err "expected type kind proof"

fn check_bool_i32_mismatch %fn Result SelfhostTypeKind SelfhostProofRefutation Result unit str \result:
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::TypeKindMismatch issue:
                    match issue.expected:
                        SelfhostTypeKind::I32:
                            match issue.actual:
                                SelfhostTypeKind::Bool:
                                    Result<unit,str>::Ok unit
                                SelfhostTypeKind::Error:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Unit:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::I32:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::I64:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::U8:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Char:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Str:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::F32:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::F64:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Never:
                                    Result<unit,str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Function:
                                    Result<unit,str>::Err "expected bool actual kind"
                        SelfhostTypeKind::Error:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Unit:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Bool:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::I64:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::U8:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Char:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Str:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::F32:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::F64:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Never:
                            Result<unit,str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Function:
                            Result<unit,str>::Err "expected i32 expected kind"
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::UnexpectedEvidence _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::OwnerTransitionInvalid _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::BorrowAccessInvalid _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::EffectBoundaryInvalid _issue:
                    Result<unit,str>::Err "expected type kind mismatch"
        Result::Ok _kind:
            Result<unit,str>::Err "bool kind was accepted as i32"

fn main %impure fn unit i32 \unit:
    let span %SelfhostSourceSpan source_span_new 0 0 3
    let i32_fact %SelfhostTypeKindFact selfhost_type_kind_fact_new SelfhostTypeKind::I32 span
    let bool_fact %SelfhostTypeKindFact selfhost_type_kind_fact_new SelfhostTypeKind::Bool span
    let checks0 checks_new
    let checks1 checks_push checks0 check_i32_kind selfhost_proof_type_kind_compatible SelfhostTypeKind::I32 i32_fact
    let checks2 checks_push checks1 check_bool_i32_mismatch selfhost_proof_type_kind_compatible SelfhostTypeKind::I32 bool_fact
    let shown checks_print_report checks2
    checks_exit_code shown
```
