use crate::ll::{Tas2563Device, Tas2563Interface};

const CFG_META_BURST: u8 = 253;

#[derive(Debug, PartialEq)]
pub struct RegisterWrite {
    register: u8,
    value: u8,
}

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    WriteSingle(RegisterWrite),
    WriteBurst(BurstCommand<'a>),
}

#[derive(Debug, PartialEq)]
pub struct CommandIterator<'a> {
    commands: &'a [u8],
}

#[derive(Debug, PartialEq)]
pub struct BurstCommand<'a> {
    /// List of bytes to send as burst write.
    /// First element is starting register.
    ///
    /// Only I2C supports burst write, for SPI iterate all registers
    data: &'a [u8],
}

impl<'a> CommandIterator<'a> {
    pub fn new(commands: &'a [u8]) -> Self {
        Self { commands }
    }

    pub async fn write<T: Tas2563Interface>(
        self,
        dest: &mut Tas2563Device<T>,
    ) -> Result<(), T::Error> {
        for c in self {
            match c {
                Command::WriteSingle(RegisterWrite { register, value }) => {
                    dest.interface().write_register(register, value).await?;
                }
                Command::WriteBurst(command) => {
                    dest.interface().write_burst(command.as_burst()).await?;
                }
            }
        }
        Ok(())
    }
}

impl<'a> Iterator for CommandIterator<'a> {
    type Item = Command<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.commands.len() == 0 {
            return None;
        }
        if self.commands.len() == 1 {
            panic!("Only one byte remaining, expected two");
        }

        let (entry, remainder) = self.commands.split_at(2);
        let (command, remainder) = if entry[0] == CFG_META_BURST {
            let data_len = entry[1] as usize;
            // Note(2): the address and first value are not part of the burst count
            let (burst, remainder) = remainder.split_at(data_len + 1);

            let remainder = if data_len % 2 == 0 {
                &remainder[1..] // Skip zero-padding
            } else {
                remainder
            };

            (Command::WriteBurst(BurstCommand { data: burst }), remainder)
        } else {
            (
                Command::WriteSingle(RegisterWrite {
                    register: entry[0],
                    value: entry[1],
                }),
                remainder,
            )
        };

        self.commands = remainder;
        Some(command)
    }
}

impl<'a> BurstCommand<'a> {
    pub fn as_burst(&self) -> &[u8] {
        self.data
    }
}

#[cfg(test)]
mod test {
    use crate::bulk::BurstCommand;

    use super::{Command, CommandIterator, RegisterWrite, CFG_META_BURST};

    #[test]
    fn commands_burst_even() {
        let mut it = CommandIterator::new(&[
            0x5a,
            0x0f,
            CFG_META_BURST,
            0x02,
            0x5c,
            0x0f,
            0xa0,
            0x00,
            0x5e,
            0x01,
        ]);

        assert_eq!(
            it.next(),
            Some(Command::WriteSingle(RegisterWrite {
                register: 0x5a,
                value: 0x0f
            }))
        );
        assert_eq!(
            it.next(),
            Some(Command::WriteBurst(BurstCommand {
                data: &[0x5c, 0x0f, 0xa0],
            }))
        );
        assert_eq!(
            it.next(),
            Some(Command::WriteSingle(RegisterWrite {
                register: 0x5e,
                value: 0x01
            }))
        );
    }

    #[test]
    fn commands_burst_odd() {
        let mut it = CommandIterator::new(&[
            0x5a,
            0x0f,
            CFG_META_BURST,
            0x03,
            0x5c,
            0x0f,
            0xa0,
            0xc5,
            0x5e,
            0x01,
        ]);

        assert_eq!(
            it.next(),
            Some(Command::WriteSingle(RegisterWrite {
                register: 0x5a,
                value: 0x0f
            }))
        );
        assert_eq!(
            it.next(),
            Some(Command::WriteBurst(BurstCommand {
                data: &[0x5c, 0x0f, 0xa0, 0xc5],
            }))
        );
        assert_eq!(
            it.next(),
            Some(Command::WriteSingle(RegisterWrite {
                register: 0x5e,
                value: 0x01
            }))
        );
    }
}
