//! Low level interface for the TAS2563 chipset providing register access.

#[cfg(test)]
mod test;

use bitvec::array::BitArray;
use device_driver::{AddressableDevice, AsyncRegisterDevice};
use embedded_hal_async::i2c::I2c;

const MAX_TRANSACTION_SIZE: usize = 3;

#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Address {
    Global = 0x48,
    Address0x4C = 0x4C,
    Address0x4D = 0x4D,
    Address0x4E = 0x4E,
    Address0x4F = 0x4F,
}

pub struct Tas2563Device<I2C> {
    address: Address,
    i2c: I2C,
    last_page: Option<u8>,
    last_book: Option<u8>,
}

pub struct RegisterAddress {
    book: u8,
    page: u8,
    register: u8,
}

impl From<u32> for RegisterAddress {
    fn from(value: u32) -> Self {
        let [_, book, page, register] = value.to_be_bytes();
        RegisterAddress {
            book,
            page,
            register,
        }
    }
}

impl Into<u32> for RegisterAddress {
    fn into(self) -> u32 {
        u32::from_be_bytes([0x00, self.book, self.page, self.register])
    }
}

impl<I2C> AddressableDevice for Tas2563Device<I2C> {
    type AddressType = u32;
}

impl<I2C> AsyncRegisterDevice for Tas2563Device<I2C>
where
    I2C: I2c,
{
    type Error = I2C::Error;

    async fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        let address = RegisterAddress::from(address);
        self.ensure_book_page(&address).await?;

        let data = data.as_raw_slice();

        let mut buf = [0u8; MAX_TRANSACTION_SIZE];
        buf[0] = address.register;
        buf[1..data.len() + 1].copy_from_slice(data);
        let buf = &buf[0..data.len() + 1];

        self.i2c.write(self.address as u8, buf).await
    }

    async fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        let address = RegisterAddress::from(address);
        self.ensure_book_page(&address).await?;

        self.i2c
            .write_read(
                self.address as u8,
                &[address.register],
                data.as_raw_mut_slice(),
            )
            .await
    }
}

impl<I2C> Tas2563Device<I2C>
where
    I2C: I2c,
{
    pub fn new(i2c: I2C, address: Address) -> Self {
        Self {
            i2c,
            address,
            last_page: None,
            last_book: None,
        }
    }

    pub fn take(self) -> I2C {
        self.i2c
    }

    async fn ensure_book_page(&mut self, address: &RegisterAddress) -> Result<(), I2C::Error> {
        if self.last_page != Some(address.page) {
            self.i2c
                .write(self.address as u8, &[0x00, address.page])
                .await?;
            self.last_page = Some(address.page);
        }
        if self.last_book != Some(address.book) {
            self.i2c
                .write(self.address as u8, &[0x7f, address.book])
                .await?;
            self.last_book = Some(address.book);
        }
        Ok(())
    }

    pub async fn write_register_direct(
        &mut self,
        book: u8,
        page: u8,
        register: u8,
        value: u8,
    ) -> Result<(), I2C::Error> {
        self.write_register(
            RegisterAddress {
                book,
                page,
                register,
            }
            .into(),
            &BitArray::new([value]),
        )
        .await
    }

    pub fn reset_assumptions(&mut self) {
        self.last_book = None;
        self.last_page = None;
    }
}

pub mod registers {
    use super::*;
    // use crate::prelude::*;

    #[device_driver_macros::implement_device_from_file(yaml = "src/ll/ll.yaml")]
    impl<I2C> Tas2563Device<I2C> {}
}
