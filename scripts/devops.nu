#!/usr/bin/env -S nu --stdin

def 'main build' [
    --target: string,
] {
    cargo build --target $target --release

    mkdir $"jj-bond-($target)"
    cp $"target/($target)/release/jb" $"jj-bond-($target)"

    cp LICENSE $"jj-bond-($target)"
}
