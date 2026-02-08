# Bug: labels transformer fields option not supported

## Issue
When using `labels` with `fields` option, kustomizer fails with:
```
Error: unknown field `fields`, expected one of pairs, includeSelectors, includeTemplates
```

## Expected behavior
Should apply labels to custom fields specified in the fields array.

## Test
```bash
kustomizer build .
```

## From kustomize test
inlinelabels_test.go:32 - TestKustomizationLabels
