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

## Performance

~20x faster than kustomize on a semi-realistic configuration (measured with [hyperfine](https://github.com/sharkdp/hyperfine)).

| Command            | Mean                | Range                |
|--------------------|---------------------|----------------------|
| `kustomize build`  | 1.599 s ± 0.297 s   | 1.031 s … 1.866 s   |
| `kustomizer build` | 83.6 ms ± 11.1 ms   | 66.8 ms … 110.3 ms  |

Reproduce with the test fixture in this repo:

```sh
KFLAGS='--load-restrictor LoadRestrictionsNone --enable-alpha-plugins --enable-exec'
DIR='kustomizer/tests/kustomizer/testdata/realistic/example-com/envs/production'
hyperfine --warmup 3 "kustomize build $KFLAGS $DIR" "kustomizer build $KFLAGS $DIR"
```

## Bug Reports

Please [open an issue](https://github.com/andyyu2004/kustomizer/issues/new) with a self-contained shell script in a code block that reproduces the problem. The script should write out the necessary files and run `kustomizer debug diff-reference` to show the discrepancy. For example:

````sh
#!/bin/sh
set -e
dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT

mkdir -p "$dir/base"

cat > "$dir/base/kustomization.yaml" << 'EOF'
resources:
  - deployment.yaml
namePrefix: dev-
EOF

cat > "$dir/base/deployment.yaml" << 'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
spec:
  replicas: 1
EOF

kustomizer debug diff-reference "$dir/base"
````

## Contributing

Contributions are welcome. Please open an issue before submitting large changes.

## License

[MIT](LICENSE)
