# kustomizer

A fast [kustomize](https://github.com/kubernetes-sigs/kustomize) implementation written in Rust.

## Features

kustomizer implements the core kustomize build pipeline

### Not implemented

- **replacements** / **vars** - field value substitution is not supported
- Container-based KRM functions (exec-based functions are supported)

## Installation

### From GitHub Releases

Download pre-built binaries from the [releases page](https://github.com/andyyu2004/kustomizer/releases). Builds are available for:

- `x86_64-unknown-linux-gnu`
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-gnu`
- `aarch64-unknown-linux-musl`

### Docker

```sh
# Alpine-based image
docker pull ghcr.io/andyyu2004/kustomizer:latest

# Minimal scratch image
docker pull ghcr.io/andyyu2004/kustomizer:latest-scratch
```

### From source

Supports latest stable Rust. Install with Cargo:

```sh
cargo install --git https://github.com/andyyu2004/kustomizer
```

## Usage

```sh
kustomizer build <directory>
```

### `debug diff-reference`

Builds the kustomization and diffs the output against the reference `kustomize` implementation using [`dyff`](https://github.com/homeport/dyff). Useful for verifying correctness. Requires `kustomize` and `dyff` on PATH.

```sh
kustomizer debug diff-reference <directory>
```

## License

[MIT](LICENSE)
