use llvm::Value;

use super::indvar::IndVar;

#[derive(Debug, PartialEq, Eq)]
pub enum IndVarType {
	/// scale == Int(1) && step.len() == 0
	Invariant,
	/// zfp.is_none() && scale == Int(1) && step.len() == 1
	Ordinary,
	/// zfp.is_none() && scale == Int(1) && step.len() > 1
	KRank,
	/// zfp.is_none() && scale != Int(1)
	WithScale,
	/// zfp.is_some() && scale == Int(1) && step.len() == 1
	OrdinaryZFP,
	/// zfp.is_some() && scale == Int(1) && step.len() > 1
	KRankZFP,
	/// zfp.is_some() && scale != Int(1)
	WithScaleZFP,
}

#[allow(unused)]
impl IndVar {
	pub fn get_type(&self) -> IndVarType {
		match (self.zfp.is_some(), self.scale.clone(), self.step.len()) {
			(_, Value::Int(1), 0) => IndVarType::Invariant,
			(false, Value::Int(1), 1) => IndVarType::Ordinary,
			(false, Value::Int(1), _) => IndVarType::KRank,
			(false, _, _) => IndVarType::WithScale,
			(true, Value::Int(1), 1) => IndVarType::OrdinaryZFP,
			(true, Value::Int(1), _) => IndVarType::KRankZFP,
			(true, _, _) => IndVarType::WithScaleZFP,
		}
	}
}
