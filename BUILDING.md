## Gotchas

### Link failed on Windows:

If you encounter linking errors such as

```console
error LNK2019: unresolved external symbol __std_mismatch_1 referenced in function "private: class onnxruntime::common::Status
```

Please make sure your visual studio is >= 17.11 (Update through Visual studio installer)

## Publish new version

```console
cargo publish -p espeak-rs-sys
cargo publish -p espeak-rs
cargo publish -p sonic-rs-sys
cargo publish -p piper-rs
cargo publish -p piper-rs-cli
```

Note: Please don't create PR from your main branch. only from new feature branch!

## Build for Linux arm64 (Raspberry PI)

Ubuntu packages: `librust-alsa-sys-dev build-essential`
Rquires [Docker](https://www.docker.com/products/docker-desktop)

```console
cargo binstall cross
# Disable default features so it won't compile espeak-ng-data
cross build --no-default-features -p espeak-rs-sys --target aarch64-unknown-linux-gnu
```