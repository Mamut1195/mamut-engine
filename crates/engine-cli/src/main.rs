// SPDX-License-Identifier: AGPL-3.0-only

#![forbid(unsafe_code)]

fn main() -> std::process::ExitCode {
    let args = match std::env::args_os()
        .skip(1)
        .map(|arg| arg.into_string())
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(args) => args,
        // El parser clasifica el argumento inválido sin reflejar bytes privados.
        Err(_) => vec![String::new()],
    };
    let mut control = engine_cli::Control::default();
    if let Some(path) = std::env::var_os("MAMUT_ENGINE_CANCEL_FILE") {
        control = control.with_cancel_file(path.into());
    }
    let cli = engine_cli::EngineCli::new(std::env::var_os("MAMUT_ENGINE_INPUT").map(Into::into))
        .with_control(control);
    #[cfg(unix)]
    let cli = cli.with_signals();
    std::process::ExitCode::from(cli.run(
        &args,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    ))
}
