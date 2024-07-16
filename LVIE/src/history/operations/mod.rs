use crate::core::{CurveType, FilterArray};

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum GeometricOperationType {
    Rotation(f32),
    Traslation(f32, f32),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum LogicOperationType {
    Reset(FilterArray),
    FileLoaded(),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum MaskOperationType {
    MainPointMoved(usize, f32, f32, f32, f32),
    ControlPointMoved(usize, usize, f32, f32, f32, f32),

    MainPointAdded(usize, f32, f32),
    MainPointRemoved(usize, f32, f32),

    MaskOpened(f32, f32),
    MaskClosed(),
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum CurveOperationType {
    CurvePointAdded(usize, f32, f32),
    CurvePointRemoved(usize, f32, f32),
    CurvePointMoved(usize, f32, f32, f32, f32),
    CurveTypeChanged(CurveType),
}
