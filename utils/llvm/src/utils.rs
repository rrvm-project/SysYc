pub fn all_equal<T: PartialEq>(slice: &[T]) -> bool {
	slice.windows(2).all(|window| window[0] == window[1])
}
