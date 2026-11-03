"""Synthetic route-parser tests; these do not execute the namespace proof."""

import importlib.util
from pathlib import Path
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "check-offline-namespace.py"
SPEC = importlib.util.spec_from_file_location("offline_namespace", SCRIPT)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("namespace parser module could not be loaded")
NAMESPACE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(NAMESPACE)

# Synthetic metadata shaped solely from the supplied /proc format contract.
BASE_FIELDS = (
    "0" * 32, "00", "0" * 32, "00", "0" * 32,
    "ffffffff", "00000001", "00000000", "00000001", "lo",
)


def route(flags=0x1, destination="0" * 32, dst_prefix="00",
          src_prefix="00", interface="lo"):
    fields = list(BASE_FIELDS)
    fields[0] = destination
    fields[1] = dst_prefix
    fields[3] = src_prefix
    fields[8] = f"{flags:08x}"
    fields[9] = interface
    return " ".join(fields)


class IPv6RouteTests(unittest.TestCase):
    def assert_rejected(self, table, message):
        with self.assertRaisesRegex(ValueError, "^" + message + "$"):
            NAMESPACE.check_ipv6_routes(table)

    def test_up_default_is_rejected(self):
        for flags in (0x1, 0x3):
            with self.subTest(flags=flags):
                self.assert_rejected(route(flags), "usable IPv6 default route present")

    def test_unusable_defaults_are_accepted(self):
        for flags in (0x200, 0x201, 0x0, 0x00200200):
            with self.subTest(flags=flags):
                NAMESPACE.check_ipv6_routes(route(flags))

    def test_valid_nondefault_up_route_is_accepted(self):
        NAMESPACE.check_ipv6_routes(
            route(destination="1" + "0" * 31, dst_prefix="40")
        )

    def test_both_destination_conditions_are_required(self):
        NAMESPACE.check_ipv6_routes(route(dst_prefix="01"))
        NAMESPACE.check_ipv6_routes(route(destination="1" + "0" * 31))

    def test_source_prefix_does_not_exempt_default(self):
        self.assert_rejected(route(src_prefix="40"),
                             "usable IPv6 default route present")

    def test_empty_and_blank_tables_are_accepted(self):
        for table in ("", " \t\n\n", "\n" + route(0x200) + "\n \t\n"):
            with self.subTest(table=table):
                NAMESPACE.check_ipv6_routes(table)

    def test_mixed_table_rejects_usable_default(self):
        table = "\n".join((route(0x200), route(),
                           route(destination="1" + "0" * 31)))
        self.assert_rejected(table, "usable IPv6 default route present")

    def test_wrong_field_counts_are_rejected(self):
        for fields in (BASE_FIELDS[:-1], BASE_FIELDS + ("extra",)):
            with self.subTest(count=len(fields)):
                self.assert_rejected(" ".join(fields),
                                     "malformed IPv6 route field count")

    def test_each_hex_field_rejects_bad_hex_and_wrong_widths(self):
        widths = (32, 2, 32, 2, 32, 8, 8, 8, 8)
        for index, width in enumerate(widths):
            for value in ("g" + "0" * (width - 1),
                          "0" * (width - 1), "0" * (width + 1)):
                with self.subTest(field=index, value=value):
                    fields = list(BASE_FIELDS)
                    fields[index] = value
                    self.assert_rejected(" ".join(fields),
                                         "malformed IPv6 route hexadecimal field")

    def test_both_prefix_overflows_are_rejected(self):
        for index in (1, 3):
            with self.subTest(field=index):
                fields = list(BASE_FIELDS)
                fields[index] = "81"
                self.assert_rejected(" ".join(fields),
                                     "malformed IPv6 route prefix length")

    def test_prefix_upper_bound_is_accepted(self):
        NAMESPACE.check_ipv6_routes(route(dst_prefix="80", src_prefix="80"))

    def test_invalid_interfaces_are_rejected(self):
        for interface in ("bad/name", "a" * 16, "bad\x00name", "é" * 8):
            with self.subTest(interface=interface):
                self.assert_rejected(route(interface=interface),
                                     "malformed IPv6 route interface")
        for interface in ("", "bad name", "bad\tname"):
            with self.subTest(interface=interface):
                self.assert_rejected(route(interface=interface),
                                     "malformed IPv6 route field count")

    def test_interface_length_uses_utf8_bytes(self):
        for interface in ("a" * 15, "é" * 7):
            with self.subTest(interface=interface):
                NAMESPACE.check_ipv6_routes(route(0x200, interface=interface))

    def test_malformed_nondefault_is_rejected(self):
        fields = route(destination="1" + "0" * 31, dst_prefix="40").split()
        fields[5] = "zzzzzzzz"
        self.assert_rejected(" ".join(fields),
                             "malformed IPv6 route hexadecimal field")

    def test_usable_default_does_not_skip_later_validation(self):
        self.assert_rejected(route() + "\n" + " ".join(BASE_FIELDS[:-1]),
                             "malformed IPv6 route field count")

    def test_uppercase_hex_is_accepted_and_parser_returns_values(self):
        fields = ["A" * width for width in (32, 2, 32, 2, 32, 8, 8, 8, 8)]
        fields[1] = fields[3] = "0A"
        line = "\t".join(fields + ["lo"])
        parsed = NAMESPACE.parse_ipv6_route(line)
        self.assertEqual(parsed[0], int("A" * 32, 16))
        self.assertEqual(parsed[1], 10)
        self.assertEqual(parsed[9], "lo")
        NAMESPACE.check_ipv6_routes(line)


if __name__ == "__main__":
    unittest.main()
