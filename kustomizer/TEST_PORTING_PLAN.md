# Kustomize Test Suite Discovery & Porting Plan

## Overview
This document catalogs all test suites in the kustomize repository that should be ported to kustomizer.

## Test Suite Categories

### 1. Strategic Merge Patch Tests (kyaml/yaml/merge2/)
**Priority: HIGH** - Core merge functionality

**Files:**
- **element_test.go**: 17 test cases, 650 lines
- **list_test.go**: 15 test cases, 490 lines
- **map_test.go**: 17 test cases, 466 lines
- **scalar_test.go**: 12 test cases, 241 lines

**Total**: 61 test cases
**Status**: ~24/61 tests ported (39%)

**Test Categories**:
- Element merging (containers, volumes, etc.)
- Map field operations
- Scalar field operations
- List operations
- Patch directives ($patch: delete/replace/merge)
- Null and empty value handling

#### Remaining Tests to Port (37 tests):

**From element_test.go (7 remaining)**:
- [ ] remove Element -- null in src
- [ ] keep list -- list missing from src
- [ ] merge Element prepend variations
- [ ] no infer merge keys merge using schema
- [ ] no infer merge keys merge using explicit schema as line comment
- [ ] infer merge keys merge
- [ ] merge_primitive_items

**From map_test.go (13 remaining)**:
- [ ] strategic merge patch delete 1
- [ ] strategic merge patch delete 2
- [ ] strategic merge patch delete 3
- [ ] strategic merge patch delete 4
- [ ] strategic merge patch replace 1
- [ ] merge Map -- add Map first
- [ ] merge Map -- add Map second
- [ ] port patch has no protocol
- [ ] keep map -- empty list in src (completed but may need verification)
- [ ] remove Map -- null in src
- [ ] Verify key style behavior

**From scalar_test.go (11 remaining)**:
- [ ] replace scalar -- different value in dest
- [ ] replace scalar -- missing from dest
- [ ] remove scalar -- empty in src
- [ ] remove scalar -- null in src, empty in dest
- [ ] remove scalar -- null in src, null in dest
- [ ] keep scalar -- preserves null marker (~)
- [ ] Other scalar edge cases

**From list_test.go (6 remaining)**:
- [ ] strategic merge patch delete for volumes
- [ ] remove k8s deployment containers -- $patch directive
- [ ] keep List -- unspecified in src (completed)
- [ ] remove List -- null in src

### 2. Krusty Integration Tests (api/krusty/)
**Priority: HIGH** - Real-world kustomization scenarios

**Total**: 72 test files, ~300+ test functions

**Key Test Files by Priority**:

#### Phase 2A: Base & Overlay Tests (High Priority)
- [ ] **baseandoverlaysmall_test.go** (8 tests, 498 lines)
  - TestOrderPreserved
  - TestBaseInResourceList
  - TestTinyOverlay
  - TestSmallBase
  - TestSmallOverlay
  - TestSharedPatchDisAllowed

- [ ] **baseandoverlaymedium_test.go** (2 tests, 307 lines)
  - Medium complexity overlay scenarios

- [ ] **complexcomposition_test.go** (6 tests, 556 lines)
  - Multi-layer composition patterns

#### Phase 2B: Patch Tests (High Priority)
- [ ] **multiplepatch_test.go** (18 tests, 1874 lines)
  - TestPatchesInOneFile
  - TestMultiplePatchesWithOnePatchDeleteDirective
  - TestSinglePatchWithMultiplePatchDeleteDirectives
  - Multiple strategic merge patch scenarios

- [ ] **patchdelete_test.go** (tests unknown)
  - TestPatchDeleteOfNotExistingAttributesShouldNotAddExtraElements (already ported as patch-delete-nonexistent-elements)

- [ ] **extendedpatch_test.go** (11 tests, 1215 lines)
  - Extended patch functionality

- [ ] **inlinepatch_test.go** (5 tests, 319 lines)
  - Inline patch scenarios

- [ ] **gvknpatch_test.go** (13 tests, 847 lines)
  - Group/Version/Kind/Name patch targeting

