use anyhow::{anyhow, Result};
use icu::{
    collator::{Collator, CollatorOptions, Strength},
    locid::Locale,
};
use icu_provider_adapters::fallback::LocaleFallbackProvider;
use log::debug;

// cargo build
// icu4x-datagen \
//     --overwrite \
//     --locales full \
//     --format mod \
//     --pretty \
//     --out ./src/icu-data \
//     --keys-for-bin ./target/debug/omegasort
//
// Original keys:
//     --keys collator/data@1 \
//     --keys collator/dia@1 \
//     --keys collator/jamo@1 \
//     --keys collator/meta@1 \
//     --keys collator/prim@1 \
//     --keys collator/reord@1  \
//     --keys normalizer/comp@1 \
//     --keys normalizer/decomp@1 \
//     --keys normalizer/nfd@1 \
//     --keys normalizer/nfdex@1 \
//     --keys normalizer/nfkd@1 \
//     --keys normalizer/nfkdex@1 \
//     --keys normalizer/uts46d@1 \

struct UnstableProvider;
include!("./icu-data/mod.rs");
impl_data_provider!(UnstableProvider);

pub(crate) fn collator_for_locale(locale_name: &str, case_insensitive: bool) -> Result<Collator> {
    debug!("Creating collator for locale: {locale_name}");
    let locale = Locale::try_from_bytes(locale_name.as_bytes())
        .map_err(|e| anyhow!("'{locale_name}' is not a valid locale: {e}"))?;
    let provider =
        LocaleFallbackProvider::try_new_unstable(UnstableProvider).map_err(|e| anyhow!("{e}"))?;
    let mut opts = CollatorOptions::new();
    if case_insensitive {
        debug!("Setting collator strength to secondary to make it case-insensitive");
        opts.strength = Some(Strength::Secondary);
    }
    Collator::try_new_unstable(&provider, &locale.into(), opts).map_err(|e| anyhow!("{e}"))
}
