# NEPLg2 self-host trait proof

## trait_impl_coherence_uses_generic_proof

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
#import "neplg2/core/ty/trait_ref" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn check_relation_different_trait <(SelfhostTraitImplRelation)->Result<(),str>> (relation):
    match relation:
        SelfhostTraitImplRelation::DifferentTrait:
            Result<(),str>::Ok ()
        SelfhostTraitImplRelation::InvalidCandidate:
            Result<(),str>::Err "expected different trait relation"
        SelfhostTraitImplRelation::InvalidExisting:
            Result<(),str>::Err "expected different trait relation"
        SelfhostTraitImplRelation::SameTraitDifferentSelfType:
            Result<(),str>::Err "expected different trait relation"
        SelfhostTraitImplRelation::SameTraitSameSelfType:
            Result<(),str>::Err "expected different trait relation"

fn check_relation_same_trait_different_self <(SelfhostTraitImplRelation)->Result<(),str>> (relation):
    match relation:
        SelfhostTraitImplRelation::SameTraitDifferentSelfType:
            Result<(),str>::Ok ()
        SelfhostTraitImplRelation::InvalidCandidate:
            Result<(),str>::Err "expected same trait different self type relation"
        SelfhostTraitImplRelation::InvalidExisting:
            Result<(),str>::Err "expected same trait different self type relation"
        SelfhostTraitImplRelation::DifferentTrait:
            Result<(),str>::Err "expected same trait different self type relation"
        SelfhostTraitImplRelation::SameTraitSameSelfType:
            Result<(),str>::Err "expected same trait different self type relation"

fn check_non_overlap_relation <(Result<SelfhostTraitImplRelation,SelfhostProofRefutation>,SelfhostTraitImplRelation)->Result<(),str>> (result, expected):
    match result:
        Result::Ok relation:
            match expected:
                SelfhostTraitImplRelation::DifferentTrait:
                    check_relation_different_trait relation
                SelfhostTraitImplRelation::SameTraitDifferentSelfType:
                    check_relation_same_trait_different_self relation
                SelfhostTraitImplRelation::InvalidCandidate:
                    Result<(),str>::Err "invalid candidate should not prove non-overlap"
                SelfhostTraitImplRelation::InvalidExisting:
                    Result<(),str>::Err "invalid existing should not prove non-overlap"
                SelfhostTraitImplRelation::SameTraitSameSelfType:
                    Result<(),str>::Err "duplicate impl should not prove non-overlap"
        Result::Err _refutation:
            Result<(),str>::Err "expected trait impl non-overlap proof"

fn check_duplicate_issue <(SelfhostTraitImplCoherenceIssue)->Result<(),str>> (issue):
    match issue.reason:
        SelfhostTraitImplCoherenceError::DuplicateImpl:
            match issue.relation:
                SelfhostTraitImplRelation::SameTraitSameSelfType:
                    Result<(),str>::Ok ()
                SelfhostTraitImplRelation::InvalidCandidate:
                    Result<(),str>::Err "duplicate impl should preserve same self relation"
                SelfhostTraitImplRelation::InvalidExisting:
                    Result<(),str>::Err "duplicate impl should preserve same self relation"
                SelfhostTraitImplRelation::DifferentTrait:
                    Result<(),str>::Err "duplicate impl should preserve same self relation"
                SelfhostTraitImplRelation::SameTraitDifferentSelfType:
                    Result<(),str>::Err "duplicate impl should preserve same self relation"
        SelfhostTraitImplCoherenceError::InvalidCandidateKey:
            Result<(),str>::Err "expected duplicate impl reason"
        SelfhostTraitImplCoherenceError::InvalidExistingKey:
            Result<(),str>::Err "expected duplicate impl reason"

fn check_invalid_candidate_issue <(SelfhostTraitImplCoherenceIssue)->Result<(),str>> (issue):
    match issue.reason:
        SelfhostTraitImplCoherenceError::InvalidCandidateKey:
            match issue.relation:
                SelfhostTraitImplRelation::InvalidCandidate:
                    Result<(),str>::Ok ()
                SelfhostTraitImplRelation::InvalidExisting:
                    Result<(),str>::Err "invalid candidate should preserve invalid relation"
                SelfhostTraitImplRelation::DifferentTrait:
                    Result<(),str>::Err "invalid candidate should preserve invalid relation"
                SelfhostTraitImplRelation::SameTraitDifferentSelfType:
                    Result<(),str>::Err "invalid candidate should preserve invalid relation"
                SelfhostTraitImplRelation::SameTraitSameSelfType:
                    Result<(),str>::Err "invalid candidate should preserve invalid relation"
        SelfhostTraitImplCoherenceError::DuplicateImpl:
            Result<(),str>::Err "expected invalid candidate reason"
        SelfhostTraitImplCoherenceError::InvalidExistingKey:
            Result<(),str>::Err "expected invalid candidate reason"

