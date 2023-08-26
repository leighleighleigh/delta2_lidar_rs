# Delta2 LiDAR Driver [WIP]

This is a driver for the 'Delta2' lidar, available through various retailers on AliExpress.

> <span><img src="https://github.com/leighleighleigh/delta2_lidar_rs/assets/19563769/c1bdc3bf-2b20-4779-9921-db1de1d9350a" width="50%" /><img src="https://github.com/leighleighleigh/delta2_lidar_rs/assets/19563769/7c0640d8-4063-4ccb-94ff-96de2c8c1ec5" width="50%" /></span><br>
> Disclaimer: I am a Rust noob. Code quality is not a priority here.

This driver is a **WORK IN PROGRESS**, pending the following TODOs:
 - [ ] ! Re-structure the code as a library, exposing the Lidar+MeasurementFrame structs
 - [ ] ! Add Python support, using [PyO3](https://pyo3.rs/v0.19.2/) 
 - [ ] ? Handle the low-RPM health message, by raising a (crate-specific) exception
 - [ ] ? Expose additional diagnostics such as scan rate, SNR, CRC error rate

## Building
There are a few ways to build this package.
```
# build and run binary program
cargo build 

# build the full library,binary,and python library, with nix
./build.sh

# build with cross build (not tested yet)
cross build --target aarch64-unknown-linux-gnu
```

## Note on Motor (M+/M-) Voltage
I was originally running both the LiDAR MCU and it's motor from 5V - but the USB port + cable impedance meant that it was only running at about ~4V.

After adding a large capacitor to the 5V supply, the motor started to spin too fast for it to report valid range data.

I recommend running the motor from a stable 3.3V source for now, although ideally ~4V would maximize scan rate.
