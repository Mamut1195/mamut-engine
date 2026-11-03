#!/usr/bin/env python3
"""Linux namespace proof with an import-safe, strict IPv6 route parser."""

import errno
import os
from pathlib import Path
import re
import socket
import struct
import sys

# Route format and flag values are from the coordinator-supplied contract.
HEX_WIDTHS = (32, 2, 32, 2, 32, 8, 8, 8, 8)
RTF_UP = 0x1
RTF_REJECT = 0x200


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def parse_ipv6_route(line):
    """Return nine integer fields and the interface; reject malformed records."""
    fields = line.split()
    if len(fields) != 10:
        raise ValueError("malformed IPv6 route field count")

    values = []
    for field, width in zip(fields[:9], HEX_WIDTHS):
        if re.fullmatch(rf"[0-9A-Fa-f]{{{width}}}", field) is None:
            raise ValueError("malformed IPv6 route hexadecimal field")
        values.append(int(field, 16))
    if values[1] > 128 or values[3] > 128:
        raise ValueError("malformed IPv6 route prefix length")

    interface = fields[9]
    try:
        interface_bytes = interface.encode("utf-8")
    except UnicodeEncodeError:
        raise ValueError("malformed IPv6 route interface") from None
    if (
        not interface
        or len(interface_bytes) > 15
        or "/" in interface
        or "\x00" in interface
        or any(character.isspace() for character in interface)
    ):
        raise ValueError("malformed IPv6 route interface")
    return tuple(values) + (interface,)


def check_ipv6_routes(table):
    """Validate every nonblank record, then reject any usable default route.

    Blank lines are ignored. Source prefixes do not exempt destination defaults.
    REJECT routes and routes without UP are unusable under the supplied contract.
    """
    usable_default = False
    for line in table.splitlines():
        if not line.strip():
            continue
        route = parse_ipv6_route(line)
        destination, destination_prefix = route[:2]
        flags = route[8]
        if (
            destination == 0
            and destination_prefix == 0
            and flags & RTF_UP
            and not (flags & RTF_REJECT)
        ):
            usable_default = True
    if usable_default:
        raise ValueError("usable IPv6 default route present")


def verify_namespace(outer_ns):
    # Keep the Linux-only import inside the proof so parser imports are portable.
    import fcntl

    require(sys.version_info >= (3, 11), "Python 3.11+ required")
    require(os.getuid() != 0 and os.geteuid() != 0, "root inside namespace rejected")
    inside = os.readlink("/proc/self/ns/net")
    require(inside != outer_ns, "network namespace is unchanged")
    status = dict(
        line.split(":", 1)
        for line in Path("/proc/self/status").read_text().splitlines()
        if ":" in line
    )
    for key in ("CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"):
        require(int(status[key].strip(), 16) == 0, "capabilities were not fully dropped")
    require(status["NoNewPrivs"].strip() == "1", "no-new-privs missing")
    require(sorted(name for _, name in socket.if_nameindex()) == ["lo"],
            "non-loopback interface present")
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as control:
        result = fcntl.ioctl(control.fileno(), 0x8913, struct.pack("256s", b"lo"))
        require(not (struct.unpack_from("H", result, 16)[0] & 1), "loopback is UP")
    for line in Path("/proc/net/route").read_text().splitlines()[1:]:
        fields = line.split()
        require(len(fields) >= 3 and fields[1] != "00000000", "IPv4 default route present")
    try:
        check_ipv6_routes(Path("/proc/net/ipv6_route").read_text())
    except ValueError as error:
        raise SystemExit(str(error)) from None
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as connection:
        connection.settimeout(3)
        try:
            connection.connect(("1.1.1.1", 443))
        except OSError as error:
            require(error.errno == errno.ENETUNREACH,
                    "numeric TCP probe failed without the required ENETUNREACH")
        else:
            raise SystemExit("numeric TCP connection unexpectedly succeeded")
    print(f"PASS kernel network namespace: outer={outer_ns} inside={inside} uid={os.getuid()}")
    print("PASS capabilities dropped; only loopback, DOWN; no IPv4 default routes; "
          "no usable IPv6 default routes; TCP errno=ENETUNREACH")


def main(argv):
    require(len(argv) == 1, "usage: check-offline-namespace.py <outer-netns>")
    verify_namespace(argv[0])
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
