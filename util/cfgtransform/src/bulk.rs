use itertools::Either;

use crate::ast::{Commands, WriteCommand};

const CFG_META_BURST: u8 = 253;

pub struct BulkGenerator;

impl BulkGenerator {
    /// Generate bulk register write files, with "burst" transfers.
    ///
    /// **Important note**: the PPC3 tooling uses a similar approach,
    /// but employs 16-bit wide blocks to encode the transfers, adding dummy values
    /// after burst transfers with an even amount of values.
    ///
    /// This method does not add dummy values to align the transfers to 16 bits.
    pub fn generate(commands: &'_ Commands) -> impl Iterator<Item = u8> + '_ {
        commands
            .0
            .iter()
            .map(|cmd| match cmd {
                crate::ast::Command::Write(WriteCommand {
                    address: _,
                    register,
                    bytes,
                }) => {
                    if bytes.len() == 1 {
                        Either::Left([*register, bytes[0]].into_iter())
                    } else {
                        Either::Right(Either::Left(
                            [CFG_META_BURST, bytes.len() as u8, *register]
                                .into_iter()
                                .chain(bytes.iter().copied()),
                        ))
                    }
                }
                crate::ast::Command::Delay(_) => {
                    Either::Right(Either::Right(std::iter::empty::<u8>()))
                }
            })
            .flatten()
    }
}
