#!/usr/bin/env python3
from time import sleep
import numpy as np
from typing import List, Tuple

import rerun as rr # pip install rerun-sdk

from delta2_lidar import Lidar, MeasurementFrame, FullScan


# start rerun session
rr.init("delta2_lidar_rerun", spawn = True)

# connect to the lidar
dev = Lidar()
dev.open("/dev/ttyUSB0")


def to_3D(points: List[Tuple[float,float]]) -> np.ndarray:
    zz = np.zeros((len(points), 1))
    return np.hstack([points, zz])


while dev.alive():
    # read a measurement frame
    try:
      scan : FullScan = dev.read_full_scan()
    except:
      print("LiDAR disconnected?")
      break

    # set the time of this data
    rr.set_time_nanos("scan", scan.timestamp)

    # convert the 2D points to 3D
    points_xyz = to_3D(scan.points)

    rr.log_points("scan", points_xyz)
    rr.log_scalar("scan/rpm", scan.rpm)
