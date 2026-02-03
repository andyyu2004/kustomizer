# Kustomize to Kustomizer Test Porting Inventory

**Generated:** 2026-02-03
**Purpose:** Comprehensive tracking of all tests to be ported from kustomize to kustomizer

## Executive Summary

- **Total Tests to Port:** ~498
- **Already Ported:** ~86 (17%)
- **Remaining:** ~412 (83%)

## 1. Krusty Integration Tests (kustomize/api/krusty)

**Total:** 290 test functions (excluding replacementtransformer_test.go and variableref_test.go)

### Files to Port (69 files)

| File | Tests | Status | Notes |
|------|-------|--------|-------|
| accumulation_test.go | 6 | Pending | |
| baseandoverlaymedium_test.go | 2 | Partial | TestMediumBase ported |
| baseandoverlaysmall_test.go | 8 | Partial | TestOrderPreserved, TestBaseInResourceList, TestTinyOverlay ported |
| basereusenameprefix_test.go | 1 | Pending | |
| basic_io_test.go | 3 | Pending | |
| blankvalues_test.go | 1 | Pending | |
| chartinflatorplugin_test.go | 1 | Pending | Skip - plugin related |
| complexcomposition_test.go | 6 | Pending | |
| component_test.go | 3 | Pending | |
| configmaps_test.go | 15 | Partial | TestGeneratorIntVsStringNoMerge, TestGeneratorIntVsStringWithMerge ported |
| crd_test.go | 3 | Pending | |
| customconfig_test.go | 5 | Pending | May contain vars - review needed |
| customconfigofbuiltinplugin_test.go | 1 | Pending | |
| customconfigreusable_test.go | 1 | Pending | |
| diamondcomposition_test.go | 8 | Pending | |
| diamondpatchref_test.go | 1 | Pending | |
| diamonds_test.go | 1 | Pending | |
| directoryarrangement_test.go | 1 | Pending | |
| disablenamesuffix_test.go | 2 | Pending | |
| duplicatekeys_test.go | 1 | Pending | |
| extendedpatch_test.go | 11 | Pending | |
| fnplugin_test.go | 14 | Pending | Skip - plugin related |
| generatormergeandreplace_test.go | 7 | Pending | |
| generatoroptions_test.go | 3 | Pending | |
| gvknpatch_test.go | 13 | Partial | TestKeepOriginalGVKN ported |
| helmchartinflationgenerator_test.go | 15 | Pending | Skip - Helm related |
| iampolicygenerator_test.go | 3 | Pending | |
| inlinelabels_test.go | 8 | Pending | |
| inlinepatch_test.go | 5 | Pending | |
| inlinetransformer_test.go | 2 | Pending | |
| intermediateresourceid_test.go | 4 | Pending | Contains vars - review needed |
| issue2896_test.go | 2 | Pending | |
| issue3377_test.go | 1 | Pending | Contains vars - review needed |
| keepemptyarray_test.go | 1 | Pending | |
| kustomizationmetadata_test.go | 1 | Pending | |
| kustomizer_test.go | 1 | Pending | |
| legacy_order_test.go | 1 | Pending | |
| legacyprefixsuffixtransformer_test.go | 1 | Pending | |
| localconfig_test.go | 1 | Pending | |
| managedbylabel_test.go | 1 | Pending | |
| mergeenvfrom_test.go | 2 | Pending | |
| multibytecharacter_test.go | 1 | Pending | |
| multiplepatch_test.go | 18 | Partial | TestPatchesInOneFile ported |
| namedspacedserviceaccounts_test.go | 1 | Pending | |
| nameprefixsuffixpatch_test.go | 1 | Done | TestNamePrefixSuffixPatch ported |
| namereference_test.go | 7 | Partial | TestIssue3489Simplified ported |
| namespacedgenerators_test.go | 2 | Pending | |
| namespaces_test.go | 8 | Partial | TestNamespacedSecrets, TestAddNamePrefixWithNamespace ported; Contains vars - review needed |
| nameupdateinroleref_test.go | 2 | Pending | |
| no_list_items_test.go | 5 | Pending | |
| nullvalues_test.go | 2 | Done | TestNullValues1, TestNullValues2 ported |
| numericcommonlabels_test.go | 1 | Pending | |
| openapicustomschema_test.go | 16 | Pending | |
| originannotation_test.go | 16 | Pending | |
| patchdelete_test.go | 1 | Pending | |
| pluginenv_test.go | 1 | Pending | Skip - plugin related |
| poddisruptionbudget_test.go | 2 | Pending | |
| remoteloader_test.go | 2 | Pending | Skip - remote loading |
| repeatbase_test.go | 1 | Pending | |
| resourceconflict_test.go | 5 | Pending | |
| rolebindingacrossnamespace_test.go | 3 | Pending | |
| simple_test.go | 1 | Done | TestSimple1 ported |
| sortordertransformer_test.go | 9 | Pending | |
| stringquoteblank_test.go | 1 | Pending | |
| transformerannotation_test.go | 7 | Pending | |
| transformerplugin_test.go | 4 | Pending | Skip - plugin related |
| transformersarrays_test.go | 1 | Pending | |
| transformersimage_test.go | 4 | Partial | TestIssue1281_JsonPatchAndImageTag ported |
| validatingwebhook_test.go | 1 | Pending | |

