# nrf-tester
A repo to play around with bluetooth libraries targeting the nRF52840-DK


## Setup
The information below details how to set things up on a Windows machine. 

### Install Rust
Install rust from here https://rustup.rs and be sure to include the "C++ build tools package" if using windows.
This project uses new toolchain features, often only to be found in nightly. Please ensure that your toolchains are up to date. We also add a target for arm cross compilation for the DK.

```
rustup install nightly
rustup target add thumbv7em-none-eabihf
```

### Install IDE

Install "Visual Studio Code" and, once installed and open, go to File | Preferences | Extensions and install "Rust Analyzer" and "Better TOML"

### Flash the nRF soft device

This example targets the nRF52840-DK board. Flashing the softdevice is required. It is NOT part of the built binary. You only need to do it once at the beginning, or after doing full chip erases.

- Download SoftDevice S140 from Nordic's website [here](https://www.nordicsemi.com/Software-and-tools/Software/S140/Download). Supported versions are 7.x.x
- Unzip
- `nrfjprog --family NRF52 --chiperase --verify --program s140_nrf52_7.2.0_softdevice.hex`
- or use the nRF Connect app which has a GUI


### Install a chip programmer

For the programmer to run you need to associate the nRF52840-DK USB peripheral to the WinUSB driver. Install https://zadig.akeo.ie/

In Zadig's graphical user interface ([credit here](https://embedded-trainings.ferrous-systems.com/installation.html)),

    Select the 'List all devices' option from the Options menu at the top.

    From the device (top) drop down menu select "BULK interface (Interface 2)"

    Once that device is selected, 1366 1015 should be displayed in the USB ID field. That's the Vendor ID - Product ID pair.

    Select 'WinUSB' as the target driver (right side)

    Click "Install WinUSB driver". The process may take a few minutes to complete.


You will also need [`probe-run`](https://ferrous-systems.com/blog/probe-run/) - a utility to enable `cargo run` to run embedded applications on a device:

```
cargo install probe-run
```

## Compiling

```
cargo build
```

## Running
Note that running will also compile if required.
```
cargo run
```

## Credit
Thanks to the authors of the embassy and nrf-softdevice repos for sample code and instructions, most of which what copied directly from those repos.

## Licence
MIT