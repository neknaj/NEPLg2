extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::HirTraitApplication;
use crate::types::{TypeCtx, TypeId};

use super::trait_identity::{MonoTraitId, MonoTraitMethodId};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MonoTraitApplication {
    pub(super) trait_id: MonoTraitId,
    pub(super) args: Vec<TypeId>,
}

impl MonoTraitApplication {
    pub(super) fn resolved(ctx: &TypeCtx, trait_id: MonoTraitId, args: &[TypeId]) -> Self {
        Self {
            trait_id,
            args: args.iter().map(|arg| ctx.resolve_id(*arg)).collect(),
        }
    }

    pub(super) fn from_hir(ctx: &TypeCtx, application: &HirTraitApplication) -> Self {
        Self::resolved(
            ctx,
            MonoTraitId::from_name(&application.base_name),
            &application.args,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MonoTraitMethodKey {
    trait_id: MonoTraitId,
    method: MonoTraitMethodId,
}

impl MonoTraitMethodKey {
    pub(super) fn new(trait_id: MonoTraitId, method: MonoTraitMethodId) -> Self {
        Self { trait_id, method }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct MonoTraitLookupKey {
    application: MonoTraitApplication,
    method: MonoTraitMethodId,
    self_ty: TypeId,
}

impl MonoTraitLookupKey {
    pub(super) fn new(
        application: MonoTraitApplication,
        method: MonoTraitMethodId,
        self_ty: TypeId,
    ) -> Self {
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