### Excluded Files (2 files, 26 tests)
- **replacementtransformer_test.go** (12 tests) - Skip: replacement transformer not in scope
- **variableref_test.go** (14 tests) - Skip: vars not supported

### Ported Krusty Tests (16 tests)

1. TestAddNamePrefixWithNamespace (namespaces_test.go:748)
2. TestBaseInResourceList (baseandoverlaysmall_test.go:102)
3. TestGeneratorIntVsStringNoMerge (configmaps_test.go:13)
4. TestGeneratorIntVsStringWithMerge (configmaps_test.go:54)
5. TestIssue1281_JsonPatchAndImageTag (transformersimage_test.go:96)
6. TestIssue3489Simplified (namereference_test.go:12)
7. TestKeepOriginalGVKN (gvknpatch_test.go:15)
8. TestMediumBase (baseandoverlaymedium_test.go:61)
9. TestNamePrefixSuffixPatch (nameprefixsuffixpatch_test.go:13)
10. TestNamespacedSecrets (namespaces_test.go:13) - appears twice
11. TestNullValues1 (nullvalues_test.go:12)
12. TestNullValues2 (nullvalues_test.go:66)
13. TestOrderPreserved (baseandoverlaysmall_test.go:15)
14. TestPatchesInOneFile (multiplepatch_test.go:13)
15. TestSimple1 (simple_test.go:12) - appears twice
16. TestTinyOverlay (baseandoverlaysmall_test.go:135)

## 2. Merge2 Tests (kyaml/yaml/merge2)

**Total:** 61 test cases (table-driven tests)

### Files and Test Cases

| File | Test Cases | Status | Notes |
|------|------------|--------|-------|
| element_test.go | 17 | Done | All test cases ported |
| list_test.go | 15 | Done | All test cases ported |
| map_test.go | 17 | Done | All test cases ported |
| scalar_test.go | 12 | Done | All test cases ported |
| merge2_old_test.go | 7 functions | Pending | Review if needed |
| merge2_test.go | 1 function | Pending | Review if needed |
| smpdirective_test.go | 1 function | Pending | Review if needed |

### Ported Merge2 Tests (~70 directories)

Based on directory naming patterns in `/home/andy/dev/kustomizer/kustomizer/tests/kustomizer/testdata/reference/`:

**Element Tests (17):**
- merge-add-element-first
- merge-add-element-second
- merge-container-empty-dest
- merge-container-missing-from-dest
- merge-element-add-list-empty
- merge-element-append
- merge-element-prepend
- keep-element-empty-list-src
- keep-element-missing-in-src
- (and more...)

