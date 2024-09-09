//! Low level interface for the TAS2563 chipset providing register access.

pub mod i2c;

#[cfg(test)]
mod test;

use bitvec::array::BitArray;
use device_driver::{AddressableDevice, AsyncRegisterDevice};

const MAX_TRANSACTION_SIZE: usize = 3;

pub struct Tas2563Device<T> {
    interface: T,
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

pub trait Tas2563Interface {
    type Error;

    async fn write(&mut self, register: u8, data: &[u8]) -> Result<(), Self::Error>;
    async fn read(&mut self, register: u8, data: &mut [u8]) -> Result<(), Self::Error>;
}

impl<T> AddressableDevice for Tas2563Device<T> {
    type AddressType = u32;
}

impl<T> Tas2563Device<T>
where
    T: Tas2563Interface,
{
    async fn ensure_book_page(&mut self, address: &RegisterAddress) -> Result<(), T::Error> {
        if self.last_page != Some(address.page) {
            self.interface.write(0x00, &[address.page]).await?;
            self.last_page = Some(address.page);
        }
        if self.last_book != Some(address.book) {
            self.interface.write(0x7f, &[address.book]).await?;
            self.last_book = Some(address.book);
        }
        Ok(())
    }
}

impl<T> AsyncRegisterDevice for Tas2563Device<T>
where
    T: Tas2563Interface,
{
    type Error = T::Error;

    async fn write_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        let address = RegisterAddress::from(address);
        self.ensure_book_page(&address).await?;

        self.interface
            .write(address.register, data.as_raw_slice())
            .await
    }

    async fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        let address = RegisterAddress::from(address);
        self.ensure_book_page(&address).await?;

        self.interface
            .read(address.register, data.as_raw_mut_slice())
            .await
    }
}

impl<T> Tas2563Device<T>
where
    Self: AsyncRegisterDevice + AddressableDevice<AddressType = u32>,
{
    pub async fn write_register_direct(
        &mut self,
        book: u8,
        page: u8,
        register: u8,
        value: u8,
    ) -> Result<(), <Self as AsyncRegisterDevice>::Error> {
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
    use crate::prelude::*;

    #[device_driver_macros::implement_device_from_file(yaml = "src/ll/ll.yaml")]
    impl<T> Tas2563Device<T> {}
}
