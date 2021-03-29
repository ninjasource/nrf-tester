# nrf-tester
A repo to play around with bluetooth libraries


## Setup
This project uses new toolchain features, often only to be found in nightly. Please ensure that your toolchains are up to date:

```
rustup install nightly
rustup target add thumbv7em-none-eabihf
```

You will also need [`probe-run`](https://ferrous-systems.com/blog/probe-run/) - a utility to enable `cargo run` to run embedded applications on a device:

```
cargo install probe-run
```
This example targets the nRF52840-DK board

Flashing the softdevice is required. It is NOT part of the built binary. You only need to do it once at the beginning, or after doing full chip erases.

- Download SoftDevice S140 from Nordic's website [here](https://www.nordicsemi.com/Software-and-tools/Software/S140/Download). Supported versions are 7.x.x
- Unzip
- `nrfjprog --family NRF52 --chiperase --verify --program s140_nrf52_7.2.0_softdevice.hex`
- or use the nRF Connect app which has a GUI


## Running
```
cargo run
```

## Credit
Thanks to the authors of the embassy and nrf-softdevice repos for sample code and instructions, most of which what copied directly from those repos.

## Licence
MIT