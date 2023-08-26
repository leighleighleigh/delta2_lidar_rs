# Delta2 LiDAR Driver [WIP]

This is a driver for the 'Delta2' lidar, available through various retailers on AliExpress.

> <img src="https://github.com/leighleighleigh/delta2_lidar_rs/assets/19563769/c1bdc3bf-2b20-4779-9921-db1de1d9350a" width="50%" /><br>
> Disclaimer: I am a Rust noob. Code quality is not a priority here.

This driver is a **WORK IN PROGRESS**, pending the following TODOs:
 - [ ] ! Re-structure the code as a library, exposing the Lidar+MeasurementFrame structs
 - [ ] ! Add Python support, using [PyO3](https://pyo3.rs/v0.19.2/) 
 - [ ] ? Handle the low-RPM health message, by raising a (crate-specific) exception
 - [ ] ? Expose additional diagnostics such as scan rate, SNR, CRC error rate



