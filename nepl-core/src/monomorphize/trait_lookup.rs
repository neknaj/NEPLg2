extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::HirTraitApplication;
use crate::types::{TypeCtx, TypeId};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MonoTraitApplication {
    pub(super) base_name: String,
    pub(super) args: Vec<TypeId>,
}

impl MonoTraitApplication {
    pub(super) fn resolved(ctx: &TypeCtx, base_name: String, args: &[TypeId]) -> Self {
        Self {
            base_name,
            args: args.iter().map(|arg| ctx.resolve_id(*arg)).collect(),
        }
    }

    pub(super) fn from_hir(ctx: &TypeCtx, application: &HirTraitApplication) -> Self {
        Self::resolved(ctx, application.base_name.clone(), &application.args)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MonoTraitMethodKey {
    trait_base_name: String,
    method: String,
}

impl MonoTraitMethodKey {
    pub(super) fn new(trait_base_name: String, method: String) -> Self {
        Self {
            trait_base_name,
            method,
        }
    }

    pub(super) fn from_names(trait_base_name: &str, method: &str) -> Self {
        Self::new(String::from(trait_base_name), String::from(method))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MonoTraitLookupKey {
    application: MonoTraitApplication,
    method: String,
    self_ty: TypeId,
}

impl MonoTraitLookupKey {
    pub(super) fn new(application: MonoTraitApplication, method: String, self_ty: TypeId) -> Self {
        Self {
            application,
            method,
            self_ty,
        }
    }
}

#[derive(Clone)]
pub(super) struct TraitImplEntry {
    pub(super) application: MonoTraitApplication,
    pub(super) type_args: Vec<TypeId>,
    pub(super) target_ty: TypeId,
    pub(super) func_name: String,
}

#[derive(Clone)]
pub(super) struct TraitImplResolution {
    pub(super) func_name: String,
    pub(super) type_args: Vec<TypeId>,
}
