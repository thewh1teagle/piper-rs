## Gotchas

### Link failed on Windows:

If you encounter linking errors such as

```console
error LNK2019: unresolved external symbol __std_mismatch_1 referenced in function "private: class onnxruntime::common::Status
```

Please make sure your visual studio is >= 17.11 (Update through Visual studio installer)

## Publish new version

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

Note: Please don't create PR from your main branch. only from new feature branch!