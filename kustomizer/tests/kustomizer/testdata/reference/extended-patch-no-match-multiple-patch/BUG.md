# Bug: Extended Patch with Multiple Non-Matching Patches

## Test
Ported from: kustomize/api/krusty/extendedpatch_test.go - TestExtendedPatchNoMatchMultiplePatch

## Description
When using multiple patches where none of the targets match any resources, kustomizer fails to parse the kustomization.yaml file with error: "data did not match any variant of untagged enum Target"

## Expected Behavior
Both patch targets should be parsed successfully, and since neither matches any resources, both patches should be silently ignored (no resources modified).

## Actual Behavior
Kustomizer fails to parse the kustomization.yaml and exits with an error.

## Test Case
```yaml
patches:
- path: patch.yaml
  target:
    name: no-match
- path: patch.yaml
  target:
    name: busybox
    kind: Job
```

Since no resource is named "no-match" and there's no Job named "busybox", both patches should be ignored and all resources should remain unchanged.

## Error
```
data did not match any variant of untagged enum Target
```

## Kustomize Behavior
Kustomize correctly handles this case - it parses successfully and ignores both patches since there are no matching resources. See output.yaml for the expected result (unchanged resources).
