use nix::unistd::Uid;

fn main() {
    // check permission
    let (uid, euid) = (Uid::current(), Uid::effective());
    if !euid.is_root() || uid == euid {
        eprintln!("error: please run `island` with SUID permission!");
        std::process::exit(1);
    }

    island::isolate();
}
