<#
.SYNOPSIS
  Sends the same HTTP requests the StudioPresence plugin sends, and checks how
  UltimatePresence reacts (via GET /status). Run while the app is running.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File tools\simulate-plugin.ps1
  powershell -ExecutionPolicy Bypass -File tools\simulate-plugin.ps1 -Keep   # leave a demo activity up
#>
param(
  [string]$Url = "http://localhost:4455/",
  [switch]$Keep
)

$ErrorActionPreference = "Stop"
$script:failures = 0
$started = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()   # os.time() in the plugin = UTC seconds

function New-Activity([string]$details, [string]$image = "serverscript", [string]$text = "Editing a SERVER Script",
                      [string]$small = "none", [string]$smallDesc = "none") {
  # Same shape as activityCreator.luau + activity.luau (note the plugin's small_image_key typo).
  @{
    updateType = "SET_ACTIVITY"
    activity   = @{
      details    = $details
      state      = "Workspace: Simulated Place"
      timestamps = @{ start = $started }
      assets     = @{ large_image = $image; large_text = $text; small_image = $small; small_image_key = $smallDesc }
    }
  }
}

function Send([object]$body, $raw = $null) {   # untyped: a [string] param would turn $null into ""
  $json = if ($null -ne $raw) { $raw } else { $body | ConvertTo-Json -Depth 6 -Compress }
  $r = Invoke-WebRequest -Uri $Url -Method Post -Body $json -ContentType "application/json" -UseBasicParsing
  return $r.Content
}

function Status { Invoke-RestMethod -Uri ($Url.TrimEnd("/") + "/status") -UseBasicParsing }

function Check([string]$name, [bool]$ok, [string]$detail = "") {
  if ($ok) { Write-Host "  PASS  $name" -ForegroundColor Green }
  else { Write-Host "  FAIL  $name $detail" -ForegroundColor Red; $script:failures++ }
}

function Shown { $s = Status; if ($s.shown) { $s.shown.details } else { $null } }

Write-Host "UltimatePresence plugin simulation -> $Url`n"

$s = Status
Check "status endpoint responds" ($s.app -eq "UltimatePresence") ($s | ConvertTo-Json -Compress)
if ($s.paused) { Write-Host "  note: sharing is paused in the app; 'shown' will stay empty." -ForegroundColor Yellow }

Write-Host "`n1. Editing a script"
$resp = Send (New-Activity "Main (Line 12 / 240)")
Check "responds 'SET Activity' like the original" ($resp -eq "SET Activity") $resp
Check "activity is set" ((Shown) -eq "Main (Line 12 / 240)" -or $s.paused) (Shown)
$st = (Status).shown
Check "timestamp converted to milliseconds" ($s.paused -or $st.startTimestamp -eq ([int64]$started * 1000)) "$($st.startTimestamp)"

Write-Host "`n2. Cursor moves"
Send (New-Activity "Main (Line 13 / 240)") | Out-Null
Check "activity follows" ((Shown) -eq "Main (Line 13 / 240)" -or $s.paused) (Shown)

Write-Host "`n3. Playtest: 'Testing' holds for 3 seconds"
Send (New-Activity "Testing" "studio" "StudioPresence by iArxic" "playtesticon" "Testing") | Out-Null
Check "Testing shown" ((Shown) -eq "Testing" -or $s.paused) (Shown)
$st = (Status).shown
Check "small tooltip taken from small_image_key" ($s.paused -or $st.smallText -eq "Testing") "$($st.smallText)"
Send (New-Activity "Main (Line 13 / 240)") | Out-Null
Check "edit-mode update ignored within 3s" ((Shown) -eq "Testing" -or $s.paused) (Shown)
Start-Sleep -Milliseconds 3200
Send (New-Activity "Main (Line 14 / 240)") | Out-Null
Check "edit-mode update accepted after 3s" ((Shown) -eq "Main (Line 14 / 240)" -or $s.paused) (Shown)

Write-Host "`n4. CLOSE (plugin disabled / Studio closing)"
Send $null '{"updateType":"CLOSE"}' | Out-Null
Check "CLOSE clears the activity" ($null -eq (Shown)) (Shown)

Write-Host "`n5. Malformed and empty bodies clear (original behaviour)"
Send (New-Activity "Developing" "studio" "StudioPresence by iArxic") | Out-Null
Send $null "{this is not json" | Out-Null
Check "invalid JSON clears" ($null -eq (Shown)) (Shown)
Send (New-Activity "Developing" "studio" "StudioPresence by iArxic") | Out-Null
Send $null "" | Out-Null
Check "empty body clears" ($null -eq (Shown)) (Shown)

Write-Host "`n6. Requests from web pages are refused"
try {
  Invoke-WebRequest -Uri $Url -Method Post -Body '{}' -Headers @{ Origin = "https://evil.example" } -UseBasicParsing | Out-Null
  Check "request with Origin header rejected" $false "got 200"
} catch {
  Check "request with Origin header rejected" ($_.Exception.Response.StatusCode.value__ -eq 403) "$($_.Exception.Message)"
}

if ($Keep) {
  Write-Host "`nLeaving a demo activity on Discord (send CLOSE or quit the app to clear)."
  Send (New-Activity "ServerHandler (Line 42 / 310)") | Out-Null
}

Write-Host ""
if ($script:failures -eq 0) { Write-Host "All checks passed." -ForegroundColor Green; exit 0 }
else { Write-Host "$script:failures check(s) failed." -ForegroundColor Red; exit 1 }
