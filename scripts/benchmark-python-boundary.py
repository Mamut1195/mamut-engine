#!/usr/bin/env python3
"""Runs a manual offline benchmark on the installed wheel; does not run in CI."""
import argparse
from copy import deepcopy
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

import mamut_engine as engine
from mamut_engine import _native

ROOT = Path(__file__).resolve().parents[1]


def model(copies):
    """Creates separate copies of the project's own analytical cantilever without solving it."""
    base = json.loads((ROOT / "crates/engine-api/fixtures/input/valid/cantilever-linear-static.json").read_text())
    result = deepcopy(base)
    for field in ("nodes", "members", "supports"):
        result["model"][field] = []
    result["cases"][0]["loads"] = []
    for index in range(copies):
        for node in base["model"]["nodes"]:
            node = deepcopy(node)
            node["id"] += 2 * index
            node["coordinates"][1] += 2 * index
            result["model"]["nodes"].append(node)
        member = deepcopy(base["model"]["members"][0])
        member["id"] = 7 + index
        member["nodes"] = [n + 2 * index for n in member["nodes"]]
        result["model"]["members"].append(member)
        support = deepcopy(base["model"]["supports"][0])
        support["id"] = f"base-{index}"
        support["node"] += 2 * index
        result["model"]["supports"].append(support)
        load = deepcopy(base["cases"][0]["loads"][0])
        load["node"] += 2 * index
        result["cases"][0]["loads"].append(load)
    return json.dumps(result, separators=(",", ":"))


def summary(samples):
    return {"samples_ns": samples, "median_ns": statistics.median(samples),
            "min_ns": min(samples), "max_ns": max(samples)}


def timed(function, repeats, warmups):
    for _ in range(warmups):
        function()
    samples = []
    for _ in range(repeats):
        start = time.perf_counter_ns()
        value = function()
        samples.append(time.perf_counter_ns() - start)
        # Destroys the output outside the interval, as in Rust.
        del value
    return summary(samples)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rust-baseline", type=Path, required=True)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--work-dir", type=Path, required=True)
    parser.add_argument("--repeats", type=int, default=7)
    parser.add_argument("--warmups", type=int, default=2)
    parser.add_argument("--sizes", type=int, nargs="+", default=[1, 16, 64])
    args = parser.parse_args()
    if args.repeats < 1 or args.warmups < 0 or any(n < 1 or n > 64 for n in args.sizes):
        parser.error("repeats >= 1, warmups >= 0 and sizes between 1 and 64")
    args.work_dir.mkdir(parents=True, exist_ok=False)
    report = {"schema": "python-boundary-benchmark/1", "python": platform.python_version(),
              "platform": platform.system(), "machine": platform.machine(),
              "engine": engine.version(), "repeats": args.repeats, "warmups": args.warmups,
              "clock": "perf_counter_ns / Rust Instant", "sizes": []}
    try:
        import numpy
    except ModuleNotFoundError as err:
        if err.name != "numpy":
            raise
        report["numpy"] = None
    else:
        report["numpy"] = numpy.__version__
    for size in args.sizes:
        source = model(size)
        path = args.work_dir / f"cantilevers-{size}.json"
        path.write_text(source)
        baseline = json.loads(subprocess.check_output(
            [str(args.rust_baseline.resolve()), str(path.resolve()),
             str(args.repeats), str(args.warmups)], text=True))
        result = engine.run(source)
        assert result.to_json() == baseline["document"], "The boundary changed results/1"
        env = dict(os.environ, MAMUT_ENGINE_INPUT=str(path.resolve()))
        env.pop("MAMUT_ENGINE_CANCEL_FILE", None)

        def cli():
            completed = subprocess.run(
                [str(args.cli.resolve()), "run", "--catalog", str(ROOT / "crates/engine-cli/catalog.json"),
                 "--workflow", str(ROOT / "crates/engine-cli/run.workflow.json")],
                capture_output=True, text=True, env=env, check=True)
            return completed.stdout

        cli_document = json.loads(cli())
        assert cli_document["engineResults"] == [json.loads(baseline["document"])], "CLI differs"
        methods = {
            "cli_process": cli,
            "native_json": lambda: _native.run(source),
            "facade_run": lambda: engine.run(source),
            "parse_results": lambda: engine.Results.from_json(baseline["document"]),
            "export_lists": result.to_arrays,
        }
        if report["numpy"] is not None:
            result.to_numpy()  # Imports outside the hot interval.
            methods["export_numpy"] = result.to_numpy
        timings = {"rust_api": summary(baseline["nanoseconds"])}
        for name, method in methods.items():
            timings[name] = timed(method, args.repeats, args.warmups)
        rust_ns = timings["rust_api"]["median_ns"]
        report["sizes"].append({"copies": size, "nodes": 2 * size, "members": size,
                                "active_dofs": 6 * size, "free_dofs": 3 * size,
                                "input_bytes": len(source.encode()),
                                "results_bytes": len(baseline["document"].encode()),
                                "results_sha256": hashlib.sha256(baseline["document"].encode()).hexdigest(),
                                "timings": timings,
                                "native_minus_rust_ns": timings["native_json"]["median_ns"] - rust_ns,
                                "facade_minus_rust_ns": timings["facade_run"]["median_ns"] - rust_ns})
    print(json.dumps(report, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
