use anyhow::{anyhow, Result};
use icu_collator::{
    options::{CollatorOptions, Strength},
    Collator, CollatorBorrowed, CollatorPreferences,
};
use icu_locale_core::{preferences::LocalePreferences, Locale};
use log::debug;

pub(crate) fn collator_for_locale(locale_name: &str, case_insensitive: bool) -> Result<Collator> {
    debug!("Creating collator for locale: {locale_name}");
    let locale = Locale::try_from_str(locale_name)
        .map_err(|e| anyhow!("Failed to parse locale '{locale_name}': {e}"))?;
    let mut prefs = CollatorPreferences::default();
    prefs.locale_preferences = LocalePreferences::from(&locale);

    let mut opts = CollatorOptions::default();
    if case_insensitive {
        debug!("Setting collator strength to secondary to make it case-insensitive");
        opts.strength = Some(Strength::Secondary);
    }
    Collator::try_new(prefs, opts)
        .map(CollatorBorrowed::static_to_owned)
        .map_err(|e| anyhow!("{e}"))
}
