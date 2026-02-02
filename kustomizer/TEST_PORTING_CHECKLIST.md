# Test Porting Checklist

Quick reference checklist for tracking test porting progress.

## Progress Overview

**Total: 498 tests | Ported: 86 (17%) | Remaining: 412 (83%)**

---

## High Priority Tests (171 tests)

### YAML Core Tests (93 tests) - 0% complete

- [ ] **rnode_test.go** (47 tests) - CRITICAL: Core YAML node operations
- [ ] **fns_test.go** (28 tests) - CRITICAL: Core YAML functions
- [ ] **types_test.go** (6 tests)
- [ ] **kfns_test.go** (4 tests)
- [ ] **match_test.go** (3 tests)
- [ ] **compatibility_test.go** (2 tests)
- [ ] **datamap_test.go** (1 test)
- [ ] **mapnode_test.go** (1 test)
- [ ] **util_test.go** (1 test)

### Types Tests (19 tests, 5 skip) - 0% complete

- [ ] **kustomization_test.go** (12 tests) - CRITICAL
- [ ] **selector_test.go** (3 tests)
- [ ] **fieldspec_test.go** (2 tests)
- [ ] **generatoroptions_test.go** (1 test)
- [ ] **patch_test.go** (1 test)
- [x] ~~helmchartargs_test.go (1 test)~~ - SKIP: Helm
- [x] ~~var_test.go (4 tests)~~ - SKIP: vars

### Krusty High Value Tests (59 tests) - 8% complete

- [ ] **openapicustomschema_test.go** (16 tests)
- [ ] **originannotation_test.go** (16 tests)
- [x] **gvknpatch_test.go** (13 tests) - 1/13 done ✓
- [x] **multiplepatch_test.go** (18 tests) - 1/18 done ✓
- [ ] **extendedpatch_test.go** (11 tests)
- [ ] **sortordertransformer_test.go** (9 tests)

---

## Medium Priority Tests (146 tests)

### Krusty Integration Tests - 7% complete

- [ ] **inlinelabels_test.go** (8 tests)
- [x] **baseandoverlaysmall_test.go** (8 tests) - 3/8 done ✓
- [ ] **diamondcomposition_test.go** (8 tests)
- [x] **namespaces_test.go** (8 tests) - 2/8 done ✓ (may have vars)
- [x] **namereference_test.go** (7 tests) - 1/7 done ✓
- [ ] **transformerannotation_test.go** (7 tests)
- [ ] **generatormergeandreplace_test.go** (7 tests)
- [ ] **complexcomposition_test.go** (6 tests)
- [ ] **accumulation_test.go** (6 tests)
- [x] **configmaps_test.go** (15 tests) - 2/15 done ✓
- [ ] **inlinepatch_test.go** (5 tests)
- [ ] **customconfig_test.go** (5 tests) - may have vars
- [ ] **no_list_items_test.go** (5 tests)
- [ ] **resourceconflict_test.go** (5 tests)
- [ ] **transformersimage_test.go** (4 tests) - 1/4 done ✓
- [ ] **intermediateresourceid_test.go** (4 tests) - has vars
- [ ] **iampolicygenerator_test.go** (3 tests)
- [ ] **generatoroptions_test.go** (3 tests)
- [ ] **rolebindingacrossnamespace_test.go** (3 tests)
- [ ] **component_test.go** (3 tests)
- [ ] **crd_test.go** (3 tests)
- [ ] **basic_io_test.go** (3 tests)
- [ ] **disablenamesuffix_test.go** (2 tests)
- [ ] **inlinetransformer_test.go** (2 tests)
- [ ] **mergeenvfrom_test.go** (2 tests)
- [ ] **namespacedgenerators_test.go** (2 tests)
- [ ] **nameupdateinroleref_test.go** (2 tests)
- [x] **nullvalues_test.go** (2 tests) - 2/2 done ✅
- [ ] **poddisruptionbudget_test.go** (2 tests)
- [ ] **issue2896_test.go** (2 tests)
- [x] **baseandoverlaymedium_test.go** (2 tests) - 1/2 done ✓

### Filter Tests (25 tests, 5 skip) - 0% complete

- [ ] **nameref/nameref_test.go** (3 tests)
- [ ] **filtersutil/setters_test.go** (3 tests)
- [ ] **fieldspec/fieldspec_test.go** (2 tests)
- [ ] **nameref/seqfilter_test.go** (2 tests)
- [ ] **replacement/replacement_test.go** (2 tests)
- [ ] **annotations/annotations_test.go** (1 test)
- [ ] **fsslice/fsslice_test.go** (1 test)
- [ ] **iampolicygenerator/iampolicygenerator_test.go** (1 test)
- [ ] **imagetag/imagetag_test.go** (1 test)
- [ ] **imagetag/legacy_test.go** (1 test)
- [ ] **labels/labels_test.go** (1 test)
- [ ] **namespace/namespace_test.go** (1 test)
- [ ] **patchjson6902/patchjson6902_test.go** (1 test)
- [ ] **patchstrategicmerge/patchstrategicmerge_test.go** (1 test)
- [ ] **prefix/prefix_test.go** (1 test)
- [ ] **replicacount/replicacount_test.go** (1 test)
- [ ] **suffix/suffix_test.go** (1 test)
- [ ] **valueadd/valueadd_test.go** (1 test)
- [x] ~~refvar/expand_test.go (3 tests)~~ - SKIP: vars
- [x] ~~refvar/refvar_test.go (2 tests)~~ - SKIP: vars

