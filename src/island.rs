use std::{env, fs};
use std::ffi::CString;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use nix::mount::{mount, MsFlags};
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{chdir, chroot, execve, execvpe, getuid, setuid};

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
        let upper_dir = base_dir.join("diff");
        let work_dir = base_dir.join("work");

        fs::create_dir_all(&upper_dir).unwrap();
        fs::create_dir_all(&work_dir).unwrap();

        check_err!(unshare(CloneFlags::CLONE_NEWNS));
        check_err!(mount::<str, str, str, str>(None, "/", None, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None));

        mount!(
            Some("island"), upper_dir.as_path(), Some("overlay"), MsFlags::empty(),
            Some(format!("lowerdir=/,upperdir={},workdir={}", upper_dir.to_str().unwrap(), work_dir.to_str().unwrap()).as_str())
        );

        Island {
            root: upper_dir
        }
    }

    fn parse_path(&self, path: &str) -> PathBuf {
        self.root.join(if path.chars().next().unwrap() == '/' {
            &path[1..]
        } else {
            path
        })
    }

    fn mount(&self, source: Option<&str>, target: &str, fstype: Option<&str>, flags: MsFlags, data: Option<&str>) {
        mount!(source, self.parse_path(target).as_path(), fstype, flags, data);
    }

    pub fn mount_fstype(&self, desc: &str, target: &str, fstype: &str) {
        self.mount(Some(desc), target, Some(fstype), MsFlags::empty(), None);
    }

    pub fn mount_bind(&self, source: &str, target: &str) {
        self.mount(Some(source), target, None, MsFlags::MS_BIND | MsFlags::MS_REC, None);
    }

    pub fn mount_dev(&self) {
        self.mount_fstype("tmpfs", "/dev", "tmpfs");

        fs::create_dir_all(self.parse_path("/dev/pts")).unwrap();
        self.mount_fstype("devpts", "/dev/pts", "devpts");

        fs::create_dir_all(self.parse_path("/dev/shm")).unwrap();
        self.mount_fstype("tmpfs", "/dev/shm", "tmpfs");

        for file in ["full", "zero", "null", "random", "urandom", "tty", "console"] {
            let target = format!("/dev/{}", file);
            fs::File::create(self.parse_path(&target)).unwrap();
            self.mount_bind(&target, &target);
        }

        symlink("/proc/self/fd", self.parse_path("/dev/fd")).unwrap();
        symlink("/proc/self/fd/0", self.parse_path("/dev/stdin")).unwrap();
        symlink("/proc/self/fd/1", self.parse_path("/dev/stdout")).unwrap();
        symlink("/proc/self/fd/2", self.parse_path("/dev/stderr")).unwrap();
        symlink("pts/ptmx", self.parse_path("/dev/ptmx")).unwrap();
        symlink("/proc/kcore", self.parse_path("/dev/core")).unwrap();
    }

    pub fn exec(&self, command: &Vec<String>) {
        check_err!(chroot(self.parse_path("/").as_path()));
        check_err!(chdir("/"));
        check_err!(setuid(getuid()));

        let envp: Vec<CString> = env::vars().map(|(k, v)| {
            CString::new(format!("{}={}", k, v)).unwrap()
        }).collect();

        if command.is_empty() {
            match env::var("SHELL") {
                Ok(shell) => {
                    let file = CString::new(shell).unwrap();
                    let argv = [&file, ];
                    check_err!(execve(&file, &argv, &envp));
                }
                Err(err) => eprintln!("error: {}", err)
            }
        } else {
            let file = CString::new(command[0].as_str()).unwrap();
            let argv: Vec<CString> = command.iter().map(|it| CString::new(it.as_str()).unwrap()).collect();
            check_err!(execvpe(&file, &argv, &envp));
        }
    }
}
