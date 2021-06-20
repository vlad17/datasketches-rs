//! `rsds` main executable, which provides count-distinct functionality
//! on the command line.

use structopt::StructOpt;

/// `dsrs` command line is not yet implemented
#[derive(Debug, StructOpt)]
#[structopt(name = "dsrs", about = "Approximate count distinct lines.")]
struct Opt {
    /// If set, the raw flag results in a base64 serialized printout of
    /// the sketch at the end of computation rather than the approximate
    /// distinct count. This is useful when combined with a downstream
    /// `dsrs --merge` operation later to merge multiple sketches.
    #[structopt(long)]
    raw: bool,
}

fn main() {
    let opt = Opt::from_args();
    let raw = if opt.raw { "raw" } else { "clean" };

    println!("Hello {} world!", raw);
}
