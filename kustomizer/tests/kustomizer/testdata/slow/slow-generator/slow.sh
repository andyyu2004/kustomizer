sleep "${1:-0.5}"

echo "
apiVersion: config.kubernetes.io/v1
kind: ResourceList
items: []
"
