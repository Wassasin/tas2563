use std::{collections::BTreeMap, iter::Peekable};

use itertools::Either;

use crate::ast::Command;

const PAGE_REGISTER: u8 = 0x00;
const BOOK_REGISTER: u8 = 0x7f;
const BURST_MAX_LEN: usize = 127;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct RegisterAddress {
    pub book: u8,
    pub page: u8,
    pub register: u8,
}

pub fn analyze<'a>(
    it: impl Iterator<Item = &'a Command> + 'a,
) -> impl Iterator<Item = (RegisterAddress, u8)> + 'a {
    let mut book = 0x00;
    let mut page = 0x00;

    it.map(|cmd| match cmd {
        Command::Write(write_command) => {
            let mut register = write_command.register;
            Either::Left(write_command.bytes.iter().copied().map(move |b| {
                let res = (register, b);
                register += 1;
                res
            }))
        }
        Command::Delay(_) => Either::Right(std::iter::empty()),
    })
    .flatten()
    .map(move |(register, value)| match register {
        PAGE_REGISTER => {
            page = value;
            None
        }
        BOOK_REGISTER => {
            book = value;
            None
        }
        register => Some((
            RegisterAddress {
                book,
                page,
                register,
            },
            value,
        )),
    })
    .flatten()
}

pub fn dedup(it: impl Iterator<Item = (RegisterAddress, u8)>) -> BTreeMap<RegisterAddress, u8> {
    it.collect()
}

pub struct RegenerateIterator<T>
where
    T: Iterator<Item = (RegisterAddress, u8)>,
{
    prev_book: Option<u8>,
    prev_page: Option<u8>,

    it: Peekable<T>,
}

impl<T> Iterator for RegenerateIterator<T>
where
    T: Iterator<Item = (RegisterAddress, u8)>,
{
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        match self.it.peek() {
            None => None,
            Some((ra, value)) => Some(Command::Write({
                let book = ra.book;
                let page = ra.page;

                if self.prev_page != Some(page) {
                    self.prev_page = Some(page);
                    crate::ast::WriteCommand {
                        register: PAGE_REGISTER,
                        bytes: vec![page],
                    }
                } else if self.prev_book != Some(book) {
                    self.prev_book = Some(book);
                    crate::ast::WriteCommand {
                        register: BOOK_REGISTER,
                        bytes: vec![book],
                    }
                } else {
                    let first_register = ra.register;
                    let mut prev_register = first_register;
                    let mut bytes: Vec<u8> = vec![*value];

                    loop {
                        let _ = self.it.next().unwrap();

                        // If max burst length is reached, or we are about to overwrite the book register.
                        if bytes.len() > BURST_MAX_LEN || prev_register == BOOK_REGISTER - 1 {
                            break;
                        }

                        match self.it.peek() {
                            Some((ra, value)) => {
                                if ra.register != prev_register + 1
                                    || ra.book != book
                                    || ra.page != page
                                {
                                    break;
                                }

                                bytes.push(*value);
                                prev_register += 1;
                            }
                            None => break,
                        }
                    }

                    crate::ast::WriteCommand {
                        register: first_register,
                        bytes: bytes,
                    }
                }
            })),
        }
    }
}

pub fn regenerate<'a, T>(it: T) -> RegenerateIterator<T>
where
    T: Iterator<Item = (RegisterAddress, u8)>,
{
    RegenerateIterator {
        prev_book: None,
        prev_page: None,
        it: it.peekable(),
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::{analyze, dedup, regenerate, RegisterAddress};

    #[test]
    fn regenerate_idem() {
        let data: BTreeMap<RegisterAddress, u8> = dedup(
            [
                (0x01, 0x01, 0x04, 0xff),
                (0x01, 0x01, 0x01, 0xff),
                (0x01, 0x01, 0x02, 0xff),
                (0x01, 0x01, 0x03, 0xff),
                (0x01, 0x01, 0x05, 0xff),
                (0x01, 0x01, 0x04, 0xfe),
                (0x01, 0x01, 0x07, 0xff),
                (0x00, 0x00, 0x07, 0xff),
            ]
            .into_iter()
            .map(|(book, page, register, value)| {
                (
                    RegisterAddress {
                        book,
                        page,
                        register,
                    },
                    value,
                )
            }),
        );

        let cmds = regenerate(data.clone().into_iter());
        let cmds: Vec<_> = cmds.collect(); // TODO make this collect unnecessary
        let result = dedup(analyze(cmds.iter()));

        assert_eq!(data, result);
    }
}
