# WARNING: This project is still in development and is not yet ready for use.全然できてないよ！！

# TODO
- 真面目なicm42688のドライバクレートと、仲間内で使うための説明や日本語を分離する

# icm42688-portable
A portable driver for the ICM42688 IMU sensor.
This driver is based on the embedded-hal and embedded-hal-async traits, and is designed to be used with any platform that implements these traits.
At present, only support for SPI is considered.

## Acknowledgements
This project was inspired by [icm42688 by oldsheep68](https://github.com/oldsheep68/icm42688).

## License
This project is licensed under the MIT License.
Additionally, `examples/usb_test.rs` is based on code from the `cortex-m-quickstart`, which is also under the MIT License.
