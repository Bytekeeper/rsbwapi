[![Documentation](https://docs.rs/rsbwapi/badge.svg)](https://docs.rs/rsbwapi)

# rsbwapi

`rsbwapi` is a Rust library for building artificial intelligence (AI) for the popular real-time strategy game Starcraft Broodwar. The library allows you to create and control Starcraft Broodwar bots through the Broodwar API (BWAPI). 

If you're not familiar with BWAPI, it is a community-developed project that provides a C++ interface to interact with Starcraft Broodwar. You can find more information about BWAPI [here](https://github.com/bwapi/bwapi).

To get started with `rsbwapi`, check out the documentation on [docs.rs](https://docs.rs/rsbwapi). Also check out the simple [ExampleBot](https://github.com/Bytekeeper/rsbwapi/tree/master/example_bot). Or take a look at my bot [Styx2](https://github.com/Bytekeeper/Styx2).

You may want to join the [SSCAIT Discord](https://discord.gg/frDVAwk), which is a community of Starcraft AI enthusiasts who run regular bot tournaments. You can also check out the [Basil Ladder](https://www.basil-ladder.net/) which is based on SSCAIT but runs a lot more games.

For more information on Starcraft AI development, you can visit the [SSCAIT website](http://www.sscaitournament.com/). There should be enough information to get you started.

If you have any questions or feedback, feel free to create an issue on the `rsbwapi` GitHub repository.

# Usage

## Windows
You should be fine to just compile your bot. The resulting x64 executable should run fine in all current tournaments/ladders.

## Linux
Note: These instructions will create a 32-bit executable. There are no 32-bit tournaments and the result is not a DLL, so you can choose to create a 64-bit executable.

### Windows-GNU target
Install support for the target:
```
rustup target add i686-pc-windows-gnu
```

Create the file '.cargo/config.toml':
```toml
[build]
target="i686-pc-windows-gnu"
```

### Windows MSVC target
Follow the installation instructions for xwin: https://github.com/rust-cross/cargo-xwin

Create the file '.cargo/config.toml':
```toml
[build]
target="i686-pc-windows-msvc"

[target.i686-pc-windows-msvc]
linker = "lld"
rustflags = [
  "-Lnative=/home/<youruser>/.xwin/crt/lib/x86",
  "-Lnative=/home/<youruser/.xwin/sdk/lib/um/x86",
  "-Lnative=/home/<youruser/.xwin/sdk/lib/ucrt/x86"
]


## Mac
It should work the same way as with Linux - but it's untested.
