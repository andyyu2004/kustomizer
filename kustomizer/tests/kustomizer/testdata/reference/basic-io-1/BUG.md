# Bug: Unquoted boolean and number values in annotations fail to load

## Issue
YAML files with unquoted boolean (`true`, `false`) or number values in annotation fields fail to load in kustomizer.

## Error
```
load resource tests/kustomizer/testdata/reference/basic-io-1/service.yaml
```

## Expected Behavior (from kustomize)
Should accept unquoted boolean and number values in annotations and automatically quote them in the output:
```yaml
metadata:
  annotations:
    port: 8080        # number - should be quoted in output
    happy: true       # boolean - should be quoted in output
    color: green      # string - stays as is
```

Output should be:
```yaml
metadata:
  annotations:
    color: green
    happy: "true"     # quoted
    port: "8080"      # quoted
```

## Actual Behavior (from kustomizer)
Fails to load the resource file entirely when annotations contain unquoted boolean or number values.

## Test Case
From kustomize test: `kustomize/api/krusty/basic_io_test.go` - "TestBasicIO_1"

## Notes
- `basic-io-2` test works fine because all annotation values are pre-quoted in the input
- This is about YAML parsing/serialization handling of scalar types in string-expected fields
- Kustomize properly handles this by accepting typed values and stringifying them
