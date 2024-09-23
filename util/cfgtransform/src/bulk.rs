use itertools::Either;

use crate::ast::{Command, WriteCommand};

const CFG_META_BURST: u8 = 253;

pub struct BulkGenerator;

impl BulkGenerator {
    /// Generate bulk register write files, with "burst" transfers.
    pub fn generate<'a>(
        commands: impl Iterator<Item = &'a Command> + 'a,
    ) -> impl Iterator<Item = u8> + 'a {
        commands
            .map(|cmd| match cmd {
                crate::ast::Command::Write(WriteCommand {
                    address: _,
                    register,
                    bytes,
                }) => {
                    if bytes.len() == 1 {
                        Either::Left([*register, bytes[0]].into_iter())
                    } else {
                        // The PPC3 tooling employs 16-bit wide blocks to encode the transfers,
                        // adding dummy values after burst transfers with an even amount of values.
                        let dummy_byte = if bytes.len() % 2 == 0 {
                            Some(0b00)
                        } else {
                            None
                        };

                        Either::Right(Either::Left(
                            [CFG_META_BURST, bytes.len() as u8, *register]
                                .into_iter()
                                .chain(bytes.iter().copied())
                                .chain(dummy_byte.into_iter()),
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

#[cfg(test)]
mod test {
    use crate::{
        ast::{Command, WriteCommand},
        bulk::CFG_META_BURST,
    };

    #[test]
    fn single() {
        let command = Command::Write(WriteCommand {
            address: 0x1b, // Not used
            register: 0x4b,
            bytes: vec![0x0f],
        });
        let bulk =
            crate::bulk::BulkGenerator::generate(std::iter::once(&command)).collect::<Vec<_>>();

        assert_eq!(bulk, &[0x4b, 0x0f]);
        assert_eq!(bulk.len() % 2, 0); // Assert that the byte buffer is composed of 16 bit words
    }

    #[test]
    fn burst_even() {
        let command = Command::Write(WriteCommand {
            address: 0x1b, // Not used
            register: 0x4c,
            bytes: vec![0x01, 0x02],
        });
        let bulk =
            crate::bulk::BulkGenerator::generate(std::iter::once(&command)).collect::<Vec<_>>();

        assert_eq!(bulk, &[CFG_META_BURST, 0x02, 0x4c, 0x01, 0x02, 0x00]);
        assert_eq!(bulk.len() % 2, 0); // Assert that the byte buffer is composed of 16 bit words
    }

    #[test]
    fn burst_odd() {
        let command = Command::Write(WriteCommand {
            address: 0x1b, // Not used
            register: 0x4d,
            bytes: vec![0x01, 0x02, 0x03],
        });
        let bulk =
            crate::bulk::BulkGenerator::generate(std::iter::once(&command)).collect::<Vec<_>>();

        assert_eq!(bulk, &[CFG_META_BURST, 0x03, 0x4d, 0x01, 0x02, 0x03]);
        assert_eq!(bulk.len() % 2, 0); // Assert that the byte buffer is composed of 16 bit words
    }

    #[test]
    fn mix() {
        let commands = vec![
            Command::Write(WriteCommand {
                address: 0x1b, // Not used
                register: 0x5a,
                bytes: vec![0x0f],
            }),
            Command::Write(WriteCommand {
                address: 0x1b, // Not used
                register: 0x5c,
                bytes: vec![0x0f, 0xa0],
            }),
            Command::Write(WriteCommand {
                address: 0x1b, // Not used
                register: 0x5e,
                bytes: vec![0x01],
            }),
        ];
        let bulk = crate::bulk::BulkGenerator::generate(commands.iter()).collect::<Vec<_>>();

        assert_eq!(
            bulk,
            &[
                0x5a,
                0x0f,
                CFG_META_BURST,
                0x02,
                0x5c,
                0x0f,
                0xa0,
                0x00,
                0x5e,
                0x01
            ]
        );
        assert_eq!(bulk.len() % 2, 0); // Assert that the byte buffer is composed of 16 bit words
    }
}
