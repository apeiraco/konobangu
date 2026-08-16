set dotenv-load
# Preserve native status and positional argv through PowerShell's scriptblock invocation.
set windows-shell := ["pwsh.exe", "-NoLogo", "-CommandWithArgs", "$ErrorActionPreference = 'Stop'; $justRecipeArgs = @($args | Select-Object -Skip 1); & ([scriptblock]::Create($args[0])) @justRecipeArgs; if ($null -ne $LASTEXITCODE) { exit $LASTEXITCODE }"]

# Native argv forwarding uses shell positional parameters without reconstructing command strings.
recipe_args := if os() == "windows" { "@($args | Select-Object -Skip 1)" } else { '"$@"' }

import "justfiles/setup.just"
import "justfiles/dev.just"
import "justfiles/build.just"
import "justfiles/quality.just"
import "justfiles/test.just"
import "justfiles/release.just"
import "justfiles/tools.just"
