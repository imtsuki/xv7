# xv7

You can view the development notes on [the Telegram Channel](https://t.me/xv7notes).

## Prerequisites

Install cargo-make:

```bash
cargo install cargo-make
```

Also, you should have the latest version of [QEMU](https://www.qemu.org) installed.

## Build and Run

Simply run the following command:

```bash
cargo make qemu
```

You can use KVM to speed up emulation:

```bash
cargo make qemu-kvm
```

## Tips

For better development experience, install these additional tools:

```bash
rustup component add llvm-tools-preview
cargo install cargo-binutils
cargo install rustfilt
```

Compose a bootable USB drive:

```bash
 cargo make compose-esp --env ESP=/path/to/where/your/drive/is/mounted
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
