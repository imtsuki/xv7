# xv7

You can view the development notes on [the Telegram Channel](https://t.me/xv7notes).

## Prerequisites

You should have the latest version of [QEMU](https://www.qemu.org) installed.

## Build and Run

Simply run the following command:

```bash
make qemu
```

You can use KVM to speed up emulation:

```bash
make qemu-kvm
```

To compose a bootable USB drive:

```bash
make compose-esp ESP=/path/to/your/drive
```

## Tips

For better development experience, install these additional tools:

```bash
cargo install cargo-binutils
cargo install rustfilt
```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
