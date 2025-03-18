# DTB Early console

## test

```shell
cargo install ostool
# test with qemu
cargo test -p hello --test test -- --show-output
# test with uboot
cargo test -p hello --test test -- --show-output --uboot
```