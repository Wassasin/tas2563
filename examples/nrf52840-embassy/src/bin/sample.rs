#![no_std]
#![no_main]

use core::f32::consts::PI;

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    i2s::{self, Channels, DoubleBuffering, MasterClock, Sample as _, SampleWidth, I2S},
    peripherals,
    twim::{self, Frequency},
};
use embassy_time::{Delay, Duration, Timer};
use embedded_hal_async::i2c::I2c;
use tas2563::{
    ll::{registers::Mode, Tas2563Device},
    prelude::*,
};
use {defmt_rtt as _, panic_probe as _};

const NUM_SAMPLES: usize = 50;
type Sample = i16;

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
    I2S => i2s::InterruptHandler<peripherals::I2S>;
});

// async fn initialize<I2C: I2c>(ll: &mut Tas2505Device<I2C>) -> Result<(), I2C::Error> {
//     // Assert Software reset (P0, R1, D0=1)
//     ll.write_register_direct(0x00, 0x01, 0x01).await?;
//     // LDO output programmed as 1.8V and Level shifters powered up. (P1, R2, D5-D4=00, D3=0)
//     ll.write_register_direct(0x01, 0x02, 0x00).await?;
//     // PLL_clkin = MCLK, codec_clkin = PLL_CLK, MCLK should be 11.2896MHz (P0, R4, D1-D0=03)
//     ll.write_register_direct(0x00, 0x04, 0x03).await?;
//     // Power up PLL, set P=1, R=1, (Page-0, Reg-5)
//     ll.write_register_direct(0x00, 0x05, 0x91).await?;
//     // Set J=4, (Page-0, Reg-6)
//     ll.write_register_direct(0x00, 0x06, 0x04).await?;
//     // D = 0000, D(13:8) = 0, (Page-0, Reg-7)
//     ll.write_register_direct(0x00, 0x07, 0x00).await?;
//     // D(7:0) = 0, (Page-0, Reg-8)
//     ll.write_register_direct(0x00, 0x08, 0x00).await?;
//     // add delay of 15 ms for PLL to lock
//     Timer::after(Duration::from_millis(15)).await;
//     // DAC NDAC Powered up, NDAC=4 (P0, R11, D7=1, D6-D0=0000100)
//     ll.write_register_direct(0x00, 0x0B, 0x84).await?;
//     // DAC MDAC Powered up, MDAC=2 (P0, R12, D7=1, D6-D0=0000010)
//     ll.write_register_direct(0x00, 0x0C, 0x82).await?;
//     // DAC OSR(9:0)-> DOSR=128 (P0, R12, D1-D0=00)
//     ll.write_register_direct(0x00, 0x0D, 0x00).await?;
//     // DAC OSR(9:0)-> DOSR=128 (P0, R13, D7-D0=10000000)
//     ll.write_register_direct(0x00, 0x0E, 0x80).await?;
//     // Codec Interface control Word length = 16bits, BCLK&WCLK inputs, I2S mode. (P0, R27, D7-D6=00, D5-D4=00, D3-D2=00)
//     ll.write_register_direct(0x00, 0x1B, 0x00).await?;
//     // Data slot offset 00 (P0, R28, D7-D0=0000)
//     ll.write_register_direct(0x00, 0x1C, 0x00).await?;
//     // Dac Instruction programming PRB #2 for Mono routing. Type interpolation (x8) and 3 programmable Biquads. (P0, R60, D4-D0=0010)
//     ll.write_register_direct(0x00, 0x3C, 0x02).await?;
//     // DAC powered up, Soft step 1 per Fs. (P0, R63, D7=1, D5-D4=01, D3-D2=00, D1-D0=00)
//     ll.write_register_direct(0x00, 0x3F, 0x90).await?;
//     // DAC digital gain 0dB (P0, R65, D7-D0=00000000)
//     ll.write_register_direct(0x00, 0x41, 0x00).await?;
//     // DAC volume not muted. (P0, R64, D3=0, D2=1)
//     ll.write_register_direct(0x00, 0x40, 0x04).await?;
//     // Master Reference Powered on (P1, R1, D4=1)
//     ll.write_register_direct(0x01, 0x01, 0x10).await?;
//     // Output common mode for DAC set to 0.9V (default) (P1, R10)
//     ll.write_register_direct(0x01, 0x0A, 0x00).await?;
//     // Mixer P output is connected to HP Out Mixer (P1, R12, D2=1)
//     ll.write_register_direct(0x01, 0x0C, 0x04).await?;
//     // HP Volume, 0dB Gain (P1, R22, D6-D0=0000000)
//     ll.write_register_direct(0x01, 0x16, 0x00).await?;
//     // No need to enable Mixer M and Mixer P, AINL Voulme, 0dB Gain (P1, R24, D7=1, D6-D0=0000000)
//     ll.write_register_direct(0x01, 0x18, 0x00).await?;
//     // Power up HP (P1, R9, D5=1)
//     ll.write_register_direct(0x01, 0x09, 0x20).await?;
//     // Unmute HP with 0dB gain (P1, R16, D4=1)
//     ll.write_register_direct(0x01, 0x10, 0x00).await?;
//     // SPK attn. Gain =0dB (P1, R46, D6-D0=000000) (0x00)
//     ll.write_register_direct(0x01, 0x2E, 0x00).await?;
//     // SPK driver Gain=6.0dB (P1, R48, D6-D4=001) (0x10)
//     ll.write_register_direct(0x01, 0x30, 0x10).await?;
//     // SPK powered up (P1, R45, D1=1)
//     ll.write_register_direct(0x01, 0x2D, 0x02).await?;

