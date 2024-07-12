use crate::core::{CurveType, FilterArray};

#[derive(Debug, Clone)]
pub enum GeometricOperationType {
    Rotation(f32),
    Traslation(f32, f32),
}

#[derive(Debug, Clone)]
pub enum LogicOperationType {
    Reset(FilterArray),
}

#[derive(Debug, Clone)]
pub enum MaskOperationType {
    MainPointMoved(usize, f32, f32),
    ControlPointMoved(usize, usize, f32, f32),

    MainPointAdded(usize),
    MainPointRemoved(usize, f32, f32),

    MaskClosed(),
}

#[derive(Debug, Clone)]
pub enum CurveOperationType {
    CurvePointAdded(usize),
    CurvePointRemoved(usize, f32, f32),
    CurvePointMoved(usize, f32, f32),
    CurveTypeChanged(CurveType),
}
