param(
    [Parameter(Mandatory = $true)][string]$Binary,
    [ValidateRange(1, 100)][int]$Repetitions = 3,
    [string[]]$Model = @(),
    [ValidateSet('static', 'modal', 'spectrum')][string[]]$Analysis = @('static', 'modal', 'spectrum'),
    [ValidateRange(1, 2147483647)][int]$TimeoutSeconds = 60,
    # Sampled working set is not a hard cap and may be exceeded between polls.
    [ValidateRange(1, 9223372036854775807)][long]$MaxObservedWorkingSetBytes = 1GB
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $root
$binaryPath = (Resolve-Path -LiteralPath $Binary).Path
$indexPath = Join-Path $root 'crates/engine-bench/fixtures/index.json'
$index = Get-Content -LiteralPath $indexPath -Raw | ConvertFrom-Json
if ($index.schema -ne 'bench-index/1') { throw 'Unexpected fixture index schema' }
foreach ($selectedModel in $Model) {
    if ($selectedModel -notin @($index.models.id)) { throw "Unknown model: $selectedModel" }
}
$rustc = (& rustc --version | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw 'rustc --version failed' }
$commit = (& git rev-parse HEAD | Out-String).Trim()
if ($LASTEXITCODE -ne 0) { throw 'git rev-parse failed' }
$dirty = @(& git status --porcelain)
if ($LASTEXITCODE -ne 0) { throw 'git status failed' }
$rows = @()
foreach ($modelItem in $index.models) {
    if ($Model.Count -gt 0 -and $modelItem.id -notin $Model) { continue }
    foreach ($analysisItem in $Analysis) {
        $fixture = $modelItem.engine_input.$analysisItem
        $path = Join-Path $root $fixture.path
        $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($hash -ne $fixture.sha256) { throw "Input hash mismatch: $($fixture.path)" }
        $inputText = [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
        $firstOutput = $null
        for ($i = 1; $i -le $Repetitions; $i++) {
            $info = [System.Diagnostics.ProcessStartInfo]::new()
            $info.FileName = $binaryPath; $info.Arguments = 'run --input -'
            $info.UseShellExecute = $false; $info.CreateNoWindow = $true
            $info.RedirectStandardInput = $true; $info.RedirectStandardOutput = $true; $info.RedirectStandardError = $true
            foreach ($key in @('MAMUT_ENGINE_INPUT', 'MAMUT_ENGINE_CANCEL_FILE', 'MAMUT_ENGINE_TIMEOUT')) { $info.EnvironmentVariables.Remove($key) }
            $process = [System.Diagnostics.Process]::new(); $process.StartInfo = $info
            $clock = [System.Diagnostics.Stopwatch]::StartNew()
            $peak = 0
            $started = $false
            try {
                if (-not $process.Start()) { throw 'Process failed to start' }
                $started = $true
                $stdoutTask = $process.StandardOutput.ReadToEndAsync()
                $stderrTask = $process.StandardError.ReadToEndAsync()
                $inputTask = $process.StandardInput.WriteAsync($inputText)
                $inputClosed = $false
                while ($true) {
                    if ($clock.Elapsed.TotalSeconds -ge $TimeoutSeconds) { throw "Engine timed out for $($modelItem.id)/$analysisItem" }
                    if (-not $process.HasExited) {
                        $process.Refresh()
                        $peak = [Math]::Max($peak, $process.WorkingSet64)
                        if ($peak -gt $MaxObservedWorkingSetBytes) { throw "Engine exceeded sampled working set threshold for $($modelItem.id)/$analysisItem" }
                    }
                    if (-not $inputClosed -and $inputTask.IsCompleted) {
                        $null = $inputTask.GetAwaiter().GetResult()
                        $process.StandardInput.Close()
                        $inputClosed = $true
                    }
                    if ($process.HasExited -and $inputClosed -and $stdoutTask.IsCompleted -and $stderrTask.IsCompleted) { break }
                    Start-Sleep -Milliseconds 10
                }
                $clock.Stop()
                $stdout = $stdoutTask.GetAwaiter().GetResult()
                $stderr = $stderrTask.GetAwaiter().GetResult()
                $exitCode = $process.ExitCode
            } finally {
                if ($started) {
                    try { if (-not $process.HasExited) { $process.Kill(); $null = $process.WaitForExit(1000) } } catch {}
                }
                try { $process.Dispose() } catch {}
            }
            if ($exitCode -notin @(0, 1)) { throw "Engine failed for $($modelItem.id)/$analysisItem, exit $exitCode : $stderr" }
            $envelope = $stdout | ConvertFrom-Json
            if ($envelope.operation -ne 'run' -or $envelope.exitCode -ne $exitCode -or $envelope.outcome -notin @('ok', 'warning') -or @($envelope.engineResults).Count -ne 1 -or $envelope.engineResults[0].schema -ne 'results/1' -or $envelope.engineResults[0].status -notin @('ok', 'warning')) { throw "Unexpected result for $($modelItem.id)/$analysisItem" }
            if (($exitCode -eq 0 -and ($envelope.outcome -cne 'ok' -or $envelope.engineResults[0].status -cne 'ok')) -or ($exitCode -eq 1 -and ($envelope.outcome -cne 'warning' -or $envelope.engineResults[0].status -cne 'warning'))) { throw "Unexpected result for $($modelItem.id)/$analysisItem" }
            if ($null -ne $firstOutput -and $stdout -cne $firstOutput) { throw "Output changed for $($modelItem.id)/$analysisItem" }; $firstOutput = $stdout
            $rows += [pscustomobject]@{ model = $modelItem.id; nodes = $modelItem.nodes; members = $modelItem.members; analysis = $analysisItem; input = $fixture.path; input_sha256 = $hash; repetition = $i; elapsed_ms = $clock.Elapsed.TotalMilliseconds; observed_peak_working_set_bytes = $peak; exit_code = $exitCode; result_status = $envelope.engineResults[0].status; engine_version = $envelope.engineVersion }
        }
    }
}
if ($rows.Count -eq 0) { throw 'No rows produced' }
[pscustomobject]@{ schema = 'eng-val-004-native-candidate/1'; commit = $commit; checkout_dirty = ($dirty.Count -gt 0); binary = $binaryPath; binary_sha256 = (Get-FileHash -LiteralPath $binaryPath -Algorithm SHA256).Hash.ToLowerInvariant(); rustc_at_execution = $rustc; powershell = $PSVersionTable.PSVersion.ToString(); os = [System.Environment]::OSVersion.VersionString; processor_count = [System.Environment]::ProcessorCount; repetitions = $Repetitions; rows = $rows; wasm = 'NOT RUN' } | ConvertTo-Json -Depth 20