**List Tests (15):**
- merge-containers-append
- keep-list-missing-from-src
- keep-list-same-value
- keep-list-unspecified
- remove-list-empty-src
- remove-list-null-src
- replace-list-different-value
- replace-list-missing-from-dest
- (and more...)

**Map Tests (17):**
- merge-map-add-field
- merge-map-add-first
- merge-map-add-second
- merge-map-empty-dest
- merge-map-missing-from-dest
- merge-map-update-field
- merge-empty-map-value
- keep-map-empty-src
- keep-map-missing-from-src
- remove-map-null-src
- (and more...)

**Scalar Tests (12):**
- keep-scalar-missing-src-null-dest
- keep-scalar-null-in-dest
- keep-scalar-preserve-null-marker
- keep-scalar-same-value
- keep-scalar-unspecified
- merge-empty-value
- remove-scalar-empty-src
- remove-scalar-null-src
- replace-scalar-different-value
- replace-scalar-missing-from-dest
- (and more...)

**Patch Tests:**
- patch-add-service-port
- patch-delete-container
- patch-delete-entire-list
- patch-delete-list-item-1
- patch-delete-list-item-2
- patch-delete-map-field
- patch-delete-multiple-resources
- patch-delete-nested-field
- patch-delete-nonexistent-elements
- patch-delete-resource
- patch-empty-list
- patch-merge-list
- patch-null-list
- patch-null-map-field
- patch-preserves-internal-annotations
- patch-replace-list
- patch-replace-map-field
- (and more...)

## 3. YAML Tests (kyaml/yaml)

**Total:** 93 test functions

### Files to Port

| File | Tests | Status | Notes |
|------|-------|--------|-------|
| compatibility_test.go | 2 | Pending | |
| datamap_test.go | 1 | Pending | |
| fns_test.go | 28 | Pending | Core YAML functions |
| kfns_test.go | 4 | Pending | |
| mapnode_test.go | 1 | Pending | |
| match_test.go | 3 | Pending | |
| rnode_test.go | 47 | Pending | Large test suite - high priority |
| types_test.go | 6 | Pending | |
| util_test.go | 1 | Pending | |

## 4. Types Tests (api/types)

**Total:** 24 test functions

### Files to Port

| File | Tests | Status | Notes |
|------|-------|--------|-------|
| fieldspec_test.go | 2 | Pending | |
| generatoroptions_test.go | 1 | Pending | |
| helmchartargs_test.go | 1 | Pending | Skip - Helm related |
| kustomization_test.go | 12 | Pending | Important - kustomization structure |
| patch_test.go | 1 | Pending | |
| selector_test.go | 3 | Pending | |
| var_test.go | 4 | Pending | Skip - vars not supported |

## 5. Filter Tests (api/filters)

**Total:** 30 test functions

### Files to Port

| File | Tests | Status | Notes |
|------|-------|--------|-------|
| annotations/annotations_test.go | 1 | Pending | |
| fieldspec/fieldspec_test.go | 2 | Pending | |
| filtersutil/setters_test.go | 3 | Pending | |
| fsslice/fsslice_test.go | 1 | Pending | |
| iampolicygenerator/iampolicygenerator_test.go | 1 | Pending | |
| imagetag/imagetag_test.go | 1 | Pending | |
| imagetag/legacy_test.go | 1 | Pending | |
| labels/labels_test.go | 1 | Pending | |
| nameref/nameref_test.go | 3 | Pending | |
| nameref/seqfilter_test.go | 2 | Pending | |
| namespace/namespace_test.go | 1 | Pending | |
| patchjson6902/patchjson6902_test.go | 1 | Pending | |
| patchstrategicmerge/patchstrategicmerge_test.go | 1 | Pending | |
| prefix/prefix_test.go | 1 | Pending | |
| refvar/expand_test.go | 3 | Pending | Skip - vars related |
| refvar/refvar_test.go | 2 | Pending | Skip - vars related |
| replacement/replacement_test.go | 2 | Pending | |
| replicacount/replicacount_test.go | 1 | Pending | |
| suffix/suffix_test.go | 1 | Pending | |
| valueadd/valueadd_test.go | 1 | Pending | |

