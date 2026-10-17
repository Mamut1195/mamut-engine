// SPDX-License-Identifier: AGPL-3.0-only

#![forbid(unsafe_code)]

fn main() -> std::process::ExitCode {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut control = engine_cli::Control::default();
    if let Some(path) = std::env::var_os("MAMUT_ENGINE_CANCEL_FILE") {
        control = control.with_cancel_file(path.into());
    }
    let cli = engine_cli::EngineCli::new(std::env::var_os("MAMUT_ENGINE_INPUT").map(Into::into))
        .with_control(control);
    std::process::ExitCode::from(cli.run(
        &args,
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    ))
}
