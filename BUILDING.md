Publish new version

```console
pushd crates/espeak-rs/sys
cargo publish
popd

pushd crates/espeak-rs
cargo publish
popd

pushd crates/sonic-rs-sys
cargo publish
popd

cargo publish
```