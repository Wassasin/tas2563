use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

use super::{Address, Tas2563Device};
use crate::prelude::*;

fn reg(register: u8, value: u8) -> Transaction {
    Transaction::write(Address::Global as u8, vec![register, value])
}

#[async_std::test]
async fn example() {
    let expectations = [
        // Set page to 0x00
        reg(0x00, 0x00),
        // Set book to 0x00
        reg(0x7f, 0x00),
        // Mute and power sensing up
        reg(0x02, 0x0d),
        // Software shutdown and power sensing up
        reg(0x02, 0x0e),
        // Software reset
        reg(0x01, 0x01),
        // reg(0x03, 0x00),
        // reg(0x01, 0x80),
        // reg(0x02, 0x02),
        // reg(0x03, 0x20),
        // reg(0x04, 0xf6),
        // reg(0x06, 0x09),
        // reg(0x07, 0x02),
        // reg(0x08, 0x7a),
        // reg(0x09, 0x10),
        // reg(0x0a, 0x03),
        // reg(0x0b, 0x44),
        // reg(0x0c, 0x40),
        // reg(0x0d, 0x04),
        // reg(0x0e, 0x05),
        // reg(0x0f, 0x06),
        // reg(0x10, 0x07),
        // reg(0x12, 0x13),
        // reg(0x13, 0x76),
        // reg(0x14, 0x01),
        // reg(0x15, 0x2e),
        // reg(0x1a, 0xfc),
        // reg(0x1b, 0xa6),
        // reg(0x1c, 0xdf),
        // reg(0x1d, 0xff),
        // reg(0x30, 0x19),
        // reg(0x31, 0x40),
        // reg(0x32, 0x81),
        // reg(0x33, 0x34),
        // reg(0x34, 0x46),
        // reg(0x35, 0x84),
        // reg(0x38, 0x20),
        // reg(0x3b, 0x38),
        // reg(0x3c, 0x38),
        // reg(0x3d, 0x08),
        // reg(0x3e, 0x10),
        // reg(0x3f, 0x00),
        // reg(0x40, 0x40),
        // reg(0x30, 0x19),
        // reg(0x02, 0x00),
        // reg(0x01, 0x80),
        // reg(0x02, 0x00),
        // reg(0x03, 0x20),
        // reg(0x04, 0xf6),
        // reg(0x06, 0x09),
        // reg(0x07, 0x02),
        // reg(0x08, 0x7a),
        // reg(0x09, 0x10),
        // reg(0x0a, 0x03),
        // reg(0x0b, 0x44),
        // reg(0x0c, 0x40),
        // reg(0x0d, 0x04),
        // reg(0x0e, 0x05),
        // reg(0x0f, 0x06),
        // reg(0x10, 0x07),
        // reg(0x12, 0x13),
        // reg(0x13, 0x76),
        // reg(0x14, 0x01),
        // reg(0x15, 0x2e),
        // reg(0x1a, 0xfc),
        // reg(0x1b, 0xa6),
        // reg(0x1c, 0xdf),
        // reg(0x1d, 0xff),
        // reg(0x30, 0x19),
        // reg(0x31, 0x40),
        // reg(0x32, 0x81),
        // reg(0x33, 0x34),
        // reg(0x34, 0x46),
        // reg(0x35, 0x84),
        // reg(0x38, 0x20),
        // reg(0x3b, 0x38),
        // reg(0x3c, 0x38),
        // reg(0x3d, 0x08),
        // reg(0x3e, 0x10),
        // reg(0x3f, 0x00),
        // reg(0x40, 0x40),
    ];
    let mut i2c = Mock::new(&expectations);

    let mut ll = Tas2563Device::new(&mut i2c, Address::Global);
    ll.pwr_ctl()
        .write_async(|w| w.mode(Mode::Mute).vsns_pd(true).isns_pd(true))
        .await
        .unwrap();

    ll.pwr_ctl()
        .write_async(|w| w.mode(Mode::SoftwareShutdown).vsns_pd(true).isns_pd(true))
        .await
        .unwrap();

    ll.software_reset()
        .write_async(|w| w.software_reset(true))
        .await
        .unwrap();

    i2c.done();
}
