
$script:CompanyDir = "Devolutions"
$script:ProgramDir = "Gateway"
$script:GatewayDataPath = [System.Environment]::ExpandEnvironmentVariables("%ProgramData%\${CompanyDir}\${ProgramDir}")

function Register-GatewayService
{
    $ServiceName = "devolutions-gateway"
    $DisplayName = "Devolutions Gateway"
    $Description = "Devolutions Gateway service"

    $Executable = "DevolutionsGateway.exe"

    $params = @{
        Name = $ServiceName
        DisplayName = $DisplayName
        Description = $Description
        WorkingDir = "%ProgramData%\${CompanyDir}\${ProgramDir}"
        BinaryPathName = "%ProgramFiles%\${CompanyDir}\${ProgramDir}\${Executable}"
    }

    New-Service @params
}

function New-JetKeyPair
{
    [CmdletBinding()]
    param(
        [string] $PublicKeyFile,
        [string] $PrivateKeyFile,
        [ValidateSet(1024,2048,4096)]
        [int] $KeySize = 2048,
        [switch] $Force
    )

    if (-Not $PublicKeyFile) {
        $PublicKeyFile = Join-Path $GatewayDataPath "public.pem"
    }
    
    if (-Not $PrivateKeyFile) {
        $PrivateKeyFile = Join-Path $GatewayDataPath "private.key"
    }

    if (-Not (Test-Path $Path $GatewayDataPath)) {
        New-Item -Path $GatewayDataPath -ItemType 'Directory' -Force | Out-Null
    }

    if ((Test-Path -Path $PublicKeyFile) -Or (Test-Path -Path $PrivateKeyFile)) {
        if (-Not $Force) {
            throw "$PublicKeyFile or $PrivateKeyFile already exists, use -Force to overwrite"
        }

        Remove-Item $PublicKeyFile -Force | Out-Null
        Remove-Item $PrivateKeyFile -Force | Out-Null
    }

    & 'step' 'crypto' 'keypair' $PublicKeyFile $PrivateKeyFile '--kty' 'RSA' '--size' $KeySize '--insecure' '--no-password'
}

function New-JetToken
{
    [CmdletBinding()]
    param(
        [string] $PublicKeyFile,
        [string] $PrivateKeyFile,
        [Parameter(Mandatory=$true)]
        [string] $DestinationHost
    )

    if (-Not $PublicKeyFile) {
        $PublicKeyFile = Join-Path $GatewayDataPath "public.pem"
    }
    
    if (-Not $PrivateKeyFile) {
        $PrivateKeyFile = Join-Path $GatewayDataPath "private.key"
    }

    if (-Not (Test-Path -Path $PrivateKeyFile)) {
        throw "$PrivateKeyFile cannot be found."
    }

    $raw_token = [PSCustomObject]@{
        jet_ap = 'rdp'
        jet_cm = 'fwd'
        dst_hst = $DestinationHost
    } | ConvertTo-Json | Out-String

    Write-Host $raw_token

    $nbf_date = Get-Date
    $exp_date = $nbf_date.AddMinutes(2).AddDays(7)

    $nbf = [System.DateTimeOffset]::new($nbf_date).ToUnixTimeSeconds()
    $exp = [System.DateTimeOffset]::new($exp_date).ToUnixTimeSeconds()

    $jwt = $raw_token | & 'step' 'crypto' 'jwt' 'sign' '-' '-nbf' $nbf '-exp' $exp '-subtle' '-key' $PrivateKeyFile

    Write-Host $jwt

    #$jwe = "$jwt" | & 'step' 'crypto' 'jwe' 'encrypt' '-alg' 'RSA-OAEP-256' '-enc' 'A256GCM' '-key' $PublicKeyFile
    #$jwe = "$jwe" | & 'step' 'crypto' 'jose' 'format'
}

#New-JetKeyPair
#New-JetToken -DestinationHost '192.168.25.158'
