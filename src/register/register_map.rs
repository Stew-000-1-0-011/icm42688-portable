trait Address<const BANK_NUM: u8> {
	const ADDR: u8;
}

pub mod bank0 {
	use super::*;

	pub struct DeviceConfig(pub u8);
	#[derive(Clone, Copy, enumn::N)]
	#[repr(u8)]
	pub enum SpiMode {
		Mode0Mode3 = 0,
		Mode1Mode2 = 1,
	}
	#[derive(Clone, Copy, enumn::N)]
	#[repr(u8)]
	pub enum SoftResetConfig {
		Normal = 0,
		EableReset = 1,  // wait 1ms before attempting any other access to the device
	}
	impl DeviceConfig {
		pub fn pack(spi_mode: SpiMode, soft_reset_config: SoftResetConfig) -> DeviceConfig {
			DeviceConfig((spi_mode as u8) << 4 | (soft_reset_config as u8))
		}

		pub fn depack(&self) -> (SpiMode, SoftResetConfig) {
			(
				SpiMode::n(self.0 >> 4 & 0b1).unwrap(),
				SoftResetConfig::n(self.0 & 0b1).unwrap(),
			)
		}
	}
	impl Address<0> for DeviceConfig {
		const ADDR: u8 = 0x75;
	}
	impl ByteReadable for DeviceConfig {}
	impl Writable for DeviceConfig {}
}