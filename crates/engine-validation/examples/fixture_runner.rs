//! Runner offline. El adaptador Node usa únicamente módulos estándar.
#[cfg(not(target_arch = "wasm32"))]
mod host {
    use engine_validation::fixtures::{self, NativeTarget, Target};
    use std::{
        fs,
        io::Write,
        path::PathBuf,
        process::{Command, Stdio},
    };

    struct NodeTarget {
        wasm: PathBuf,
    }
    impl Target for NodeTarget {
        fn name(&self) -> &str {
            "wasm32-node"
        }
        fn execute(&self, input: &str) -> Result<String, String> {
            let mut child = Command::new("node")
                .arg(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/scripts/run-fixture-wasm.cjs"
                ))
                .arg(&self.wasm)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Node: {e}"))?;
            let write_result = match child.stdin.take() {
                Some(mut stdin) => stdin.write_all(input.as_bytes()).map_err(|e| e.to_string()),
                None => Err("No se pudo abrir stdin de Node".into()),
            };
            let output = child.wait_with_output().map_err(|e| e.to_string())?;
            if !output.status.success() {
                return Err(format!(
                    "Node falló: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            write_result?;
            String::from_utf8(output.stdout).map_err(|e| e.to_string())
        }
    }

    pub fn run() -> Result<bool, String> {
        let mut json = false;
        let mut wasm = None;
        let mut paths = Vec::new();
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--json" => json = true,
                "--wasm" => wasm = Some(PathBuf::from(args.next().ok_or("--wasm requiere ruta")?)),
                _ if arg.starts_with('-') => return Err(format!("Opción desconocida: {arg}")),
                _ => paths.push(PathBuf::from(arg)),
            }
        }
        if paths.is_empty() {
            for entry in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures"))
                .map_err(|e| e.to_string())?
            {
                let path = entry.map_err(|e| e.to_string())?.path();
                if path
                    .extension()
                    .is_some_and(|extension| extension == "json")
                {
                    paths.push(path);
                }
            }
        }
        paths.sort();
        let documents: Vec<_> = paths
            .iter()
            .map(|path| {
                let name = path
                    .file_stem()
                    .and_then(|n| n.to_str())
                    .ok_or("Nombre de fixture no UTF-8")?
                    .to_owned();
                let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
                Ok((name, text))
            })
            .collect::<Result<_, String>>()?;
        let documents: Vec<_> = documents
            .iter()
            .map(|(name, text)| (name.as_str(), text.as_str()))
            .collect();
        let native = NativeTarget;
        let node = wasm.map(|wasm| NodeTarget { wasm });
        let mut targets: Vec<&dyn Target> = vec![&native];
        if let Some(node) = &node {
            targets.push(node);
        }
        let report = fixtures::run(&documents, &targets);
        if json {
            println!("{}", report.json()?);
        } else {
            print!("{}", report.text());
        }
        Ok(report.passed())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    match host::run() {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {}
