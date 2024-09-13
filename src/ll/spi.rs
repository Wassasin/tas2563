use embedded_hal_async::spi::SpiDevice;

use super::{Tas2563Device, Tas2563Interface};

pub struct SPIInterface<T: SpiDevice> {
    spi: T,
}

impl<T: SpiDevice> Tas2563Interface for SPIInterface<T> {
    type Error = T::Error;

    async fn write_burst(&mut self, data: &[u8]) -> Result<(), T::Error> {
        let mut register = data[0];
        for b in &data[1..] {
            self.write_register(register, *b).await?;
            register += 1;
        }
        Ok(())
    }

    async fn read_registers(&mut self, mut register: u8, data: &mut [u8]) -> Result<(), T::Error> {
        let mut buf = [0u8; 2];
        for b in data {
            buf[0] = register << 1 | 0b1;
            buf[1] = 0x00;
            self.spi.transfer_in_place(&mut buf).await?;
            *b = buf[1];
            register += 1;
        }
        Ok(())
    }

    async fn write_register(&mut self, register: u8, value: u8) -> Result<(), Self::Error> {
        self.spi.write(&[register << 1, value]).await
    }
}

impl<T> Tas2563Device<SPIInterface<T>>
where
    T: SpiDevice,
{
    pub fn new_spi(spi: T) -> Self {
        Self {
            iface: SPIInterface { spi },
            last_page: None,
            last_book: None,
        }
    }

    pub fn take(self) -> T {
        self.iface.spi
    }
}
