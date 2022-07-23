use std::{env, fs};
use std::ffi::CString;
use std::path::Path;
use std::process;

use nix::mount::{mount, MsFlags};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{chdir, chroot, execve, getuid, setuid};

macro_rules! check_err {
    ( $name:ident$($body:tt)* ) => {
        if let Err(err) = $name$($body)* {
            eprintln!("{}: {}", stringify!($name), err.desc());
            process::exit(1);
        }
    };
}

macro_rules! mount {
    ( $($arg:expr),* ) => {
        check_err!(mount::<str, Path, str, str>($($arg),*))
    };
}


fn main() {
    if !nix::unistd::geteuid().is_root() {
        eprintln!("error: please run `island` as superuser!");
        return;
    }

    let base_dir = Path::new(".island");
    let jail_dir = base_dir.join("jail");
    let work_dir = base_dir.join("work");

    fs::create_dir_all(&jail_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    check_err!(unshare(CloneFlags::CLONE_NEWNS));
    check_err!(mount::<str, str, str, str>(None, "/", None, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None));

    mount!(
        Some("island"), jail_dir.as_path(), Some("overlay"), MsFlags::empty(),
        Some(format!("lowerdir=/,upperdir={},workdir={}", jail_dir.to_str().unwrap(), work_dir.to_str().unwrap()).as_str())
    );
    mount!(Some("proc"), jail_dir.join("proc").as_path(), Some("proc"), MsFlags::empty(), None);
    mount!(Some("sysfs"), jail_dir.join("sys").as_path(), Some("sysfs"), MsFlags::empty(), None);
    mount!(Some("/run"), jail_dir.join("run").as_path(), None, MsFlags::MS_BIND | MsFlags::MS_REC, None);
    mount!(Some("/dev"), jail_dir.join("dev").as_path(), None, MsFlags::MS_BIND | MsFlags::MS_REC, None);

    check_err!(chroot(jail_dir.as_path()));
    check_err!(chdir("/"));
    check_err!(setuid(getuid()));

    match env::var("SHELL") {
        Ok(shell) => {
            let path = CString::new(shell).unwrap();
            let argv = [&path, ];
            let envp: Vec<CString> = env::vars().map(|(k, v)| {
                CString::new(format!("{}={}", k, v)).unwrap()
            }).collect();
            execve(&path, &argv, &envp).unwrap();
        }
        Err(err) => eprintln!("error: {}", err)
    }
}