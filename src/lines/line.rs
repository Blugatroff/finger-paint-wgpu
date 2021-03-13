use crate::lines::LineVertex;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Line {
    p1: LineVertex,
    p2: LineVertex,
}

impl Line {
    pub fn new(p1: LineVertex, p2: LineVertex) -> Self {
        Self {
            p1,
            p2,
        }
    }
}