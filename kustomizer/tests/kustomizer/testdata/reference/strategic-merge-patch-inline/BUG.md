# Bug: Inline strategic merge patches not working

## Issue
Inline strategic merge patches in the `patchesStrategicMerge` field are not working. Getting error: "loading strategic merge patches"

## Expected Behavior (from kustomize)
Should accept inline YAML patches in the `patchesStrategicMerge` array and apply them correctly.

## Actual Behavior (from kustomizer)
Fails with error: "loading strategic merge patches"

This suggests kustomizer is not properly parsing inline strategic merge patches.

## Test Case
From kustomize test: `kustomize/api/krusty/inlinepatch_test.go` - "TestStrategicMergePatchInline"

## Configuration
```yaml
patchesStrategicMerge:
- |-
  apiVersion: apps/v1
  kind: Deployment
  metadata:
    name: nginx
  spec:
    template:
      spec:
        containers:
          - name: nginx
            image: image1
```

The array contains an inline YAML patch (not a file path). Kustomize supports this but kustomizer does not.
