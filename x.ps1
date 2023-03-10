param (
    [Parameter(Position = 0)]
    [ValidateSet("build", "watch")]
    $mode = "watch"
)

$build_commands = @{
    # "shared" = "cargo build -r";
    "."         = "trunk build";
    "src-tauri" = "cargo build -r";
}

$watch_services = @{
    # "shared" = "cargo watch -x 'build'"
    "."         = "trunk serve";
    "src-tauri" = "cargo watch -x 'run r' ";
}

switch ($mode) {
    "build" { 
        foreach ($path in $build_commands.Keys) {
            Start-Process -Wait -WorkingDirectory $path -FilePath "pwsh" -ArgumentList @("-Command", $build_commands.$path) -NoNewWindow
        }
    }
    "watch" { 
        $watch_services.Keys | ForEach-Object {
            "start"
            Start-Job -Name $_ -ScriptBlock {
                param($path, $command)
                # "$path $command"
                Start-Process -Wait -WorkingDirectory $path -FilePath "pwsh" -ArgumentList @("-Command", $command) -NoNewWindow
            } -ArgumentList $_, $watch_services.$_
        }
        
        $watch_services.Keys | ForEach-Object {Receive-Job $_ -Wait -AutoRemoveJob}
    }
    Default { "Invalid mode" }
}