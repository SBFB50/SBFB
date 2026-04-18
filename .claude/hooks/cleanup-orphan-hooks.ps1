# .claude/hooks/cleanup-orphan-hooks.ps1
#
# Kill orphan node.exe processes that are stuck hook scripts.
# Short-lived hooks (narrate-action.js, nexus-statusline.js,
# sidecar-drain-on-post-tool.js, sidecar-drain-on-stop.js) should exit
# within ~25s max. Anything older than 60s is assumed leaked.
#
# Long-lived viewers (narration-viewer.js, sidecar-input.js) are
# explicitly preserved.
#
# Invoked by SessionStart hook via cleanup-orphan-hooks.sh wrapper.
# Can also be run manually : powershell -File .claude\hooks\cleanup-orphan-hooks.ps1
#
# Safety: uses -Force but only kills processes that (a) match a
# narrow commandline pattern AND (b) are older than 60s. A hook that
# just spawned and is legitimately running <60s is never touched.

$ErrorActionPreference = 'Stop'

$killedCount = 0
$preservedCount = 0

try {
    $procs = Get-CimInstance -ClassName Win32_Process -Filter "Name = 'node.exe'" -ErrorAction SilentlyContinue

    foreach ($p in $procs) {
        $cmd = $p.CommandLine
        if (-not $cmd) { continue }

        # Preserve long-lived viewers explicitly
        if ($cmd -match 'narration-viewer\.js' -or $cmd -match 'sidecar-input\.js') {
            $preservedCount++
            continue
        }

        # Target only known short-lived hook scripts
        $isHookScript = $cmd -match 'narrate-action\.js' -or `
                        $cmd -match 'nexus-statusline\.js' -or `
                        $cmd -match 'sidecar-drain-on-post-tool\.js' -or `
                        $cmd -match 'sidecar-drain-on-stop\.js'
        if (-not $isHookScript) { continue }

        # Age check : creation date -> now. Get-CimInstance returns
        # CreationDate as native DateTime (unlike Get-WmiObject which
        # returns DMTF string). Handle both defensively.
        $createdRaw = $p.CreationDate
        if (-not $createdRaw) { continue }
        try {
            if ($createdRaw -is [DateTime]) {
                $created = $createdRaw
            } else {
                $created = [Management.ManagementDateTimeConverter]::ToDateTime($createdRaw)
            }
        } catch {
            continue
        }
        $ageSec = [int]((Get-Date) - $created).TotalSeconds

        if ($ageSec -gt 60) {
            try {
                Stop-Process -Id $p.ProcessId -Force -ErrorAction Stop
                $killedCount++
                Write-Output ("killed orphan node.exe PID {0} age {1}s" -f $p.ProcessId, $ageSec)
            } catch {
                # Process may have exited between listing and kill; ignore.
            }
        }
    }

    if ($killedCount -eq 0 -and $preservedCount -eq 0) {
        Write-Output "cleanup-orphan-hooks: no node.exe hook processes found"
    } elseif ($killedCount -eq 0) {
        Write-Output ("cleanup-orphan-hooks: {0} viewer(s) preserved, 0 orphans" -f $preservedCount)
    } else {
        Write-Output ("cleanup-orphan-hooks: killed {0} orphan(s), preserved {1} viewer(s)" -f $killedCount, $preservedCount)
    }
} catch {
    Write-Output ("cleanup-orphan-hooks: error during scan: {0}" -f $_.Exception.Message)
    exit 0
}

exit 0
