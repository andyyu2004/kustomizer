# IAMPolicyGenerator Not Supported

## Test Cases
All 3 tests from kustomize/api/krusty/iampolicygenerator_test.go:
1. `gke-generator` - Ported from TestGkeGenerator
2. `gke-generator-with-namespace` - Ported from TestGkeGeneratorWithNamespace  
3. `gke-generator-with-two` - Ported from TestGkeGeneratorWithTwo

## Issue
Kustomizer does not support the `IAMPolicyGenerator` builtin plugin.

## Error
```
building generator at `apiVersion: builtin
kind: IAMPolicyGenerator
...

Caused by:
    No such file or directory (os error 2)
```

Or:
```
unknown builtin generator kind `IAMPolicyGenerator` at ...
```

## Expected Behavior
The IAMPolicyGenerator should generate Kubernetes ServiceAccounts with the appropriate IAM annotations for GKE workload identity. This allows Kubernetes service accounts to be bound to Google Cloud service accounts.

## Test Files
- `/home/andy/dev/kustomizer/kustomizer/tests/kustomizer/testdata/reference/gke-generator/`
- `/home/andy/dev/kustomizer/kustomizer/tests/kustomizer/testdata/reference/gke-generator-with-namespace/`
- `/home/andy/dev/kustomizer/kustomizer/tests/kustomizer/testdata/reference/gke-generator-with-two/`

## Feature Description
The IAMPolicyGenerator is a builtin generator in kustomize that creates ServiceAccount resources with cloud provider-specific IAM annotations. For GKE, it adds the `iam.gke.io/gcp-service-account` annotation to enable workload identity binding.
