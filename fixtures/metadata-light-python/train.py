import cv2
import numpy
import torch
from ultralytics import YOLO


def describe_runtime() -> tuple[str, str, str]:
    return torch.__version__, cv2.__version__, numpy.__version__


def build_model() -> YOLO:
    return YOLO("synthetic-model.yaml")
