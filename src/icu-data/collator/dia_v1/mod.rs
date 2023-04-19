// @generated
type DataStruct =
    <::icu::collator::provider::CollationDiacriticsV1Marker as ::icu_provider::DataMarker>::Yokeable;
pub fn lookup(locale: &icu_provider::DataLocale) -> Option<&'static DataStruct> {
    static KEYS: [&str; 4usize] = ["ee", "und", "vi", "vi-u-co-trad"];
    static DATA: [&DataStruct; 4usize] = [&EE, &UND, &VI, &VI];
    KEYS.binary_search_by(|k| locale.strict_cmp(k.as_bytes()).reverse())
        .ok()
        .map(|i| unsafe { *DATA.get_unchecked(i) })
}
static EE: DataStruct = include!("ee.rs.data");
static UND: DataStruct = include!("und.rs.data");
static VI: DataStruct = include!("vi.rs.data");
