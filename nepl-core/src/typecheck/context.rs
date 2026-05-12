use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::hir::HirFunction;
use crate::resolve::ImportResolution;
use crate::source_map::SourceMap;
use crate::types::{TypeCtx, TypeId};

use super::env::Env;
use super::model::{EnumInfo, StructInfo};
use super::traits::{BoundEnv, ImplInfo, PendingTraitCheck, TraitInfo};
use super::type_expr::{LabelEnv, StringTable};
// ---------------------------------------------------------------------
// Block checker
// ---------------------------------------------------------------------

pub(super) struct BlockChecker<'a> {
    pub(super) ctx: &'a mut TypeCtx,
    pub(super) env: &'a mut Env,
    pub(super) labels: &'a mut LabelEnv,
    pub(super) string_table: &'a mut StringTable,
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) pending_trait_bound_checks: Vec<PendingTraitCheck>,
    pub(super) current_effect: Effect,
    pub(super) enums: &'a BTreeMap<String, EnumInfo>,
    pub(super) structs: &'a BTreeMap<String, StructInfo>,
    pub(super) instantiations: &'a mut BTreeMap<String, Vec<Vec<TypeId>>>,
    pub(super) type_param_bounds: BoundEnv,
    pub(super) import_resolution: &'a ImportResolution,
    pub(super) traits: &'a BTreeMap<String, TraitInfo>,
    pub(super) impls: &'a Vec<ImplInfo>,
    pub(super) generated_functions: &'a mut Vec<HirFunction>,
    pub(super) target: CompileTarget,
    pub(super) profile: BuildProfile,
    pub(super) source_map: Option<&'a SourceMap>,
}
