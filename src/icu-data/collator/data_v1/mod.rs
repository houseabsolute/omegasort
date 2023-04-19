// @generated
type DataStruct =
    <::icu::collator::provider::CollationDataV1Marker as ::icu_provider::DataMarker>::Yokeable;
pub fn lookup(locale: &icu_provider::DataLocale) -> Option<&'static DataStruct> {
    static KEYS: [&str; 106usize] = [
        "af",
        "ar",
        "ar-u-co-compat",
        "as",
        "az",
        "be",
        "bn",
        "bn-u-co-trad",
        "bo",
        "br",
        "bs",
        "bs-Cyrl",
        "ceb",
        "cs",
        "cy",
        "da",
        "de-AT-u-co-phonebk",
        "de-u-co-phonebk",
        "dsb",
        "ee",
        "en-US-posix",
        "eo",
        "es",
        "es-u-co-trad",
        "et",
        "fa",
        "fa-AF",
        "ff-Adlm",
        "fi",
        "fi-u-co-trad",
        "fil",
        "fo",
        "fy",
        "gl",
        "gu",
        "ha",
        "haw",
        "hi",
        "hr",
        "hsb",
        "hu",
        "hy",
        "ig",
        "is",
        "ja",
        "ja-u-co-unihan",
        "kk",
        "kl",
        "km",
        "kn",
        "kn-u-co-trad",
        "ko",
        "ko-u-co-unihan",
        "kok",
        "ku",
        "ky",
        "lkt",
        "ln",
        "ln-u-co-phonetic",
        "lt",
        "lv",
        "mk",
        "ml",
        "mr",
        "mt",
        "my",
        "no",
        "om",
        "or",
        "pa",
        "pl",
        "ps",
        "ro",
        "se",
        "si",
        "si-u-co-dict",
        "sk",
        "sl",
        "smn",
        "sq",
        "sr",
        "sr-Latn",
        "sv",
        "sv-u-co-trad",
        "ta",
        "te",
        "th",
        "tk",
        "to",
        "tr",
        "ug",
        "uk",
        "und",
        "und-u-co-emoji",
        "und-u-co-eor",
        "ur",
        "uz",
        "vi",
        "vi-u-co-trad",
        "wo",
        "yi",
        "yo",
        "zh",
        "zh-u-co-stroke",
        "zh-u-co-unihan",
        "zh-u-co-zhuyin",
    ];
    static DATA: [&DataStruct; 106usize] = [
        &AF,
        &AR,
        &AR_U_CO_COMPAT,
        &AS,
        &AZ,
        &BE,
        &BN,
        &BN_U_CO_TRAD,
        &BO,
        &BR,
        &BS,
        &BS_CYRL,
        &CEB,
        &CS,
        &CY,
        &DA,
        &DE_AT_U_CO_PHONEBK,
        &DE_U_CO_PHONEBK,
        &DSB,
        &EE,
        &EN_US_POSIX,
        &EO,
        &ES,
        &ES_U_CO_TRAD,
        &ET,
        &FA,
        &FA_AF,
        &FF_ADLM,
        &FI,
        &FI_U_CO_TRAD,
        &CEB,
        &FO,
        &FY,
        &ES,
        &GU,
        &HA,
        &HAW,
        &HI,
        &BS,
        &HSB,
        &HU,
        &HY,
        &IG,
        &IS,
        &JA,
        &JA_U_CO_UNIHAN,
        &KK,
        &KL,
        &KM,
        &KN,
        &KN_U_CO_TRAD,
        &KO,
        &KO_U_CO_UNIHAN,
        &KOK,
        &KU,
        &KY,
        &LKT,
        &LN,
        &LN_U_CO_PHONETIC,
        &LT,
        &LV,
        &MK,
        &ML,
        &MR,
        &MT,
        &MY,
        &NO,
        &OM,
        &OR,
        &PA,
        &PL,
        &FA_AF,
        &RO,
        &SE,
        &SI,
        &SI_U_CO_DICT,
        &SK,
        &SL,
        &SMN,
        &SQ,
        &BS_CYRL,
        &BS,
        &SV,
        &SV_U_CO_TRAD,
        &TA,
        &TE,
        &TH,
        &TK,
        &TO,
        &TR,
        &UG,
        &UK,
        &UND,
        &UND_U_CO_EMOJI,
        &UND_U_CO_EOR,
        &UR,
        &UZ,
        &VI,
        &VI_U_CO_TRAD,
        &WO,
        &YI,
        &YO,
        &ZH,
        &ZH_U_CO_STROKE,
        &ZH_U_CO_UNIHAN,
        &ZH_U_CO_ZHUYIN,
    ];
    KEYS.binary_search_by(|k| locale.strict_cmp(k.as_bytes()).reverse())
        .ok()
        .map(|i| unsafe { *DATA.get_unchecked(i) })
}
static AF: DataStruct = include!("af.rs.data");
static AR_U_CO_COMPAT: DataStruct = include!("ar-u-co-compat.rs.data");
static AR: DataStruct = include!("ar.rs.data");
static AS: DataStruct = include!("as.rs.data");
static AZ: DataStruct = include!("az.rs.data");
static BE: DataStruct = include!("be.rs.data");
static BN_U_CO_TRAD: DataStruct = include!("bn-u-co-trad.rs.data");
static BN: DataStruct = include!("bn.rs.data");
static BO: DataStruct = include!("bo.rs.data");
static BR: DataStruct = include!("br.rs.data");
static BS_CYRL: DataStruct = include!("bs-Cyrl.rs.data");
static BS: DataStruct = include!("bs.rs.data");
static CEB: DataStruct = include!("ceb.rs.data");
static CS: DataStruct = include!("cs.rs.data");
static CY: DataStruct = include!("cy.rs.data");
static DA: DataStruct = include!("da.rs.data");
static DE_AT_U_CO_PHONEBK: DataStruct = include!("de-AT-u-co-phonebk.rs.data");
static DE_U_CO_PHONEBK: DataStruct = include!("de-u-co-phonebk.rs.data");
static DSB: DataStruct = include!("dsb.rs.data");
static EE: DataStruct = include!("ee.rs.data");
static EN_US_POSIX: DataStruct = include!("en-US-posix.rs.data");
static EO: DataStruct = include!("eo.rs.data");
static ES_U_CO_TRAD: DataStruct = include!("es-u-co-trad.rs.data");
static ES: DataStruct = include!("es.rs.data");
static ET: DataStruct = include!("et.rs.data");
static FA_AF: DataStruct = include!("fa-AF.rs.data");
static FA: DataStruct = include!("fa.rs.data");
static FF_ADLM: DataStruct = include!("ff-Adlm.rs.data");
static FI_U_CO_TRAD: DataStruct = include!("fi-u-co-trad.rs.data");
static FI: DataStruct = include!("fi.rs.data");
static FO: DataStruct = include!("fo.rs.data");
static FY: DataStruct = include!("fy.rs.data");
static GU: DataStruct = include!("gu.rs.data");
static HA: DataStruct = include!("ha.rs.data");
static HAW: DataStruct = include!("haw.rs.data");
static HI: DataStruct = include!("hi.rs.data");
static HSB: DataStruct = include!("hsb.rs.data");
static HU: DataStruct = include!("hu.rs.data");
static HY: DataStruct = include!("hy.rs.data");
static IG: DataStruct = include!("ig.rs.data");
static IS: DataStruct = include!("is.rs.data");
static JA_U_CO_UNIHAN: DataStruct = include!("ja-u-co-unihan.rs.data");
static JA: DataStruct = include!("ja.rs.data");
static KK: DataStruct = include!("kk.rs.data");
static KL: DataStruct = include!("kl.rs.data");
static KM: DataStruct = include!("km.rs.data");
static KN_U_CO_TRAD: DataStruct = include!("kn-u-co-trad.rs.data");
static KN: DataStruct = include!("kn.rs.data");
static KO_U_CO_UNIHAN: DataStruct = include!("ko-u-co-unihan.rs.data");
static KO: DataStruct = include!("ko.rs.data");
static KOK: DataStruct = include!("kok.rs.data");
static KU: DataStruct = include!("ku.rs.data");
static KY: DataStruct = include!("ky.rs.data");
static LKT: DataStruct = include!("lkt.rs.data");
static LN_U_CO_PHONETIC: DataStruct = include!("ln-u-co-phonetic.rs.data");
static LN: DataStruct = include!("ln.rs.data");
static LT: DataStruct = include!("lt.rs.data");
static LV: DataStruct = include!("lv.rs.data");
static MK: DataStruct = include!("mk.rs.data");
static ML: DataStruct = include!("ml.rs.data");
static MR: DataStruct = include!("mr.rs.data");
static MT: DataStruct = include!("mt.rs.data");
static MY: DataStruct = include!("my.rs.data");
static NO: DataStruct = include!("no.rs.data");
static OM: DataStruct = include!("om.rs.data");
static OR: DataStruct = include!("or.rs.data");
static PA: DataStruct = include!("pa.rs.data");
static PL: DataStruct = include!("pl.rs.data");
static RO: DataStruct = include!("ro.rs.data");
static SE: DataStruct = include!("se.rs.data");
static SI_U_CO_DICT: DataStruct = include!("si-u-co-dict.rs.data");
static SI: DataStruct = include!("si.rs.data");
static SK: DataStruct = include!("sk.rs.data");
static SL: DataStruct = include!("sl.rs.data");
static SMN: DataStruct = include!("smn.rs.data");
static SQ: DataStruct = include!("sq.rs.data");
static SV_U_CO_TRAD: DataStruct = include!("sv-u-co-trad.rs.data");
static SV: DataStruct = include!("sv.rs.data");
static TA: DataStruct = include!("ta.rs.data");
static TE: DataStruct = include!("te.rs.data");
static TH: DataStruct = include!("th.rs.data");
static TK: DataStruct = include!("tk.rs.data");
static TO: DataStruct = include!("to.rs.data");
static TR: DataStruct = include!("tr.rs.data");
static UG: DataStruct = include!("ug.rs.data");
static UK: DataStruct = include!("uk.rs.data");
static UND_U_CO_EMOJI: DataStruct = include!("und-u-co-emoji.rs.data");
static UND_U_CO_EOR: DataStruct = include!("und-u-co-eor.rs.data");
static UND: DataStruct = include!("und.rs.data");
static UR: DataStruct = include!("ur.rs.data");
static UZ: DataStruct = include!("uz.rs.data");
static VI_U_CO_TRAD: DataStruct = include!("vi-u-co-trad.rs.data");
static VI: DataStruct = include!("vi.rs.data");
static WO: DataStruct = include!("wo.rs.data");
static YI: DataStruct = include!("yi.rs.data");
static YO: DataStruct = include!("yo.rs.data");
static ZH_U_CO_STROKE: DataStruct = include!("zh-u-co-stroke.rs.data");
static ZH_U_CO_UNIHAN: DataStruct = include!("zh-u-co-unihan.rs.data");
static ZH_U_CO_ZHUYIN: DataStruct = include!("zh-u-co-zhuyin.rs.data");
static ZH: DataStruct = include!("zh.rs.data");
