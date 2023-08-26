#!/usr/bin/env python3
import rerun as rr # pip install rerun-sdk
import math
import delta2_lidar
from time import sleep

# start rerun session
rr.init("delta2_lidar_rerun", spawn = True)

# connect to hardware, using a rudimentary 'reconnect' method
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

