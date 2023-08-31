#!/usr/bin/env python3
from time import sleep
import numpy as np
import math
from typing import List, Tuple

from delta2_lidar import Lidar, MeasurementFrame, FullScan

import rclpy
from rclpy.node import Node
from rclpy.duration import Duration
from rclpy.time import Time
from std_msgs.msg import Header, String, Float32
from sensor_msgs_py.point_cloud2 import create_cloud_xyz32, PointCloud2

def to_3D(points: List[Tuple[float,float]]) -> np.ndarray:
    zz = np.zeros((len(points), 1))
    return np.hstack([points, zz])

def timer_callback():
    global node
    global dev
    global pub
    global pubrpm
    node.get_logger().info("Scanning")

    if dev.alive():
        # read a measurement frame
        try:
          scan : FullScan = dev.read_full_scan()
        except:
          print("LiDAR disconnected?")
          return

        # convert the 2D points to 3D
        points_xyz = to_3D(scan.points)

        hd = Header()
        hd.frame_id = "delta2"
        hd.stamp = Time(seconds=scan.timestamp // 1e9, nanoseconds=scan.timestamp % 1e9).to_msg()

        pclmsg = create_cloud_xyz32(hd, points_xyz)
        pub.publish(pclmsg)

        rpmmsg = Float32()
        rpmmsg.data = float(scan.rpm)
        pubrpm.publish(rpmmsg)

def main(args=None):
    rclpy.init(args=args)
    global node
    global dev 
    global pub
    global pubrpm
    global pubscan
    # connect to the lidar
    dev = Lidar()
    dev.open("/dev/ttyUSB0")
    node = Node('delta2')
    node.create_timer(0.01, timer_callback)
    pub = node.create_publisher(PointCloud2, "/cloud", 0)
    pubrpm = node.create_publisher(Float32, "/rpm", 0)
    rclpy.spin(node)
    rclpy.shutdown()

if __name__ == '__main__':
    main()
