# Test Porting Notes

## Tests to Skip (For Now)

### Replacement-related tests
- replacementtransformer_test.go - All tests (variable replacement feature)
- variableref_test.go - All tests (variable reference feature)
- issue3377_test.go - Uses vars feature
- Any test using "vars:" in kustomization.yaml

### Reason
These features require variable replacement functionality that may not be fully implemented yet.

## Tests to Port

All other krusty tests should be ported.
