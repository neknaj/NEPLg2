extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::Span;
use crate::types::TypeId;

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
    Construct {
        output: Place,
        kind: AggregateKind,
        inputs: Vec<Place>,
        span: Span,
    },
    Branch {
        output: Place,
        condition: Place,
        then_ops: Vec<ResourceOp>,
        then_value: Place,
        else_ops: Vec<ResourceOp>,
        else_value: Place,
        span: Span,
    },
    Loop {
        condition_ops: Vec<ResourceOp>,
        condition: Place,
        body_ops: Vec<ResourceOp>,
        span: Span,
    },
    Match {
        output: Place,
        scrutinee: Place,
        arms: Vec<ResourceMatchArm>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceExprKind {
    Literal,
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
pub enum RawMemoryOp {
    Alloc,
    Dealloc,
    Realloc,
    Load,
    Store,
    BulkCopy,
    BulkMove,
    MemorySize,
    MemoryGrow,
    Fill,
    Other { name: String },
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
        trait_name: String,
        trait_args: Vec<TypeId>,
        method: String,
        self_ty: TypeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AggregateKind {
    Enum { name: String, variant: String },
    Struct { name: String },
    Tuple,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceMatchArm {
    pub pattern: ResourceMatchPattern,
    pub bind_local: Option<Place>,
    pub ops: Vec<ResourceOp>,
    pub value: Place,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceMatchPattern {
    Variant(String),
    IntLiteral(i32),
    BoolLiteral(bool),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Place {
    pub root: PlaceRoot,
    pub projections: Vec<PlaceProjection>,
    pub ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceRoot {
    Local(String),
    Temporary(ResourceId),
    Return,
    Storage(StorageId),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlaceProjection {
    Field { index: usize, offset_bytes: usize },
    TupleField { index: usize, offset_bytes: usize },
    EnumPayload { variant: String },
    Deref,
    StorageOffset(ResourceOffset),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceOffset {
    pub bytes: Option<usize>,
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
    InternalAlloc,
    UnsafeMemory { operation: String },
    ExternalIo { operation: String },
    Nondet { operation: String },
    Unknown { reason: String },
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
pub enum OwnerState {
    NoFreeObligation,
    Live { storage: StorageId },
    Moved,
    Freed,
    MaybeFreed,
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
            borrows: Vec::new(),
        }
    }
}
