// use embedded_hal_async::spi;

// struct SpiSampleDevice<SpiDevice: spi::SpiDevice> {
// 	spi_device: SpiDevice,
// }

// impl<SpiDevice: spi::SpiDevice> SpiSampleDevice<SpiDevice> {
// 	pub fn new(spi_device: SpiDevice) -> Self {
// 		Self {
// 			spi_device,
// 		}
// 	}

// 	pub async fn read(&mut self, buffer: &mut [u8]) -> Result<(), SpiDevice::Error> {
// 		// push "hello" to front of buffer
// 		buffer[0] = b'h';
// 		buffer[1] = b'e';
// 		buffer[2] = b'l';
// 		buffer[3] = b'l';
// 		buffer[4] = b'o';
// 		buffer[5] = b'!';
		
// 		// read from spi device and append to buffer
// 		self.spi_device.read(&mut buffer[6..]).await?;

// 		Ok(())
// 	}

// 	pub async fn write(&mut self, buffer: &[u8]) -> Result<(), SpiDevice::Error> {
// 		// write buffer to spi device
// 		self.spi_device.write(buffer).await?;

// 		// write "world" to spi device
// 		self.spi_device.write(b"world").await?;
		
// 		Ok(())
// 	}
// }