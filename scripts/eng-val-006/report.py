"""Tablas y gráficos leídos solo de los CSV crudos."""

import csv
from pathlib import Path

from stats import summarize


def load_measurements(directory):
    rows = []
    for path in sorted(Path(directory).glob("*.csv")):
        with path.open(newline="", encoding="utf-8") as handle:
            rows.extend(csv.DictReader(handle))
    return rows


def measurement_rows(rows):
    return [row for row in rows if row["phase"] == "measure"]


def groups(rows):
    buckets = {}
    for row in measurement_rows(rows):
        key = (row["model"], row["analysis"], int(row["threads"]))
        buckets.setdefault(key, {"elapsed_s": [], "max_rss_kb": []})
        buckets[key]["elapsed_s"].append(float(row["elapsed_s"]))
        buckets[key]["max_rss_kb"].append(float(row["max_rss_kb"]))
    return buckets


def table_markdown(rows):
    lines = [
        "| modelo | análisis | hilos | n | mediana s | IQR s | mín s | máx s | mediana KiB | IQR KiB | mín KiB | máx KiB |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for key in sorted(groups(rows)):
        model, analysis, threads = key
        bucket = groups(rows)[key]
        elapsed = summarize(bucket["elapsed_s"])
        memory = summarize(bucket["max_rss_kb"])
        lines.append(
            "| {model} | {analysis} | {threads} | {n} | {median:.6g} | {iqr:.6g} | {min:.6g} | {max:.6g} | {mmed:.6g} | {miqr:.6g} | {mmin:.6g} | {mmax:.6g} |".format(
                model=model,
                analysis=analysis,
                threads=threads,
                n=elapsed["n"],
                median=elapsed["median"],
                iqr=elapsed["iqr"],
                min=elapsed["min"],
                max=elapsed["max"],
                mmed=memory["median"],
                miqr=memory["iqr"],
                mmin=memory["min"],
                mmax=memory["max"],
            )
        )
    return "\n".join(lines) + "\n"


def _series(rows, field):
    summary = []
    for key in sorted(groups(rows)):
        stats = summarize(groups(rows)[key][field])
        summary.append((key, stats))
    return summary


def svg_chart(rows, field, title, unit):
    series = _series(rows, field)
    if not series:
        raise ValueError("el CSV no tiene medidas")
    width = 80 + 140 * len(series)
    height = 360
    peak = max(item[1]["max"] for item in series) or 1.0
    parts = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
        '<rect width="100%" height="100%" fill="#ffffff"/>',
        f'<text x="24" y="28" font-family="sans-serif" font-size="16">{_xml(title)}</text>',
    ]
    for index, (key, stats) in enumerate(series):
        x = 60 + index * 140
        top = 50
        bottom = 280
        scale = (bottom - top) / peak

        def y_of(value, scale=scale, bottom=bottom):
            return bottom - value * scale

        parts.append(
            f'<line x1="{x + 30}" y1="{y_of(stats["min"])}" x2="{x + 30}" y2="{y_of(stats["max"])}" stroke="#1f4e79" stroke-width="2"/>'
        )
        bar_top = y_of(stats["median"])
        parts.append(
            f'<rect x="{x}" y="{bar_top}" width="60" height="{max(bottom - bar_top, 1)}" fill="#4c78a8"/>'
        )
        label = f"{key[0]} {key[1]} t={key[2]}"
        parts.append(
            f'<text x="{x}" y="310" font-family="sans-serif" font-size="11">{_xml(label)}</text>'
        )
        parts.append(
            f'<text x="{x}" y="328" font-family="sans-serif" font-size="11">{stats["median"]:.4g} {_xml(unit)}</text>'
        )
    parts.append("</svg>")
    return "\n".join(parts) + "\n"


def _xml(text):
    return (
        str(text)
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
    )


def write_report(directory):
    directory = Path(directory)
    rows = load_measurements(directory)
    (directory / "tables.md").write_text(table_markdown(rows), encoding="utf-8")
    (directory / "elapsed.svg").write_text(
        svg_chart(rows, "elapsed_s", "Tiempo de pared", "s"), encoding="utf-8"
    )
    (directory / "rss.svg").write_text(
        svg_chart(rows, "max_rss_kb", "Memoria residente máxima", "KiB"),
        encoding="utf-8",
    )
