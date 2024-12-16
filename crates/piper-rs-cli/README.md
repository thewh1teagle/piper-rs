# piper-rs-cli

CLI interface for [piper-rs](https://github.com/thewh1teagle/piper-rs)

## Usage

```console
cargo install piper-rs-cli

wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx
wget https://huggingface.co/rhasspy/piper-voices/resolve/v1.0.0/en/en_US/hfc_female/medium/en_US-hfc_female-medium.onnx.json
wget https://github.com/thewh1teagle/piper-rs/releases/download/espeak-ng-files/espeak-ng-data.tar.gz
tar xf espeak-ng-data.tar.gz
```

Create audio from text

```console
piper-rs-cli en_US-hfc_female-medium.onnx.json "Hello from piper-rs-cli!"
```
