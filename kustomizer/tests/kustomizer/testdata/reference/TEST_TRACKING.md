# Go Kustomize Test Porting Tracker

This file tracks which tests from the Go kustomize test suite have been ported to the Rust implementation.

## Status Legend
- ✅ PORTED - Test successfully ported and passing
- 🔄 IN_PROGRESS - Currently being worked on
- ❌ FAILED - Attempted but failed due to missing functionality
- ⏸️ SKIPPED - Intentionally skipped (e.g. plugin tests, complex features)
- ⏳ TODO - Not yet attempted

## Test Files and Functions

### baseandoverlaysmall_test.go
- ✅ TestOrderPreserved - order-preserved (already ported in ai-generated)
- ✅ TestBaseInResourceList - base-in-resource-list (already ported in ai-generated) 
- ✅ TestTinyOverlay - tiny-overlay (already ported in ai-generated)
- ✅ TestSmallBase - small-base (already ported in ai-generated)
- ⏳ TestSmallOverlay - Complex overlay with labels, images, patches
- ❌ TestSharedPatchDisAllowed - Load restrictions testing
- ❌ TestSharedPatchAllowed - Load restrictions testing  
- ❌ TestSmallOverlayJSONPatch - JSON 6902 patches (not implemented)

### simple_test.go
- ✅ TestSimple1 - simple-patch (already ported in ai-generated)

### configmaps_test.go  
- ⏳ TestGeneratorIntVsStringNoMerge
- ⏳ TestGeneratorIntVsStringWithMerge
- ⏳ TestGeneratorFromProperties
- ⏳ TestGeneratorBasics
- ⏳ TestGeneratorRepeatsInKustomization
- ⏳ TestIssue3393
- ⏳ TestGeneratorSimpleOverlay
- ⏳ TestGeneratorOverlaysBinaryData
- ⏳ TestGeneratorOverlays
- ⏳ TestConfigMapGeneratorMergeNamePrefix
- ⏳ TestConfigMapGeneratorLiteralNewline
- ⏳ TestDataEndsWithQuotes
- ⏳ TestDataIsSingleQuote
- ⏳ TestPrefixSuffix
- ⏳ TestPrefixSuffix2

### component_test.go
- ⏳ TestComponent
- ⏳ TestComponentErrors
- ⏳ TestOrderOfAccumulatedComponent

### generatoroptions_test.go
- ⏳ TestSecretGenerator
- ⏳ TestGeneratorOptionsWithBases
- ⏳ TestGeneratorOptionsOverlayDisableNameSuffixHash

### Basic I/O Tests (basic_io_test.go)
- ⏳ TestBasicIO_1
- ⏳ TestBasicIO_2  
- ⏳ TestBasicIO3812

### Namespace Tests
- ⏳ TestBlankNamespace4240 (blankvalues_test.go)

### Name/Label Transformers
- ⏳ TestAddManagedbyLabel (managedbylabel_test.go)
- ⏳ TestNameUpdateInRoleRef (nameupdateinroleref_test.go)
- ⏳ TestNameUpdateInRoleRef2

### Extended Patch Tests (extendedpatch_test.go)
- ⏳ TestExtendedPatchNameSelector
- ⏳ TestExtendedPatchGvkSelector
- ⏳ TestExtendedPatchLabelSelector
- ⏳ TestExtendedPatchNameGvkSelector
- ⏳ TestExtendedPatchNameLabelSelector
- ⏳ TestExtendedPatchGvkLabelSelector
- ⏳ TestExtendedPatchNameGvkLabelSelector
- ⏳ TestExtendedPatchNoMatch
- ⏳ TestExtendedPatchWithoutTarget
- ⏳ TestExtendedPatchNoMatchMultiplePatch
- ⏳ TestExtendedPatchMultiplePatchOverlapping

### Replacement Transformer Tests (replacementtransformer_test.go)
- ⏳ TestReplacementsField
- ⏳ TestReplacementsFieldWithPath
- ⏳ TestReplacementsFieldWithPathMultiple
- ⏳ TestReplacementTransformerWithDiamondShape
- ⏳ TestReplacementTransformerWithOriginalName
- ⏳ TestReplacementTransformerWithNamePrefixOverlay
- ⏳ TestReplacementTransformerWithNamespaceOverlay
- ⏳ TestReplacementTransformerWithConfigMapGenerator
- ⏳ TestReplacementTransformerWithSuffixTransformerAndReject
- ⏳ TestReplacementTransformerAppendToAnnotationUsingRegex
- ⏳ TestReplacementTransformerServiceNamespaceUrlUsingRegex
- ⏳ TestReplacementTransformerWithSuffixTransformerAndRejectUsingRegex

## Plugin Tests (Likely to Skip)
- ⏸️ All tests in fnplugin_test.go (exec/container plugins)
- ⏸️ All tests in transformerannotation_test.go (transformer plugins)
- ⏸️ All tests in customconfig_test.go (custom configs)

## Feature Coverage Status
- ✅ Basic patches (modern format)
- ✅ namePrefix/nameSuffix  
- ✅ labels
- ✅ commonAnnotations
- ✅ configMapGenerator/secretGenerator
- ✅ replicas transformation
- ✅ components
- ✅ namespace setting
- ❌ patchesJson6902 (JSON patches)
- ❌ patchesStrategicMerge (legacy format)
- ❌ images transformation
- ❌ commonLabels (legacy format)

## Next Tests to Port (Priority Order)
1. TestSmallOverlay - Complex overlay example
2. TestGeneratorBasics - More generator testing  
3. TestComponent - Component functionality
4. TestSecretGenerator - Secret generation
5. Basic I/O tests for fundamental functionality