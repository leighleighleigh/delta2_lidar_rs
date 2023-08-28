#!/usr/bin/env python3
import rerun as rr  # pip install rerun-sdk
import math
from typing import List, Tuple, Optional
from delta2_lidar import Lidar, MeasurementFrame, FullScan
import numpy as np
from time import sleep, time_ns
from icp import iterative_closest_point, apply_transform
from gaussian import gaussian

"""
Uses 'icp.py' from https://github.com/OmarJItani/Iterative-Closest-Point-Algorithm/
"""

rr.init("delta2_lidar_icp", spawn=True)
dev = Lidar()
dev.open("/dev/ttyUSB0")

POS_VAR = 1.0
MEASUREMENT_VAR = 20.0
STATIC_VAR = 20.0
odom_x = gaussian(0.0,POS_VAR,"x")
odom_y = gaussian(0.0,POS_VAR,"y")
odom_theta = gaussian(0.0,POS_VAR,"theta")

scan_memory = 32  # buffer some scans, which we can use to build up a history of our odometry
scans: List[MeasurementFrame] = []
scans_matched: List[MeasurementFrame] = []

# thanks to https://alexsm.com/homogeneous-transforms/
def rotation_matrix(theta):
    c = np.cos(theta)
    s = np.sin(theta)
    return np.array([[c, -s], [s, c]])

def create_transform(t_x, t_y, theta):
    translation = np.array([t_x, t_y])
    rotation = rotation_matrix(theta)
    transform = np.eye(3, dtype=float)
    transform[:2, :2] = rotation
    transform[:2, 2] = translation
    return transform

def transform_to_cartesian(transform) -> Tuple[float, float, float]:
    #  x,y, theta
    ab = -math.asin(transform[0][1])
    lx = -transform[0][2]
    ly = -transform[1][2]
    return lx, ly, -math.degrees(ab)

def odometry_to_origin_transform(x : float, y : float, theta_deg : float) -> np.ndarray:
    return create_transform(-x,-y,-math.radians(theta_deg))

def icp_match(next: np.ndarray, last: np.ndarray, max_iterations : int = 20, tolerance : float = 0.005):
    # Performs ICP against these new points, and the last scan -
    # if the delta movement (rot,dx,dy) is below a threshold, we don't add it to the scans list.
    # trim to the shorter length
    L = min(len(next), len(last))
    next = next[:L]
    last = last[:L]
    # find the match between the two scans
    return iterative_closest_point(last, next, max_iterations=max_iterations,tolerance=tolerance)


def register_new_scan(next: np.array, last: np.array) -> Tuple[np.array, bool]:
    # find the match between the two scans
    transform, matched, error, iterations = icp_match(next, last)

    dx,dy,dtheta = transform_to_cartesian(transform)
    print(f"x,y,theta: {dx:+3.4f}, {dy:+3.4f}, {dtheta:+3.2f}\nerror: {error:+4.4f}\niter: {iterations:02}")

    # if dx, or dy are > 5cm, we add this scan.
    # if angle is > +-1 deg, we add the scan

    if abs(dx) >= 0.01 or abs(dy) >= 0.01 or abs(dtheta) >= 0.75:
        # returns the matched points and transform if successful
        return matched, transform, True, error
    else:
        return matched, transform, False, error


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
        scans_matched.append(points)
        continue

    # previous set of points
    last_scan = scans[-1]
    # use register_new_scan to determine if we add the new points
    matched_scan, transform, different, err = register_new_scan(points, last_scan)
    # update our odometry, 'velocity method' - faster updated by comparing to previous scan.
    if different:
        dx,dy,dtheta = transform_to_cartesian(transform)
        odom_theta = (odom_theta * gaussian(odom_theta.mean + dtheta, 100.0))
        rad = math.radians(odom_theta.mean)
        odx = dx*math.cos(rad) - dy*math.sin(rad)
        ody = dy*math.cos(rad) + dx*math.sin(rad)

        odom_x = (odom_x * gaussian(odom_x.mean + odx, MEASUREMENT_VAR)) + gaussian(0.0, MEASUREMENT_VAR)
        odom_y = (odom_y * gaussian(odom_x.mean + ody, MEASUREMENT_VAR)) + gaussian(0.0, MEASUREMENT_VAR)
        # Add to the scans list
        scans.append(points)
        # clamp the scans list length
        scans = scans[-scan_memory:]

    # if not different, we do the same thing - but update using a mapping to the origin point.
    transform, matched_back, error, iterations = icp_match(points, scans_matched[0],max_iterations=100,tolerance=0.0001)
    scans_matched.append(matched_back)
    scans_matched = [scans_matched[0]] + scans_matched[-scan_memory:]

    # update location - treating this as a 'zero-point'
    dx,dy,dtheta = transform_to_cartesian(transform)
    print(f"x,y,theta: {dx:+3.4f}, {dy:+3.4f}, {dtheta:+3.2f}\nerror: {error:+4.4f}\niter: {iterations:02}")
    # nearest 360 degree
    next_zero = odom_theta.mean % 360.0

    if next_zero >= 180:
        next_zero *= -1

    odom_theta = odom_theta * gaussian(odom_theta.mean - next_zero, STATIC_VAR * max(1.0,min(error/0.005,100.0)))
    rad = math.radians(odom_theta.mean)
    odx = dx*math.cos(rad) - dy*math.sin(rad)
    ody = dy*math.cos(rad) + dx*math.sin(rad)
    odom_x = odom_x * gaussian(odx, STATIC_VAR * max(1.0,min(error/0.005,100.0)))
    odom_y = odom_y * gaussian(ody, STATIC_VAR * max(1.0,min(error/0.005,100.0)))


    rr.set_time_nanos("scan", scan.timestamp)

    # make a transform for the scan sweep
    # rr.log_transform3d("scan", transform=rr.TranslationRotationScale3D(translation=[0.0,0.0,0.0],rotation=rr.RotationAxisAngle([0,0,1], degrees=-odom_theta)),from_parent=True)
    rr.log_points("scan", to_3D(points))

    # merge all points of the scan memory into a mega merged scan
    # merged_scans = None
    # for s in scans_matched:
    #     if merged_scans is None:
    #         merged_scans = s
    #         continue

    #     merged_scans = np.concatenate([merged_scans, s])

    rr.log_transform3d("map", transform=rr.TranslationRotationScale3D(translation=[-odom_x.mean,-odom_y.mean,0.0],rotation=rr.RotationAxisAngle([0,0,1], degrees=odom_theta.mean)),from_parent=True)
    rr.log_points("map", to_3D(points))

    # rr.log_points("scan/icp", [[x[0],x[1],0.0] for x in matched.tolist()])
    # rr.log_scalar("scan/rpm", scan.rpm)
    # rr.log_scalar("scan/icp_error", error)
    # rr.log_scalar("odometry/vel/dx", -dx)
    # rr.log_scalar("odometry/vel/dy", -dy)
    rr.log_scalar("odometry/x", odom_x.mean)
    rr.log_scalar("odometry/y", odom_y.mean)
    rr.log_scalar("odometry/x-var", odom_x.var)
    rr.log_scalar("odometry/y-var", odom_y.var)
    rr.log_scalar("odometry/theta", odom_theta.mean % 360.0)
