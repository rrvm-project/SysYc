use std::cmp::max;

use utils::{errors::Result, SysycError::*};

use crate::{BType, BinaryOp, VarType};

fn upgrade(x: &VarType, y: &VarType) -> Result<VarType> {
	if !x.dims.is_empty() || !y.dims.is_empty() {
		Err(TypeError(
			"Can not do arith operation with pointer".to_string(),
		))
	} else {
		Ok(VarType {
			is_lval: false,
			type_t: max(x.type_t, y.type_t),
			dims: Vec::new(),
		})
	}
}

fn to_bool(x: &VarType, y: &VarType) -> Result<VarType> {
	if !x.dims.is_empty() || !y.dims.is_empty() {
		Err(TypeError(
			"Can not do arith operation with pointer".to_string(),
		))
	} else {
		Ok(VarType {
			is_lval: false,
			type_t: BType::Int,
			dims: Vec::new(),
		})
	}
}

#[rustfmt::skip]
pub fn type_binaryop(x: &VarType, op: BinaryOp, y: &VarType) -> Result<VarType> {
	match op {
		BinaryOp::IDX => {
      if x.dims.is_empty() || y.type_t != BType::Int || !y.dims.is_empty() {
        return Err(TypeError("array can only be indexed by int".to_string()));
      }
      Ok(VarType{
        dims: x.dims[1..].to_vec(),
        ..*x
      })
		}
    BinaryOp::Assign => {
      if x.is_lval {
        Err(TypeError("Only lvalue can be assigned".to_string()))
      }
      else if !x.dims.is_empty() || !y.dims.is_empty() {
        Err(TypeError("Can not do assign to pointer".to_string()))
      }
      else {
        Ok(x.clone())
      }
    }
		BinaryOp::Add => upgrade(x, y),
		BinaryOp::Sub => upgrade(x, y),
		BinaryOp::Mul => upgrade(x, y),
		BinaryOp::Div => upgrade(x, y),
		BinaryOp::Mod => upgrade(x, y),
		BinaryOp::LT => to_bool(x, y),
		BinaryOp::LE => to_bool(x, y),
		BinaryOp::GT => to_bool(x, y),
		BinaryOp::GE => to_bool(x, y),
		BinaryOp::EQ => to_bool(x, y),
		BinaryOp::NE => to_bool(x, y),
	}
}