---

## Lower Priority Tests (95 tests)

### Krusty Remaining Tests

- [x] **simple_test.go** (1 test) - 1/1 done ✅
- [x] **nameprefixsuffixpatch_test.go** (1 test) - 1/1 done ✅
- [ ] **basereusenameprefix_test.go** (1 test)
- [ ] **blankvalues_test.go** (1 test)
- [ ] **customconfigofbuiltinplugin_test.go** (1 test)
- [ ] **customconfigreusable_test.go** (1 test)
- [ ] **diamondpatchref_test.go** (1 test)
- [ ] **diamonds_test.go** (1 test)
- [ ] **directoryarrangement_test.go** (1 test)
- [ ] **duplicatekeys_test.go** (1 test)
- [ ] **issue3377_test.go** (1 test) - has vars
- [ ] **keepemptyarray_test.go** (1 test)
- [ ] **kustomizationmetadata_test.go** (1 test)
- [ ] **kustomizer_test.go** (1 test)
- [ ] **legacy_order_test.go** (1 test)
- [ ] **legacyprefixsuffixtransformer_test.go** (1 test)
- [ ] **localconfig_test.go** (1 test)
- [ ] **managedbylabel_test.go** (1 test)
- [ ] **multibytecharacter_test.go** (1 test)
- [ ] **namedspacedserviceaccounts_test.go** (1 test)
- [ ] **numericcommonlabels_test.go** (1 test)
- [ ] **patchdelete_test.go** (1 test)
- [ ] **repeatbase_test.go** (1 test)
- [ ] **stringquoteblank_test.go** (1 test)
- [ ] **transformersarrays_test.go** (1 test)
- [ ] **validatingwebhook_test.go** (1 test)

---

## Completed Tests (86 tests)

### Merge2 Tests - 100% complete ✅

- [x] **element_test.go** (17 test cases) - ALL DONE ✅
- [x] **list_test.go** (15 test cases) - ALL DONE ✅
- [x] **map_test.go** (17 test cases) - ALL DONE ✅
- [x] **scalar_test.go** (12 test cases) - ALL DONE ✅

~70 test directories created in `testdata/reference/` covering all merge2 scenarios.

### Ported Krusty Tests (16 tests) ✅

- [x] TestAddNamePrefixWithNamespace
- [x] TestBaseInResourceList
- [x] TestGeneratorIntVsStringNoMerge
- [x] TestGeneratorIntVsStringWithMerge
- [x] TestIssue1281_JsonPatchAndImageTag
- [x] TestIssue3489Simplified
- [x] TestKeepOriginalGVKN
- [x] TestMediumBase
- [x] TestNamePrefixSuffixPatch
- [x] TestNamespacedSecrets
- [x] TestNullValues1
- [x] TestNullValues2
- [x] TestOrderPreserved
- [x] TestPatchesInOneFile
- [x] TestSimple1
- [x] TestTinyOverlay

---

## Tests to Skip (31 tests)

### By Design (26 tests)
- [x] ~~replacementtransformer_test.go (12 tests)~~ - Replacements not in scope
- [x] ~~variableref_test.go (14 tests)~~ - Vars not supported

### External Dependencies (5 tests)
- [x] ~~helmchartinflationgenerator_test.go (15 tests)~~ - Helm
- [x] ~~fnplugin_test.go (14 tests)~~ - Plugins
- [x] ~~transformerplugin_test.go (4 tests)~~ - Plugins
- [x] ~~chartinflatorplugin_test.go (1 test)~~ - Plugins
- [x] ~~pluginenv_test.go (1 test)~~ - Plugins
- [x] ~~remoteloader_test.go (2 tests)~~ - Remote loading

---

## Suggested Work Order

1. **Phase 1: Core YAML (93 tests)**
   - Start with rnode_test.go - foundation for everything
   - Then fns_test.go - core operations
   - Complete other YAML tests

2. **Phase 2: Types (19 tests)**
   - kustomization_test.go - critical structure
   - Other types tests

3. **Phase 3: High Value Krusty (59 tests)**
   - Patch-related tests (extendedpatch, gvknpatch, multiplepatch)
   - Schema and annotation tests

4. **Phase 4: Filters (25 tests)**
   - Core filters first (namespace, labels, annotations)
   - Then specialized filters

5. **Phase 5: Remaining Krusty (95 tests)**
   - Generator tests
   - Composition tests
   - Edge cases

---

## Quick Stats

```
Category          Total    Ported   Remaining   % Complete
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Merge2             61       61         0         100%
Krusty            290       16       274           6%
YAML               93        0        93           0%
Types              24        0        24           0%
Filters            30        0        30           0%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
TOTAL             498       86       412          17%
```

**Next up: rnode_test.go (47 tests) - The foundation!**
