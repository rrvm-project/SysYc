
pub trait ir_type {
    fn is_base(&self) -> bool;
    fn is_array(&self) -> bool;
    fn indexed(&self) -> Option<&Self>;
    fn size(&self) -> usize;
    fn name(&self) -> String;
    fn dims(&self) -> Vec<u32>;
}