//! Strongly typed register values used by this crate.
pub use crate::ll::registers::{
    AmpLevel, BopAtkRt, BopAtkSt, BopHldTm, BstClassHStepTime, BstIr, BstLr, BstMode, BstPa,
    BstPfml, BstVreg, FrameStart, IrqzPinCfg, IrqzPol, IvmonLen, LimbAtkRt, LimbAtkSt, LimbHldTm,
    LimbRlsRt, LimbRlsSt, Mode, RxScfg, RxSlen, RxWlen, SampRate, Tg1En, Tg1Pinen, TxEdge, TxFill,
    VbatLimThSelection,
};

use derive_more::{From, Into};

#[derive(From, Into, Debug, PartialEq)]
pub struct BoostPeakCurrentMaxRun(pub u8);

impl BoostPeakCurrentMaxRun {
    pub fn from_milliamps(i: u16) -> Self {
        let i = i.clamp(990, 4000);

        if i == 4000 {
            BoostPeakCurrentMaxRun(0x37)
        } else {
            let i = (i - 990) / 55;
            BoostPeakCurrentMaxRun(i as u8)
        }
    }
}

#[derive(From, Into, Debug, PartialEq)]
pub struct VBatCnv(pub u16);

impl VBatCnv {
    pub fn to_millivolts(&self) -> u16 {
        (self.0 as u32 * 1000 / 64) as u16
    }
}

#[derive(From, Into, Debug, PartialEq)]
pub struct PVDDCnv(pub u16);

impl PVDDCnv {
    pub fn to_millivolts(&self) -> u16 {
        todo!()
    }
}

#[derive(From, Into, Debug, PartialEq)]
pub struct TempCnv(pub u8);

impl TempCnv {
    pub fn to_celcius(&self) -> i16 {
        self.0 as i16 - 93
    }
}

#[derive(Debug)]
pub struct ADCReadout {
    pub pvdd: PVDDCnv,
    pub vbat: VBatCnv,
    pub temp: TempCnv,
}

#[cfg(test)]
mod test {
    use crate::prelude::{BoostPeakCurrentMaxRun, TempCnv, VBatCnv};

    #[test]
    fn boost_peak_current_max_run() {
        assert_eq!(
            BoostPeakCurrentMaxRun::from_milliamps(990),
            BoostPeakCurrentMaxRun(0)
        );
        assert_eq!(
            BoostPeakCurrentMaxRun::from_milliamps(3999),
            BoostPeakCurrentMaxRun(0x36)
        );
        assert_eq!(
            BoostPeakCurrentMaxRun::from_milliamps(4000),
            BoostPeakCurrentMaxRun(0x37)
        );
    }

    #[test]
    fn vbat_voltage() {
        assert_eq!(VBatCnv(0x000).to_millivolts(), 0);
        assert_eq!(VBatCnv(0x001).to_millivolts(), 15);
        assert_eq!(VBatCnv(0x100).to_millivolts(), 4000);
        assert_eq!(VBatCnv(0x180).to_millivolts(), 6000);
    }

    #[test]
    fn temperature() {
        assert_eq!(TempCnv(0x00).to_celcius(), -93);
        assert_eq!(TempCnv(0x76).to_celcius(), 25);
        assert_eq!(TempCnv(0xFF).to_celcius(), 162);
    }
}
