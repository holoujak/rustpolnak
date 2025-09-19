#!/usr/bin/env python3
import serial
from pathlib import Path
import subprocess
import time
import struct

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

    def close(self) -> None:
        self.serial.close()
        self.socat.terminate()

    def checksum(self, data: bytes) -> int:
        val = sum(data) & 0xFF
        return ((1 << 8) - val) & 0xFF

    def send_frame(self, addr: int, cmd: int, status: int, payload: bytearray) -> None:
        frame = (
            struct.pack(">2bhbbb", 0x43, 0x54, len(payload) + 4, addr, cmd, status)
            + payload
        )
        frame += bytearray([self.checksum(frame)])
        print(frame.hex(" "))
        self.serial.write(frame)

    def send_tags(self, tags: list[str]) -> None:
        dev_sn = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]
        payload = bytearray(dev_sn) + bytearray([len(tags)])
        for tag in tags:
            tag_type = 1
            ant2 = 2
            rssi = 34
            content = bytearray([tag_type, ant2, *bytes.fromhex(tag), rssi])
            payload += bytearray(bytearray([len(content)]) + content)

        self.send_frame(1, 0x45, 1, payload)


DEVICE_DIR.mkdir(parents=True, exist_ok=True)

rfids = [
    RFID(0),
    RFID(1),
]


try:
    i = 0
    while True:
        for device_id, device in enumerate(rfids):
            device.send_tags([f"{device_id:02x}801191A50300642CABB6{i:02x}"])
        i = (i + 1) & 0xFF
        time.sleep(1)
finally:
    for rfid in rfids:
        rfid.close()