//     Ok(())
// }

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("running!");

    let sda = p.P1_01;
    let scl = p.P1_02;

    let mck = p.P1_03;
    let bck_sck = p.P1_04;
    let wck_lrck = p.P1_05;
    let din = p.P1_06;

    let mut config = twim::Config::default();
    config.frequency = Frequency::K400;
    config.sda_pullup = true;
    config.scl_pullup = true;
    let twim = twim::Twim::new(p.TWISPI0, Irqs, sda, scl, config);

    let master_clock: MasterClock = i2s::ApproxSampleRate::_48000.into();

    let sample_rate = master_clock.sample_rate();
    info!("Sample rate: {}", sample_rate);

    let mut config = i2s::Config::default();
    config.sample_width = SampleWidth::_16bit;
    config.channels = Channels::MonoLeft;

    let buffers = DoubleBuffering::<Sample, NUM_SAMPLES>::new();
    // let mut output_stream =
    //     I2S::new_master(p.I2S, Irqs, mck, bck_sck, wck_lrck, master_clock, config)
    //         .output(din, buffers);

    let mut ll = tas2563::ll::Tas2563Device::new(twim, tas2563::ll::Address::Global);

    ll.software_reset()
        .write_async(|w| w.software_reset(true))
        .await
        .unwrap();

    let v = ll.software_reset().read_async().await.unwrap();
    defmt::info!("{:?}", defmt::Debug2Format(&v));

    let v = ll.rev_id().read_async().await.unwrap();
    defmt::info!("{:?}", defmt::Debug2Format(&v));

    ll.bst_ilim_cfg_0()
        .write_async(|w| w.bst_ilim(0x00))
        .await
        .unwrap();

    ll.tg_cfg_0()
        .modify_async(|w| w.tg_1_en(tas2563::ll::registers::Tg1En::Enabled))
        .await
        .unwrap();

    ll.pwr_ctl()
        .modify_async(|w| w.vsns_pd(true).mode(Mode::LoadDiagnosticsActive))
        .await
        .unwrap();

    loop {
        let v = ll.temp().read_async().await.unwrap();
        defmt::info!("{:?}", defmt::Debug2Format(&v));
        let v = ll.vbat().read_async().await.unwrap();
        defmt::info!("{:?}", defmt::Debug2Format(&v));
        let v = ll.pvdd().read_async().await.unwrap();
        defmt::info!("{:?}", defmt::Debug2Format(&v));
        Timer::after_secs(1).await;
    }

    // initialize(&mut ll).await.unwrap();

    // let mut waveform = Waveform::new(1.0 / sample_rate as f32);
    // waveform.process(output_stream.buffer());

    // const SAMPLE: &'static [u8] = include_bytes!("../../../samples/windows95.raw");

    // output_stream.start().await.expect("I2S Start");

    // let mut i = 0;
    // loop {
    //     for j in output_stream.buffer() {
    //         *j = i16::from_le_bytes([SAMPLE[i], SAMPLE[i + 1]]);
    //         i = (i + 2) % SAMPLE.len();
    //     }

    //     // waveform.process(output_stream.buffer());

    //     if let Err(err) = output_stream.send().await {
    //         defmt::error!("{}", err);
    //     }
    // }
}

