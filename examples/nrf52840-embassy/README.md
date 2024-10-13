Tested with the devkits of the [NRF52840](https://docs.nordicsemi.com/bundle/ug_nrf52840_dk/page/UG/dk/intro.html) and TAS2563 ([TAS2X63EVM](https://www.ti.com/tool/TAS2X63EVM)). Using dupont cables to connect:

* P1.01: SCL
* P1.02: SDA
* P1.03: SBCLK (BCK_SCK)
* P1.04: FSYNC (WCK_LRCK)
* P1.05: SDIN (DIN)
* P1.06: GPIO (MCK)
* GND

The configuration for the TAS2563 devkit jumpers for these examples is as follows:

* I2S: EXT (J27)
* I2C: EXT (J29)
* ShutDown: SW CTRL (J19)
* IOVDD: 3.3V (J18)
* VCC-IO: 3.3V (J3)
* CTRL SEL: I2C (J21)
* SS SEL: SSO (J14)
* SPI CTRL: DISABLE (J5)
* All other jumpers connected

VBAT was powered using an external power supply at my minimal battery voltage of 2.5V. Speaker connected was the stock speaker in the TAS2X63EVM.

## How to run
```bash
cargo run --release
```