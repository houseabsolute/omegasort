// @generated
type DataStruct =
    <::icu::collator::provider::CollationReorderingV1Marker as ::icu_provider::DataMarker>::Yokeable;
pub fn lookup(locale: &icu_provider::DataLocale) -> Option<&'static DataStruct> {
    static KEYS: [&str; 59usize] = [
        "am",
        "ar",
        "ar-u-co-compat",
        "as",
        "az",
        "be",
        "bg",
        "bn",
        "bn-u-co-trad",
        "bo",
        "bs",
        "bs-Cyrl",
        "chr",
        "el",
        "fa",
        "fa-AF",
        "gu",
        "he",
        "hi",
        "hr",
        "hy",
        "ja",
        "ja-u-co-unihan",
        "ka",
        "kk",
        "km",
        "kn",
        "kn-u-co-trad",
        "ko",
        "ko-u-co-unihan",
        "kok",
        "ku",
        "ky",
        "lo",
        "mk",
        "ml",
        "mn",
        "mr",
        "my",
        "ne",
        "or",
        "pa",
        "ps",
        "ru",
        "si",
        "si-u-co-dict",
        "sr",
        "sr-Latn",
        "ta",
        "te",
        "th",
        "ug",
        "uk",
        "ur",
        "yi",
        "zh",
        "zh-u-co-stroke",
        "zh-u-co-unihan",
        "zh-u-co-zhuyin",
    ];
    static DATA: [&DataStruct; 59usize] = [
        &AM,
        &AR,
        &AR,
        &AS,
        &AZ,
        &BE,
        &BE,
        &AS,
        &AS,
        &BO,
        &AZ,
        &BE,
        &CHR,
        &EL,
        &AR,
        &AR,
        &GU,
        &HE,
        &HI,
        &AZ,
        &HY,
        &JA,
        &JA,
        &KA,
        &BE,
        &KM,
        &KN,
        &KN,
        &KO,
        &KO,
        &HI,
        &KU,
        &BE,
        &LO,
        &BE,
        &ML,
        &MN,
        &HI,
        &MY,
        &NE,
        &OR,
        &PA,
        &AR,
        &BE,
        &SI,
        &SI,
        &BE,
        &AZ,
        &TA,
        &TE,
        &TH,
        &AR,
        &BE,
        &AR,
        &HE,
        &ZH,
        &ZH_U_CO_STROKE,
        &ZH_U_CO_STROKE,
        &ZH_U_CO_STROKE,
    ];
    KEYS.binary_search_by(|k| locale.strict_cmp(k.as_bytes()).reverse())
        .ok()
        .map(|i| unsafe { *DATA.get_unchecked(i) })
}
static AM: DataStruct = include!("am.rs.data");
static AR: DataStruct = include!("ar.rs.data");
static AS: DataStruct = include!("as.rs.data");
static AZ: DataStruct = include!("az.rs.data");
static BE: DataStruct = include!("be.rs.data");
static BO: DataStruct = include!("bo.rs.data");
static CHR: DataStruct = include!("chr.rs.data");
static EL: DataStruct = include!("el.rs.data");
static GU: DataStruct = include!("gu.rs.data");
static HE: DataStruct = include!("he.rs.data");
static HI: DataStruct = include!("hi.rs.data");
static HY: DataStruct = include!("hy.rs.data");
static JA: DataStruct = include!("ja.rs.data");
static KA: DataStruct = include!("ka.rs.data");
static KM: DataStruct = include!("km.rs.data");
static KN: DataStruct = include!("kn.rs.data");
static KO: DataStruct = include!("ko.rs.data");
static KU: DataStruct = include!("ku.rs.data");
static LO: DataStruct = include!("lo.rs.data");
static ML: DataStruct = include!("ml.rs.data");
static MN: DataStruct = include!("mn.rs.data");
static MY: DataStruct = include!("my.rs.data");
static NE: DataStruct = include!("ne.rs.data");
static OR: DataStruct = include!("or.rs.data");
static PA: DataStruct = include!("pa.rs.data");
static SI: DataStruct = include!("si.rs.data");
static TA: DataStruct = include!("ta.rs.data");
static TE: DataStruct = include!("te.rs.data");
static TH: DataStruct = include!("th.rs.data");
static ZH_U_CO_STROKE: DataStruct = include!("zh-u-co-stroke.rs.data");
static ZH: DataStruct = include!("zh.rs.data");
