trait U8T {
	const N: u8;
}
struct U8<const N: u8>;
impl<const N: u8> U8T for U8<N> {
	const N: u8 = N;
}
