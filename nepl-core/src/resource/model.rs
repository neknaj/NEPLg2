extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::ast::Effect;
pub use crate::effects::{ExternalIoOp, NondetOp, RawMemoryOp};
use crate::span::Span;
use crate::types::TypeId;

use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
pub use super::trait_identity::{ResourceTraitApplication, ResourceTraitMethodId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StorageId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ResourceBlockId(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceModule {
    pub functions: Vec<ResourceFunction>,
    pub entry: Option<String>,
    pub string_literals: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceFunction {
    pub name: String,
    pub origin_name: String,
    pub type_params: Vec<TypeId>,
    pub params: Vec<ResourceLocal>,
    pub result: TypeId,
    pub effect: Effect,
    pub entry_block: ResourceBlockId,
    pub blocks: Vec<ResourceBlock>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceLocal {
    pub name: String,
    pub ty: TypeId,
    pub mutable: bool,
    pub place: Place,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceBlock {
    pub id: ResourceBlockId,
    pub ops: Vec<ResourceOp>,
    pub terminator: ResourceTerminator,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceTerminator {
    Return { value: Option<Place>, span: Span },
    Unreachable { span: Span },
    RawBody { kind: RawBodyKind, span: Span },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawBodyKind {
    Wasm,
    LlvmIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceOp {
    Expr {
        kind: ResourceExprKind,
        output: Place,
        ty: TypeId,
        span: Span,
    },
    DeclareLocal {
        place: Place,
        source_name: String,
        mutable: bool,
        initializer: Option<Place>,
        span: Span,
    },
    Read {
        source: Place,
        output: Place,
        span: Span,
    },
    Assign {
        target: Place,
        value: Place,
        span: Span,
    },
    Borrow {
        source: Place,
        output: Place,
        kind: BorrowKind,
        synthetic: bool,
        span: Span,
    },
    Move {
        source: Place,
        output: Place,
        span: Span,
    },
    Drop {
        place: Place,
        span: Span,
    },
    EndScope {
        locals: Vec<Place>,
        result: Option<Place>,
        span: Span,
    },
    CallEffect {
        effect: EffectOp,
        span: Span,
    },
    FunctionValue {
        output: Place,
        name: String,
        effect: EffectOp,
        span: Span,
    },
    Call {
        output: Place,
        target: ResourceCallTarget,
        args: Vec<Place>,
        effect: EffectOp,
        span: Span,
    },
    IndirectCall {
        output: Place,
        callee: Place,
        params: Vec<TypeId>,
        result: TypeId,
        args: Vec<Place>,
        effect: EffectOp,
        span: Span,
    },
    RawMemory {
        operation: RawMemoryOp,
        output: Place,
        args: Vec<Place>,
        span: Span,
    },
    RawAddressAlias {
        source: Place,
        target: Place,
        kind: RawAddressAliasKind,
        span: Span,
    },
    RawAddressView {
        source: Place,
        target: Place,
        kind: RawAddressViewKind,
        span: Span,
    },
    StorageOrigin {
        target: Place,
        origin: StorageOrigin,
        span: Span,
    },
    CollectionSlotLifecycle {
        target: Place,
        event: CollectionSlotLifecycleEvent,
        span: Span,
    },
    CollectionStorageRelocate {
        old_storage: Place,
        new_storage: Place,
        span: Span,
    },
    CollectionSlotDropTraversal {
        storage: Place,
        initialized_count: Place,
        expected_ty: TypeId,
        span: Span,
    },
    Construct {
        output: Place,
        kind: AggregateKind,
        inputs: Vec<Place>,
        span: Span,
    },
    Branch {
        output: Place,
        condition: Place,
        condition_fact: Option<ResourceConditionFact>,
        then_ops: Vec<ResourceOp>,
        then_value: Place,
        else_ops: Vec<ResourceOp>,
        else_value: Place,
        span: Span,
    },
    Loop {
        condition_ops: Vec<ResourceOp>,
        condition: Place,
        condition_fact: Option<ResourceConditionFact>,
        body_ops: Vec<ResourceOp>,
        span: Span,
    },
    Match {
        output: Place,
        scrutinee: Place,
        scrutinee_is_borrow_target: bool,
        arms: Vec<ResourceMatchArm>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawAddressViewKind {
    Offset,
    MemPtrOffset,
    NonOwningProjection,
    InternalHelper,
}

impl fmt::Display for RawAddressViewKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            RawAddressViewKind::Offset => "offset",
            RawAddressViewKind::MemPtrOffset => "mem_ptr_offset",
            RawAddressViewKind::NonOwningProjection => "non_owning_projection",
            RawAddressViewKind::InternalHelper => "internal_helper",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawAddressAliasKind {
    Transparent,
    InternalHelper,
    OwnerTokenConstruct,
}

impl fmt::Display for RawAddressAliasKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            RawAddressAliasKind::Transparent => "transparent",
            RawAddressAliasKind::InternalHelper => "internal_helper",
            RawAddressAliasKind::OwnerTokenConstruct => "owner_token_construct",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceExprKind {
    Literal,
    LiteralI32(i32),
    LayoutSizeOf(TypeId),
    LocalRead,
    FunctionValue,
    Call,
    IndirectCall,
    Branch,
    Loop,
    Match,
    Construct,
    Block,
    Let,
    Set,
    Intrinsic,
    Borrow,
    Deref,
    Drop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceCallTarget {
    Builtin {
        name: String,
    },
    User {
        name: String,
        type_args: Vec<TypeId>,
    },
    Trait {
        application: ResourceTraitApplication,
        method: ResourceTraitMethodId,
        self_ty: TypeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateKind {
    Enum {
        name: String,
        variant: String,
    },
    Struct {
        name: String,
        field_offsets: Vec<usize>,
    },
    Tuple {
        field_offsets: Vec<usize>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceMatchArm {
    pub pattern: ResourceMatchPattern,
    pub bind_local: Option<Place>,
    pub bind_source_name: Option<String>,
    pub bind_mode: Option<ResourceMatchBindMode>,
    pub ops: Vec<ResourceOp>,
    pub value: Place,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceMatchBindMode {
    Owned,
    Borrowed { is_mut: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceMatchPattern {
    Variant(String),
    IntLiteral(i32),
    BoolLiteral(bool),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceConditionFact {
    EqZero {
        place: Place,
    },
    NeZero {
        place: Place,
    },
    Positive {
        place: Place,
    },
    NonPositive {
        place: Place,
    },
    Negative {
        place: Place,
    },
    NonNegative {
        place: Place,
    },
    I32Relation {
        left: Place,
        op: ResourceI32RelationOp,
        right: Place,
    },
    Any(Vec<ResourceConditionFact>),
    All(Vec<ResourceConditionFact>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceI32RelationOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum I32ValueCondition {
    EqZero,
    NeZero,
    Positive,
    NonPositive,
    Negative,
    NonNegative,
}

impl I32ValueCondition {
    pub fn holds(self, value: i32) -> bool {
        match self {
            I32ValueCondition::EqZero => value == 0,
            I32ValueCondition::NeZero => value != 0,
            I32ValueCondition::Positive => value > 0,
            I32ValueCondition::NonPositive => value <= 0,
            I32ValueCondition::Negative => value < 0,
            I32ValueCondition::NonNegative => value >= 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Place {
    pub root: PlaceRoot,
    pub projections: Vec<PlaceProjection>,
    pub ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlaceRoot {
    Local(String),
    Temporary(ResourceId),
    I32Constant(i32),
    Return,
    Storage(StorageId),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlaceProjection {
    Field { index: usize, offset_bytes: usize },
    TupleField { index: usize, offset_bytes: usize },
    EnumPayload { variant: String },
    Deref,
    StorageOffset(ResourceOffset),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceOffset {
    Known(usize),
    Symbolic { place: Box<Place> },
    ScaledSymbolic { place: Box<Place>, scale: usize },
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorrowKind {
    Shared,
    Unique,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectOp {
    Pure,
    UserCall { name: String, effect: Effect },
    IndirectCall { effect: Effect },
    InternalAlloc { operation: RawMemoryOp },
    UnsafeMemory { operation: RawMemoryOp },
    ExternalIo { operation: ExternalIoOp },
    Nondet { operation: NondetOp },
    Unknown { reason: UnknownEffectReason },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownEffectReason {
    FunctionValueWithoutKnownEffect,
    AssignedCallbackWithoutKnownEffect,
    FunctionParameterWithoutKnownEffect,
    CallbackParameterWithoutKnownEffect,
    SyntheticTestFixture,
}

impl UnknownEffectReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            UnknownEffectReason::FunctionValueWithoutKnownEffect => {
                "function_value_without_known_effect"
            }
            UnknownEffectReason::AssignedCallbackWithoutKnownEffect => {
                "assigned_callback_without_known_effect"
            }
            UnknownEffectReason::FunctionParameterWithoutKnownEffect => {
                "function_parameter_without_known_effect"
            }
            UnknownEffectReason::CallbackParameterWithoutKnownEffect => {
                "callback_parameter_without_known_effect"
            }
            UnknownEffectReason::SyntheticTestFixture => "synthetic_test_fixture",
        }
    }
}

impl fmt::Display for UnknownEffectReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PointerProvenance {
    pub base: Place,
    pub offset: ResourceOffset,
    pub may_alias_unknown_offset: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceState {
    pub cells: Vec<CellStateEntry>,
    pub owners: Vec<OwnerStateEntry>,
    pub storage_origins: Vec<StorageOriginEntry>,
    pub borrows: Vec<BorrowStateEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellStateEntry {
    pub place: Place,
    pub state: CellState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellState {
    Uninit,
    Initialized(TypeId),
    Moved,
    Dropped,
    MaybeMoved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerStateEntry {
    pub place: Place,
    pub state: OwnerState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageOriginEntry {
    pub place: Place,
    pub origin: StorageOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StorageOrigin {
    Owned,
    Unmanaged,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnerState {
    NoFreeObligation,
    Live {
        storage: StorageId,
        extent: OwnerStorageExtent,
    },
    Reserved {
        storage: Option<StorageId>,
    },
    Moved,
    Freed,
    MaybeFreed {
        storage: Option<StorageId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnerStorageExtent {
    Unknown,
    PayloadBytes { bytes: Box<Place> },
    PayloadBytesScaled { source: Box<Place>, scale: usize },
    RegionTokenSize,
}

impl OwnerStorageExtent {
    pub fn payload_bytes(bytes: &Place) -> Self {
        Self::PayloadBytes {
            bytes: Box::new(bytes.clone()),
        }
    }

    pub fn payload_bytes_scaled(source: &Place, scale: usize) -> Self {
        if scale == 1 {
            Self::payload_bytes(source)
        } else {
            Self::PayloadBytesScaled {
                source: Box::new(source.clone()),
                scale,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BorrowStateEntry {
    pub place: Place,
    pub state: BorrowState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorrowState {
    Unborrowed,
    Shared { count: usize },
    Unique { source: Box<Place> },
    Released,
}

impl Place {
    pub fn local(name: String, ty: TypeId) -> Self {
        Self {
            root: PlaceRoot::Local(name),
            projections: Vec::new(),
            ty,
        }
    }

    pub fn unknown(ty: TypeId) -> Self {
        Self {
            root: PlaceRoot::Unknown,
            projections: Vec::new(),
            ty,
        }
    }

    pub fn temporary(id: ResourceId, ty: TypeId) -> Self {
        Self {
            root: PlaceRoot::Temporary(id),
            projections: Vec::new(),
            ty,
        }
    }

    pub fn i32_constant(value: i32, ty: TypeId) -> Self {
        Self {
            root: PlaceRoot::I32Constant(value),
            projections: Vec::new(),
            ty,
        }
    }

    pub fn with_projection(mut self, projection: PlaceProjection, ty: TypeId) -> Self {
        self.projections.push(projection);
        self.ty = ty;
        self
    }
}

impl Default for ResourceState {
    fn default() -> Self {
        Self {
            cells: Vec::new(),
            owners: Vec::new(),
            storage_origins: Vec::new(),
            borrows: Vec::new(),
        }
    }
}
