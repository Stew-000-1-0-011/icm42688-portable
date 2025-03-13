use register_map::

// ジェネリクス引数に一般の構造比較可能でトリビアルにDropできる型を入れさせてくれ～

pub struct Icm42688 <
	const BANK_NUM: u8,
	const ACCEL_MODE: u8,
	const GYRO_MODE: u8,
>;

impl<const BANK_NUM: u8, const ACCEL_MODE: u8, const GYRO_MODE: u8> Icm42688<BANK_NUM, ACCEL_MODE, GYRO_MODE> {
	pub fn register_read(&self, )
}