## Priority Recommendations

### High Priority (Core Functionality)
1. **rnode_test.go** (47 tests) - Core YAML node operations
2. **fns_test.go** (28 tests) - Core YAML functions
3. **kustomization_test.go** (12 tests) - Kustomization file structure
4. **extendedpatch_test.go** (11 tests) - Extended patch functionality
5. **gvknpatch_test.go** (13 tests) - GVK/Name patching (12 remaining)

### Medium Priority (Integration & Transform)
1. **multiplepatch_test.go** (18 tests) - Multiple patch scenarios (17 remaining)
2. **originannotation_test.go** (16 tests) - Origin tracking
3. **openapicustomschema_test.go** (16 tests) - Schema validation
4. **configmaps_test.go** (15 tests) - ConfigMap generation (13 remaining)
5. **sortordertransformer_test.go** (9 tests) - Resource ordering

### Low Priority (Edge Cases & Specialized)
1. **diamondcomposition_test.go** (8 tests) - Diamond dependency patterns
2. **complexcomposition_test.go** (6 tests) - Complex compositions
3. **accumulation_test.go** (6 tests) - Resource accumulation
4. Plugin-related tests - Skip unless plugin support added
5. Helm-related tests - Skip unless Helm support added

## Tests to Skip

### By Design (Not in Scope)
- **replacementtransformer_test.go** - Replacement transformer
- **variableref_test.go** - Variable references (vars)
- **refvar/** tests - Variable references
- Tests using "vars:" in kustomization files

### External Dependencies
- **helmchartinflationgenerator_test.go** - Helm integration
- **helmchartargs_test.go** - Helm configuration
- **fnplugin_test.go** - KRM function plugins
- **transformerplugin_test.go** - Transformer plugins
- **pluginenv_test.go** - Plugin environment
- **chartinflatorplugin_test.go** - Helm chart plugins
- **remoteloader_test.go** - Remote resource loading

## Progress Tracking

### Summary Statistics
```
Total Test Count: 498
├─ Krusty: 290 (16 ported, 274 remaining)
├─ Merge2: 61 (61 ported via ~70 test dirs)
├─ YAML: 93 (0 ported, 93 remaining)
├─ Types: 24 (0 ported, 24 remaining - 5 skip)
└─ Filters: 30 (0 ported, 30 remaining - 5 skip)

Ported: ~86 (17%)
Remaining: ~412 (83%)
To Skip: ~31 (6%)
```

### Completion by Category
- ✅ **Merge2 Tests:** 100% (61/61 test cases)
- 🔄 **Krusty Tests:** 6% (16/290 functions)
- ⏳ **YAML Tests:** 0% (0/93 functions)
- ⏳ **Types Tests:** 0% (0/19 applicable functions)
- ⏳ **Filter Tests:** 0% (0/25 applicable functions)

## Next Steps

1. **Complete High Priority YAML Tests**
   - Start with rnode_test.go (47 tests) - foundational
   - Then fns_test.go (28 tests) - core operations

2. **Port High Priority Krusty Tests**
   - extendedpatch_test.go (11 tests)
   - gvknpatch_test.go (12 remaining)
   - multiplepatch_test.go (17 remaining)

3. **Port Types Tests**
   - kustomization_test.go (12 tests)
   - Other types tests (7 tests)

4. **Port Filter Tests**
   - Start with core filters (annotations, labels, namespace)
   - Then patch-related filters

5. **Port Remaining Krusty Tests**
   - Generator tests
   - Composition tests
   - Transformer tests

## Notes

- Test counts are based on `func Test` patterns and table-driven test cases
- Some tests may need modification to work without vars support
- Plugin-related tests are skipped unless plugin support is added to kustomizer
- Remote loading tests are skipped unless remote support is added
- Merge2 tests appear to be fully ported based on directory structure analysis
- Attribution comments in test.yaml files follow format: `# Source: TestName from file.go:line`
