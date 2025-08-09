set -euo pipefail

a=$(mktemp)
b=$(mktemp)

path="../../../partly-argocd/clusters/hq/resources/partly-dev9"

kustomize --load-restrictor LoadRestrictionsNone --enable-alpha-plugins --enable-exec build $path >"$a" &
cargo run -- build $path >"$b"
wait

dyff between --color=on --ignore-order-changes --ignore-whitespace-changes --set-exit-code "$a" "$b"
