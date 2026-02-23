# Branch Triage Notes

## Status
All 49 original `bug/*` branches have been triaged. 34 renamed to `unimplemented/`, 5 deleted (incorrect duplicates), 10 remain as `bug/`.

## Remaining bug/ branches (10) — actual bugs with wrong output
All rebased onto master except 3 with rebase conflicts.

| Branch | Issue |
|--------|-------|
| bug/change-name-and-kind | snapshot mismatch on simple-multiple-patches-overlay |
| bug/inline-transformer | snapshot mismatch on disable-name-suffix-hash (uses generatorOptions) |
| bug/merge-and-replace-disable-name-suffix-hash-generators | snapshot mismatch (merge/replace behavior with disableNameSuffixHash) |
| bug/name-reference-after-gvkn-change | snapshot mismatch on own output |
| bug/configmap-generator-repeats-in-kustomization | YAML duplicate keys rejected (kustomize allows merging them) |
| bug/base-reuse-name-and-kind-conflict | resource merging: "already registered id" when same name+kind in different bases |
| bug/component-can-add-same-base-if-first-renames | resource merging: same issue with components that rename |
| bug/name-reference-after-json-patch | **REBASE CONFLICT** on extended-patch-gvk-label-selector |
| bug/patch-original-name | **REBASE CONFLICT** on extended-patch-no-match |
| bug/patch-original-name-and-kind | **REBASE CONFLICT** on extended-patch-gvk-label-selector |

## Unimplemented features (34 branches)
Grouped by missing feature:

- **`buildMetadata` field** (3): anno-origin-and-transformer-builtin-local, anno-transformer-builtin-inline, anno-transformer-builtin-local
- **`configurations` field** (3): empty-field-spec-value, issue-4682-name-references-to-self, issue-4884-local-config-name-ref
- **`replacements` field** (1): component-applied-before-replacements
- **Test manifest `dir` field** (8): anno-origin-remote-builtin-transformer, anno-origin-custom-inline-transformer, anno-transformer-local-files-with-overlay, build-metadata-origin-annotations, configmap-generating-into-same-namespace, secret-generating-into-same-namespace, sortOptions-not-supported, patch-original-name-and-new-kind
- **List kind resources** (11): empty-list, rnode-from-map, inline-generator, legacy-prefix-suffix-transformer, list-to-individual-resources, managed-by-label, namespaced-service-accounts-overlap, skip-local-config-after-transform, transformers-image-custom-config, transformers-image-default-config, transformers-image-knative-config
  - Note: 9 of these (inline-generator through transformers-image-knative-config) are duplicates that only add the empty-list test
- **!!binary YAML** (2): rnode-new-string-rnode-binary, rnode-set-data-map (duplicates)
- **Bare resources (no apiVersion)** (1): rnode-marshal-json
- **PrefixSuffixTransformer** (1): custom-name-prefixer
- **IAMPolicyGenerator** (1): gke-generator
- **Generator from directory** (2): complex-composition-dev-base-transformers, complex-composition-prod-base-transformers
- **Both snapshots exist** (1): empty-patches-strategic-merge-fails
- **Test `dir` field** (1): patch-original-name-and-new-kind

## Deleted branches (5)
anno-origin-local-files, change-kind, change-name, configmap-duplicate-keys, custom-ordering — all identical incorrect snapshot fix for accumulate-malformed-yaml/error.stderr (the existing snapshot on master is already correct).

## Next steps
- Fix the 3 rebase conflicts on bug/name-reference-after-json-patch, bug/patch-original-name, bug/patch-original-name-and-kind
- Consider deduplicating unimplemented/ branches (many empty-list duplicates, rnode-set-data-map duplicates rnode-new-string-rnode-binary)
- Work on fixing the 10 remaining bugs
