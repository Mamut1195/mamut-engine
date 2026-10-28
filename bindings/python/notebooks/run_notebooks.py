"""Ejecuta notebooks con el intérprete actual, sin guardar sus outputs."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import sys
from tempfile import TemporaryDirectory


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "notebooks", nargs="*", type=Path,
        help="Notebooks a ejecutar; por defecto, todos los de esta carpeta",
    )
    args = parser.parse_args()
    try:
        from nbclient import NotebookClient
    except ImportError:
        print("ERROR: falta nbclient; provisiona nbclient, nbformat e ipykernel "
              "en este entorno Python antes de ejecutar los notebooks.", file=sys.stderr)
        return 2
    try:
        import ipykernel  # noqa: F401
        import nbformat
        from jupyter_client import AsyncKernelManager
        from jupyter_client.kernelspec import KernelSpecManager
    except ImportError as error:
        print(f"ERROR: dependencia de notebooks no disponible ({error.name}); "
              "se requieren nbformat, ipykernel y jupyter_client.", file=sys.stderr)
        return 2

    paths = args.notebooks or sorted(Path(__file__).resolve().parent.glob("*.ipynb"))
    if not paths:
        print("ERROR: no hay notebooks para ejecutar.", file=sys.stderr)
        return 2

    with TemporaryDirectory(prefix="mamut-notebooks-") as temporary:
        temporary_root = Path(temporary)
        kernels = temporary_root / "kernels"
        kernel_name = "mamut-notebooks"
        specification = kernels / kernel_name
        specification.mkdir(parents=True)
        (specification / "kernel.json").write_text(json.dumps({
            "argv": [sys.executable, "-m", "ipykernel_launcher", "-f", "{connection_file}"],
            "display_name": "MAMUT notebooks", "language": "python",
        }), encoding="utf-8")
        environment = dict(os.environ)
        environment["PYTHONDONTWRITEBYTECODE"] = "1"
        for key in ("IPYTHONDIR", "JUPYTER_RUNTIME_DIR", "JUPYTER_CONFIG_DIR", "MPLCONFIGDIR"):
            directory = temporary_root / key.lower()
            directory.mkdir()
            environment[key] = str(directory)

        failures: list[str] = []
        for path in paths:
            print(f"Ejecutando {path.name}...", flush=True)
            try:
                notebook = nbformat.read(path, as_version=4)
                nbformat.validate(notebook)
                manager = AsyncKernelManager(
                    kernel_name=kernel_name,
                    kernel_spec_manager=KernelSpecManager(kernel_dirs=[str(kernels)]),
                    connection_file=str(temporary_root / "connection.json"),
                )
                client = NotebookClient(
                    notebook, km=manager, timeout=120, allow_errors=False,
                    force_raise_errors=True,
                    resources={"metadata": {"path": str(path.resolve().parent)}},
                )
                # Cerrar también el manager suministrado, tanto en éxito como en error.
                client.execute(env=environment, cleanup_kc=True)
            except Exception as error:
                print(f"ERROR en {path.name}: {error}", file=sys.stderr)
                failures.append(path.name)
                continue
            print(f"OK: {path.name}", flush=True)

    if failures:
        print(f"ERROR: fallaron {len(failures)} de {len(paths)} notebooks: "
              + ", ".join(failures), file=sys.stderr)
        return 1

    print(f"{len(paths)} notebooks completados; outputs solo en memoria.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
