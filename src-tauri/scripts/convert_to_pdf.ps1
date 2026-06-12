param(
    [Parameter(Mandatory=$true)][string]$InputPath,
    [Parameter(Mandatory=$true)][string]$OutputPath,
    [Parameter(Mandatory=$true)][int]$Format
)

$ext = [System.IO.Path]::GetExtension($InputPath).ToLower()

function Convert-Word($inputPath, $outputPath, $format) {
    $app = $null; $doc = $null
    try {
        try { $app = New-Object -ComObject Word.Application } catch {
            $app = New-Object -ComObject KWPS.Application
        }
        $app.Visible = $false
        $app.DisplayAlerts = 0
        $doc = $app.Documents.Open($inputPath, $false, $true)
        $doc.SaveAs([ref]$outputPath, [ref]$format)
    } finally {
        if ($doc) { try { $doc.Close($false) } catch {} }
        if ($app) { try { $app.Quit() } catch {} }
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($doc) 2>$null
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($app) 2>$null
        [GC]::Collect()
    }
}

function Convert-Excel($inputPath, $outputPath, $format) {
    $app = $null; $wb = $null
    try {
        try { $app = New-Object -ComObject Excel.Application } catch {
            $app = New-Object -ComObject KET.Application
        }
        $app.Visible = $false
        $app.DisplayAlerts = $false
        $wb = $app.Workbooks.Open($inputPath, $false, $true)
        if ($format -eq 0) {
            # PDF
            $wb.ExportAsFixedFormat(0, $outputPath)
        } else {
            $wb.SaveAs([ref]$outputPath, [ref]$format)
        }
    } finally {
        if ($wb) { try { $wb.Close($false) } catch {} }
        if ($app) { try { $app.Quit() } catch {} }
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($wb) 2>$null
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($app) 2>$null
        [GC]::Collect()
    }
}

function Convert-PowerPoint($inputPath, $outputPath, $format) {
    $app = $null; $pres = $null
    try {
        try { $app = New-Object -ComObject PowerPoint.Application } catch {
            $app = New-Object -ComObject KWPP.Application
        }
        $pres = $app.Presentations.Open($inputPath, $false, $false, $false)
        $pres.SaveAs($outputPath)
    } finally {
        if ($pres) { try { $pres.Close() } catch {} }
        if ($app) { try { $app.Quit() } catch {} }
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($pres) 2>$null
        [System.Runtime.Interopservices.Marshal]::ReleaseComObject($app) 2>$null
        [GC]::Collect()
    }
}

switch ($ext) {
    { $_ -in '.doc','.docx','.rtf','.txt' } { Convert-Word $InputPath $OutputPath $Format }
    { $_ -in '.xls','.xlsx','.csv' } { Convert-Excel $InputPath $OutputPath $Format }
    { $_ -in '.ppt','.pptx' } { Convert-PowerPoint $InputPath $OutputPath $Format }
    default { Write-Error "不支持的格式: $ext"; exit 1 }
}
