# Kustomize Test Coverage Tracking

This document tracks which tests from the reference `kustomize` repository have been ported to kustomizer.

## Legend
- ✅ Ported and passing
- ❌ Not yet ported
- 🔧 Ported but failing (bug identified)
- ⏭️ Skipped (uses unimplemented features)

## Tests from krusty (api/krusty/*_test.go)

### Name Reference Tests (namereference_test.go)
- ✅ TestIssue3489Simplified → `reference/name-reference-suffix`
- ❌ TestIssue3489
- ❌ TestIssue4682_NameReferencesToSelfInAnnotations
- ❌ TestIssue4884_UseLocalConfigAsNameRefSource
- ❌ TestUnrelatedNameReferenceReplacement_Issue4254_Issue3418
- ❌ TestBackReferenceAdmissionPolicy
- ❌ TestEmptyFieldSpecValue

### Namespace Tests (namespaces_test.go)
- ❌ TestNamespacedSecrets
- ❌ TestNameReferenceDeploymentIssue3489
- ❌ TestAddNamePrefixWithNamespace
- ❌ TestNameAndNsTransformation
- ❌ TestNameNotOveriddenForNonCoreApiVersionOnANamespaceKind
- ❌ TestVariablesAmbiguous
- ❌ TestVariablesAmbiguousWorkaround
- ❌ TestVariablesDisambiguatedWithNamespace

### ConfigMap/Generator Tests (configmaps_test.go)
- ✅ TestConfigMapGeneratorLiteralNewline → `reference/configmap-generator-literal-newline`
- ✅ TestConfigMapGeneratorMergeNamePrefix → `reference/configmap-generator-merge-name-prefix`
- ✅ TestDataEndsWithQuotes → `reference/data-ends-with-quotes`
- ✅ TestDataIsSingleQuote → `reference/data-is-single-quote`
- ✅ TestGeneratorBasics → `reference/generator-basics`
- ✅ TestGeneratorFromProperties → `reference/generator-from-properties`
- ✅ TestGeneratorIntVsStringNoMerge → `reference/generator-int-vs-string-no-merge`
- ✅ TestGeneratorIntVsStringWithMerge → `reference/generator-int-vs-string-with-merge`
- ✅ TestGeneratorOverlaysBinaryData → `reference/generator-overlays-binary-data`
- ✅ TestGeneratorOverlays → `reference/generator-overlays`
- ✅ TestGeneratorSimpleOverlay → `reference/generator-simple-overlay`
- ✅ TestIssue3393 → `reference/issue-3393`
- ✅ TestPrefixSuffix → `reference/prefix-suffix`
- ✅ TestPrefixSuffix2 → `reference/prefix-suffix2`
- ❌ TestGeneratorRepeatsInKustomization

### Component Tests (component_test.go)
- ✅ TestComponent cases → multiple tests in `reference/`
- ✅ TestComponentErrors cases → multiple tests in `reference/`
- ✅ TestOrderOfAccumulatedComponent → `reference/order-components-using-a-generated-resource-by-configmapgenerator`

### Basic I/O Tests (basic_io_test.go)
- ✅ TestBasicIO_1 → `reference/basic-io-1`
- ✅ TestBasicIO_2 → `reference/basic-io-2`
- ✅ TestBasicIO3812 → `reference/basic-io-3812`

### Blank Values Tests (blankvalues_test.go)
- ✅ TestBlankNamespace4240 → `reference/blank-namespace-4240`

### Generator Options Tests (generatoroptions_test.go)
- ✅ TestGeneratorOptionsWithBases → `reference/generator-options-with-bases`
- ✅ TestGeneratorOptionsOverlayDisableNameSuffixHash → `reference/generator-options-overlay-disable-name-suffix-hash`
- ✅ TestSecretGenerator → `reference/secret-generator`

### Image Transformer Tests (transformersimage_test.go)
- ❌ TestIssue1281_JsonPatchAndImageTag
- 🔧 TestTransfomersImageDefaultConfig → `reference/transformer-configs-images` (needs digest support)
- ❌ TestTransfomersImageCustomConfig
- ❌ TestTransfomersImageKnativeConfig

### Patch Tests (multiplepatch_test.go)
- 🔧 TestRemoveEmptyDirWithNullFieldInSmp (needs null preservation)
- ❌ TestSimpleMultiplePatches
- ❌ TestPatchPreservesInternalAnnotations
- ❌ TestNonCommutablePatches
- ❌ TestMultiplePatchesNoConflict
- ❌ TestMultiplePatchesWithOnePatchDeleteDirective
- ❌ TestEmptyPatchFilesShouldBeIgnored
- ❌ Many more patch tests...

### Inline Labels Tests (inlinelabels_test.go)
- ❌ TestKustomizationLabels
- ❌ TestKustomizationLabelsInDeploymentTemplate
- ❌ TestKustomizationLabelsInJobTemplate
- ❌ etc...

### Variable Reference Tests (variableref_test.go)
- ⏭️ All tests (vars field not implemented)

### Replacement Transformer Tests (replacementtransformer_test.go)
- ⏭️ All tests (replacements field not implemented)

### Helm Chart Tests (helmchartinflationgenerator_test.go)
- ⏭️ All tests (helm not implemented)

### Remote Loader Tests (remoteloader_test.go)
- ⏭️ All tests (remote loading not implemented)

### Plugin Tests (fnplugin_test.go, transformerplugin_test.go)
- ⏭️ All tests (plugins not implemented)

## Tests from examples/

- ✅ multibases → `reference/multibases`
- ✅ ldap/overlays/production → `reference/ldap-production`
- ✅ ldap/overlays/staging → `reference/ldap-staging`
- ✅ springboot/overlays/production → `reference/springboot-production`
- ✅ springboot/overlays/staging → `reference/springboot-staging`
- ✅ generatorOptions.md → `reference/generator-options-labels-annotations`
- ✅ replicas.md → `reference/replicas-transformer`
- ✅ jsonpatch.md → `reference/json-patch-ingress`
- ✅ patchMultipleObjects.md → `reference/patch-multiple-deployments`
- ❌ helloWorld (uses commonLabels)
- ❌ wordpress (uses vars)
- ❌ mySql
- ❌ loadHttp (uses remote loading)
- ❌ All other examples

## Summary

**Total Tests Ported:** ~46
**Total Tests Passing:** ~77 (including non-reference tests)
**Known Bugs Identified:** 5

- bug-image-digest
- bug-env-file-parsing
- bug-emptydir-null
- bug-multidoc-yaml (fixed)
- bug-name-reference-suffix-cross-contamination

## Next Tests to Port

High priority transformation tests to port next:

1. **Inline Labels/Annotations** - TestKustomizationLabels and related
2. **Extended Patch** - TestExtendedPatch* series (patch with selectors)
3. **Name Reference** - TestIssue3489 (full version), TestIssue4682
4. **Namespace** - TestNamespacedSecrets, TestNameReferenceDeploymentIssue3489
5. **Patch** - TestSimpleMultiplePatches, TestMultiplePatchesNoConflict
6. **Base/Overlay** - TestSmallBase, TestSmallOverlay, TestMediumBase, TestMediumOverlay
