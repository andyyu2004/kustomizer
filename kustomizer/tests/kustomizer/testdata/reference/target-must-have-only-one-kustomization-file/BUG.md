# Bug: Multiple kustomization files not detected

## Issue
Kustomizer should fail when a directory contains multiple kustomization files (both `kustomization.yaml` and `kustomization.yml`), but it currently succeeds.

## Expected Behavior (from kustomize)
Should fail with error message: "Found multiple kustomization files"

## Actual Behavior (from kustomizer)
Succeeds without error (presumably uses one of the files and ignores the other)

## Test Case
From kustomize test: `kustomize/api/krusty/accumulation_test.go` - "TestTargetMustHaveOnlyOneKustomizationFile"

This directory contains:
- kustomization.yaml
- kustomization.yml

Both are valid kustomization files. Kustomize properly detects this ambiguity and fails, but kustomizer does not.
