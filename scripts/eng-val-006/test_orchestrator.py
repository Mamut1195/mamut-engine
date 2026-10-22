#!/usr/bin/env python3
"""Pruebas del orquestador que no publican tiempos del Engine."""

import json
from pathlib import Path
import tempfile
import unittest

from orchestrate import parse_time_verbose, run_block, verify_index
from report import table_markdown, write_report
from stats import quantile, summarize


class StatsTest(unittest.TestCase):
    def test_type_6_on_fifteen_points(self):
        sample = list(range(1, 16))
        self.assertEqual(quantile(sample, 0.25), 4)
        self.assertEqual(quantile(sample, 0.5), 8)
        self.assertEqual(quantile(sample, 0.75), 12)
        stats = summarize(sample)
        self.assertEqual(stats["median"], 8)
        self.assertEqual(stats["iqr"], 8)
        self.assertEqual(stats["min"], 1)
        self.assertEqual(stats["max"], 15)

    def test_interpolation(self):
        self.assertEqual(quantile([1, 2, 3, 4], 0.25), 1.25)
        self.assertEqual(quantile([1, 2, 3, 4], 0.75), 3.75)


class TimeParseTest(unittest.TestCase):
    def test_gnu_time_verbose(self):
        sample = "\n".join(
            [
                "Command being timed: \"mamut-engine\"",
                "User time (seconds): 0.01",
                "Elapsed (wall clock) time (h:mm:ss or m:ss): 0:01.25",
                "Maximum resident set size (kbytes): 4096",
                "Exit status: 0",
            ]
        )
        self.assertEqual(parse_time_verbose(sample), ("1.25", 4096))

    def test_hour_form(self):
        sample = "\n".join(
            [
                "Elapsed (wall clock) time (h:mm:ss or m:ss): 1:02:03.04",
                "Maximum resident set size (kbytes): 8",
            ]
        )
        self.assertEqual(parse_time_verbose(sample), ("3723.04", 8))


class BlockTest(unittest.TestCase):
    def test_discards_a_block_when_a_job_appears(self):
        class Host:
            def __init__(self):
                self.idle = [True, False, True, True]
                self.measures = 0

            def runners_idle(self):
                return self.idle.pop(0)

            def loadavg(self):
                return [0.0, 0.0, 0.0]

            def sleep(self, _seconds):
                return None

            def measure(self, _engine_input, _threads, _cpu_count):
                self.measures += 1
                return "0.10", 12, 0

        host = Host()
        result = run_block(
            host,
            {
                "model": "rc3d-3s",
                "analysis": "static",
                "threads": 1,
                "cpu_count": 2,
                "engine_input": "modelo.json",
            },
            warmup=1,
            measures=1,
            max_attempts=3,
            pause_s=0,
        )
        self.assertEqual(result["discarded_attempts"], 1)
        self.assertEqual(host.measures, 4)
        self.assertEqual(len(result["rows"]), 2)
        self.assertEqual(result["rows"][0]["phase"], "warmup")
        self.assertEqual(result["rows"][1]["phase"], "measure")


class ReportTest(unittest.TestCase):
    def test_tables_come_from_csv_only(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            path.joinpath("sample.csv").write_text(
                "model,analysis,threads,phase,ordinal,elapsed_s,max_rss_kb,exit_code\n"
                "rc3d-3s,static,1,warmup,1,9.00,1,0\n"
                "rc3d-3s,static,1,measure,1,1.00,10,0\n"
                "rc3d-3s,static,1,measure,2,3.00,30,0\n",
                encoding="utf-8",
            )
            path.joinpath("manifest.json").write_text(
                json.dumps({"blocks": [{"elapsed": 999}]}), encoding="utf-8"
            )
            write_report(path)
            text = path.joinpath("tables.md").read_text(encoding="utf-8")
            self.assertIn("rc3d-3s", text)
            self.assertNotIn("999", text)
            self.assertIn("2", text)
            rows = [
                {
                    "model": "rc3d-3s",
                    "analysis": "static",
                    "threads": "1",
                    "phase": "measure",
                    "elapsed_s": "1.00",
                    "max_rss_kb": "10",
                },
                {
                    "model": "rc3d-3s",
                    "analysis": "static",
                    "threads": "1",
                    "phase": "measure",
                    "elapsed_s": "3.00",
                    "max_rss_kb": "30",
                },
            ]
            self.assertEqual(table_markdown(rows), text)
            self.assertTrue(path.joinpath("elapsed.svg").is_file())
            self.assertTrue(path.joinpath("rss.svg").is_file())


class IndexTest(unittest.TestCase):
    def test_published_index_hashes_match_the_files(self):
        verify_index(json.loads(INDEX_TEXT()))


def INDEX_TEXT():
    return (Path(__file__).resolve().parents[2] / "crates/engine-bench/fixtures/index.json").read_text(
        encoding="utf-8"
    )


if __name__ == "__main__":
    unittest.main()
