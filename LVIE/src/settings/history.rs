#![allow(dead_code)]
use std::vec::Vec;

use super::xml;

#[derive(PartialEq, Clone)]
pub enum OpType {
    Saturation,
    Contrast,
    Filter,
}

#[derive(Clone)]
pub struct Operation {
    optype: OpType,
    parameters: Vec<f32>,
}

pub struct History {
    history: Vec<Operation>,
}

impl History {
    pub fn append(&mut self, operation: Operation) {
        self.history.push(operation);
    }

    pub fn get_by_type(&self, optype: OpType) -> Option<Vec<&Operation>> {
        let mut out: Vec<&Operation> = Vec::new();
        for x in &self.history {
            if x.optype == optype {
                out.push(x);
            }
        }

        if out.len() > 0 {
            Option::Some(out)
        } else {
            Option::None
        }
    }
}
