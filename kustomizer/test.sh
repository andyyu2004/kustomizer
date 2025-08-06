set -euo pipefail

a=$(mktemp)
b=$(mktemp)

cargo run -- build ../../../partly-argocd/shared/resources/partly >"$a" &

kustomize --load-restrictor LoadRestrictionsNone --enable-alpha-plugins --enable-exec build ../../../partly-argocd/shared/resources/partly >"$b" &

wait

dyff between --color=on --ignore-order-changes --ignore-whitespace-changes --set-exit-code "$a" "$b"
