extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::TraitCapability;
use crate::ast::{Effect, LlvmIrBlock, WasmBlock};
use crate::resolve::DefId;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

#[derive(Debug, Clone)]
pub struct HirModule {
    pub functions: Vec<HirFunction>,
    pub entry: Option<String>,
    pub externs: Vec<HirExtern>,
    pub string_literals: Vec<String>,
    pub traits: Vec<HirTrait>,
    pub impls: Vec<HirImpl>,
}

#[derive(Debug, Clone)]
pub struct HirFunction {
    pub doc: Option<String>,
    pub name: String,
    pub origin_name: String,
    pub func_ty: TypeId, // new
    pub params: Vec<HirParam>,
    pub result: TypeId,
    pub effect: Effect,
    pub body: HirBody,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirParam {
    pub name: String,
    pub ty: TypeId,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct HirExtern {
    pub module: String,
    pub name: String,
    pub local_name: String,
    pub params: Vec<TypeId>,
    pub result: TypeId,
    pub effect: Effect,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirBody {
    Block(HirBlock),
    Wasm(WasmBlock),
    LlvmIr(LlvmIrBlock),
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirBlock {
    pub lines: Vec<HirLine>,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirLine {
    pub expr: HirExpr,
    pub drop_result: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirExpr {
    pub ty: TypeId,
    pub kind: HirExprKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirExprKind {
    LiteralI32(i32),
    LiteralF32(f32),
    LiteralBool(bool),
    LiteralStr(u32),
    Unit,
    Var(String),
    /// Explicit function-value reference created by `@fn_name`.
    FnValue(String),
    Call {
        callee: FuncRef,
        args: Vec<HirExpr>,
    },
    CallIndirect {
        callee: Box<HirExpr>,
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
        args: Vec<HirExpr>,
    },
    If {
        cond: Box<HirExpr>,
        then_branch: Box<HirExpr>,
        else_branch: Box<HirExpr>,
    },
    While {
        cond: Box<HirExpr>,
        body: Box<HirExpr>,
    },
    Match {
        scrutinee: Box<HirExpr>,
        arms: Vec<HirMatchArm>,
    },
    EnumConstruct {
        name: String,
        variant: String,
        type_args: Vec<TypeId>,
        payload: Option<Box<HirExpr>>,
    },
    StructConstruct {
        name: String,
        type_args: Vec<TypeId>,
        fields: Vec<HirExpr>,
    },
    TupleConstruct {
        items: Vec<HirExpr>,
    },
    Block(HirBlock),
    Let {
        name: String,
        mutable: bool,
        value: Box<HirExpr>,
    },
    Set {
        name: String,
        value: Box<HirExpr>,
    },
    Intrinsic {
        name: String,
        type_args: Vec<TypeId>,
        args: Vec<HirExpr>,
    },
    AddrOf(Box<HirExpr>),
    Deref(Box<HirExpr>),
    Drop {
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTraitId(String);

impl HirTraitId {
    pub fn from_name(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTraitMethodId(String);

impl HirTraitMethodId {
    pub fn from_name(name: String) -> Self {
        Self(name)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTraitApplication {
    pub trait_id: HirTraitId,
    pub args: Vec<TypeId>,
}

impl HirTraitApplication {
    pub fn new(base_name: String, args: Vec<TypeId>) -> Self {
        Self {
            trait_id: HirTraitId::from_name(base_name),
            args,
        }
    }

    pub fn display_name(&self, ctx: &TypeCtx) -> String {
        if self.args.is_empty() {
            return self.trait_id.as_str().to_string();
        }
        let mut name = self.trait_id.as_str().to_string();
        name.push('<');
        for (index, arg) in self.args.iter().enumerate() {
            if index > 0 {
                name.push(',');
            }
            name.push_str(&ctx.type_to_string(*arg));
        }
        name.push('>');
        name
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FuncRef {
    Builtin(String),
    User(String, Vec<TypeId>, Option<DefId>),
    Trait {
        application: HirTraitApplication,
        method: HirTraitMethodId,
        self_ty: TypeId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirMatchPattern {
    Variant(String),
    IntLiteral(i32),
    BoolLiteral(bool),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HirMatchArm {
    pub pattern: HirMatchPattern,
    pub bind_local: Option<String>,
    pub bind_ty: Option<TypeId>,
    pub body: HirExpr,
}
#[derive(Debug, Clone)]
pub struct HirTrait {
    pub doc: Option<String>,
    pub name: String,
    pub type_params: Vec<TypeId>,
    pub capabilities: Vec<TraitCapability>,
    pub methods: alloc::collections::BTreeMap<String, TypeId>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirImpl {
    pub doc: Option<String>,
    pub trait_application: HirTraitApplication,
    pub type_args: Vec<TypeId>,
    pub target_ty: TypeId,
    pub methods: Vec<HirImplMethod>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct HirImplMethod {
    pub name: String,
    pub func: HirFunction,
}
