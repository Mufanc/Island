extern crate core;

use std::env;
use std::ffi::OsString;
use std::path::Component::Normal;
use std::path::PathBuf;

use nix::unistd::Uid;

mod argparse;
mod island;

#[macro_export]
macro_rules! check_err {
    ( $name:ident$($body:tt)* ) => {
        match $name$($body)* {
            Err(err) => {
                eprintln!("{}: {}", stringify!($name), err.desc());
                std::process::exit(1)
            }
            Ok(result) => result
        }
    };
}


pub fn isolate() {
    let args = argparse::parse();

    let (uid, euid) = (Uid::current(), Uid::effective());
    if !euid.is_root() || uid == euid {
        eprintln!("error: please run `island` with SUID permission!");
        std::process::exit(1);
    }

    let homedir = OsString::from(env::var("HOME").unwrap());
    let workdir = args.workdir.components().map(|it| match it {
        Normal(x) => {
            if x == &OsString::from("~") {
                Normal(&homedir)
            } else {
                it
            }
        }
        _ => it,
    }).collect::<PathBuf>();

    let island = island::Island::new(&workdir);

    island.init();

    if args.procfs {
        island.mount_fs("proc", &PathBuf::from("/proc"), "proc");
    }

    if args.sysfs {
        island.mount_fs("sysfs", &PathBuf::from("/sys"), "sysfs");
    }

    if args.dev {
        island.mount_dev();
    }

    for dst in args.tmpfs {
        island.mount_fs("tmpfs", &dst, "tmpfs");
    }

    if !args.bind.is_empty() {
        for (src, dst) in args.bind.iter().zip(args.bind[1..].iter()) {
            island.mount_bind(src, dst);
        }
    }

    island.exec(&args.command);
}
