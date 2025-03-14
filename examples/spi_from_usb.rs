#![no_std]
#![no_main]
#![feature(try_blocks)]

use panic_halt as _;

use core::fmt::{self, Write};

use cortex_m::asm;

use stm32f1xx_hal as _;
use stm32f1xx_hal:: {
	pac,
	rcc,
	flash,
	afio,
	spi,
	usb,
	prelude::*,
};

use embedded_halv02::spi as spiv02;
use spiv02::FullDuplex;

use usb_device::prelude::*;
use usbd_serial:: {
	SerialPort,
	USB_CLASS_CDC,
};
use fugit::RateExtU32;

// core::fmt::Writeを実装したバッファが欲しかった
struct WriteTo {
	buf: [u8; 128],
	pos: usize,
}

impl WriteTo {
	fn new() -> Self {
		Self { buf: [0u8; 128], pos: 0 }
	}

	fn as_str(&self) -> Result<&str, core::str::Utf8Error> {
		core::str::from_utf8(&self.buf[..self.pos])
	}
}

impl Write for WriteTo {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		let bytes = s.as_bytes();
		let len = bytes.len().min(self.buf.len() - self.pos);
		if self.pos + len > self.buf.len() {
			return Err(fmt::Error);
		}
		self.buf[self.pos..self.pos + len].copy_from_slice(&bytes[..len]);
		self.pos += len;
		Ok(())
	}
}

// writerのライフタイムの保証が面倒だったんで、ErrorMessageはスライスでなく配列とWriteToを持つようにした。
struct ErrorMessage<'a>(&'a str);
fn error_message<'a>(writer: &'a mut WriteTo, args: core::fmt::Arguments) -> ErrorMessage<'a> {
	write!(writer, "E{}", args).or_else(|_| write!(writer, "Efmt error")).ok();
	ErrorMessage(writer.as_str().unwrap_or("EUTF-8 error"))
}

fn assert(condition: bool, message: ErrorMessage) -> Result<(), ErrorMessage> {
	condition.then_some(()).ok_or(message)
}

