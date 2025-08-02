set -euo pipefail

cargo run -- build ../../../partly-argocd/clusters/prod-eu/resources/partly/
cargo run -- build ../../../partly-argocd/clusters/staging-au/resources/partly/
cargo run -- build ../../../partly-argocd/clusters/hq/resources/partly-dev9/
