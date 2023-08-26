#!/usr/bin/env bash

# Need to move to a different folder, otherwise it tries to import in here.
x=$(pwd)

# Adds the nix build to pythonpath, opens python
export PYTHONPATH="${PYTHONPATH}:${x}/result/lib/python3.10/site-packages/"

pushd $(mktemp -d)

cat << EOF > test.py
#!/usr/bin/env python3
print("Import")
import delta2_lidar_py
print("Init")
dev = delta2_lidar_py.Lidar()
print("Open")
dev.open("/dev/ttyUSB0")

while True:
    f = dev.read()
    print(str(f))
    for m in f.measurements:
        print(str(m))

EOF

python3 -i test.py 

popd
