fn main() {
    divan::main();
}

#[divan::bench]
#[tokio::main]
async fn build_production() -> kustomizer::ResourceMap {
    kustomizer::build("tests/kustomizer/testdata/full/envs/production/")
        .await
        .unwrap()
}

#[divan::bench]
#[tokio::main]
async fn build_staging() -> kustomizer::ResourceMap {
    kustomizer::build("tests/kustomizer/testdata/full/envs/staging/")
        .await
        .unwrap()
}

#[divan::bench]
#[tokio::main]
async fn build_resources() -> kustomizer::ResourceMap {
    kustomizer::build("tests/kustomizer/testdata/full/resources")
        .await
        .unwrap()
}
