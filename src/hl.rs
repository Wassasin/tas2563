//! High level interface for the TAS2563 chipset providing convenience methods and a Rust-style interface.

use embedded_hal_async::{i2c::I2c, spi::SpiDevice};

use crate::ll::{
    i2c::{Address, I2CInterface},
    spi::SPIInterface,
    Tas2563Device, Tas2563Interface,
};
use crate::prelude::*;

/// High level interface for the TAS2563 chipset.
pub struct Tas2563<T> {
    dev: Tas2563Device<T>,
}

impl<T> Tas2563<I2CInterface<T>>
where
    T: I2c,
{
    pub fn new_i2c(i2c: T, address: Address) -> Self {
        Self {
            dev: Tas2563Device::new_i2c(i2c, address),
        }
    }

    pub fn take(self) -> T {
        self.dev.take()
    }
}

impl<T> Tas2563<SPIInterface<T>>
where
    T: SpiDevice,
{
    pub fn new_spi(spi: T) -> Self {
        Self {
            dev: Tas2563Device::new_spi(spi),
        }
    }

    pub fn take(self) -> T {
        self.dev.take()
    }
}

impl<T> Tas2563<T> {
    /// Get access to the underlying low level device.
    pub fn ll(&mut self) -> &mut Tas2563Device<T> {
        &mut self.dev
    }
}

impl<T> Tas2563<T>
where
    T: Tas2563Interface,
{
    pub async fn amplification_level_or_mute(
        &mut self,
        level: Option<AmpLevel>,
    ) -> Result<(), T::Error> {
        if let Some(level) = level {
            self.dev
                .pb_cfg_1()
                .write_async(|w| w.amp_level(level))
                .await?;

            self.dev
                .pwr_ctl()
                .modify_async(|w| w.mode(Mode::Active))
                .await?;
        } else {
            self.dev
                .pwr_ctl()
                .modify_async(|w| w.mode(Mode::Mute))
                .await?;
        }

        Ok(())
    }

    pub async fn adc(&mut self) -> Result<ADCReadout, T::Error> {
        Ok(ADCReadout {
            pvdd: self.dev.pvdd().read_async().await?.pvdd_cnv_dsp(),
            vbat: self.dev.vbat().read_async().await?.vbat_cnv(),
            temp: self.dev.temp().read_async().await?.tmp_cnv(),
        })
    }

    // /// Configure the LDO pin.
    // pub async fn ldo(&mut self, config: LdoConfig) -> Result<(), I2C::Error> {
    //     self.dev
    //         .ldoctrl()
    //         .write_async(|w| match config {
    //             LdoConfig::Off => w.en_ls_ldo(false),
    //             LdoConfig::Switch => w.en_ls_ldo(true).ldo_switch_config(LdoSwitchConfig::Switch),
    //             LdoConfig::Ldo(v) => w
    //                 .en_ls_ldo(true)
    //                 .ldo_switch_config(LdoSwitchConfig::Ldo)
    //                 .vldo(v),
    //         })
    //         .await
    // }
}
