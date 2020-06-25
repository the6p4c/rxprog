# rxprog
[![crates.io badge](https://img.shields.io/crates/v/rxprog)](https://crates.io/crates/rxprog)

Library implementing the Boot Mode protocol for specific Renesas microcontrollers and CLI for programming firmware images

[View library documentation on docs.rs](https://docs.rs/rxprog/)

# `rxprog-cli`
## Installation
    $ cargo install rxprog --features rxprog-cli

The `rxprog-cli` binary will now be available.

## Usage
See `rxprog-cli --help` for more details.

To connect to a target on COM3 to query the available devices:

    $ rxprog-cli "p=COM3"

To connect to a target on `/dev/ttyS4` and program an image:

    $ rxprog-cli "p=/dev/ttyS4;d=7805;cm=0;if=3200;mr=x1,x1;br=115200" image.ihex

## Examples
Querying multiplication ratios and input frequency ranges:

    $ rxprog-cli "p=COM3;d=7805;cm=0"
    Connecting to target on COM3
    The selected debug adapter does not support automatic reset. Please reset the target into boot mode and press ENTER.
    
    Continuing...
    Initial connection succeeded
    
    No input frequency, multiplication ratio and/or bit rate specified in connection string. Querying target for supported multiplication ratios and operating frequency ranges:
    Clock    Multiplication ratios
    ==============================
    0        x1
    1        x1
    Clock    Minimum frequency    Maximum frequency
    ===============================================
    0        3200                 3200
    1        3200                 3200
    
    Hint: select an input frequency, multiplication ratio and bit rate with if=<input frequency>;mr=<ratio 1>,<ratio 2>,...;br=<bit rate>

Programming an image:

    $ rxprog-cli "p=COM3;d=7805;cm=0;if=3200;mr=x1,x1;br=115200" blink.ihex
    Connecting to target on COM3
    The selected debug adapter does not support automatic reset. Please reset the target into boot mode and press ENTER.
    
    Continuing...
    Initial connection succeeded
    Detected ihex image from extension
    Transitioned to programming/erasure state successfully
    
    Programming...
    Programming complete.
    Verifying...
    Verification complete.
