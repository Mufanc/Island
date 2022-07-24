# Island🏝️

* 基于 overlayfs 的存储隔离工具，并提供一些其它小功能
* 灵感来源于 [这篇文章](https://ouonline.net/overlayfs-and-chroot)，同时也参考了 [BubbleWrap](https://github.com/containers/bubblewrap) 的模式
* 仅为学习 Rust 的练手作，不保证能够完全隔离

## Usage🎯

```shell
git clone https://github.com/Mufanc/Island
cd island 
cargo build --release
cd target/release
sudo chown root island
sudo chmod 4777 island
./island 
```

## Todo📅

- [x] 分离业务逻辑到 `lib.rs`，并完善异常处理
- [ ] 使用 `pivot_root` 代替 `chroot`
- [ ] 将隔离环境的 overlayfs 封装到单独的 img 镜像中
- [ ] 支持诸如 `--ro-bind`、`--procfs` 此类的个性化挂载参数
- [ ] 使用 capabilities 机制代替 suid，提升安全性
