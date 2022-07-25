use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[clap(version, about, long_about=None)]
pub struct Args {
    #[clap(long, help="A directory to store overlayfs data", default_value="~/.island")]
    pub workdir: PathBuf,

    #[clap(long, help="Mount new procfs on /proc")]
    pub procfs: bool,

    #[clap(long, help="Mount new sysfs on /sys")]
    pub sysfs: bool,

    #[clap(long, help="Mount a minimum viable tmpfs on /dev")]
    pub dev: bool,

    #[clap(long, value_names=&["DST"], help="Mount new tmpfs on DST")]
    pub tmpfs: Vec<String>,

    #[clap(long, value_names=&["SRC", "DST"], number_of_values=2, help="Bind mount the host path SRC on DST")]
    pub bind: Vec<String>,

    #[clap(multiple=true, help="Executable and arguments, default to \"$SHELL\"")]
    pub command: Vec<String>,
}

pub fn parse() -> Args {
    return Args::parse();
}