struct Waveform {
    inv_sample_rate: f32,
    carrier: SineOsc,
    freq_mod: SineOsc,
    amp_mod: SineOsc,
}

impl Waveform {
    fn new(inv_sample_rate: f32) -> Self {
        let mut carrier = SineOsc::new();
        carrier.set_frequency(110.0, inv_sample_rate);

        let mut freq_mod = SineOsc::new();
        freq_mod.set_frequency(1.0, inv_sample_rate);
        freq_mod.set_amplitude(1.0);

        let mut amp_mod = SineOsc::new();
        amp_mod.set_frequency(16.0, inv_sample_rate);
        amp_mod.set_amplitude(0.5);

        Self {
            inv_sample_rate,
            carrier,
            freq_mod,
            amp_mod,
        }
    }

    fn process(&mut self, buf: &mut [Sample]) {
        for sample in buf.chunks_mut(1) {
            let freq_modulation = bipolar_to_unipolar(self.freq_mod.generate());
            self.carrier
                .set_frequency(110.0 + 440.0 * freq_modulation, self.inv_sample_rate);

            let amp_modulation = bipolar_to_unipolar(self.amp_mod.generate());
            self.carrier.set_amplitude(amp_modulation);

            let signal = self.carrier.generate();

            sample[0] = (Sample::SCALE as f32 * signal) as Sample;
        }
    }
}

struct SineOsc {
    amplitude: f32,
    modulo: f32,
    phase_inc: f32,
}

impl SineOsc {
    const B: f32 = 4.0 / PI;
    const C: f32 = -4.0 / (PI * PI);
    const P: f32 = 0.225;

    pub fn new() -> Self {
        Self {
            amplitude: 1.0,
            modulo: 0.0,
            phase_inc: 0.0,
        }
    }

    pub fn set_frequency(&mut self, freq: f32, inv_sample_rate: f32) {
        self.phase_inc = freq * inv_sample_rate;
    }

    pub fn set_amplitude(&mut self, amplitude: f32) {
        self.amplitude = amplitude;
    }

    pub fn generate(&mut self) -> f32 {
        let signal = self.parabolic_sin(self.modulo);
        self.modulo += self.phase_inc;
        if self.modulo < 0.0 {
            self.modulo += 1.0;
        } else if self.modulo > 1.0 {
            self.modulo -= 1.0;
        }
        signal * self.amplitude
    }

    fn parabolic_sin(&mut self, modulo: f32) -> f32 {
        let angle = PI - modulo * 2.0 * PI;
        let y = Self::B * angle + Self::C * angle * abs(angle);
        Self::P * (y * abs(y) - y) + y
    }
}

#[inline]
fn abs(value: f32) -> f32 {
    if value < 0.0 {
        -value
    } else {
        value
    }
}

#[inline]
fn bipolar_to_unipolar(value: f32) -> f32 {
    (value + 1.0) / 2.0
}
