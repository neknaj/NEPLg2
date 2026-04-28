extern crate alloc;
#[cfg(not(target_os = "none"))]
extern crate std;

mod ascription;
mod binding_rules;
mod block_check;
mod call_reduction;
mod call_resolution;
mod driver;
mod effect_check;
mod env;
mod field_access;
mod function_apply;
mod function_check;
mod hir_finalize;
mod match_check;
mod model;
mod name_lookup;
mod prefix_check;
mod signature;
mod syntax_helpers;
mod trait_check;
mod traits;
mod type_expr;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::*;
use crate::compiler::{BuildProfile, CompileTarget};
use crate::diagnostic::Diagnostic;
use crate::hir::*;
use crate::resolve::ImportResolution;
use crate::source_map::SourceMap;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

use env::Env;
use function_check::check_function;
use model::{
    AssignKind, EnumInfo, FieldAccessorKind, FieldIdx, ScalarMatchKind, StackEntry, StructInfo,
};
use traits::{ImplInfo, TraitBoundRef, TraitInfo};
use type_expr::{LabelEnv, StringTable};

pub use driver::{typecheck, TypeCheckResult};

// ---------------------------------------------------------------------
// Block checker
// ---------------------------------------------------------------------

struct BlockChecker<'a> {
    ctx: &'a mut TypeCtx,
    env: &'a mut Env,
    labels: &'a mut LabelEnv,
    string_table: &'a mut StringTable,
    diagnostics: Vec<Diagnostic>,
    pending_trait_bound_checks: Vec<(TraitBoundRef, TypeId, Span)>,
    current_effect: Effect,
    enums: &'a BTreeMap<String, EnumInfo>,
    structs: &'a BTreeMap<String, StructInfo>,
    instantiations: &'a mut BTreeMap<String, Vec<Vec<TypeId>>>, // new
    type_param_bounds: BTreeMap<TypeId, Vec<TraitBoundRef>>,
    import_resolution: &'a ImportResolution,
    traits: &'a BTreeMap<String, TraitInfo>,
    impls: &'a Vec<ImplInfo>,
    generated_functions: &'a mut Vec<HirFunction>,
    target: CompileTarget,
    profile: BuildProfile,
    source_map: Option<&'a SourceMap>,
}
