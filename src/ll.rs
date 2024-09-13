//! Low level interface for the TAS2563 chipset providing register access.

pub mod i2c;
pub mod spi;

#[cfg(test)]
mod test;

use bitvec::array::BitArray;
use device_driver::{AddressableDevice, AsyncRegisterDevice};

pub struct Tas2563Device<T> {
    iface: T,
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

    /// Write data in burst to the peripheral.
    ///
    /// The first element in data is the first register address to write to.
    /// If the interface does not support burst write, it is required to unwrap the burst write into
    /// separate single register writes.
    async fn write_burst(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Read a series of register values from the peripheral.
    ///
    /// If the interface does not support multibyte read, it is required to unwrap the read into
    /// separate single register reads.
    async fn read_registers(&mut self, register: u8, values: &mut [u8]) -> Result<(), Self::Error>;

    /// Convenience function to create a single register write.
    async fn write_register(&mut self, register: u8, value: u8) -> Result<(), Self::Error> {
        self.write_burst(&[register, value]).await
    }

    /// Convenience function to create a multi register write.
    async fn write_registers(
        &mut self,
        mut register: u8,
        values: &[u8],
    ) -> Result<(), Self::Error> {
        for v in values {
            self.write_register(register, *v).await?;
            register += 1;
        }
        Ok(())
    }
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
            self.iface.write_register(0x00, address.page).await?;
            self.last_page = Some(address.page);
        }
        if self.last_book != Some(address.book) {
            self.iface.write_register(0x7f, address.book).await?;
            self.last_book = Some(address.book);
        }
        Ok(())
    }

    pub fn interface(&mut self) -> &mut T {
        &mut self.iface
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

        self.iface
            .write_registers(address.register, data.as_raw_slice())
            .await
    }

    async fn read_register<const SIZE_BYTES: usize>(
        &mut self,
        address: Self::AddressType,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error> {
        let address = RegisterAddress::from(address);
        self.ensure_book_page(&address).await?;

        self.iface
            .read_registers(address.register, data.as_raw_mut_slice())
            .await
    }
}

impl<T> Tas2563Device<T>
where
    Self: AsyncRegisterDevice + AddressableDevice<AddressType = u32>,
{
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
