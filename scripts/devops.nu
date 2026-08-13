#!/usr/bin/env -S nu --stdin

def main [] {}

def 'main build' [
    --target: string,
] {
    if ((uname | get operating-system) =~ 'Linux') and (which '^zip' | is-empty) {
        apt-get update
        apt-get install -yq zip
    }

    cargo build --target $target --release

    mkdir $"jj-bond-($target)"
    let exe_ext = if ((uname | get operating-system) =~ 'Windows') { '.exe' } else { '' }
    cp $"target/($target)/release/jb($exe_ext)" $"jj-bond-($target)"

    cp LICENSE $"jj-bond-($target)"
}
