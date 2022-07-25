# Island 🏝️

* 基于 overlayfs 的存储隔离工具，并提供一些其它小功能
* 灵感来源于 [这篇文章](https://ouonline.net/overlayfs-and-chroot)，同时也参考了 [BubbleWrap](https://github.com/containers/bubblewrap) 的模式
* 仅为学习 Rust 的练手作，不保证能够完全隔离

## Usage 🎯

```shell
git clone https://github.com/Mufanc/Island
cd Island 
make run -- --help
```

## Todo 📅

- [x] 分离业务逻辑到 `lib.rs`，并完善异常处理
- [ ] 使用 `pivot_root` 代替 `chroot`
- [ ] 支持将隔离环境的 overlayfs 封装到单独的 img 镜像中
- [x] 支持诸如 `--ro-bind`、`--procfs` 此类的个性化挂载参数
- [ ] 使用 capabilities 机制代替 suid，提升安全性
- [x] 使用 GNU Make 快捷构建和安装
- [ ] 更优雅地处理异常
