use anyhow::{anyhow, Result};
use icu::{
    collator::{Collator, CollatorOptions, Strength},
    locid::Locale,
};
use log::debug;

pub(crate) fn collator_for_locale(locale_name: &str, case_insensitive: bool) -> Result<Collator> {
    debug!("Creating collator for locale: {locale_name}");
    let locale = Locale::try_from_bytes(locale_name.as_bytes())
        .map_err(|e| anyhow!("'{locale_name}' is not a valid locale: {e}"))?;
    let mut opts = CollatorOptions::new();
    if case_insensitive {
        debug!("Setting collator strength to secondary to make it case-insensitive");
        opts.strength = Some(Strength::Secondary);
    }
    Collator::try_new(&locale.into(), opts).map_err(|e| anyhow!("{e}"))
}
