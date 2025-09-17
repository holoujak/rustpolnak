#!/usr/bin/env python3
import serial
from pathlib import Path
import subprocess
import time

DEVICE_DIR = Path("dev")


class RFID:
    def __init__(self, num: int) -> None:
        device_path = DEVICE_DIR / f"rfid{num}"
        our_path = f"{device_path}_"

        self.socat = subprocess.Popen(
            [
                "socat",
                "-d",
                "-d",
                f"PTY,link={device_path},raw,echo=0",
                f"PTY,link={our_path},raw,echo=0",
            ]
        )
        time.sleep(1)
        self.serial = serial.Serial(our_path, 115200)

    def close(self):
        self.serial.close()
        self.socat.terminate()

    def send_tag(self, tag: str) -> None:
        self.serial.write(tag.encode("utf-8") + b"\r\n")


DEVICE_DIR.mkdir(parents=True, exist_ok=True)

rfids = [
    RFID(0),
    RFID(1),
]


try:
    i = 0
    while True:
        for device_id, device in enumerate(rfids):
            device.send_tag(f"tag{device_id}_{i}")
        i += 1
        time.sleep(1)
finally:
    for rfid in rfids:
        rfid.close()
