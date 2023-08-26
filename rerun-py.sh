#!/usr/bin/env bash

# Need to move to a different folder, otherwise it tries to import in here.
x=$(pwd)

# Adds the nix build to pythonpath, opens python
export PYTHONPATH="${PYTHONPATH}:${x}/result/lib/python3.10/site-packages/"

pushd $(mktemp -d)

cat << EOF > test.py
#!/usr/bin/env python3
import rerun as rr
import math
import delta2_lidar_py

# start rerun session
rr.init("delta2_lidar_rerun", spawn = True)

# connect to hardware
dev = delta2_lidar_py.Lidar()
dev.open("/dev/ttyUSB0")

while True:
    # read a measurement frame
    f = dev.read()

    # set the time of this data
    rr.set_time_nanos("scan", f.timestamp)

    # convert to cartesian points
    points = []

    for m in f.measurements:
        dx = m.distance_mm * math.sin(math.radians(m.angle)) / 1000.0;
        dy = m.distance_mm * math.cos(math.radians(m.angle)) / 1000.0;
        dz = 0.0
        points.append([dx,dy,dz])

    rr.log_points("scan", points)

EOF

python3 -i test.py 

popd
