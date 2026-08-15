$locator = New-Object -ComObject WbemScripting.SWbemLocator
$service = $locator.ConnectServer(".", "root\wmi")

# 0x20000 = wbemFlagUseAmendedQualifiers
# 0x10    = wbemFlagReturnImmediately
$flags = 0x20000 -bor 0x10

# Query matching FileIo, FileIo_V1, FileIo_V2 and all their subclasses
$wql = @"
SELECT * FROM meta_class
WHERE __CLASS LIKE 'FileIo%'
   OR __SUPERCLASS LIKE 'FileIo%'
"@

$classes = $service.ExecQuery($wql, "WQL", $flags)

foreach ($c in $classes) {
    Write-Output $c.GetObjectText_(0)
}
