use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use std::path::PathBuf;

use crate::diagnostic::Severity;
use crate::loader::Loader;
use crate::resource::{lower_hir_module, PlaceProjection};
use crate::{BuildProfile, CompileTarget};

use super::super::summary::OwnerProjectionReturnOwner;
use super::{
    compute_owner_return_summaries_for_root_for_test,
    compute_owner_return_summaries_with_recomputations,
};

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

#[test]
fn deep_distinct_owner_variant_summary_preserves_path_conditioned_mapping() {
    let source = r#"
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/result" as *

enum LeafState:
    Ready <RegionToken<u8>>
    Empty

struct LeafOwner:
    state <LeafState>

struct InnerOwner:
    leaf <LeafOwner>

struct OuterOwner:
    inner <InnerOwner>

struct OwnerPair:
    source <OuterOwner>
    writer <OuterOwner>

struct Step:
    owner <OwnerPair>

struct NestedError:
    owner <OwnerPair>

enum StepError:
    Direct <OwnerPair>
    Nested <NestedError>
    SourceOnly <OuterOwner>

fn free_outer <(OuterOwner)*>()> (owner):
    let inner <InnerOwner> field::get owner "inner"
    let leaf <LeafOwner> field::get inner "leaf"
    match field::get leaf "state":
        LeafState::Ready region:
            match dealloc_region<u8> region:
                Result::Ok _:
                    ()
                Result::Err _:
                    #intrinsic "unreachable" <> ()
        LeafState::Empty:
            ()

fn route <(OwnerPair,bool,bool,bool)*>Result<Step,StepError>> (owner, ok_path, direct_error, source_only):
    if:
        ok_path
        then:
            Result<Step,StepError>::Ok Step owner
        else if:
            direct_error
            then:
                Result<Step,StepError>::Err StepError::Direct owner
            else if:
                source_only
                then:
                    let source <OuterOwner> field::get owner "source"
                    let writer <OuterOwner> field::get owner "writer"
                    free_outer writer
                    Result<Step,StepError>::Err StepError::SourceOnly source
                else:
                    Result<Step,StepError>::Err StepError::Nested NestedError owner

fn probe <(OwnerPair,bool,bool,bool,bool,bool)*>Result<Step,StepError>> (owner, completed, exhausted, lower_ok, direct_error, source_only):
    budget owner completed exhausted lower_ok direct_error source_only

fn budget <(OwnerPair,bool,bool,bool,bool,bool)*>Result<Step,StepError>> (owner, completed, exhausted, lower_ok, direct_error, source_only):
    if:
        completed
        then:
            Result<Step,StepError>::Ok Step owner
        else if:
            exhausted
            then:
                Result<Step,StepError>::Ok Step owner
            else:
                route owner lower_ok direct_error source_only
"#;
    let root = stdlib_root();
    let mut loader = Loader::new(root.clone());
    let loaded = loader
        .load_inline(
            root.join(
                "alloc/gui/font/registered_face/simple_glyph/indexed/owner_summary_test.nepl",
            ),
            source.to_string(),
        )
        .expect("load owner summary fixture");
    let checked = crate::typecheck::typecheck(
        &loaded.module,
        CompileTarget::Wasm,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    );
    assert!(
        checked
            .diagnostics
            .iter()
            .all(|diagnostic| !matches!(diagnostic.severity, Severity::Error)),
        "typecheck diagnostics: {:#?}",
        checked.diagnostics
    );
    let module = checked.module.expect("typechecked owner summary fixture");
    let resource = lower_hir_module(&module, &checked.types);
    let (summaries, _) =
        compute_owner_return_summaries_with_recomputations(&resource, &checked.types, None);
    let budget_index = resource
        .functions
        .iter()
        .position(|function| function.name.starts_with("budget__"))
        .expect("budget function index");
    let rooted_summaries =
        compute_owner_return_summaries_for_root_for_test(&resource, &checked.types, budget_index);
    assert_eq!(rooted_summaries.len(), 4, "unexpected rooted closure");
    assert!(rooted_summaries.iter().all(|summary| {
        summary.function.starts_with("route__")
            || summary.function.starts_with("budget__")
            || summary.function.starts_with("free_outer__")
            || summary.function.starts_with("dealloc_region__")
    }));

    for prefix in ["route__", "budget__"] {
        let full = summaries
            .iter()
            .find(|summary| summary.function.starts_with(prefix))
            .unwrap_or_else(|| panic!("missing full {prefix} summary"));
        let rooted = rooted_summaries
            .iter()
            .find(|summary| summary.function.starts_with(prefix))
            .unwrap_or_else(|| panic!("missing rooted {prefix} summary"));
        assert_eq!(rooted, full, "rooted summary must equal full fixed point");
    }
    let rooted_free = rooted_summaries
        .iter()
        .find(|summary| summary.function.starts_with("free_outer__"))
        .expect("rooted free_outer summary");
    let full_free = summaries
        .iter()
        .find(|summary| summary.function.starts_with("free_outer__"))
        .expect("full free_outer summary");
    assert_eq!(rooted_free, full_free);
    let rooted_dealloc = rooted_summaries
        .iter()
        .find(|summary| summary.function.starts_with("dealloc_region__"))
        .expect("rooted dealloc_region summary");
    let full_dealloc = summaries
        .iter()
        .find(|summary| summary.function.starts_with("dealloc_region__"))
        .expect("full dealloc_region summary");
    assert_eq!(rooted_dealloc, full_dealloc);

    for prefix in ["route__", "probe__", "budget__"] {
        let summary = summaries
            .iter()
            .find(|summary| summary.function.starts_with(prefix))
            .unwrap_or_else(|| panic!("missing {prefix} summary"));
        assert_eq!(
            summary.variant_projection_returns.len(),
            7,
            "{prefix} must contain only the seven path-conditioned returns: {summary:#?}"
        );
        assert!(
            summary.projection_returns.is_empty(),
            "{prefix} must not retain unconditional owner returns alongside variant returns: {summary:#?}"
        );
        assert!(summary.parameter_indices.is_empty());
        assert!(summary.parameter_sources.is_empty());
        assert!(summary.parameter_return_extents.is_empty());
        assert!(!summary.returns_fresh_owner);
        assert!(!summary.returns_maybe_owner);
        let parameter_returns = summary
            .variant_projection_returns
            .iter()
            .map(|entry| match &entry.owner {
                OwnerProjectionReturnOwner::Parameter { source, .. } => (entry, source),
                owner => panic!("{prefix} must not degrade a return source to {owner:#?}"),
            })
            .collect::<Vec<_>>();
        assert_eq!(parameter_returns.len(), 7);
        let sources = parameter_returns
            .iter()
            .map(|(_, source)| (*source).clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            sources.len(),
            2,
            "{prefix} must retain distinct authorities"
        );
        assert!(sources.iter().all(|source| source.parameter_index == 0));
        let source_fields = sources
            .iter()
            .filter_map(|source| match source.suffix.first() {
                Some(PlaceProjection::Field { index, .. }) => Some(*index),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(source_fields, BTreeSet::from([0, 1]));
        for source in &sources {
            let paths = parameter_returns
                .iter()
                .filter(|(_, found)| *found == source)
                .map(|(entry, _)| {
                    if entry.variant == "Ok" {
                        String::from("Ok")
                    } else if entry.variant == "Err" && entry.suffix.iter().any(|projection| {
                        matches!(
                            projection,
                            PlaceProjection::EnumPayload { variant } if variant.ends_with("Direct")
                        )
                    }) {
                        String::from("Direct")
                    } else if entry.variant == "Err" && entry.suffix.iter().any(|projection| {
                        matches!(
                            projection,
                            PlaceProjection::EnumPayload { variant } if variant.ends_with("Nested")
                        )
                    }) {
                        String::from("Nested")
                    } else if entry.variant == "Err" && entry.suffix.iter().any(|projection| {
                        matches!(
                            projection,
                            PlaceProjection::EnumPayload { variant } if variant.ends_with("SourceOnly")
                        )
                    }) {
                        String::from("SourceOnly")
                    } else {
                        String::from("Unknown")
                    }
                })
                .collect::<BTreeSet<_>>();
            let expected = if matches!(
                source.suffix.first(),
                Some(PlaceProjection::Field { index: 0, .. })
            ) {
                BTreeSet::from([
                    String::from("Direct"),
                    String::from("Nested"),
                    String::from("Ok"),
                    String::from("SourceOnly"),
                ])
            } else {
                BTreeSet::from([
                    String::from("Direct"),
                    String::from("Nested"),
                    String::from("Ok"),
                ])
            };
            assert_eq!(paths, expected, "{prefix} must preserve asymmetric paths");
        }
        let targets = parameter_returns
            .iter()
            .map(|(entry, _)| (entry.variant.clone(), entry.suffix.clone(), entry.ty))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            targets.len(),
            7,
            "{prefix} must not collapse return targets"
        );
    }
}