#[cortex_m_rt::entry]
fn main() -> ! {
	// To not have main optimize to abort in release mode, remove when you add code
	asm::nop();

	// ペリフェラルを纏めて取得。組み込みRust仕草。
	let dp = pac::Peripherals::take().unwrap();
	
	let rcc_p: rcc::Rcc = dp.RCC.constrain();
	let mut flash_p: flash::Parts = dp.FLASH.constrain();

	// CubeMXのコードだとFlashのprefetch bufferを有効にしている
	// ...が、これはデフォルトで有効になっているので、特に何もしなくても良い。

	// CubeMXのコードだと、ここでNVICを弄り割り込みグループの優先度の設定をしているが、デフォルトでも大丈夫じゃないかな。
	// とりあえず、何もしない。

	// CubeMXのコードだと、ここでSysTickの設定をしているが、デフォルトでも大丈夫じゃないかな。
	// とりあえず、何もしない。

	// CubeMXのコードだと、ここでAFIO関連とPWR関連へのクロックの供給をしているが、デフォルトでも大丈夫じゃないかな。
	// うち、少なくともAFIOのクロックは供給する必要がある。すぐ下でAFIO_MAPRを変更しているが、これはAFIOのクロックが供給されていないといけない。
	unsafe {
		let rcc_p = pac::Peripherals::steal().RCC;
		rcc_p.apb2enr.write(|w| { w.afioen().set_bit() });
	}

	// JTAGの無効化かつSWDの有効化
	// disable_jtagに通してやらないとpa15, pb4, pb5は他の用途に使えない(ように型で制約されている)。
	let mut gpioa = dp.GPIOA.split();
	let mut gpiob = dp.GPIOB.split();

	let mut afio_p: afio::Parts = dp.AFIO.constrain();
	let (_pa15, pb3, pb4) = afio_p.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
	afio_p.mapr.modify_mapr(|_, w| unsafe {w.swj_cfg().bits(0b010)});

	// いつものクロックコンフィグに合わせている。
	// 罠1: CubeMXのClock Configurationには表示されていないが、usb clockというのがあり、
	// これは48MHzになるようプリスケーラを設定しなければならない。
	// 罠2: 同じくCubeMxには表示されていないが、adc clockというのがあり、これも制約の範囲内となるよう設定しなければならない。
	// 罠3: RCCの設定に合わせ、Flashメモリを読む速度をFLASHのADRレジスタに設定しなければならない。だからadrを渡している。
	let clocks = rcc_p.cfgr.freeze_with_config (
		rcc::Config {
			hse: Some(8_000_000u32),
			pllmul: Some(0b0111),  // **ATTENTION** 8MHz * (pllmul + 2) = 72MHz
			hpre: rcc::HPre::Div1,
			ppre1: rcc::PPre::Div2,
			ppre2: rcc::PPre::Div1,
			usbpre: rcc::UsbPre::Div15,
			adcpre: rcc::AdcPre::Div6,
		},
		&mut flash_p.acr,
	);

	// USBクロックが48MHzになっているか確認する。
	assert!(clocks.usbclk_valid(), "USB clock is not valid!");

	// 通信するたびPC13に繋がってるBluePillのLEDを光らせる。
	let mut gpioc = dp.GPIOC.split();
	let mut pc13led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
	pc13led.set_high(); // LEDを消灯しておく。

	// spiペリフェラルの初期化
	let pins = (
		pb3.into_alternate_push_pull(&mut gpiob.crl), // SCK
		pb4.into_floating_input(&mut gpiob.crl), // MISO
		gpiob.pb5.into_alternate_push_pull(&mut gpiob.crl), // MOSI
	);

	// ICM42688参照
	let spi_mode = spi::Mode {
		polarity: spi::Polarity::IdleHigh,  // 極性、データ送信してないときSCLKがHighかLowか
		phase: spi::Phase::CaptureOnSecondTransition,  // 位相、立ち上がりと立ち下がりどちらでサンプリングするか
	};

	// Hz, MsbFirstはICM42688参照
	let mut spi_p = spi::Spi::spi1 (
		dp.SPI1,
		pins,
		&mut afio_p.mapr,
		spi_mode,
		24u32.MHz(),
		clocks,
	).frame_size_16bit();
	spi_p.bit_format(spi::SpiBitFormat::MsbFirst);

	// USBペリフェラルの初期化
	let mut pa10_usb_dp_pullup = gpioa.pa10.into_push_pull_output(&mut gpioa.crh);
	pa10_usb_dp_pullup.set_high();  // CRSの基板では何故か慣習的にPA10がD+のプルアップ抵抗

	// USBからこのマイコンにプログラム書き込んでいる場合、ここでD+ピンを下げてバスをリセットする必要がある。
	let mut usb_dp = gpioa.pa12.into_push_pull_output(&mut gpioa.crh);
	usb_dp.set_low();
	// このリセットを解除するために、少し待つ。
	asm::delay(clocks.sysclk().raw() / 100);

	// USBのペリフェラルを初期化
	let usb_p = usb::Peripheral {
		usb: dp.USB,
		pin_dm: gpioa.pa11,
		pin_dp: usb_dp.into_floating_input(&mut gpioa.crh),
	};
	
	// usb-serialに繋ぐ
	let usb_bus = usb::UsbBus::new(usb_p);
	let mut serial = SerialPort::new(&usb_bus);
	// // ベンダIDとプロダクトIDはテキトー(一応推奨されているものだが、本来は何らかの保証をする必要があるとか)。
	let mut usb_stack = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
		.device_class(USB_CLASS_CDC)  // USB-CDCを使う
		.manufacturer("CRS")  // 以下テキトー
		.product("Test device")
		.serial_number("TEST")
		.build()
	;

	loop {
		let writer = &mut WriteTo::new();

		if usb_stack.poll(&mut [&mut serial]) {
			let res: Result<(), ErrorMessage> = try {
				let mut buf = [0u8; 256];
				match serial.read(&mut buf) {
					Ok(count) if (2 <= count) => {
						let mode = buf[0];

						// 先頭がb'w'かb'r'で、countが奇数であることを確認
						let hoge = assert (
							mode == b'w' || mode == b'r',
							error_message(writer, format_args!("mode invalid. mode must be b'w'({}) or b'r'({}): mode={}", b'w', b'r', mode))
						);
						hoge?;
						assert(
							count % 2 == 1,
							error_message(writer, format_args!("count is invalid. count must be odd: count={}", count))
						)?;

						serial.write(b"EInput seems valid\r\n").ok();

						match mode {
							b'w' => {  // write only mode
								for i in 0..((count - 1) / 2) {
									// 2byteずつbig-endianで送信(というよりは、先頭から順に2byteずつ送信)
									let send_word = (buf[2 * i + 1] as u16) << 8 | buf[2 * i + 2] as u16;
									spi_p.send(send_word)
										.or_else(|e| {
											Err(error_message(writer, format_args! (
												"spi send error: {:?}", e
											)))
										})?;
								}

								// 送れたことを通知
								serial.write(b"OOK\r\n")
									.or_else(|e| {
										Err(error_message(writer, format_args! (
											"seral write error: {:?}", e
										)))
									})?;
							},
							b'r' => {  // write and read
								for i in 0..((count - 1) / 2) {
									// 2byteずつbig-endianで送信(というよりは、先頭から順に2byteずつ送信)
									let send_word = (buf[2 * i + 1] as u16) << 8 | buf[2 * i + 2] as u16;
									spi_p.send(send_word)
										.or_else(|e| {
											Err(error_message(writer, format_args! (
												"spi send error: {:?}", e
											)))
										})?;
									// 受信
									let recv_word = spi_p.read()
										.or_else(|e| {
											Err(error_message(writer, format_args! (
												"spi read error: {:?}", e
											)))
										})?;
									// bufにインプレースで書き込む
									buf[2 * i + 1] = (recv_word >> 8) as u8;
									buf[2 * i + 2] = recv_word as u8;
								}

								// 受信内容を送信(先頭のb'r'は送信しない)
								serial.write(&[b'O']).or_else(|e| {
									Err(error_message(writer, format_args! (
										"seral write error: {:?}", e
									)))
								})?;
								serial.write(&buf[1..count])
									.or_else(|e| {
										Err(error_message(writer, format_args! (
											"seral write error: {:?}", e
										)))
									})?;
							}
							_ => {},
						}
					}
					// 何も読み込めなかった場合は何もしない
					Err(UsbError::WouldBlock) => {},
					// それ以外のエラーが発生した場合はエラーメッセージを送信
					Err(e) => {
						Err(error_message(writer, format_args! (
							"seral read error: {:?}", e
						)))?;
					},
					_ => {},
				}
			};
			res.or_else(|error_mes| {
				serial.write(error_mes.0.as_bytes()).ok();
				serial.write(b"\r\n").ok();
				Ok::<(), ()>(())  // suppress error
			}).ok();
		}
	}
}
