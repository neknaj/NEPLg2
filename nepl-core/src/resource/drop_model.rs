extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::drop_point_path::ResourceDropPointPath;
use super::drop_requirement::ResourceDropRequirement;
use super::model::Place;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropPlan {
    pub functions: Vec<ResourceDropFunctionPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropFunctionPlan {
    pub name: String,
    pub auto_drops: Vec<ResourceAutoDrop>,
    pub drop_points: Vec<ResourceDropPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDropPoint {
    pub path: ResourceDropPointPath,
    pub span: Span,
    pub auto_drops: Vec<ResourceAutoDrop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceAutoDrop {
    pub place: Place,
    pub kind: ResourceAutoDropKind,
    pub requirement: ResourceDropRequirement,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceAutoDropKind {
    ScopeLocal,
    AssignmentOverwrite,
}
