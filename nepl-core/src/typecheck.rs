extern crate alloc;
#[cfg(not(target_os = "none"))]
extern crate std;

mod ascription;
mod assignment_apply;
mod binding_rules;
mod block_check;
mod call_binding_lookup;
mod call_pipe;
mod call_reduction;
mod call_resolution;
mod compiler_memory_type;
mod constructor_apply;
mod context;
mod control_apply;
mod control_special;
mod copy_capability;
mod diagnostics;
mod driver;
mod driver_entry;
mod driver_span;
mod effect_check;
mod env;
mod extern_import;
mod field_access;
mod field_apply;
mod function_apply;
mod function_check;
mod generic_call_constraints;
mod hir_finalize;
mod indirect_apply;
mod match_check;
mod memo_call;
mod model;
mod name_lookup;
mod overload_candidate;
mod overload_narrowing;
mod overload_selection;
mod prefix_check;
mod public_signature;
mod public_surface;
mod selected_call_apply;
mod signature;
mod struct_shape;
mod syntax_helpers;
mod trait_bound_apply;
mod trait_call_apply;
mod trait_check;
mod traits;
mod type_argument_inference;
mod type_expectation;
mod type_expr;
use crate::intrinsic_kinds::{
    CoreIntrinsicKind, CoreIntrinsicResultKind, FieldAccessorKind, ScalarIntrinsicKind,
    ScalarIntrinsicType,
};
use context::BlockChecker;
use function_check::check_function;
use model::{AssignKind, FieldIdx, ScalarMatchKind, StackEntry};
use traits::BoundEnv;

pub use driver::{typecheck, TypeCheckResult};
pub use public_signature::{
    TypedPublicSignatureEntry, TypedPublicSignatureKind, TypedPublicSignatureTable,
};
pub use public_surface::{
    PublicCallableSurface, PublicEffect, PublicEnumSurface, PublicEnumVariantSurface,
    PublicFieldSurface, PublicImplKind, PublicImplSurface, PublicStructConstructorPolicy,
    PublicStructSurface, PublicSurfaceShape, PublicTraitCapability, PublicTraitMethodSurface,
    PublicTraitRef, PublicTraitSurface, PublicTypeParam, PublicTypeParamBounds, PublicTypeTerm,
    TypedPublicSurfaceEntry, TypedPublicSurfaceTable,
};
