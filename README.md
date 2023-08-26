# Delta2 LiDAR Driver [WIP]

This is a driver for the 'Delta2' lidar, available through various retailers on AliExpress.

> <span><img src="https://github.com/leighleighleigh/delta2_lidar_rs/assets/19563769/c1bdc3bf-2b20-4779-9921-db1de1d9350a" width="50%" /><img src="https://github.com/leighleighleigh/delta2_lidar_rs/assets/19563769/7c0640d8-4063-4ccb-94ff-96de2c8c1ec5" width="50%" /></span><br>
> Disclaimer: I am a Rust noob. Code quality is not a priority here.

This driver is a **WORK IN PROGRESS**, pending the following TODOs:
 - [x] ! Re-structure the code as a library, exposing the Lidar+MeasurementFrame structs
 - [x] ! Add Python support, using [PyO3](https://pyo3.rs/v0.19.2/)
 - [ ] ! Test python build on Raspberry Pi
 - [ ] ! Publish python builds to PyPy
 - [ ] ! Handle the low-RPM health messages, serial disconnection, etc, with cleaner crate-specific exceptions
 - [ ] ? Expose additional diagnostics such as scan rate, SNR, CRC error rates
 - [ ] ? Handle serial re-connection ?

## Install + Use (Python)

```bash
pip install git+https://github.com/leighleighleigh/delta2_lidar_rs
```

`$ cat ./examples/stream-rerun.py`
```python
#!/usr/bin/env python3
import rerun as rr # pip install rerun-sdk
import math
import delta2_lidar
from time import sleep

# start rerun session
rr.init("delta2_lidar_rerun", spawn = True)

# connect to hardware
dev = delta2_lidar.Lidar()
dev.open("/dev/ttyUSB0")

while dev.alive():
    # read a measurement frame
    try:
      f = dev.read()
    except:
      print("LiDAR disconnected?")
      break

    # set the time of this data
    rr.set_time_nanos("scan", f.timestamp)

    # make a list of XYZ points
    points = []

    for (dx,dy) in f.points:
        points.append([dx,dy,0.0])

    rr.log_points("scan", points)
    rr.log_scalar("scan/rpm", f.rpm)

```

## Building

There are a few ways to build this package.

```
# build rust library
cargo build 

# build the full library,binary,and python library, with nix
./build.sh

# build with cross build (not tested)
cross build --target aarch64-unknown-linux-gnu

# build using python setuptools (this is pretty good)
python setup.py bdist_wheel --plat-name x86_64-unknown-linux-gnu --py-limited-api=cp38
```

## Note on Motor (M+/M-) Voltage
I was originally running both the LiDAR MCU and it's motor from 5V - but the USB port + cable impedance meant that it was only running at about ~4V.

After adding a large capacitor to the 5V supply, the motor started to spin too fast for it to report valid range data.

I recommend running the motor from a stable 3.3V source for now, although ideally ~4V would maximize scan rate.
