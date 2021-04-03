[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
### Description
pictas-interface-rs is the host interface software for the [PICTAS replay device](https://github.com/bigbass1997/PICTAS), written in Rust. This project is still an early prototype, code is likely to be messy and buggy. Others are allowed to write their own interface software if they wish; the PICTAS does _not_ require that this particular software is used for communication.

### Usage / Communication
The PICTAS currently uses a USB to TTL module to communicate with a host computer, however this will be replaced with another microcontroller similar to how some Arduino boards connect over USB. The general process looks like this: Load movie into this software. Program the movie's inputs onto the PICTAS device. And finally, initiate the playback (there are multiple ways to do this).

The replay device contains a 16MB FLASH memory IC that stores the movie's inputs. It does _not_ need to be reprogrammed each time you wish to playback the TAS, only when changing/updating the run. Communication details will be found in the PICTAS's readme file.

### Movie File Compatibility
Currently only `.r08` files are supported, but there will be support for `.r16`, `.m64`, and any others as more console support is added.

### Building
Rust is highly integrated with the `cargo` build system. To install Rust and `cargo`, just follow [these instructions](https://doc.rust-lang.org/cargo/getting-started/installation.html). Once installed, while in the project directory, just run `cargo build --release` or to run directly, you can use `cargo run`.
