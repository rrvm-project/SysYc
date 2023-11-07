
pub trait tp {
    fn is_base(&self) -> bool;
    fn is_array(&self) -> bool;
    fn indexed(&self) -> Option<&Self>;
    fn size(&self) -> usize;
}