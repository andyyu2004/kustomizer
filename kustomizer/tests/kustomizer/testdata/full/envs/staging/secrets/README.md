# Staging-specific secret env variables override

The file `base-secrets-patch-secret.enc.yaml` hold only the differences (overrides) from `shared/resources/example/base-secrets`. Plus, it is encrypted with Mozilla SOPS.

When staging-au postgres db credentials change, e.g. after recreating the environment, update related env variables by first retrieving the last terraform `gcp/staging-au` output json file:

> terraform output -json > staging-output.json

Then run the script to update related env secret overrides:

> ./update-staging-db-creds.sh

Then commit and push the changes to `argocd` repo.
