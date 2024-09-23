#[derive(Debug)]
pub struct Commands(pub Vec<Command>);

impl Commands {
    pub fn iter(&self) -> impl Iterator<Item = &'_ Command> + '_ {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a Commands {
    type Item = &'a Command;

    type IntoIter = std::slice::Iter<'a, Command>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Debug)]
pub struct WriteCommand {
    pub address: u8,
    pub register: u8,
    pub bytes: Vec<u8>,
}

#[derive(Debug)]
pub enum Command {
    Write(WriteCommand),
    Delay(u8),
}
