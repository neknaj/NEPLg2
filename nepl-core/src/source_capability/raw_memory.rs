use crate::ast::{Block, FnBody, Module, PrefixExpr, PrefixItem, Stmt, Symbol};
use crate::effects::{
    raw_body_direct_callees, raw_body_memory_operations, raw_memory_op_from_name,
};
use crate::hir::HirBody;
use crate::runtime_helpers::helper_base_name;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawMemoryBoundaryEvidence {
    RawBodyInstruction,
    RawAddressBoundaryHelper,
    RawHelperCall,
    RawOwnerBoundaryHelper,
    RawIntrinsic,
    RestrictedConstructor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawAddressBoundaryHelper {
    MemPtrWrap,
    MemPtrAddr,
    MemPtrAdd,
    RegionNew,
    RegionPtr,
    RegionPtrAt,
    RegionTokenPtrRef,
    StrAddr,
    StrFromAddrUnchecked,
}

impl RawAddressBoundaryHelper {
    fn from_symbol(name: &str) -> Option<Self> {
        let helper = match helper_base_name(name) {
            "mem_ptr_wrap" => Self::MemPtrWrap,
            "mem_ptr_addr" => Self::MemPtrAddr,
            "mem_ptr_add" => Self::MemPtrAdd,
            "region_new" => Self::RegionNew,
            "region_ptr" => Self::RegionPtr,
            "region_ptr_at" => Self::RegionPtrAt,
            "region_token_ptr_ref" => Self::RegionTokenPtrRef,
            "str_addr" => Self::StrAddr,
            "str_from_addr_unchecked" => Self::StrFromAddrUnchecked,
            _ => return None,
        };
        Some(helper)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawOwnerBoundaryHelper {
    AllocPtr,
    ReallocPtr,
    DeallocPtr,
    AllocRegion,
    AllocRegionBytes,
    DeallocRegion,
}

impl RawOwnerBoundaryHelper {
    fn from_symbol(name: &str) -> Option<Self> {
        let helper = match helper_base_name(name) {
            "alloc_ptr" => Self::AllocPtr,
            "realloc_ptr" => Self::ReallocPtr,
            "dealloc_ptr" => Self::DeallocPtr,
            "alloc_region" => Self::AllocRegion,
            "alloc_region_bytes" => Self::AllocRegionBytes,
            "dealloc_region" => Self::DeallocRegion,
            _ => return None,
        };
        Some(helper)
    }
}

impl RawMemoryBoundaryEvidence {
    fn from_symbol(name: &str) -> Option<Self> {
        if matches!(name, "MemPtr" | "RegionToken") {
            return Some(Self::RestrictedConstructor);
        }
        if RawAddressBoundaryHelper::from_symbol(name).is_some() {
            return Some(Self::RawAddressBoundaryHelper);
        }
        if RawOwnerBoundaryHelper::from_symbol(name).is_some() {
            return Some(Self::RawOwnerBoundaryHelper);
        }
        raw_memory_op_from_name(name).map(|_| Self::RawHelperCall)
    }
}

pub(crate) fn module_has_raw_memory_boundary_evidence(module: &Module) -> bool {
    block_raw_memory_boundary_evidence(&module.root).is_some()
}

fn block_raw_memory_boundary_evidence(block: &Block) -> Option<RawMemoryBoundaryEvidence> {
    block
        .items
        .iter()
        .find_map(stmt_raw_memory_boundary_evidence)
}

fn stmt_raw_memory_boundary_evidence(stmt: &Stmt) -> Option<RawMemoryBoundaryEvidence> {
    match stmt {
        Stmt::FnDef(def) => fn_body_raw_memory_boundary_evidence(&def.body),
        Stmt::Impl(def) => def
            .methods
            .iter()
            .find_map(|method| fn_body_raw_memory_boundary_evidence(&method.body)),
        Stmt::FnAlias(alias) => RawMemoryBoundaryEvidence::from_symbol(alias.target.name.as_str()),
        Stmt::Wasm(body) => raw_body_evidence(HirBody::Wasm(body.clone())),
        Stmt::LlvmIr(body) => raw_body_evidence(HirBody::LlvmIr(body.clone())),
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => expr_raw_memory_boundary_evidence(expr),
        Stmt::Directive(_) | Stmt::StructDef(_) | Stmt::EnumDef(_) | Stmt::Trait(_) => None,
    }
}

fn fn_body_raw_memory_boundary_evidence(body: &FnBody) -> Option<RawMemoryBoundaryEvidence> {
    match body {
        FnBody::Parsed(block) => block_raw_memory_boundary_evidence(block),
        FnBody::Wasm(body) => raw_body_evidence(HirBody::Wasm(body.clone())),
        FnBody::LlvmIr(body) => raw_body_evidence(HirBody::LlvmIr(body.clone())),
    }
}

fn raw_body_evidence(body: HirBody) -> Option<RawMemoryBoundaryEvidence> {
    if raw_body_memory_operations(&body).is_empty()
        && raw_body_direct_callees(&body)
            .iter()
            .all(|callee| raw_memory_op_from_name(callee).is_none())
    {
        None
    } else {
        Some(RawMemoryBoundaryEvidence::RawBodyInstruction)
    }
}

fn expr_raw_memory_boundary_evidence(expr: &PrefixExpr) -> Option<RawMemoryBoundaryEvidence> {
    expr.items
        .iter()
        .find_map(prefix_item_raw_memory_boundary_evidence)
}

fn prefix_item_raw_memory_boundary_evidence(
    item: &PrefixItem,
) -> Option<RawMemoryBoundaryEvidence> {
    match item {
        PrefixItem::Symbol(Symbol::Ident(ident, _, _)) => {
            RawMemoryBoundaryEvidence::from_symbol(ident.name.as_str())
        }
        PrefixItem::Intrinsic(intrinsic, _) => {
            if RawAddressBoundaryHelper::from_symbol(intrinsic.name.as_str()).is_some()
                || raw_memory_op_from_name(intrinsic.name.as_str()).is_some()
            {
                Some(RawMemoryBoundaryEvidence::RawIntrinsic)
            } else {
                intrinsic
                    .args
                    .iter()
                    .find_map(expr_raw_memory_boundary_evidence)
            }
        }
        PrefixItem::Block(block, _) => block_raw_memory_boundary_evidence(block),
        PrefixItem::Match(m, _) => m
            .arms
            .iter()
            .find_map(|arm| block_raw_memory_boundary_evidence(&arm.body))
            .or_else(|| expr_raw_memory_boundary_evidence(&m.scrutinee)),
        PrefixItem::Tuple(items, _) => items.iter().find_map(expr_raw_memory_boundary_evidence),
        PrefixItem::Group(inner, _) => expr_raw_memory_boundary_evidence(inner),
        PrefixItem::Literal(_, _)
        | PrefixItem::TypeAnnotation(_, _)
        | PrefixItem::Pipe(_)
        | PrefixItem::Symbol(_) => None,
    }
}
