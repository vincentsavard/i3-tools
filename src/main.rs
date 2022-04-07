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

fn main() {
    let opt: Opt = Opt::from_args();
    println!(
        "Performing <{}> on <{}>",
        opt.action,
        opt.socket.to_str().unwrap()
    );
}
