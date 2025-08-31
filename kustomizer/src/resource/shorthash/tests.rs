use crate::resource::Resource;

fn create_resource_from_yaml(yaml: &str) -> anyhow::Result<Resource> {
    let value: serde_json::Value = serde_yaml::from_str(yaml)?;
    let resource: Resource = serde_json::from_value(value)?;
    Ok(resource)
}

fn skip_rest(desc: &str, result: &anyhow::Result<String>, expected_err: &str) -> bool {
    match result {
        Err(err) => {
            if expected_err.is_empty() {
                panic!("case '{}', expect nil error but got '{}'", desc, err);
            } else if !err.to_string().contains(expected_err) {
                panic!(
                    "case '{}', expect error to contain '{}' but got '{}'",
                    desc, expected_err, err
                );
            }
            true
        }
        Ok(_) => {
            if !expected_err.is_empty() {
                panic!(
                    "case '{}', expect error to contain '{}' but got nil error",
                    desc, expected_err
                );
            }
            false
        }
    }
}

#[test]
fn test_config_map_hash() -> anyhow::Result<()> {
    let cases = vec![
        // empty map
        (
            "empty data",
            r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: """#,
            "6ct58987ht",
            "",
        ),
        // one key
        (
            "one key",
            r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
data:
  one: """#,
            "9g67k2htb6",
            "",
        ),
        // three keys (tests sorting order)
        (
            "three keys",
            r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
data:
  two: 2
  one: ""
  three: 3"#,
            "7757f9kkct",
            "",
        ),
        // empty binary data map
        (
            "empty binary data",
            r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: """#,
            "6ct58987ht",
            "",
        ),
        // one key with binary data
        (
            "one key with binary data",
            r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
binaryData:
  one: """#,
            "6mtk2m274t",
            "",
        ),
        // three keys with binary data (tests sorting order)
        (
            "three keys with binary data",
            r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
binaryData:
  two: 2
  one: ""
  three: 3"#,
            "9th7kc28dg",
            "",
        ),
        // two keys, one with string and another with binary data
        (
            "two keys with one each",
            r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: ""
data:
  one: ""
binaryData:
  two: """#,
            "698h7c7t9m",
            "",
        ),
    ];

    for (desc, cm_yaml, expected_hash, err_msg) in cases {
        let resource = create_resource_from_yaml(cm_yaml)?;
        let result = resource.shorthash().map(|s| s.to_string());

        if skip_rest(desc, &result, err_msg) {
            continue;
        }

        let hashed = result?;
        assert_eq!(
            expected_hash, hashed,
            "case '{}', expect hash '{}' but got '{}'",
            desc, expected_hash, hashed
        );
    }
    Ok(())
}

#[test]
fn test_secret_hash() -> anyhow::Result<()> {
    let cases = vec![
        // empty map
        (
            "empty data",
            r#"
apiVersion: v1
kind: Secret
metadata:
  name: ""
type: my-type"#,
            "5gmgkf8578",
            "",
        ),
        // one key
        (
            "one key",
            r#"
apiVersion: v1
kind: Secret
metadata:
  name: ""
type: my-type
data:
  one: """#,
            "74bd68bm66",
            "",
        ),
        // three keys (tests sorting order)
        (
            "three keys",
            r#"
apiVersion: v1
kind: Secret
metadata:
  name: ""
type: my-type
data:
  two: 2
  one: ""
  three: 3"#,
            "4gf75c7476",
            "",
        ),
        // with stringdata
        (
            "stringdata",
            r#"
apiVersion: v1
kind: Secret
metadata:
  name: ""
type: my-type
data:
  one: ""
stringData:
  two: 2"#,
            "c4h4264gdb",
            "",
        ),
        // empty stringdata
        (
            "empty stringdata",
            r#"
apiVersion: v1
kind: Secret
metadata:
  name: ""
type: my-type
data:
  one: """#,
            "74bd68bm66",
            "",
        ),
    ];

    for (desc, secret_yaml, expected_hash, err_msg) in cases {
        let resource = create_resource_from_yaml(secret_yaml)?;
        let result = resource.shorthash().map(|s| s.to_string());

        if skip_rest(desc, &result, err_msg) {
            continue;
        }

        let hashed = result?;
        assert_eq!(
            expected_hash, hashed,
            "case '{}', expect hash '{}' but got '{}'",
            desc, expected_hash, hashed
        );
    }
    Ok(())
}
