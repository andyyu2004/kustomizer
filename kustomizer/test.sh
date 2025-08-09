set -euo pipefail

a=$(mktemp)
b=$(mktemp)

path="../../../partly-argocd/shared/resources/partly"

kustomize --load-restrictor LoadRestrictionsNone --enable-alpha-plugins --enable-exec build $path >"$a" &
cargo run -r -- build $path >"$b"
wait

dyff between --color=on --ignore-order-changes --ignore-whitespace-changes --set-exit-code --exclude spec.compatibilityDate "$a" "$b"
