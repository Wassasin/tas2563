#![no_std]
#![no_main]

use core::f32::consts::PI;

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    i2s::{self, Channels, DoubleBuffering, MasterClock, Sample as _, SampleWidth, I2S},
    peripherals,
    twim::{self, Frequency},
};
use embassy_time::Timer;
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

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("running!");

    let scl = p.P1_01;
    let sda = p.P1_02;

    let sbclk = p.P1_03;
    let fsync = p.P1_04;
    let sdin = p.P1_05;
    let gpio = p.P1_06;

    let mck = gpio;
    let bck_sck = sbclk;
    let wck_lrck = fsync;
    let din = sdin;

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
    let mut output_stream =
        I2S::new_master(p.I2S, Irqs, mck, bck_sck, wck_lrck, master_clock, config)
            .output(din, buffers);

    let mut hl = tas2563::hl::Tas2563::new_i2c(twim, tas2563::ll::i2c::Address::Global);

    {
        let ll = hl.ll();

        ll.software_reset()
            .write_async(|w| w.software_reset(true))
            .await
            .unwrap();

        let v = ll.software_reset().read_async().await.unwrap();
        defmt::info!("{:?}", defmt::Debug2Format(&v));

        let v = ll.rev_id().read_async().await.unwrap();
        defmt::info!("{:?}", defmt::Debug2Format(&v));

        ll.pwr_ctl()
            .write_async(|w| w.mode(Mode::Mute).vsns_pd(true).isns_pd(true))
            .await
            .unwrap();

        ll.pwr_ctl()
            .write_async(|w| w.mode(Mode::SoftwareShutdown).vsns_pd(true).isns_pd(true))
            .await
            .unwrap();

        ll.software_reset()
            .write_async(|w| w.software_reset(true))
            .await
            .unwrap();

        ll.pb_cfg_1()
            .write_async(|w| w.dis_dc_blocker(false).amp_level(AmpLevel::Unknown))
            .await
            .unwrap();

        ll.pwr_ctl()
            .write_async(|w| w.mode(Mode::SoftwareShutdown).vsns_pd(false).isns_pd(false))
            .await
            .unwrap();

        ll.pb_cfg_1()
            .write_async(|w| w.dis_dc_blocker(false).amp_level(AmpLevel::Amp16DBv0))
            .await
            .unwrap();

        ll.misc_cfg_1()
            .write_async(|w| {
                w.amp_ss(true)
                    .irqz_pu(false)
                    .ote_retry(true)
                    .oce_retry(true)
                    .vbat_por_retry(true)
                    .cp_pg_retry(true)
            })
            .await
            .unwrap();

        ll.tdm_cfg_0()
            .write_async(|w| {
                w.frame_start(FrameStart::HighToLow)
                    .samp_rate(SampRate::Rate48Khz)
            })
            .await
            .unwrap();

        ll.tdm_cfg_1()
            .write_async(|w| w.rx_offset(0x01))
            .await
            .unwrap();

        ll.tdm_cfg_2()
            .write_async(|w| {
                w.rx_slen(RxSlen::Length32Bits)
                    .rx_wlen(RxWlen::Length24Bits)
                    .rx_scfg(RxScfg::StereoDownmix)
                    .ivmon_len(IvmonLen::Length16Bits)
            })
            .await
            .unwrap();

        ll.tdm_cfg_3()
            .write_async(|w| w.rx_slot_l(0x0).rx_slot_r(0x1))
            .await
            .unwrap();

        ll.tdm_cfg_4()
            .write_async(|w| {
                w.tx_edge(TxEdge::FallingEdge)
                    .tx_offset(0x1)
                    .tx_fill(TxFill::Transmit0)
            })
            .await
            .unwrap();

        ll.tdm_cfg_5()
            .write_async(|w| w.vsns_slot(0x4).vsns_tx(true))
            .await
            .unwrap();

        ll.tdm_cfg_6()
            .write_async(|w| w.isns_slot(0x0).isns_tx(true))
            .await
            .unwrap();

        ll.tdm_cfg_7()
            .write_async(|w| w.vbat_slot(0x4).vbat_tx(false))
            .await
            .unwrap();

        ll.tdm_cfg_8()
            .write_async(|w| w.temp_slot(0x5).temp_tx(false))
            .await
            .unwrap();

        ll.tdm_cfg_9()
            .write_async(|w| w.gain_slot(0x6).gain_tx(false))
            .await
            .unwrap();

        ll.tdm_cfg_10()
            .write_async(|w| w.bst_slot(0x7).bst_tx(false))
            .await
            .unwrap();

        ll.lim_cfg_0()
            .write_async(|w| {
                w.limb_en(true)
                    .limb_atk_rt(LimbAtkRt::Step2Samples)
                    .limb_atk_st(LimbAtkSt::Step0DB5)
            })
            .await
            .unwrap();

        ll.lim_cfg_1()
            .write_async(|w| {
                w.limb_hld_tm(LimbHldTm::Time500Ms)
                    .limb_rls_rt(LimbRlsRt::Step640Ms)
                    .limb_rls_st(LimbRlsSt::Step0DB5)
            })
            .await
            .unwrap();

        ll.dsp_frequency_bop_cfg_0()
            .write_async(|w| w.bop_en(true))
            .await
            .unwrap();

        ll.bop_cfg_0()
            .write_async(|w| {
                w.bop_hld_tm(BopHldTm::Time500Ms)
                    .bop_atk_st(BopAtkSt::Step1DB0)
                    .bop_atk_rt(BopAtkRt::Step2Samples)
            })
            .await
            .unwrap();

        ll.int_mask_0().write_async(|w| w).await.unwrap();
        ll.int_mask_1().write_async(|w| w).await.unwrap();
        ll.int_mask_2().write_async(|w| w).await.unwrap();
        ll.int_mask_3().write_async(|w| w).await.unwrap();

        ll.int_clk_cfg()
            .write_async(|w| w.irqz_pin_cfg(IrqzPinCfg::UnmaskedLatched))
            .await
            .unwrap();

        ll.din_pd().write_async(|w| w).await.unwrap();

        ll.misc()
            .write_async(|w| w.irqz_val(true).irqz_pol(IrqzPol::ActiveLow))
            .await
            .unwrap();

        ll.boost_cfg_1()
            .write_async(|w| {
                w.bst_dynamic_ilim_en(false)
                    .bst_pfml(BstPfml::Frequency50Khz)
                    .bst_en(true)
                    .bst_mode(BstMode::ClassH)
            })
            .await
            .unwrap();

        ll.boost_cfg_2()
            .write_async(|w| {
                w.bst_vreg(BstVreg::Boost8V5)
                    .bst_pa(BstPa::Phase0Deg)
                    .bst_sync(false)
                    .bst_ir(BstIr::Between0UH6And1UH3)
            })
            .await
            .unwrap();

        ll.boost_cfg_3()
            .write_async(|w| {
                w.bst_lr(BstLr::LoadReg1V03APerV)
                    .bst_class_h_step_time(BstClassHStepTime::Step162Us)
            })
            .await
            .unwrap();

        ll.clock_configuration()
            .write_async(|w| {
                w.auto_clk(false)
                    .sbclk_fs_ratio(0b1000)
                    .sel_madc_div_rev(0x0)
                    .inv_dac_out_phase(false)
            })
            .await
            .unwrap();

        ll.ramp_frame_select()
            .write_async(|w| {
                w.sar_vbat_avg_config(0b11)
                    .vbat_dsp_lpf_reg(0b01)
                    .haptic_en(false)
            })
            .await
            .unwrap();

        ll.retry_timer().write_async(|w| w).await.unwrap();
        ll.hold_sar_update().write_async(|w| w).await.unwrap();
        ll.idle_channel().write_async(|w| w).await.unwrap();

        ll.tg_cfg_0()
            .write_async(|w| {
                w.tg_1_pinen(Tg1Pinen::Disabled)
                    .tg_1_en(Tg1En::PinTriggered)
            })
            .await
            .unwrap();

        ll.bst_ilim_cfg_0()
            .write_async(|w| w.bst_ilim(BoostPeakCurrentMaxRun(0)).bst_ssl(0x1))
            .await
            .unwrap();
    }

    tas2563::bulk::CommandIterator::new(include_bytes!(
        "../../../../util/cfgtransform/example/program_0_Tuning Mode.bulk"
    ))
    .write(hl.ll())
    .await
    .unwrap();

    tas2563::bulk::CommandIterator::new(include_bytes!(
        "../../../../util/cfgtransform/example/configuration_0_TuningMode_48KHz_DEV_A_COEFF.bulk"
    ))
    .write(hl.ll())
    .await
    .unwrap();

    defmt::info!("Configuration loaded");

    hl.ll().reset_assumptions();

    hl.ll()
        .tdm_cfg_2()
        .write_async(|w| {
            w.rx_slen(RxSlen::Length32Bits)
                .rx_wlen(RxWlen::Length24Bits)
                .rx_scfg(RxScfg::MonoLeftChannel)
                .ivmon_len(IvmonLen::Length16Bits)
        })
        .await
        .unwrap();

    hl.ll()
        .pb_cfg_1()
        .write_async(|w| w.dis_dc_blocker(false).amp_level(AmpLevel::Amp16DBv0))
        .await
        .unwrap();

    hl.ll()
        .pwr_ctl()
        .modify_async(|w| w.vsns_pd(false).isns_pd(false).mode(Mode::Active))
        .await
        .unwrap();

    defmt::info!("Speaker activated");

    // loop {
    //     let adc = hl.adc().await.unwrap();
    //     defmt::info!("{}", defmt::Debug2Format(&adc));

    //     let adc: ADCReadOutReadable = adc.into();
    //     defmt::info!("{}", defmt::Debug2Format(&adc));

    //     Timer::after_secs(1).await;
    // }

    // initialize(&mut ll).await.unwrap();

    // let mut waveform = Waveform::new(1.0 / sample_rate as f32);
    // waveform.process(output_stream.buffer());

    const SAMPLE: &'static [u8] = include_bytes!("../../../samples/windows-xp.raw");

    output_stream.start().await.expect("I2S Start");

    let mut i = 0;
    loop {
        for j in output_stream.buffer() {
            *j = i16::from_le_bytes([SAMPLE[i], SAMPLE[i + 1]]);
            i = (i + 2) % SAMPLE.len();
        }

        // waveform.process(output_stream.buffer());

        if let Err(err) = output_stream.send().await {
            defmt::error!("{}", err);
        }
    }
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
