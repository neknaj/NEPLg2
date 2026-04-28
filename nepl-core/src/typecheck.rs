extern crate alloc;
#[cfg(not(target_os = "none"))]
extern crate std;

mod ascription;
mod assignment_apply;
mod binding_rules;
mod block_check;
mod call_reduction;
mod call_resolution;
mod context;
mod control_apply;
mod driver;
mod effect_check;
mod env;
mod field_access;
mod field_apply;
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
use context::BlockChecker;
use function_check::check_function;
use model::{AssignKind, FieldAccessorKind, FieldIdx, ScalarMatchKind, StackEntry};
use traits::TraitBoundRef;

pub use driver::{typecheck, TypeCheckResult};
