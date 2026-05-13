extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};

use crate::ast::Effect;
use crate::effects::{intrinsic_internal_effect, raw_callee_internal_effect, InternalEffect};
use crate::hir::FuncRef;
use crate::runtime_helpers::helper_base_name;

use super::lower::LoweringEnvironment;
use super::model::{EffectOp, ResourceCallTarget, ResourceTraitApplication, ResourceTraitMethodId};

pub(super) fn call_effect_skeleton(callee: &FuncRef, env: &LoweringEnvironment) -> EffectOp {
    match callee {
        FuncRef::Builtin(name) => {
            if let Some(effect) = raw_callee_internal_effect(name.as_str()) {
                resource_effect_from_internal(effect)
            } else {
                EffectOp::UserCall {
                    name: name.clone(),
                    effect: Effect::Pure,
                }
            }
        }
        FuncRef::User(name, _, _) => {
            if let Some(effect) = raw_callee_internal_effect(name.as_str()) {
                resource_effect_from_internal(effect)
            } else {
                EffectOp::UserCall {
                    name: name.clone(),
                    effect: env.function_effect(name),
                }
            }
        }
        FuncRef::Trait {
            application,
            method,
            ..
        } => EffectOp::UserCall {
            name: format!("{}::{}", application.trait_id.as_str(), method.as_str()),
            effect: Effect::Pure,
        },
    }
}

pub(super) fn function_value_effect(name: &str, env: &LoweringEnvironment) -> EffectOp {
    env.known_function_value_effect(name)
        .unwrap_or_else(|| EffectOp::UserCall {
            name: String::from(name),
            effect: env.function_effect(name),
        })
}

pub(super) fn resource_effect_from_internal(effect: InternalEffect) -> EffectOp {
    match effect {
        InternalEffect::Pure => EffectOp::Pure,
        InternalEffect::InternalAlloc { operation } => EffectOp::InternalAlloc { operation },
        InternalEffect::UnsafeMemory { operation } => EffectOp::UnsafeMemory { operation },
        InternalEffect::ExternalIo { operation } => EffectOp::ExternalIo { operation },
        InternalEffect::Nondet { operation } => EffectOp::Nondet { operation },
    }
}

pub(super) fn intrinsic_effect_skeleton(name: &str) -> Option<EffectOp> {
    let effect = intrinsic_internal_effect(name);
    if matches!(effect, InternalEffect::Pure) {
        None
    } else {
        Some(resource_effect_from_internal(effect))
    }
}

pub(super) fn lower_call_target(callee: &FuncRef) -> ResourceCallTarget {
    match callee {
        FuncRef::Builtin(name) => ResourceCallTarget::Builtin { name: name.clone() },
        FuncRef::User(name, type_args, _) => ResourceCallTarget::User {
            name: name.clone(),
            type_args: type_args.clone(),
        },
        FuncRef::Trait {
            application,
            method,
            self_ty,
        } => ResourceCallTarget::Trait {
            application: ResourceTraitApplication::new(
                application.trait_id.as_str().to_string(),
                application.args.clone(),
            ),
            method: ResourceTraitMethodId::from_name(method.as_str().to_string()),
            self_ty: *self_ty,
        },
    }
}

pub(super) fn func_ref_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}
