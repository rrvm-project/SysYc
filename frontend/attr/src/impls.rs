use rrvm_symbol::{FuncSymbol, VarSymbol};
use value::{Value, VarType};

use crate::Attr;

// TODO: use derive macro to generate this

impl From<&Attr> for VarType {
	fn from(value: &Attr) -> Self {
		if let Attr::VarType(v) = value {
			v.clone()
		} else {
			unreachable!("Don't downcast if you do not zhe shi shenma leixing")
		}
	}
}

impl From<&Attr> for Value {
	fn from(value: &Attr) -> Self {
		if let Attr::Value(v) = value {
			v.clone()
		} else {
			unreachable!("Don't downcast if you do not zhe shi shenma leixing")
		}
	}
}

impl From<&Attr> for VarSymbol {
	fn from(value: &Attr) -> Self {
		if let Attr::VarSymbol(v) = value {
			v.clone()
		} else {
			unreachable!("Don't downcast if you do not zhe shi shenma leixing")
		}
	}
}

impl From<&Attr> for FuncSymbol {
	fn from(value: &Attr) -> Self {
		if let Attr::FuncSymbol(v) = value {
			v.clone()
		} else {
			unreachable!("Don't downcast if you do not zhe shi shenma leixing")
		}
	}
}

impl From<VarType> for Attr {
	fn from(value: VarType) -> Self {
		Self::VarType(value)
	}
}

impl From<Value> for Attr {
	fn from(value: Value) -> Self {
		Self::Value(value)
	}
}

impl From<VarSymbol> for Attr {
	fn from(value: VarSymbol) -> Self {
		Self::VarSymbol(value)
	}
}

impl From<FuncSymbol> for Attr {
	fn from(value: FuncSymbol) -> Self {
		Self::FuncSymbol(value)
	}
}
