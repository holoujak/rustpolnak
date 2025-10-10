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
    from typing import List
    from prompt_toolkit import PromptSession
    from prompt_toolkit.key_binding import KeyBindings
    from prompt_toolkit.formatted_text import HTML
    from dataclasses import dataclass
    from app import registrations
    import asyncio

    @dataclass
    class RacerWithRFID:
        start_number: int
        firstname: str
        lastname: str
        tag: str
        sent: bool = False

    class Menu:
        def __init__(self, racers: List[RacerWithRFID]):
            self.racers = racers
            self.selected_index = 0
            self.rfids: List[RFID] = []

        def create_keybindings(self):
            kb = KeyBindings()

            @kb.add("up")
            def up(event):
                self.selected_index = (self.selected_index - 1) % len(self.racers)

            @kb.add("down")
            def down(event):
                self.selected_index = (self.selected_index + 1) % len(self.racers)

            @kb.add("enter")
            def enter(event):
                selected = self.racers[self.selected_index]
                selected.sent = True

                for rfid in self.rfids:
                    rfid.send_tags([selected.tag])

            @kb.add("c-r")
            def reset(event):
                for item in self.racers:
                    item.sent = False

            return kb

        def create_prompt_text(self):
            result = "\n"
            for i, racer in enumerate(self.racers):
                start, end = "", ""
                if racer.sent:
                    start, end = "<ansigreen>", "</ansigreen>"
                prefix = ">" if i == self.selected_index else " "
                result += f"{prefix} {start}#{racer.start_number:04} {racer.tag} {racer.firstname} {racer.lastname}{end}\n"
            result += "[Enter] send tag   [Up/Down] Navigate racers  [Ctrl+R] reset sent colors\n"
            return HTML(result)

        async def run(self):
            self.rfids = await asyncio.gather(
                RFID.create(0),
                RFID.create(1),
            )

            session = PromptSession(key_bindings=self.create_keybindings())
            return await session.prompt_async(self.create_prompt_text)

    async def main():
        racers = [
            RacerWithRFID(
                racer.startNumber, racer.firstName, racer.lastName, racer.tagId
            )
            for racer in registrations(0)
            if racer.startNumber and racer.tagId
        ]

        await Menu(racers).run()

    asyncio.run(main())
