mod osr;
pub mod impls;
pub struct StrengthReduce {
    // 此 pass 需要创建新变量，这里记一个新变量的总量，起到一个 TempManager 的作用
    total_new_temp: u32,
}
