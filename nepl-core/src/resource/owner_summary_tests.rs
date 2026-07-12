use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use std::path::PathBuf;

use crate::diagnostic::Severity;
use crate::loader::Loader;
use crate::resource::{
    check_resource_owner_obligations, lower_hir_module, OwnerState, PlaceProjection,
    ResourceOwnerDiagnostic, ResourceOwnerOperation,
};
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

struct SourceOwner:
    items <OuterOwner>
    spans <OuterOwner>
    scalar_slots <OuterOwner>

struct WriterOwner:
    path_sink_scalars <OuterOwner>
    raster_mask_scalars <OuterOwner>

struct OwnerPair:
    source <SourceOwner>
    writer <WriterOwner>

struct Step:
    owner <OwnerPair>

struct NestedError:
    owner <OwnerPair>

struct ReadRetainedError:
    retained <OwnerPair>
    read_kind <i32>

enum StepError:
    Direct <OwnerPair>
    Nested <NestedError>
    ReadFailed <ReadRetainedError>
    SourceOnly <SourceOwner>

struct LowerStep:
    writer <WriterOwner>

struct LowerError:
    kind <i32>
    writer <WriterOwner>
    rejected_value <i32>
    capacity_error <i32>
    rejected_scalar <i32>
    storage_error <i32>

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

fn free_writer <(WriterOwner)*>()> (owner):
    free_outer field::get owner "path_sink_scalars"
    free_outer field::get owner "raster_mask_scalars"

fn read_source <(&SourceOwner,bool)->Result<i32,i32>> (source, ok_path):
    if ok_path then Result<i32,i32>::Ok 0 else Result<i32,i32>::Err 1

fn lower_scalar <(WriterOwner,bool,i32)*>Result<LowerStep,LowerError>> (writer, ok_path, scalar):
    if:
        ok_path
        then Result<LowerStep,LowerError>::Ok LowerStep writer
        else Result<LowerStep,LowerError>::Err LowerError 1 writer scalar 0 scalar 0

fn lower_push_three <(WriterOwner,bool,bool,bool)*>Result<LowerStep,LowerError>> (writer, first_ok, second_ok, third_ok):
    match lower_scalar writer first_ok 0:
        Result::Err lower: Result::Err lower
        Result::Ok first:
            let first_writer <WriterOwner> field::get first "writer"
            match lower_scalar first_writer second_ok 1:
                Result::Err lower: Result::Err lower
                Result::Ok second:
                    let second_writer <WriterOwner> field::get second "writer"
                    lower_scalar second_writer third_ok 2

fn lower_push <(WriterOwner,bool,bool,bool,bool,bool)*>Result<LowerStep,LowerError>> (writer, first_ok, second_ok, third_ok, fourth_ok, fifth_ok):
    match lower_push_three writer first_ok second_ok third_ok:
        Result::Err lower: Result::Err lower
        Result::Ok third:
            let third_writer <WriterOwner> field::get third "writer"
            match lower_scalar third_writer fourth_ok 3:
                Result::Err lower: Result::Err lower
                Result::Ok fourth:
                    let fourth_writer <WriterOwner> field::get fourth "writer"
                    lower_scalar fourth_writer fifth_ok 4

fn route <(OwnerPair,bool,bool,bool,bool,bool,bool,bool,bool)*>Result<Step,StepError>> (owner, read_ok, first_ok, second_ok, third_ok, fourth_ok, fifth_ok, direct_error, nested_error):
    let source <SourceOwner> field::get owner "source"
    let writer <WriterOwner> field::get owner "writer"
    match read_source &source read_ok:
        Result::Err read_kind:
            Result<Step,StepError>::Err StepError::ReadFailed ReadRetainedError OwnerPair source writer read_kind
        Result::Ok observed:
            if:
                direct_error
                then:
                    Result<Step,StepError>::Err StepError::Direct OwnerPair source writer
                else if:
                    nested_error
                    then:
                        Result<Step,StepError>::Err StepError::Nested NestedError OwnerPair source writer
                    else match lower_push writer first_ok second_ok third_ok fourth_ok fifth_ok:
                        Result::Ok lower:
                            let next_writer <WriterOwner> field::get lower "writer"
                            Result<Step,StepError>::Ok Step OwnerPair source next_writer
                        Result::Err lower:
                            let lower_kind <i32> *field::get_ref &lower "kind"
                            let rejected_value <i32> *field::get_ref &lower "rejected_value"
                            let capacity_error <i32> *field::get_ref &lower "capacity_error"
                            let rejected_scalar <i32> *field::get_ref &lower "rejected_scalar"
                            let storage_error <i32> *field::get_ref &lower "storage_error"
                            let failed_writer <WriterOwner> field::get lower "writer"
                            free_writer failed_writer
                            Result<Step,StepError>::Err StepError::SourceOnly source

