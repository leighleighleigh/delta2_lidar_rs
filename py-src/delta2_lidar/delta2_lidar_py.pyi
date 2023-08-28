from typing import Any, List, Tuple, Optional

class FullScan:
    complete: bool
    frames: List[MeasurementFrame]
    points: List[Tuple[float,float]]
    rpm: float
    timestamp: int
    timestamp_range: int
    @classmethod
    def __init__(cls, *args, **kwargs) -> None: ...
    def as_json(self, *args, **kwargs) -> str: ...

class Lidar:
    @classmethod
    def __init__(cls, *args, **kwargs) -> None: ...
    def alive(self, *args, **kwargs) -> bool: ...
    def open(self, *args, **kwargs) -> None: ...
    def read_frame(self, *args, **kwargs) -> MeasurementFrame: ...
    def read_full_scan(self, *args, **kwargs) -> FullScan: ...

class Measurement:
    angle: float
    distance_mm: float
    point: Tuple[float,float]
    signal_quality: int
    @classmethod
    def __init__(cls, *args, **kwargs) -> None: ...

class MeasurementFrame:
    end_angle: float
    measurements: List[Measurement]
    offset_angle: float
    points: List[Tuple[float,float]]
    rpm: float
    sector_angle: float
    start_angle: float
    timestamp: int
    @classmethod
    def __init__(cls, *args, **kwargs) -> None: ...
    def as_json(self, *args, **kwargs) -> str: ...
