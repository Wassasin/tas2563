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

#[cfg(test)]
mod test {
    use super::BoostPeakCurrentMaxRun;

    #[async_std::test]
    async fn boost_peak_current_max_run() {
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
}
