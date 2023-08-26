#!/usr/bin/env python3
import rerun as rr
import math
import delta2_lidar_py
from time import sleep

# start rerun session
rr.init("delta2_lidar_rerun", spawn = True)

# connect to hardware, using a rudimentary 'reconnect' method
dev = delta2_lidar_py.Lidar()
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

    # convert to cartesian points
    points = []

    for (dx,dy) in f.points:
        #dx = m.distance_mm * math.sin(math.radians(m.angle)) / 1000.0;
        #dy = m.distance_mm * math.cos(math.radians(m.angle)) / 1000.0;
        dz = 0.0
        points.append([dx,dy,dz])

    rr.log_points("scan", points)






