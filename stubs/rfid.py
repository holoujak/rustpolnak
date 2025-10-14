#!/usr/bin/env python3
import asyncio
import serial
from pathlib import Path
import struct

DEVICE_DIR = Path("dev")


class RFID:
    @classmethod
    async def create(cls, num: int) -> "RFID":
        device_path = DEVICE_DIR / f"rfid{num}"
        our_path = f"{device_path}_"

        DEVICE_DIR.mkdir(parents=True, exist_ok=True)
        socat = await asyncio.create_subprocess_exec(
            "socat",
            "-d",
            "-d",
            f"PTY,link={device_path},raw,echo=0",
            f"PTY,link={our_path},raw,echo=0",
        )
        # wait until socat starts
        await asyncio.sleep(1)
        return RFID(socat, serial.Serial(our_path, 115200, write_timeout=0))

    def __init__(self, socat: asyncio.subprocess.Process, serial: serial.Serial):
        self.socat = socat
        self.serial = serial

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


if __name__ == "__main__":
    from app import registrations
    import asyncio
    import argparse

    from prompt_toolkit.key_binding import KeyBindings
    from prompt_toolkit.application import Application
    from prompt_toolkit.layout import (
        HSplit,
        Layout,
    )
    from prompt_toolkit.widgets import RadioList

    async def main():
        p = argparse.ArgumentParser()
        p.add_argument("race_id", nargs="?", default=3, type=int)
        args = p.parse_args()

        racers = [
            (
                racer.tagId,
                f"#{racer.startNumber:04} {racer.tagId} {racer.firstName} {racer.lastName}",
            )
            for racer in registrations(args.race_id)
            if racer.startNumber and racer.tagId
        ]

        rfids = await asyncio.gather(
            RFID.create(0),
            RFID.create(1),
        )

        radiolist = RadioList(
            values=racers,
            select_on_focus=True,
            open_character="",
            select_character=">",
            close_character="",
            show_cursor=False,
            show_scrollbar=False,
        )

        kb = KeyBindings()

        @kb.add("enter", eager=True)
        def _on_enter(event) -> None:
            for rfid in rfids:
                rfid.send_tags([radiolist.current_value])

        @kb.add("c-c")
        @kb.add("<sigint>")
        @kb.add("escape")
        @kb.add("q")
        def _keyboard_interrupt(event) -> None:
            event.app.exit(style="class:aborting")

        @kb.add("c-z")
        def _suspend(event) -> None:
            event.app.suspend_to_background()

        app = Application(
            layout=Layout(HSplit([radiolist])),
            full_screen=True,
            mouse_support=True,
            key_bindings=kb,
        )

        await app.run_async()

    asyncio.run(main())
