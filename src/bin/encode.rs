use std::path::PathBuf;

use structopt::StructOpt;

use starsoldier_ground_compress as ssgc;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long, parse(try_from_str=parse_int::parse), default_value="0xD90C")]
    origin: usize,

    #[structopt(parse(from_os_str))]
    path_in: PathBuf,

    #[structopt(parse(from_os_str))]
    path_out: PathBuf,
}

fn main() -> eyre::Result<()> {
    let opt = Opt::from_args();

    let buf_in = std::fs::read(opt.path_in)?;

    let buf_out = ssgc::encode(buf_in, opt.origin)?;

    std::fs::write(opt.path_out, buf_out)?;

    Ok(())
}
