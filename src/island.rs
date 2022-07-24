use std::{env, fs};
use std::ffi::CString;
use std::path::{Path, PathBuf};

use nix::mount::{mount, MsFlags};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{chdir, chroot, execve, getuid, setuid};

use crate::check_err;

macro_rules! mount {
    ( $($arg:expr),* ) => {
        check_err!(mount::<str, Path, str, str>($($arg),*))
    };
}

pub struct Island {
    root: PathBuf
}

impl Island {
    pub fn new(base_dir: &PathBuf) -> Island {
        let base_dir = base_dir;
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

        Island {
            root: jail_dir
        }
    }

    pub fn mount(&self, source: Option<&str>, target: &str, fstype: Option<&str>, flags: MsFlags, data: Option<&str>) {
        let target = if target.chars().next().unwrap() == '/' {
            &target[1..]
        } else {
            target
        };
        mount!(source, self.root.join(target).as_path(), fstype, flags, data);
    }

    pub fn mount_fstype(&self, desc: &str, target: &str, fstype: &str) {
        self.mount(Some(desc), target, Some(fstype), MsFlags::empty(), None);
    }

    pub fn mount_bind(&self, source: &str, target: &str) {
        self.mount(Some(source), target, None, MsFlags::MS_BIND | MsFlags::MS_REC, None);
    }

    pub fn exec(&self) {
        check_err!(chroot(self.root.as_path()));
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
}
