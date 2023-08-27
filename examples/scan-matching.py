#!/usr/bin/env python3
import rerun as rr # pip install rerun-sdk
import math
from typing import List
from delta2_lidar import Lidar, MeasurementFrame, FullScan
import numpy as np
from time import sleep
from icp import iterative_closest_point

"""
Uses 'icp.py' from https://github.com/OmarJItani/Iterative-Closest-Point-Algorithm/
"""

rr.init("delta2_lidar_icp", spawn = True)
dev = Lidar()
dev.open("/dev/ttyUSB0")

scans : List[MeasurementFrame] = []

odom_x, odom_y = 0.0,0.0

while dev.alive():
    # read a measurement frame
    try:
      scan : FullScan = dev.read_full_scan()
    except:
      print("LiDAR disconnected?")
      break

    # push to frames
    scans.append(scan.points)
    # only keep two full scans
    scans = scans[-2:]

    if len(scans) == 2:
      next = np.array(scans[-1])
      last = np.array(scans[-2])

      # trim to the shorter length
      L = min(len(next),len(last))
      next = next[:L]
      last = last[:L]

      # find the match between the two scans
      transform, matched, error, iterations = iterative_closest_point(last,next)

      rr.set_time_nanos("scan", scan.timestamp)
      rr.log_points("scan", [[x[0],x[1],0.0] for x in next])
      rr.log_points("scan/icp", [[x[0],x[1],0.0] for x in matched.tolist()])

      # rr.log_scalar("scan/rpm", scan.rpm)
      rr.log_scalar("scan/icp_error", error)

      dx,dy = transform[0][2], transform[1][2]
      odom_x -= dx
      odom_y -= dy

      rr.log_scalar("odometry/vel/dx", -dx)
      rr.log_scalar("odometry/vel/dy", -dy)

      rr.log_scalar("odometry/pos/x", odom_x)
      rr.log_scalar("odometry/pos/y", odom_y)
