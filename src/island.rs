use std::{env, fs};
use std::ffi::CString;
use std::os::unix::fs::symlink;
use std::path::Component::RootDir;
use std::path::PathBuf;

use nix::mount::{mount, MsFlags};
use nix::NixPath;
use nix::sched::{CloneFlags, unshare};
use nix::unistd::{chdir, chroot, execve, execvpe, getuid, setuid};

use crate::check_err;

pub struct Island(PathBuf);

impl Island {
    pub fn new(base_dir: &PathBuf) -> Island {
        Island(base_dir.join("diff"))
    }

    pub fn init(&self) {
        let upper_dir = &self.0;
        let work_dir = &upper_dir.join("../work");

        fs::create_dir_all(upper_dir).unwrap();
        fs::create_dir_all(work_dir).unwrap();

        check_err!(unshare(CloneFlags::CLONE_NEWNS));
        check_err!(mount::<str, _, str, str>(None, "/", None, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None));

        self.mount(
            Some("island"), upper_dir, Some("overlay"), MsFlags::empty(),
            Some(format!("lowerdir=/,upperdir={},workdir={}", upper_dir.to_str().unwrap(), work_dir.to_str().unwrap()).as_str())
        );
    }

    /**
     * arguments:
     * - fstype:  str,  Path, str,  None
     * - bind:    Path, Path, None, None
     * - overlay: str,  Path, str,  str
     */
    pub fn mount<T: ?Sized + NixPath>(&self, source: Option<&T>, target: &PathBuf, fstype: Option<&str>, flags: MsFlags, data: Option<&str>) {
        match (fstype, data) {
            (Some(_), Some(_)) => {  // overlayfs
                check_err!(mount(source, target, fstype, flags, data));
            }
            (Some(_), None) => {     // some filesystem type
                check_err!(mount::<_, _, _, str>(source, target, fstype, flags, data));
            }
            (None, None) => {        // bind mount
                check_err!(mount::<_, _, str, str>(source, target, None, MsFlags::MS_BIND | flags, None));
            }
            _ => panic!("unknown operation!"),
        }
    }

    pub fn inner_path(&self, sub_path: &PathBuf) -> PathBuf {
        self.0.join(sub_path.components().filter(|it| match it {
            RootDir => false,
            _ => true,
        }).collect::<PathBuf>())
    }

    pub fn mount_fs(&self, desc: &str, target: &PathBuf, fstype: &str) {
        self.mount(Some(desc), target, Some(fstype), MsFlags::empty(), None);
    }

    pub fn mount_bind(&self, source: &PathBuf, target: &PathBuf) {
        self.mount(Some(source), target, None, MsFlags::MS_REC, None);
    }

    pub fn mount_dev(&self) {
        self.mount_fs("tmpfs", &PathBuf::from("/dev"), "tmpfs");

        let dev_pts = PathBuf::from("/dev/pts");
        fs::create_dir_all(self.inner_path(&dev_pts)).unwrap();
        self.mount_fs("devpts", &dev_pts, "devpts");

        let dev_shm = PathBuf::from("/dev/shm");
        fs::create_dir_all(self.inner_path(&dev_shm)).unwrap();
        self.mount_fs("tmpfs", &dev_shm, "tmpfs");

        for file in ["full", "zero", "null", "random", "urandom", "tty", "console"] {
            let target = PathBuf::from(format!("/dev/{}", file));
            fs::File::create(self.inner_path(&target)).unwrap();
            self.mount_bind(&target, &target);
        }

        symlink("/proc/self/fd", self.inner_path(&PathBuf::from("/dev/fd"))).unwrap();
        symlink("/proc/self/fd/0", self.inner_path(&PathBuf::from("/dev/stdin"))).unwrap();
        symlink("/proc/self/fd/1", self.inner_path(&PathBuf::from("/dev/stdout"))).unwrap();
        symlink("/proc/self/fd/2", self.inner_path(&PathBuf::from("/dev/stderr"))).unwrap();
        symlink("pts/ptmx", self.inner_path(&PathBuf::from("/dev/ptmx"))).unwrap();
        symlink("/proc/kcore", self.inner_path(&PathBuf::from("/dev/core"))).unwrap();
    }

    pub fn exec(&self, command: &Vec<String>) {
        check_err!(chroot(self.inner_path(&PathBuf::from("/")).as_path()));
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
