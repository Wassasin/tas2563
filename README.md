# TAS2563
Rust no-std device driver for the Texas Instruments [TAS2563](https://www.ti.com/product/TAS2563) smart amplifier.

## Support
* All publicly known low level registers

## Do I still need the configuration desktop software?
Yes, you still need to measure, calibrate and tune your specific speaker for your specific application. The configuration files that are generated by the proprietary desktop app can be fed to this driver. This driver can enable ROM-mode for debugging purposes, or if your speaker is in no danger of being damaged under maximum settings. (meaning not a microspeaker)

## Example
See the `examples/` folder in this repository on how to use the library crate.
