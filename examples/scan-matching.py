#!/usr/bin/env python3
import rerun as rr  # pip install rerun-sdk
import math
from typing import List, Tuple, Optional
from delta2_lidar import Lidar, MeasurementFrame, FullScan
import numpy as np
from time import sleep
from icp import iterative_closest_point

"""
Uses 'icp.py' from https://github.com/OmarJItani/Iterative-Closest-Point-Algorithm/
"""

rr.init("delta2_lidar_icp", spawn=True)
dev = Lidar()
dev.open("/dev/ttyUSB0")

odom_x, odom_y, odom_theta = 0.0, 0.0, 0.0

scan_memory = 32  # buffer some scans, which we can use to build up a history of our odometry
scans: List[MeasurementFrame] = []


def transform_to_cartesian(transform) -> Tuple[float, float, float]:
    #  x,y, theta
    ab = -math.asin(transform[0][1])
    lx = -transform[0][2]
    ly = -transform[1][2]

    return lx, ly, -math.degrees(ab)

def register_new_scan(next: np.array, last: np.array) -> Tuple[np.array, bool]:
    # Performs ICP against these new points, and the last scan -
    # if the delta movement (rot,dx,dy) is below a threshold, we don't add it to the scans list.
    # trim to the shorter length
    L = min(len(next), len(last))
    next = next[:L]
    last = last[:L]

    # find the match between the two scans
    transform, matched, error, iterations = iterative_closest_point(last, next, max_iterations=50)
    dx, dy, dtheta = transform_to_cartesian(transform)

    print(f"x,y,theta: {dx:+3.4f}, {dy:+3.4f}, {dtheta:+3.2f}\nerror: {error:+4.4f}\niter: {iterations:02}")

    # if dx, or dy are > 5cm, we add this scan.
    # if angle is > +-1 deg, we add the scan

    if abs(dx) >= 0.01 or abs(dy) >= 0.01 or abs(dtheta) >= 2.0:
      # returns the matched points and transform if successful
      return matched, (dx,dy,dtheta), True
    else:
      return matched, (dx,dy,dtheta), False


def to_3D(points: np.array) -> np.array:
    zz = np.zeros((len(points), 1))
    return np.hstack([points, zz])


while dev.alive():
    # read a measurement frame
    try:
        scan: FullScan = dev.read_full_scan()
    except:
        break

    points = np.array(scan.points)

    if len(scans) == 0:
        # add first scan once-off
        scans.append(points)
        continue
      
    # previous set of points
    last_scan = scans[-1]

    # use register_new_scan to determine if we add the new points
    matched_scan, transform, different = register_new_scan(points, last_scan)

    if different: 
      # use the transform to update our odometry
      odom_theta += transform[2]
      rad = math.radians(odom_theta)
      x = transform[0]*math.cos(rad) - transform[1]*math.sin(rad)
      y = transform[1]*math.cos(rad) + transform[0]*math.sin(rad)
      odom_x += x
      odom_y += y
      # Add to the scans list
      scans.append(points)
      # clamp the scans list length
      scans = scans[-scan_memory:]

    rr.set_time_nanos("scan", scan.timestamp)
    rr.log_points("scan", to_3D(points))
    rr.log_points("scan/last", to_3D(last_scan))
    rr.log_points("scan/matched", to_3D(matched_scan))

    # rr.log_points("scan/icp", [[x[0],x[1],0.0] for x in matched.tolist()])
    # rr.log_scalar("scan/rpm", scan.rpm)
    # rr.log_scalar("scan/icp_error", error)
    # rr.log_scalar("odometry/vel/dx", -dx)
    # rr.log_scalar("odometry/vel/dy", -dy)
    rr.log_scalar("odometry/x", odom_x)
    rr.log_scalar("odometry/y", odom_y)
    rr.log_scalar("odometry/theta", odom_theta % 360.0)
