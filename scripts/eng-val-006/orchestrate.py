#!/usr/bin/env python3
"""Orquestador del Engine para ENG-VAL-006.

Mide el binario release. No lanza la campaña contra el VPS: el valor por defecto
queda disponible para esa corrida, y --smoke hace una pasada local mínima.
Las tablas salen después, solo de los CSV.

`LocalHost.measure` retiene en memoria, hasta que el proceso termina, el stdout
del motor y el stderr de GNU time (`subprocess.run` con `capture_output=True`).
No lee esa salida en flujo: una corrida completa cabe en RAM antes de extraer
el bloque de `time -v`.
"""

import argparse
import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import subprocess
import time

from report import write_report

ROOT = Path(__file__).resolve().parents[2]
INDEX = ROOT / "crates/engine-bench/fixtures/index.json"
CATALOG = ROOT / "crates/engine-cli/catalog.json"
WORKFLOW = ROOT / "crates/engine-cli/run.workflow.json"
TIME_BIN = "/usr/bin/time"
BLAS_VARIABLES = (
    "OMP_NUM_THREADS",
    "OPENBLAS_NUM_THREADS",
    "MKL_NUM_THREADS",
    "MKL_DYNAMIC",
    "BLIS_NUM_THREADS",
    "VECLIB_MAXIMUM_THREADS",
    "NUMEXPR_NUM_THREADS",
    "GOTO_NUM_THREADS",
)
ELAPSED_HMS = re.compile(
    r"Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):\s+(\d+):(\d{2}):(\d{2})\.(\d{2})$"
)
ELAPSED_MS = re.compile(
    r"Elapsed \(wall clock\) time \(h:mm:ss or m:ss\):\s+(\d+):(\d{2})\.(\d{2})$"
)
RSS = re.compile(r"Maximum resident set size \(kbytes\):\s+(\d+)")
ANALYSES = ("static", "modal", "spectrum")


class CampaignError(RuntimeError):
    pass


