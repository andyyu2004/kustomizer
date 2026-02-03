# Bug: NetworkPolicy commonLabels field spec application fails

## Issue
When applying `commonLabels` to a NetworkPolicy resource, kustomizer fails with an error applying the field spec.

## Error
```
applying field spec `NetworkPolicy.networking.k8s.io` `spec/ingress/from/podSelector/matchLabels` 
to resource networking.k8s.io.v1.NetworkPolicy/nginx
```

## Expected Behavior (from kustomize)
CommonLabels should be successfully applied to:
- NetworkPolicy metadata labels
- NetworkPolicy spec.podSelector.matchLabels
- NetworkPolicy spec.ingress[].from[].podSelector.matchLabels

## Actual Behavior (from kustomizer)
Fails when trying to apply commonLabels to the nested `spec/ingress/from/podSelector/matchLabels` field.

## Test Case
From kustomize test: `kustomize/api/krusty/generatormergeandreplace_test.go` - "TestSimpleBase"

## NetworkPolicy Resource
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: nginx
spec:
  podSelector:
    matchExpressions:
      - {key: app, operator: In, values: [test]}
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: nginx  # commonLabels should merge here
```

The field spec for NetworkPolicy appears to be incorrectly configured or the transformation logic has a bug when dealing with nested podSelector.matchLabels fields.
