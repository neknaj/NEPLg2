extern crate alloc;

use alloc::vec::Vec;

use super::initialized::ResourceCheckEngine;
use super::report::ResourceCheckDeferred;

pub(super) fn summary_check_engine<'a>(
    engine: &ResourceCheckEngine<'a>,
) -> ResourceCheckEngine<'a> {
    ResourceCheckEngine {
        function: engine.function,
        types: engine.types,
        raw_alias_summaries: engine.raw_alias_summaries,
        i32_scalar_summaries: engine.i32_scalar_summaries,
        raw_init_summaries: engine.raw_init_summaries,
        collection_slot_summaries: engine.collection_slot_summaries,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    }
}
