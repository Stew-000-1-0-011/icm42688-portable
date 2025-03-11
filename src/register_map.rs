trait Adress {
	const ADDR: u8;
}

trait Bank<const BANK_NUM: u8> {}
trait ByteReadable {}
trait WordReadable {}
trait Writable {}
trait ReadClearable {}
trait WriteClearble {}

pub mod bank0 {

	struct DeviceConfig(u8);
	enum SpiMode {
		Mode0Mode3 = 0,
		Mode1Mode2 = 1,
	}
	enum SoftResetConfig {
		Normal = 0,
		EableReset = 1,  // wait 1ms before attempting any other access to the device
	}
}