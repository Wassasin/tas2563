use embedded_hal_mock::eh1::i2c::{Mock, Transaction};

use super::{Address, Tas2563Device};
use crate::prelude::*;

fn reg(register: u8, value: u8) -> Transaction {
    Transaction::write(Address::Global as u8, vec![register, value])
}

#[async_std::test]
async fn example() {
    let expectations = [
        // Set page to 0x00
        reg(0x00, 0x00),
        // Set book to 0x00
        reg(0x7f, 0x00),
        // Mute and power sensing up
        reg(0x02, 0x0d),
        // Software shutdown and power sensing down
        reg(0x02, 0x0e),
        // Software reset
        reg(0x01, 0x01),
        // Set amp level to unknown invalid value, and enable DC blocker
        reg(0x03, 0x00),
        // Non-compliant register write
        // reg(0x01, 0x80),

        // Software shutdown but power sensing enabled
        reg(0x02, 0x02),
        // Set amp level to 16dBv, and enable DC blocker
        reg(0x03, 0x20),
        // Enable low EMI spread spectrum, all retries
        reg(0x04, 0xf6),
        // Set sample rate to 44.1/48kHz
        reg(0x06, 0x09),
        // Set TDM RX offset to 1 cycle
        reg(0x07, 0x02),
        // SLEN=32bits, WLEN=24bits, SCFG=stereodownmix, IVMON=16bits
        reg(0x08, 0x7a),
        // RX Timeslots L=0 R=1
        reg(0x09, 0x10),
        // Use TX offset 1
        reg(0x0a, 0x03),
        // Send voltage sensing data on slot 4
        reg(0x0b, 0x44),
        // Send current sensing data on slot 0
        reg(0x0c, 0x40),
        // Configure vbat to slot 4, but do not transmit
        reg(0x0d, 0x04),
        // Configure temp to slot 5, but do not transmit
        reg(0x0e, 0x05),
        // Configure gain to slot 6, but do not transmit
        reg(0x0f, 0x06),
        // Configure bst to slot 7, but do not transmit
        reg(0x10, 0x07),
        // Enable limiter with 2 sample steps of 0.5dB
        reg(0x12, 0x13),
        // Enable limiter with 500ms hold time, 640ms/step release rate, and 0.5dB step size
        reg(0x13, 0x76),
        // Enable brownout prevention
        reg(0x14, 0x01),
        // Brownout hold time=500ms, attack step size=1dB, rate=2 samples/step
        reg(0x15, 0x2e),
        // Set int_masks to default
        reg(0x1a, 0xfc),
        reg(0x1b, 0xa6),
        reg(0x1c, 0xdf),
        reg(0x1d, 0xff),
        // Configure IRQZ pin to assert on any unmasked latched interrupts
        reg(0x30, 0x19),
        // Enable no pull-downs
        reg(0x31, 0x40),
        // Set IRQZ polarity active low, reset value
        reg(0x32, 0x81),
        // Boost PFM limit frequency=50kHz, enable, Boost Mode Class-H
        reg(0x33, 0x34),
        // Set Boost to 8.5V with 0 degree phase, no synchronisation, and an 1uH inductor
        reg(0x34, 0x46),
        // Set load regulation slope to 3A/V, with 162us step time
        reg(0x35, 0x84),
        // Set clock configuration to manual
        reg(0x38, 0x20),
        // Set vbat averaging and filter
        reg(0x3b, 0x38),
        // reg(0x3c, 0x38),
        // reg(0x3d, 0x08),
        // reg(0x3e, 0x10),
        // reg(0x3f, 0x00),
        // reg(0x40, 0x40),
        // reg(0x30, 0x19),
        // reg(0x02, 0x00),
        // reg(0x01, 0x80),
        // reg(0x02, 0x00),
        // reg(0x03, 0x20),
        // reg(0x04, 0xf6),
        // reg(0x06, 0x09),
        // reg(0x07, 0x02),
        // reg(0x08, 0x7a),
        // reg(0x09, 0x10),
        // reg(0x0a, 0x03),
        // reg(0x0b, 0x44),
        // reg(0x0c, 0x40),
        // reg(0x0d, 0x04),
        // reg(0x0e, 0x05),
        // reg(0x0f, 0x06),
        // reg(0x10, 0x07),
        // reg(0x12, 0x13),
        // reg(0x13, 0x76),
        // reg(0x14, 0x01),
        // reg(0x15, 0x2e),
        // reg(0x1a, 0xfc),
        // reg(0x1b, 0xa6),
        // reg(0x1c, 0xdf),
        // reg(0x1d, 0xff),
        // reg(0x30, 0x19),
        // reg(0x31, 0x40),
        // reg(0x32, 0x81),
        // reg(0x33, 0x34),
        // reg(0x34, 0x46),
        // reg(0x35, 0x84),
        // reg(0x38, 0x20),
        // reg(0x3b, 0x38),
        // reg(0x3c, 0x38),
        // reg(0x3d, 0x08),
        // reg(0x3e, 0x10),
        // reg(0x3f, 0x00),
        // reg(0x40, 0x40),
    ];
    let mut i2c = Mock::new(&expectations);

    let mut ll = Tas2563Device::new(&mut i2c, Address::Global);
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

    i2c.done();
}
