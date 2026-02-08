# Bug: patch target labelSelector not working

## Issue
When using `patches` with `target.labelSelector`, kustomizer fails with:
```
Error: data did not match any variant of untagged enum Target
```

## Expected behavior
Should patch only resources matching the label selector (app=nginx in this case).

## Test
```bash
kustomizer build .
```

## From kustomize test
extendedpatch_test.go:295 - TestExtendedPatchLabelSelector
