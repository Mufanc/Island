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

    let island = island::Island::new(&args.workdir);

    if args.procfs {
        island.mount_fstype("proc", "/proc", "proc");
    }

    if args.sysfs {
        island.mount_fstype("sysfs", "/sys", "sysfs");
    }

    for dst in args.tmpfs {
        island.mount_fstype("tmpfs", &dst, "tmpfs");
    }

    if args.bind.len() != 0 {
        for (src, dst) in args.bind.iter().zip(args.bind[1..].iter()) {
            island.mount_bind(src, dst);
        }
    }

    island.exec();
}
