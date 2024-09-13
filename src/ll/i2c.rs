use embedded_hal::i2c::Operation;
use embedded_hal_async::i2c::I2c;

use super::{Tas2563Device, Tas2563Interface};

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Address {
    Global = 0x48,
    Address0x4C = 0x4C,
    Address0x4D = 0x4D,
    Address0x4E = 0x4E,
    Address0x4F = 0x4F,
}

pub struct I2CInterface<T: I2c> {
    address: Address,
    i2c: T,
}

impl<T: I2c> Tas2563Interface for I2CInterface<T> {
    type Error = T::Error;

    async fn write_burst(&mut self, data: &[u8]) -> Result<(), T::Error> {
        self.i2c.write(self.address as u8, data).await
    }

    async fn read_registers(&mut self, register: u8, values: &mut [u8]) -> Result<(), T::Error> {
        self.i2c
            .write_read(self.address as u8, &[register], values)
            .await
    }
}

impl<T> Tas2563Device<I2CInterface<T>>
where
    T: I2c,
{
    pub fn new_i2c(i2c: T, address: Address) -> Self {
        Self {
            iface: I2CInterface { i2c, address },
            last_page: None,
            last_book: None,
        }
    }

    pub fn take(self) -> T {
        self.iface.i2c
    }
}