fn check_coherence_refutation <(SelfhostProofRefutation,(SelfhostTraitImplCoherenceIssue)->Result<(),str>)->Result<(),str>> (refutation, checker):
    match refutation:
        SelfhostProofRefutation::TraitImplCoherenceInvalid issue:
            checker issue
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result<(),str>::Err "expected trait impl coherence refutation"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result<(),str>::Err "expected trait impl coherence refutation"

fn check_coherence_error <(Result<SelfhostTraitImplRelation,SelfhostProofRefutation>,(SelfhostTraitImplCoherenceIssue)->Result<(),str>)->Result<(),str>> (result, checker):
    match result:
        Result::Err refutation:
            check_coherence_refutation refutation checker
        Result::Ok _relation:
            Result<(),str>::Err "trait impl coherence error was accepted"

fn run_with_types <(SelfhostTypeArena,SelfhostTypeId,SelfhostTypeId)*>i32> (arena, i32_id, bool_id):
    let span <SelfhostSourceSpan> source_span_new 0 0 5
    let trait0 <SelfhostTraitId> selfhost_trait_id_new 0
    let trait1 <SelfhostTraitId> selfhost_trait_id_new 1
    let impl_trait0_i32 <SelfhostTraitImplKey> selfhost_trait_impl_key_new trait0 i32_id
    let impl_trait0_bool <SelfhostTraitImplKey> selfhost_trait_impl_key_new trait0 bool_id
    let impl_trait1_i32 <SelfhostTraitImplKey> selfhost_trait_impl_key_new trait1 i32_id
    let invalid_type <SelfhostTypeId> selfhost_type_id_new -1
    let impl_trait0_invalid <SelfhostTraitImplKey> selfhost_trait_impl_key_new trait0 invalid_type
    let fact_different_type <SelfhostTraitImplPairFact> selfhost_trait_impl_pair_fact_new &arena impl_trait0_bool impl_trait0_i32 span
    let fact_different_trait <SelfhostTraitImplPairFact> selfhost_trait_impl_pair_fact_new &arena impl_trait1_i32 impl_trait0_i32 span
    let fact_duplicate <SelfhostTraitImplPairFact> selfhost_trait_impl_pair_fact_new &arena impl_trait0_i32 impl_trait0_i32 span
    let fact_invalid_candidate <SelfhostTraitImplPairFact> selfhost_trait_impl_pair_fact_new &arena impl_trait0_invalid impl_trait0_i32 span
    let checks0 checks_new
    let checks1 checks_push checks0 check_non_overlap_relation (selfhost_proof_trait_impl_non_overlapping fact_different_type) SelfhostTraitImplRelation::SameTraitDifferentSelfType
    let checks2 checks_push checks1 check_non_overlap_relation (selfhost_proof_trait_impl_non_overlapping fact_different_trait) SelfhostTraitImplRelation::DifferentTrait
    let checks3 checks_push checks2 check_coherence_error (selfhost_proof_trait_impl_non_overlapping fact_duplicate) check_duplicate_issue
    let checks4 checks_push checks3 check_coherence_error (selfhost_proof_trait_impl_non_overlapping fact_invalid_candidate) check_invalid_candidate_issue
    let shown checks_print_report checks4
    let code <i32> checks_exit_code shown
    selfhost_type_arena_free arena
    code

fn main <()*>i32> ():
    match selfhost_type_arena_new:
        Result::Ok arena0:
            match selfhost_type_arena_add_primitive arena0 SelfhostPrimitiveTypeKind::I32:
                Result::Ok allocated_i32:
                    let i32_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &allocated_i32
                    let arena1 <SelfhostTypeArena> selfhost_type_arena_alloc_into_arena allocated_i32
                    match selfhost_type_arena_add_primitive arena1 SelfhostPrimitiveTypeKind::Bool:
                        Result::Ok allocated_bool:
                            let bool_id <SelfhostTypeId> selfhost_type_arena_alloc_type_id &allocated_bool
                            let arena2 <SelfhostTypeArena> selfhost_type_arena_alloc_into_arena allocated_bool
                            run_with_types arena2 i32_id bool_id
                        Result::Err _e:
                            1
                Result::Err _e:
                    1
        Result::Err _e:
            1
```
