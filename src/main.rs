extern crate core;

use i3_tools::{FocusTarget, I3Service};
use std::io::Error;
use std::path::PathBuf;
use structopt::StructOpt;

const ACTIONS: &[&str] = &["previous", "next"];

#[derive(StructOpt)]
#[structopt(name = "i3-tools", about = "Tools for the i3 window manager.")]
struct Opt {
    /// Sets the filename of the i3 socket for communication
    #[structopt(short, long, parse(from_os_str), env = "I3SOCK")]
    socket: PathBuf,

    /// Performs the action
    #[structopt(possible_values(ACTIONS))]
    action: String,
}

fn main() -> Result<(), Error> {
    let opt: Opt = Opt::from_args();
    let mut i3service = I3Service::connect(opt.socket)?;
    let target = match opt.action.as_str() {
        "previous" => FocusTarget::Previous,
        "next" => FocusTarget::Next,
        _ => unreachable!(),
    };

    i3service.focus(target)
}
