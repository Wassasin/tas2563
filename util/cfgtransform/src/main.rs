pub mod ast;
pub mod bulk;

use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::Parser;
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(grammar);

#[derive(Parser, Debug)]
struct Args {
    /// Input file (of the cfg format)
    input: PathBuf,

    /// Output file (of the byte buffer format)
    #[arg(default_value = "./out.bulk")]
    output: PathBuf,
}

fn main() {
    pretty_env_logger::init();

    let args = Args::parse();

    log::info!("Reading {:?}", args.input);
    let f = std::fs::read_to_string(&args.input).unwrap();

    let cmds = grammar::CommandsParser::new().parse(&f).unwrap();
    log::info!("Read {} commands", cmds.0.len());

    let mut f = BufWriter::new(std::fs::File::create(&args.output).unwrap());

    let mut i = 0usize;
    for b in bulk::BulkGenerator::generate(cmds.into_iter()) {
        f.write(&[b]).unwrap();
        i += 1;
    }
    f.flush().unwrap();

    log::info!("Spooled {} bytes to {:?}", i, args.output);
}

#[cfg(test)]
mod test {
    #[test]
    fn test_file() {
        let src = r"
            #########################################################
            # TAS2563QFN_Mono debug cfg file
            # PPC3 File: stock.ppc3
            # TAS2563QFN_Mono version: 3.0.0
            # DDC Name: stock
            # Binary Version: 1
            #########################################################
            w 98 00 00
            w 98 7f 00
            w 98 00 05
            w 98 54 00
            > 00
            > 00
            > 00
            w 98 08 3f
        ";

        let cmds = crate::grammar::CommandsParser::new().parse(src).unwrap();
        let bulk = crate::bulk::BulkGenerator::generate(cmds.into_iter()).collect::<Vec<_>>();

        assert_eq!(
            bulk,
            &[0, 0, 127, 0, 0, 5, 253, 4, 84, 0, 0, 0, 0, 0, 8, 63]
        );
    }
}
