use embedded_hal_async::i2c::I2c;

use super::{Tas2563Device, Tas2563Interface, MAX_TRANSACTION_SIZE};

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

    async fn write(&mut self, register: u8, data: &[u8]) -> Result<(), T::Error> {
        let mut buf: [u8; 3] = [0u8; MAX_TRANSACTION_SIZE];
        buf[0] = register;
        buf[1..data.len() + 1].copy_from_slice(data);
        let buf = &buf[0..data.len() + 1];
        self.i2c.write(self.address as u8, buf).await
    }

    async fn read(&mut self, register: u8, data: &mut [u8]) -> Result<(), T::Error> {
        self.i2c
            .write_read(self.address as u8, &[register], data)
            .await
    }
}

impl<T> Tas2563Device<I2CInterface<T>>
where
    T: I2c,
{
    pub fn new_i2c(i2c: T, address: Address) -> Self {
        Self {
            interface: I2CInterface { i2c, address },
            last_page: None,
            last_book: None,
        }
    }

    pub fn take(self) -> T {
        self.interface.i2c
    }
}