fn take_read_owner <(ReadRetainedError)->OwnerPair> (error):
    field::get error "retained"

fn double_take_read_owner <(ReadRetainedError)*>OwnerPair> (error):
    let _first <OwnerPair> take_read_owner error
    take_read_owner error

fn retry_read <(StepError,bool,bool,bool,bool,bool,bool,bool)*>Result<Step,StepError>> (error, first_ok, second_ok, third_ok, fourth_ok, fifth_ok, direct_error, nested_error):
    match error:
        StepError::ReadFailed read_error:
            let owner <OwnerPair> take_read_owner read_error
            route owner true first_ok second_ok third_ok fourth_ok fifth_ok direct_error nested_error
        StepError::Direct owner:
            Result<Step,StepError>::Err StepError::Direct owner
        StepError::Nested nested:
            Result<Step,StepError>::Err StepError::Nested nested
        StepError::SourceOnly source:
            Result<Step,StepError>::Err StepError::SourceOnly source

fn attempt <(OwnerPair,bool,bool,bool,bool,bool,bool,bool)*>Result<Step,StepError>> (owner, first_ok, second_ok, third_ok, fourth_ok, fifth_ok, direct_error, nested_error):
    match route owner false first_ok second_ok third_ok fourth_ok fifth_ok direct_error nested_error:
        Result::Ok step:
            Result<Step,StepError>::Ok step
        Result::Err error:
            retry_read error first_ok second_ok third_ok fourth_ok fifth_ok direct_error nested_error

fn probe <(OwnerPair,bool,bool,bool,bool,bool,bool,bool,bool,bool,bool)*>Result<Step,StepError>> (owner, completed, exhausted, read_ok, first_ok, second_ok, third_ok, fourth_ok, fifth_ok, direct_error, nested_error):
    budget owner completed exhausted read_ok first_ok second_ok third_ok fourth_ok fifth_ok direct_error nested_error