def sha256_file(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_index():
    return json.loads(INDEX.read_text(encoding="utf-8"))


def verify_index(index):
    for model in index["models"]:
        _check_hash(ROOT / model["path"], model["sha256"])
        for kind in ANALYSES:
            item = model["engine_input"][kind]
            _check_hash(ROOT / item["path"], item["sha256"])


def _check_hash(path, expected):
    actual = sha256_file(path)
    if actual != expected:
        raise CampaignError(f"sha256 distinto en {path}")


def parse_time_verbose(stderr):
    elapsed = None
    rss = None
    for raw in stderr.splitlines():
        line = raw.strip()
        matched = ELAPSED_HMS.match(line)
        if matched:
            hours, minutes, seconds, hundredths = (int(part) for part in matched.groups())
            total = ((hours * 60 + minutes) * 60 + seconds) * 100 + hundredths
            elapsed = _format_hundredths(total)
            continue
        matched = ELAPSED_MS.match(line)
        if matched:
            minutes, seconds, hundredths = (int(part) for part in matched.groups())
            total = (minutes * 60 + seconds) * 100 + hundredths
            elapsed = _format_hundredths(total)
            continue
        matched = RSS.search(line)
        if matched:
            rss = int(matched.group(1))
    if elapsed is None or rss is None:
        raise CampaignError("la salida de time -v no trae tiempo de pared o memoria")
    return elapsed, rss


def _format_hundredths(hundredths):
    whole, fraction = divmod(hundredths, 100)
    return f"{whole}.{fraction:02d}"


def runner_worker_pids():
    found = []
    proc = Path("/proc")
    for entry in proc.iterdir():
        if not entry.name.isdigit():
            continue
        if _is_runner_worker(entry):
            found.append(int(entry.name))
    return found


def _is_runner_worker(entry):
    try:
        command = (entry / "comm").read_text(encoding="utf-8", errors="replace").strip()
    except OSError:
        command = ""
    if command == "Runner.Worker":
        return True
    try:
        raw = (entry / "cmdline").read_bytes()
    except OSError:
        return False
    text = raw.replace(b"\x00", b" ").decode("utf-8", "replace")
    return "Runner.Worker" in text


def read_loadavg():
    parts = Path("/proc/loadavg").read_text(encoding="utf-8").split()
    return [float(parts[0]), float(parts[1]), float(parts[2])]


def cpu_model():
    for line in Path("/proc/cpuinfo").read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("model name"):
            return line.split(":", 1)[1].strip()
    return platform.processor()


def operating_system():
    data = {}
    for line in Path("/etc/os-release").read_text(encoding="utf-8").splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            data[key] = value.strip().strip('"')
    return data.get("PRETTY_NAME", platform.system())


class LocalHost:
    def __init__(self, time_bin, engine_bin, timeout_s):
        self.time_bin = time_bin
        self.engine_bin = engine_bin
        self.timeout_s = timeout_s

    def runners_idle(self):
        return not runner_worker_pids()

    def loadavg(self):
        return read_loadavg()

    def sleep(self, seconds):
        time.sleep(seconds)

    def measure(self, engine_input, threads, cpu_count):
        env = os.environ.copy()
        env["LC_ALL"] = "C"
        env["LANG"] = "C"
        env["MAMUT_ENGINE_INPUT"] = str(engine_input)
        for name in BLAS_VARIABLES:
            env[name] = "FALSE" if name == "MKL_DYNAMIC" else str(threads)
        cpus = "0" if threads == 1 else f"0-{min(threads, cpu_count) - 1}"
        command = [
            "taskset",
            "-c",
            cpus,
            self.time_bin,
            "-v",
            str(self.engine_bin),
            "run",
            "--catalog",
            str(CATALOG),
            "--workflow",
            str(WORKFLOW),
        ]
        try:
            # stdout del motor y stderr de time quedan en memoria hasta el
            # fin del proceso. No hay lector en flujo.
            completed = subprocess.run(
                command,
                cwd=ROOT,
                env=env,
                check=False,
                capture_output=True,
                timeout=self.timeout_s,
            )
        except subprocess.TimeoutExpired as error:
            raise CampaignError("la corrida superó el tiempo límite") from error
        if completed.returncode not in (0, 1):
            tail = completed.stderr.decode("utf-8", "replace")[-2000:]
            raise CampaignError(f"el Engine terminó con código {completed.returncode}: {tail}")
        elapsed, rss = parse_time_verbose(completed.stderr.decode("utf-8", "replace"))
        return elapsed, rss, completed.returncode


def run_block(host, spec, warmup, measures, max_attempts, pause_s):
    discarded = 0
    while True:
        if not host.runners_idle():
            discarded += 1
            if discarded > max_attempts:
                raise CampaignError("los runners no quedaron idle antes del bloque")
            host.sleep(pause_s)
            continue
        started = host.loadavg()
        rows = []
        for phase, count in (("warmup", warmup), ("measure", measures)):
            for ordinal in range(1, count + 1):
                elapsed, rss, code = host.measure(
                    spec["engine_input"], spec["threads"], spec["cpu_count"]
                )
                rows.append(
                    {
                        "model": spec["model"],
                        "analysis": spec["analysis"],
                        "threads": spec["threads"],
                        "phase": phase,
                        "ordinal": ordinal,
                        "elapsed_s": elapsed,
                        "max_rss_kb": rss,
                        "exit_code": code,
                    }
                )
        finished = host.loadavg()
        if not host.runners_idle():
            discarded += 1
            if discarded > max_attempts:
                raise CampaignError("un job apareció durante el bloque; no se publica")
            host.sleep(pause_s)
            continue
        return {
            "rows": rows,
            "loadavg_start": started,
            "loadavg_end": finished,
            "discarded_attempts": discarded,
        }


def write_csv(path, rows):
    fields = [
        "model",
        "analysis",
        "threads",
        "phase",
        "ordinal",
        "elapsed_s",
        "max_rss_kb",
        "exit_code",
    ]
    with Path(path).open("w", newline="\n", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=fields, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def git_commit():
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout.strip()


def assert_release(path):
    parts = Path(path).resolve().parts
    if "release" not in parts or "debug" in parts:
        raise CampaignError("se requiere el binario release de mamut-engine")
    if not os.access(path, os.X_OK):
        raise CampaignError("el binario release no es ejecutable")


def select_models(index, smoke):
    models = index["models"]
    if smoke:
        return [model for model in models if model["id"] == "rc3d-3s"]
    return models


def campaign(args, host):
    index = load_index()
    verify_index(index)
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    cpu_count = os.cpu_count() or 1
    warmup = 1 if args.smoke else 3
    measures = 1 if args.smoke else 15
    analyses = ("static",) if args.smoke else ANALYSES
    thread_counts = (1,) if args.smoke else (1, cpu_count)
    commit = git_commit()
    blocks = []
    for model in select_models(index, args.smoke):
        for analysis in analyses:
            for threads in thread_counts:
                spec = {
                    "model": model["id"],
                    "analysis": analysis,
                    "threads": threads,
                    "cpu_count": cpu_count,
                    "engine_input": ROOT / model["engine_input"][analysis]["path"],
                }
                result = run_block(
                    host, spec, warmup, measures, args.max_attempts, args.pause_s
                )
                csv_name = f"{model['id']}-{analysis}-t{threads}.csv"
                write_csv(output / csv_name, result["rows"])
                blocks.append(
                    {
                        "id": f"{model['id']}-{analysis}-t{threads}",
                        "model": model["id"],
                        "analysis": analysis,
                        "threads": threads,
                        "taskset": "0" if threads == 1 else f"0-{threads - 1}",
                        "loadavg_start": result["loadavg_start"],
                        "loadavg_end": result["loadavg_end"],
                        "runners_idle_before": True,
                        "runners_idle_after": True,
                        "discarded_attempts": result["discarded_attempts"],
                        "csv": csv_name,
                    }
                )
    manifest = {
        "schema": "bench-manifest/1",
        "commit": commit,
        "binary": {
            "path": str(Path(args.binary).resolve()),
            "sha256": sha256_file(args.binary),
            "profile": "release",
        },
        "cpu": {"model": cpu_model(), "cores": cpu_count},
        "os": operating_system(),
        "kernel": platform.release(),
        "protocol": {
            "warmup": warmup,
            "measures": measures,
            "analyses": list(analyses),
            "threads": list(thread_counts),
            "time_bin": str(host.time_bin),
            "blas_variables": list(BLAS_VARIABLES),
            "accepted_exit_codes": [0, 1],
        },
        "blocks": blocks,
    }
    (output / "manifest.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    write_report(output)
    return manifest


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--smoke", action="store_true")
    parser.add_argument("--time-bin", type=Path, default=Path(TIME_BIN))
    parser.add_argument("--max-attempts", type=int, default=8)
    parser.add_argument("--pause-s", type=float, default=5.0)
    parser.add_argument("--timeout-s", type=float, default=3600.0)
    parser.add_argument("--allow-nonrelease", action="store_true")
    args = parser.parse_args(argv)
    if not args.time_bin.is_file():
        parser.error(f"no está el binario de time: {args.time_bin}")
    if args.allow_nonrelease:
        if not os.access(args.binary, os.X_OK):
            parser.error("el binario no es ejecutable")
    else:
        assert_release(args.binary)
    host = LocalHost(args.time_bin, args.binary, args.timeout_s)
    campaign(args, host)


if __name__ == "__main__":
    main()
