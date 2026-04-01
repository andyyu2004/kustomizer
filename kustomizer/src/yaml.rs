use std::io::Read;

use serde::de::DeserializeOwned;

/// Default options for YAML deserialization.
///
/// Uses `strict_booleans` (YAML 1.2 behavior) so that only `true`/`false` are
/// parsed as booleans. YAML 1.1 forms like `on`/`off`/`yes`/`no` are treated
/// as plain strings, avoiding the "Norway problem" and ensuring non-string map
/// keys don't appear when these words are used as field names (e.g. `on:` in
/// Kubernetes CRDs).
const DESER_OPTS: serde_saphyr::Options = serde_saphyr::Options {
    strict_booleans: true,
    budget: Some(serde_saphyr::Budget {
        max_reader_input_bytes: Some(256 * 1024 * 1024),
        max_events: 1_000_000,
        max_aliases: 50_000,
        max_anchors: 50_000,
        max_depth: 2_000,
        max_documents: 1_024,
        max_nodes: 250_000,
        max_total_scalar_bytes: 64 * 1024 * 1024,
        max_merge_keys: 10_000,
        enforce_alias_anchor_ratio: true,
        alias_anchor_min_aliases: 100,
        alias_anchor_ratio_multiplier: 10,
    }),
    budget_report: None,
    duplicate_keys: serde_saphyr::options::DuplicateKeyPolicy::Error,
    alias_limits: serde_saphyr::options::AliasLimits {
        max_total_replayed_events: 1_000_000,
        max_replay_stack_depth: 64,
        max_alias_expansions_per_anchor: usize::MAX,
    },
    legacy_octal_numbers: false,
    angle_conversions: false,
    ignore_binary_tag_for_string: false,
    no_schema: false,
    with_snippet: true,
    crop_radius: 64,
};

pub fn from_str<T>(s: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    serde_saphyr::from_str_with_options(s, DESER_OPTS).map_err(Into::into)
}

pub fn from_slice<T>(s: &[u8]) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    serde_saphyr::from_slice_with_options(s, DESER_OPTS).map_err(Into::into)
}

pub fn from_reader<T>(reader: impl Read) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    serde_saphyr::from_reader_with_options(reader, DESER_OPTS).map_err(Into::into)
}

pub fn from_reader_multi<T>(mut reader: impl Read) -> anyhow::Result<Box<[T]>>
where
    T: DeserializeOwned,
{
    serde_saphyr::read_with_options(&mut reader, DESER_OPTS)
        .map(|res| res.map_err(Into::into))
        .collect()
}

const OPTS: serde_saphyr::SerializerOptions = serde_saphyr::SerializerOptions {
    prefer_block_scalars: true,
    empty_as_braces: true,
    indent_step: 2,
    anchor_generator: None,
    min_fold_chars: 32,
    folded_wrap_chars: 80,
    tagged_enums: false,
};

pub fn to_string<T>(value: &T) -> anyhow::Result<String>
where
    T: serde::Serialize,
{
    serde_saphyr::to_string_with_options(value, OPTS).map_err(Into::into)
}

pub fn to_fmt_writer<W: std::fmt::Write, T: serde::Serialize>(
    output: &mut W,
    value: &T,
) -> anyhow::Result<()> {
    serde_saphyr::to_fmt_writer_with_options(output, value, OPTS).map_err(Into::into)
}