fn budget <(OwnerPair,bool,bool,bool,bool,bool,bool,bool,bool,bool,bool)*>Result<Step,StepError>> (owner, completed, exhausted, read_ok, first_ok, second_ok, third_ok, fourth_ok, fifth_ok, direct_error, nested_error):
    if:
        completed
        then:
            Result<Step,StepError>::Ok Step owner
        else if:
            exhausted
            then:
                Result<Step,StepError>::Ok Step owner
            else:
                route owner read_ok first_ok second_ok third_ok fourth_ok fifth_ok direct_error nested_error
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
    let owner_report = check_resource_owner_obligations(&resource, &checked.types);
    assert!(
        owner_report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                operation: ResourceOwnerOperation::Read,
                state: OwnerState::Moved,
                ..
            } if function.starts_with("double_take_read_owner__")
        )),
        "double take must remain a genuine use-after-move: {:#?}",
        owner_report.diagnostics
    );
    let (summaries, _) =
        compute_owner_return_summaries_with_recomputations(&resource, &checked.types, None);
    let budget_index = resource
        .functions
        .iter()
        .position(|function| function.name.starts_with("budget__"))
        .expect("budget function index");
    let rooted_summaries =
        compute_owner_return_summaries_for_root_for_test(&resource, &checked.types, budget_index);
    assert_eq!(rooted_summaries.len(), 8, "unexpected rooted closure");
    assert!(rooted_summaries.iter().all(|summary| {
        summary.function.starts_with("route__")
            || summary.function.starts_with("budget__")
            || summary.function.starts_with("lower_push__")
            || summary.function.starts_with("lower_push_three__")
            || summary.function.starts_with("lower_scalar__")
            || summary.function.starts_with("read_source__")
            || summary.function.starts_with("free_writer__")
            || summary.function.starts_with("free_outer__")
            || summary.function.starts_with("dealloc_region__")
    }));
    assert!(
        rooted_summaries
            .iter()
            .any(|summary| summary.function.starts_with("lower_push__")),
        "writer-only lower Result helper must belong to the rooted owner-summary closure"
    );
    assert!(
        rooted_summaries
            .iter()
            .any(|summary| summary.function.starts_with("lower_push_three__")),
        "three-scalar helper must belong to the rooted owner-summary closure"
    );
    assert!(
        rooted_summaries
            .iter()
            .any(|summary| summary.function.starts_with("lower_scalar__")),
        "sequential scalar Result helper must belong to the rooted owner-summary closure"
    );
    assert!(
        rooted_summaries
            .iter()
            .all(|summary| !summary.function.starts_with("read_source__")),
        "owner-free source read must not enter the owner-summary closure"
    );
    for rooted in &rooted_summaries {
        let full = summaries
            .iter()
            .find(|summary| summary.function == rooted.function)
            .unwrap_or_else(|| panic!("missing full {} summary", rooted.function));
        assert_eq!(rooted, full, "rooted summary must equal full fixed point");
    }

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

    let take_read = summaries
        .iter()
        .find(|summary| summary.function.starts_with("take_read_owner__"))
        .expect("missing take_read_owner summary");
    assert_eq!(take_read.projection_returns.len(), 5, "{take_read:#?}");
    assert!(take_read.variant_projection_returns.is_empty());
    assert!(take_read.parameter_indices.is_empty());
    assert!(take_read.parameter_sources.is_empty());
    assert!(!take_read.returns_fresh_owner);
    assert!(!take_read.returns_maybe_owner);
    let take_returns = take_read
        .projection_returns
        .iter()
        .map(|entry| match entry.parameter_sources.as_slice() {
            [source] => (source, entry),
            sources => panic!("take_read_owner must have one exact source: {sources:#?}"),
        })
        .collect::<Vec<_>>();
    assert!(take_returns
        .iter()
        .all(|(source, _)| source.parameter_index == 0));
    assert_eq!(
        take_returns
            .iter()
            .map(|(source, _)| (*source).clone())
            .collect::<BTreeSet<_>>()
            .len(),
        5
    );
    assert_eq!(
        take_returns
            .iter()
            .map(|(_, entry)| (entry.suffix.clone(), entry.ty))
            .collect::<BTreeSet<_>>()
            .len(),
        5
    );
    assert!(take_returns.iter().all(|(source, entry)| {
        matches!(
            source.suffix.first(),
            Some(PlaceProjection::Field { index: 0, .. })
        ) && source.suffix[1..] == entry.suffix
            && source.ty == entry.ty
    }));
    let retry_read = summaries
        .iter()
        .find(|summary| summary.function.starts_with("retry_read__"))
        .expect("missing retry_read summary");
    assert_eq!(
        retry_read.variant_projection_returns.len(),
        36,
        "{retry_read:#?}"
    );
    assert_eq!(retry_read.projection_returns.len(), 5);
    assert!(retry_read.projection_returns.iter().all(|projection| {
        projection.returns_fresh_owner
            && projection.parameter_indices.is_empty()
            && projection.parameter_sources.is_empty()
            && matches!(
                projection.suffix.as_slice(),
                [PlaceProjection::EnumPayload { variant }, PlaceProjection::EnumPayload { variant: error_variant }, ..]
                    if variant == "Err" && error_variant == "Direct"
            )
    }));
    assert!(retry_read.parameter_indices.is_empty());
    assert!(retry_read.parameter_sources.is_empty());
    assert!(!retry_read.returns_fresh_owner);
    assert!(!retry_read.returns_maybe_owner);
    let retry_returns = retry_read
        .variant_projection_returns
        .iter()
        .map(|entry| match &entry.owner {
            OwnerProjectionReturnOwner::Parameter { source, .. } => (source, entry),
            owner => panic!("retry_read must preserve an exact parameter source: {owner:#?}"),
        })
        .collect::<Vec<_>>();
    assert!(retry_returns
        .iter()
        .all(|(source, _)| source.parameter_index == 0));
    assert_eq!(
        retry_returns
            .iter()
            .map(|(source, _)| (*source).clone())
            .collect::<BTreeSet<_>>()
            .len(),
        18
    );
    assert_eq!(
        retry_returns
            .iter()
            .map(|(source, entry)| ((*source).clone(), entry.suffix.clone(), entry.ty))
            .collect::<BTreeSet<_>>()
            .len(),
        36
    );
    let mut retry_targets_by_source = BTreeMap::<_, BTreeSet<_>>::new();
    for (source, entry) in &retry_returns {
        let source_variant = source
            .suffix
            .iter()
            .find_map(|projection| match projection {
                PlaceProjection::EnumPayload { variant } => Some(variant.as_str()),
                _ => None,
            });
        let target_variants = entry
            .suffix
            .iter()
            .filter_map(|projection| match projection {
                PlaceProjection::EnumPayload { variant } => Some(variant.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>();
        retry_targets_by_source
            .entry((*source).clone())
            .or_default()
            .insert(
                target_variants
                    .iter()
                    .map(|variant| (*variant).to_string())
                    .collect(),
            );
        match source_variant {
            Some("Direct") => assert_eq!(target_variants, ["Err", "Direct", "Ready"]),
            Some("Nested") => assert_eq!(target_variants, ["Err", "Nested", "Ready"]),
            Some("SourceOnly") => {
                assert_eq!(target_variants, ["Err", "SourceOnly", "Ready"])
            }
            Some("ReadFailed") => assert!(matches!(
                target_variants.as_slice(),
                ["Ok", "Ready"]
                    | ["Err", "Direct", "Ready"]
                    | ["Err", "Nested", "Ready"]
                    | ["Err", "ReadFailed", "Ready"]
                    | ["Err", "SourceOnly", "Ready"]
            )),
            variant => panic!("unexpected retry source variant {variant:?}"),
        }
    }
    for (source, targets) in retry_targets_by_source {
        let source_variant = source
            .suffix
            .iter()
            .find_map(|projection| match projection {
                PlaceProjection::EnumPayload { variant } => Some(variant.as_str()),
                _ => None,
            });
        let expected = match source_variant {
            Some("Direct") => BTreeSet::from([vec!["Err", "Direct", "Ready"]]),
            Some("Nested") => BTreeSet::from([vec!["Err", "Nested", "Ready"]]),
            Some("SourceOnly") => BTreeSet::from([vec!["Err", "SourceOnly", "Ready"]]),
            Some("ReadFailed") => {
                let owner_pair_field = source
                    .suffix
                    .iter()
                    .filter_map(|projection| match projection {
                        PlaceProjection::Field { index, .. } => Some(*index),
                        _ => None,
                    })
                    .nth(1)
                    .expect("ReadFailed retained OwnerPair field");
                if owner_pair_field == 0 {
                    BTreeSet::from([
                        vec!["Ok", "Ready"],
                        vec!["Err", "Direct", "Ready"],
                        vec!["Err", "Nested", "Ready"],
                        vec!["Err", "ReadFailed", "Ready"],
                        vec!["Err", "SourceOnly", "Ready"],
                    ])
                } else {
                    assert_eq!(owner_pair_field, 1);
                    BTreeSet::from([
                        vec!["Ok", "Ready"],
                        vec!["Err", "Direct", "Ready"],
                        vec!["Err", "Nested", "Ready"],
                        vec!["Err", "ReadFailed", "Ready"],
                    ])
                }
            }
            variant => panic!("unexpected retry source variant {variant:?}"),
        }
        .into_iter()
        .map(|variants| variants.into_iter().map(String::from).collect::<Vec<_>>())
        .collect::<BTreeSet<_>>();
        assert_eq!(
            targets, expected,
            "incomplete retry targets for {source:#?}"
        );
    }

    let attempt_index = resource
        .functions
        .iter()
        .position(|function| function.name.starts_with("attempt__"))
        .expect("attempt function index");
    let rooted_attempt =
        compute_owner_return_summaries_for_root_for_test(&resource, &checked.types, attempt_index);
    assert!(rooted_attempt
        .iter()
        .any(|summary| summary.function.starts_with("retry_read__")));
    assert!(rooted_attempt
        .iter()
        .any(|summary| summary.function.starts_with("take_read_owner__")));
    for rooted in &rooted_attempt {
        let full = summaries
            .iter()
            .find(|summary| summary.function == rooted.function)
            .unwrap_or_else(|| panic!("missing full {} summary", rooted.function));
        assert_eq!(rooted, full, "attempt closure must equal full fixed point");
    }

    for prefix in ["lower_scalar__", "lower_push_three__", "lower_push__"] {
        let summary = summaries
            .iter()
            .find(|summary| summary.function.starts_with(prefix))
            .unwrap_or_else(|| panic!("missing {prefix} summary"));
        assert_eq!(
            summary.variant_projection_returns.len(),
            4,
            "{prefix} must return two writer authorities through Ok and Err: {summary:#?}"
        );
        assert!(summary.projection_returns.is_empty());
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
                owner => panic!("{prefix} must not degrade a writer source to {owner:#?}"),
            })
            .collect::<Vec<_>>();
        let sources = parameter_returns
            .iter()
            .map(|(_, source)| (*source).clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(sources.len(), 2, "{prefix} must retain both writer leaves");
        for source in &sources {
            let returns = parameter_returns
                .iter()
                .filter(|(_, found)| *found == source)
                .collect::<Vec<_>>();
            assert_eq!(returns.len(), 2);
            assert_eq!(
                returns
                    .iter()
                    .map(|(entry, _)| entry.variant.clone())
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from([String::from("Err"), String::from("Ok")])
            );
            assert!(returns
                .iter()
                .all(|(entry, _)| entry.suffix.ends_with(&source.suffix)));
        }
        assert_eq!(
            parameter_returns
                .iter()
                .map(|(entry, _)| (entry.variant.clone(), entry.suffix.clone(), entry.ty))
                .collect::<BTreeSet<_>>()
                .len(),
            4,
            "{prefix} must not collapse writer return targets"
        );
    }

    for prefix in ["route__", "probe__", "budget__"] {
        let summary = summaries
            .iter()
            .find(|summary| summary.function.starts_with(prefix))
            .unwrap_or_else(|| panic!("missing {prefix} summary"));
        assert_eq!(
            summary.variant_projection_returns.len(),
            23,
            "{prefix} must contain only the twenty-three path-conditioned returns: {summary:#?}"
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
        assert_eq!(parameter_returns.len(), 23);
        let sources = parameter_returns
            .iter()
            .map(|(_, source)| (*source).clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            sources.len(),
            5,
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
        assert_eq!(
            sources
                .iter()
                .filter(|source| matches!(
                    source.suffix.first(),
                    Some(PlaceProjection::Field { index: 0, .. })
                ))
                .count(),
            3,
            "{prefix} must retain three source authorities"
        );
        assert_eq!(
            sources
                .iter()
                .filter(|source| matches!(
                    source.suffix.first(),
                    Some(PlaceProjection::Field { index: 1, .. })
                ))
                .count(),
            2,
            "{prefix} must retain two writer authorities"
        );
        for source in &sources {
            let returns_for_source = parameter_returns
                .iter()
                .filter(|(_, found)| *found == source)
                .collect::<Vec<_>>();
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
                            PlaceProjection::EnumPayload { variant } if variant.ends_with("ReadFailed")
                        )
                    }) {
                        String::from("ReadFailed")
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
                    String::from("ReadFailed"),
                    String::from("SourceOnly"),
                ])
            } else {
                BTreeSet::from([
                    String::from("Direct"),
                    String::from("Nested"),
                    String::from("Ok"),
                    String::from("ReadFailed"),
                ])
            };
            assert_eq!(paths, expected, "{prefix} must preserve asymmetric paths");
            assert_eq!(
                returns_for_source.len(),
                if matches!(
                    source.suffix.first(),
                    Some(PlaceProjection::Field { index: 0, .. })
                ) {
                    5
                } else {
                    4
                },
                "{prefix} must preserve the mapping count for each authority"
            );
            for (entry, _) in returns_for_source {
                let source_only = entry.suffix.iter().any(|projection| {
                    matches!(
                        projection,
                        PlaceProjection::EnumPayload { variant }
                            if variant.ends_with("SourceOnly")
                    )
                });
                let expected_suffix = if source_only {
                    &source.suffix[1..]
                } else {
                    source.suffix.as_slice()
                };
                assert!(
                    entry.suffix.ends_with(expected_suffix),
                    "{prefix} must map each source authority to its matching return leaf: source={source:#?}, entry={entry:#?}"
                );
            }
        }
        let targets = parameter_returns
            .iter()
            .map(|(entry, _)| (entry.variant.clone(), entry.suffix.clone(), entry.ty))
            .collect::<BTreeSet<_>>();
        assert_eq!(
            targets.len(),
            23,
            "{prefix} must not collapse return targets"
        );
    }
}
