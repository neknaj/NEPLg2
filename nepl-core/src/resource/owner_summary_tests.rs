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
#import "core/mem" as *
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

fn route <(OwnerPair,bool,bool)*>Result<Step,StepError>> (owner, ok_path, direct_error):
    if:
        ok_path
        then:
            Result<Step,StepError>::Ok Step owner
        else if:
            direct_error
            then:
                Result<Step,StepError>::Err StepError::Direct owner
            else:
                Result<Step,StepError>::Err StepError::Nested NestedError owner

fn probe <(OwnerPair,bool,bool)*>Result<Step,StepError>> (owner, ok_path, direct_error):
    route owner ok_path direct_error
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
    let probe_index = resource
        .functions
        .iter()
        .position(|function| function.name.starts_with("probe__"))
        .expect("probe function index");
    let rooted_summaries =
        compute_owner_return_summaries_for_root_for_test(&resource, &checked.types, probe_index);
    assert_eq!(
        rooted_summaries.len(),
        2,
        "rooted fixture must exclude summaries outside probe -> route closure: {rooted_summaries:#?}"
    );
    assert!(rooted_summaries.iter().all(|summary| {
        summary.function.starts_with("route__") || summary.function.starts_with("probe__")
    }));

    for prefix in ["route__", "probe__"] {
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

    for prefix in ["route__", "probe__"] {
        let summary = summaries
            .iter()
            .find(|summary| summary.function.starts_with(prefix))
            .unwrap_or_else(|| panic!("missing {prefix} summary"));
        assert_eq!(
            summary.variant_projection_returns.len(),
            6,
            "{prefix} must contain only the six path-conditioned returns: {summary:#?}"
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
        assert_eq!(
            parameter_returns.len(),
            6,
            "{prefix} must retain two sources across three exclusive return paths: {summary:#?}"
        );
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
                    } else {
                        String::from("Unknown")
                    }
                })
                .collect::<BTreeSet<_>>();
            assert_eq!(
                paths,
                BTreeSet::from([
                    String::from("Direct"),
                    String::from("Nested"),
                    String::from("Ok"),
                ]),
                "{prefix} must map each source once to every exclusive path"
            );
        }
        let targets = parameter_returns
            .iter()
            .map(|(entry, _)| (entry.variant.clone(), entry.suffix.clone(), entry.ty))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            targets.len(),
            6,
            "{prefix} must not collapse return targets"
        );
    }
}
