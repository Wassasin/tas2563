#[derive(Debug)]
pub struct Commands(pub Vec<Command>);

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