#### Phase 3: Generator Tests (Medium Priority)
- [ ] **configmaps_test.go** (15 tests, 774 lines)
  - TestGeneratorIntVsStringNoMerge
  - TestGeneratorIntVsStringWithMerge
  - TestGeneratorFromProperties
  - TestGeneratorBasics
  - ConfigMap generation variations

- [ ] **generatormergeandreplace_test.go** (7 tests, 750 lines)
  - Generator merge behavior
  - Generator replace behavior

#### Phase 4: Component Tests (Medium Priority)
- [ ] **component_test.go** (3 tests, 769 lines)
  - TestComponent (basic component)
  - TestComponentErrors (error handling)
  - Multiple components scenarios

#### Phase 5: Transformer Tests (Medium Priority)
- [ ] **namespaces_test.go** (8 tests, 830 lines)
  - TestNamespacedSecrets (already ported)
  - Namespace transformer behavior

- [ ] **transformerannotation_test.go** (7 tests, 492 lines)
  - Annotation transformer tests

- [ ] **transformersimage_test.go** (4 tests, 403 lines)
  - Image transformer tests

- [ ] **variableref_test.go** (14 tests, 2276 lines)
  - Variable reference transformations

- [ ] **replacementtransformer_test.go** (12 tests, 1004 lines)
  - Replacement transformer tests

- [ ] **inlinelabels_test.go** (8 tests, 527 lines)
  - Inline label tests

- [ ] **sortordertransformer_test.go** (9 tests, 331 lines)
  - Sort order behavior

#### Phase 6: Advanced & Edge Cases (Lower Priority)
- [ ] **issue2896_test.go** - Name suffix with volumeMounts
- [ ] **issue3377_test.go** - Variable substitution with patches
- [ ] **openapicustomschema_test.go** (16 tests, 645 lines)
- [ ] **customconfig_test.go** (5 tests, 734 lines)
- [ ] **crd_test.go** (3 tests, 356 lines)
- [ ] **diamondcomposition_test.go** (8 tests, 459 lines)

### 3. Test Statistics

**Total tests discovered**: ~500+
- merge2 tests: 61 test cases
- krusty tests: ~300+ test functions across 72 files
- Other test suites: ~150+

**Current progress**:
- Total ported: 169 tests
- Percentage complete: ~34%

### 4. Test Porting Strategy

#### Current Phase: Complete merge2 Suite
**Goal**: Port all 61 merge2 test cases (37 remaining)
**Estimated effort**: 2-3 sessions
**Priority**: Critical for core merge functionality

#### Next Phase: Basic Krusty Integration
**Goal**: Port fundamental overlay and patch tests
**Files**: baseandoverlaysmall, multiplepatch, patchdelete
**Estimated effort**: 3-4 sessions
**Priority**: High - validates real-world scenarios

#### Subsequent Phases:
1. Generators (ConfigMaps, Secrets)
2. Components
3. Transformers (Namespace, Image, Annotations)
4. Advanced features and edge cases

### 5. Porting Checklist Template

For each test ported:
- [ ] Create test directory structure
- [ ] Add test.yaml with source comment
- [ ] Create base resource files
- [ ] Create kustomization.yaml
- [ ] Create expected output.yaml
- [ ] Verify against reference kustomize
- [ ] Run test and update snapshots
- [ ] Document any bugs found

### 6. Known Bugs Found During Porting

1. **bug-primitive-list-merge** (branch created)
   - Primitive lists like finalizers produce duplicates instead of merging unique values
   - Test: merge-primitive-finalizers

2. **Container ordering** (not yet branched)
   - Container prepend order not preserved correctly
   - Test: merge-element-prepend

3. **Port ordering** (not yet branched)
   - Service port patch order incorrect
   - Test: port-patch-different-port-number

### 7. Next Immediate Steps

1. ✅ Complete source documentation for recent tests
2. ⏳ Port remaining element_test.go cases (7 tests)
3. ⏳ Port remaining map_test.go cases (13 tests)
4. ⏳ Port remaining scalar_test.go cases (11 tests)
5. ⏳ Port remaining list_test.go cases (6 tests)
6. ⏳ Begin baseandoverlaysmall_test.go (8 tests)
7. ⏳ Begin multiplepatch_test.go (18 tests)

---

**Last Updated**: Session continuing from context budget overflow
**Current Branch**: bug-patch-delete-directive
**Tests Passing**: 169
