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
        if self.commands.len() == 1 {
            panic!("Only one byte remaining, expected two");
        }

        let (entry, remainder) = self.commands.split_at(2);
        let (command, remainder) = if entry[0] == CFG_META_BURST {
            let (burst, remainder) = remainder.split_at(entry[1] as usize);
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

    pub const TEST_COMMANDS: [u8; 12] = [
        0x00,
        0x00,
        0x7f,
        0x00,
        CFG_META_BURST,
        4,
        0x08,
        0x00,
        0xfe,
        0x20,
        0x05,
        0x44,
    ];

    #[test]
    fn commands() {
        let mut it = CommandIterator::new(&TEST_COMMANDS);

        assert_eq!(
            it.next(),
            Some(Command::WriteSingle(RegisterWrite {
                register: 0x00,
                value: 0x00
            }))
        );
        assert_eq!(
            it.next(),
            Some(Command::WriteSingle(RegisterWrite {
                register: 0x7f,
                value: 0x00
            }))
        );
        assert_eq!(
            it.next(),
            Some(Command::WriteBurst(BurstCommand {
                data: &[0x08, 0x00, 0xfe, 0x20,]
            }))
        );
        assert_eq!(
            it.next(),
            Some(Command::WriteSingle(RegisterWrite {
                register: 0x05,
                value: 0x44
            }))
        );
    }
}
