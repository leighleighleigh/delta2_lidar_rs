#!/usr/bin/env python3
import rerun as rr # pip install rerun-sdk
import math
import numpy as np
from delta2_lidar import Lidar, MeasurementFrame, FullScan
from time import sleep

# start rerun session
rr.init("delta2_lidar_rerun", spawn = True)

# connect to hardware, using a rudimentary 'reconnect' method
dev = Lidar()
dev.open("/dev/ttyUSB0")

while dev.alive():
    # read a measurement frame
    try:
      scan : FullScan = dev.read_full_scan()
    except:
      print("LiDAR disconnected?")
      break

    # set the time of this data
    rr.set_time_nanos("scan", scan.timestamp)

    # make a list of XYZ points
    points = []

    for (dx,dy) in scan.points:
        points.append([dx,dy,0.0])

    rr.log_points("scan", np.array(points).copy())
    # rr.log_scalar("scan/rpm", scan.rpm)
    # rr.log_scalar("scan/scan_sector", sum(list(map(lambda f: f.sector_angle, scan.frames))))
