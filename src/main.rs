extern crate core;

use clap::Parser;
use i3_tools::{FocusTarget, I3Service};
use std::io::Error;
use std::path::PathBuf;
use std::time::Duration;

const ACTIONS: &[&str] = &["previous", "next"];

#[derive(Parser)]
#[clap(name = "i3-tools", about = "Tools for the i3 window manager.")]
struct Opt {
    /// Sets the filename of the i3 socket for communication
    #[clap(short, long, parse(from_os_str), env = "I3SOCK")]
    socket: PathBuf,

    /// Sets the connect and read timeout duration of the socket, in milliseconds
    #[clap(short, long, default_value = "250")]
    timeout: u64,

    /// Performs the action
    #[clap(possible_values(ACTIONS))]
    action: String,
}

fn main() -> Result<(), Error> {
    let opt: Opt = Opt::parse();
    let timeout = match opt.timeout {
        0 => None,
        timeout => Some(Duration::from_millis(timeout)),
    };
    let mut i3service = I3Service::connect(opt.socket, timeout)?;
    let target = match opt.action.as_str() {
        "previous" => FocusTarget::Previous,
        "next" => FocusTarget::Next,
        _ => unreachable!(),
    };

    i3service.focus(target)
}
