use nepl_core::diagnostic::Severity;
use nepl_core::hir::HirModule;
use nepl_core::loader::Loader;
use nepl_core::resource::{
    check_resource_initialized_moves, lower_hir_module, CollectionSlotLifecycleRefutation,
    CollectionSlotState, ResourceCheckDiagnostic,
};
use nepl_core::types::TypeCtx;
use nepl_core::{BuildProfile, CompileTarget};
use std::path::PathBuf;

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

fn typecheck_stdlib_source(source: &str, relative_path: &str) -> (HirModule, TypeCtx) {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline(stdlib_root().join(relative_path), source.to_string())
        .expect("load stdlib source with canonical source capabilities");
    let checked = nepl_core::typecheck::typecheck(
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
    (checked.module.expect("typechecked module"), checked.types)
}

#[test]
fn source_loop_drop_traversal_summary_cleans_caller_initialized_range() {
    let source = full_range_drop_traversal_source("0", "add i 1");
    let (module, types) = typecheck_stdlib_source(
        &source,
        "alloc/collections/vec/source_full_range_drop_traversal.nepl",
    );
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);

    assert!(
        report.diagnostics.is_empty(),
        "source-level loop cleanup must produce a full initialized-range traversal proof that caller storage release can consume: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
    assert!(
        report
            .functions
            .iter()
            .find(|function| function.name.starts_with("caller__"))
            .is_some_and(|function| {
                function
                    .final_collection_slots
                    .iter()
                    .all(|entry| !matches!(entry.state, CollectionSlotState::Initialized(_)))
            }),
        "caller must not retain initialized collection slots after summary replay: {:#?}",
        report.functions
    );
}

#[test]
fn source_loop_drop_traversal_rejects_non_zero_start_as_full_range_proof() {
    let source = full_range_drop_traversal_source("1", "add i 1");
    let (module, types) = typecheck_stdlib_source(
        &source,
        "alloc/collections/vec/source_incomplete_drop_traversal_start.nepl",
    );
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let owned_ty = types.lookup_named("LocalOwner").expect("LocalOwner type");

    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CollectionSlotRefuted {
                function,
                reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { slot_ty },
                ..
            } if function.starts_with("caller__") && *slot_ty == owned_ty
        )),
        "loop starting at 1 must not certify that every initialized slot was dropped: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn source_loop_drop_traversal_rejects_step_two_as_full_range_proof() {
    let source = full_range_drop_traversal_source("0", "add i 2");
    let (module, types) = typecheck_stdlib_source(
        &source,
        "alloc/collections/vec/source_incomplete_drop_traversal_step.nepl",
    );
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let owned_ty = types.lookup_named("LocalOwner").expect("LocalOwner type");

    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CollectionSlotRefuted {
                function,
                reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { slot_ty },
                ..
            } if function.starts_with("caller__") && *slot_ty == owned_ty
        )),
        "loop incrementing by 2 must not certify that every initialized slot was dropped: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn source_loop_drop_traversal_rejects_after_witness_raw_load_as_full_range_proof() {
    let source = full_range_drop_traversal_source_with_after_drop(
        "0",
        "add i 1",
        "            let again <LocalOwner> load<LocalOwner> raw\n",
    );
    let (module, types) = typecheck_stdlib_source(
        &source,
        "alloc/collections/vec/source_after_witness_raw_load.nepl",
    );
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let owned_ty = types.lookup_named("LocalOwner").expect("LocalOwner type");

    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CollectionSlotRefuted {
                function,
                reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { slot_ty },
                ..
            } if function.starts_with("caller__") && *slot_ty == owned_ty
        )),
        "an extra source-level raw load after the witness drop must not certify the full initialized range: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

fn full_range_drop_traversal_source(start_index: &str, increment_expr: &str) -> String {
    full_range_drop_traversal_source_with_after_drop(start_index, increment_expr, "")
}

fn full_range_drop_traversal_source_with_after_drop(
    start_index: &str,
    increment_expr: &str,
    after_drop: &str,
) -> String {
    format!(
        r#"
#indent 4
#target wasm
#no_prelude

#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as allocator
#import "core/mem/internal" as *
#import "core/mem/raw" as *
#import "core/mem/types" as *
#import "core/traits/drop" as *

struct LocalOwner:
    value <i32>

impl Drop for LocalOwner:
    fn drop <(&LocalOwner)*>()> (_self):
        ()

fn cleanup_all <(&RegionToken<LocalOwner>,i32)*>i32> (storage, initialized_len):
    let data <MemPtr<LocalOwner>> region_ptr storage
    let mut i <i32> {start_index}
    while lt i initialized_len:
        do:
            let byte_off <i32> mul i size_of<LocalOwner>
            let slot <MemPtr<LocalOwner>> mem_ptr_add<LocalOwner> data byte_off
            let raw <i32> mem_ptr_addr slot
            let loaded <LocalOwner> load<LocalOwner> raw
            Drop::drop &loaded
{after_drop}            set i {increment_expr}
    #intrinsic "collection_slot_drop_traversal" <LocalOwner> (storage, initialized_len)
    0

fn caller <(&RegionToken<LocalOwner>,LocalOwner,LocalOwner)*>i32> (storage, first, second):
    let data <MemPtr<LocalOwner>> region_ptr storage
    let raw0 <i32> mem_ptr_addr data
    let slot1 <MemPtr<LocalOwner>> mem_ptr_add<LocalOwner> data size_of<LocalOwner>
    let raw1 <i32> mem_ptr_addr slot1
    store<LocalOwner> raw0 first
    #intrinsic "collection_slot_initialize_empty" <LocalOwner> (storage, 0)
    store<LocalOwner> raw1 second
    #intrinsic "collection_slot_initialize_empty" <LocalOwner> (storage, size_of<LocalOwner>)
    cleanup_all storage 2
    let total_size <i32> region_size storage
    allocator::dealloc_raw raw0 total_size
    #intrinsic "collection_slot_storage_dealloc" <> (storage)
    0
"#
    )
}
