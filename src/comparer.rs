use anyhow::Result;
use chrono::{DateTime, Utc};
use dateparser::DateTimeUtc;
use icu::collator::Collator;
use ipnet::IpNet;
use lazy_regex::regex;
use log::debug;
use std::cmp::Ordering;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use typed_path::windows::Utf8WindowsComponent;
use typed_path::{Utf8Component, Utf8Encoding, Utf8Path, Utf8UnixPath, Utf8WindowsPath};

pub(crate) trait Comparer {
    fn is_ordered(&self, str1: &str, str2: &str, reverse: bool) -> Result<bool> {
        let ord = self.cmp(str1, str2)?;
        Ok(if reverse { !ord.is_gt() } else { ord.is_gt() })
    }

    fn cmp(&self, str1: &str, str2: &str) -> Result<Ordering>;
}

pub(crate) struct TextComparer {
    collator: Option<Collator>,
    case_insensitive: bool,
}

impl Comparer for TextComparer {
    fn cmp(&self, str1: &str, str2: &str) -> Result<Ordering> {
        Ok(compare_two_strings(
            self.collator.as_ref(),
            self.case_insensitive,
            str1,
            str2,
        ))
    }
}

impl TextComparer {
    pub(crate) fn new(collator: Option<Collator>, case_insensitive: bool) -> Self {
        Self {
            collator,
            case_insensitive,
        }
    }
}

pub(crate) struct NumberedTextComparer {
    collator: Option<Collator>,
    case_insensitive: bool,
}

impl Comparer for NumberedTextComparer {
    fn cmp(&self, str1: &str, str2: &str) -> Result<Ordering> {
        debug!("NumberedTextComparer comparing `{str1}` <=> `{str2}`");
        let numbered_text_re = regex!(
            r#"(?x)
            \A
            (?P<number>
                [0-9]+
                (?:\.[0-9]+)?
            )?
            (?P<rest>.*)
            \z
        "#
        );

        // This regex will always match since it matches even an empty string.
        let caps1 = numbered_text_re.captures(str1).unwrap();
        let caps2 = numbered_text_re.captures(str2).unwrap();

        // This always has to be at least an empty string.
        let rest1 = caps1.name("rest").unwrap();
        let rest2 = caps2.name("rest").unwrap();

        match (caps1.name("number"), caps2.name("number")) {
            (Some(num1), Some(num2)) => {
                let num1 = num1.as_str();
                let num2 = num2.as_str();

                debug!("  Both strings match the number regex: `{num1}` <=> `{num2}`");
                if num1 == num2 {
                    debug!("  The numbers are equal so the comparison will look at the rest of each string");
                    return Ok(compare_two_strings(
                        self.collator.as_ref(),
                        self.case_insensitive,
                        rest1.as_str(),
                        rest2.as_str(),
                    ));
                }

                let f1 = num1.parse::<f64>();
                let f2 = num2.parse::<f64>();
                debug!(
                    "  Parsed numbers as: `{}` <=> `{}`",
                    f1.as_ref()
                        .map_or_else(|e| e.to_string(), |r| r.to_string()),
                    f2.as_ref()
                        .map_or_else(|e| e.to_string(), |r| r.to_string()),
                );

                match (f1, f2) {
                    (Ok(f1), Ok(f2)) => {
                        debug!("  Both strings start with valid numbers");
                        Ok(f1.total_cmp(&f2))
                    }
                    (Ok(_), Err(_)) => {
                        debug!("  Only the left side has a valid number ");
                        Ok(Ordering::Less)
                    }
                    (Err(_), Ok(_)) => {
                        debug!("  Only the right side has a valid number");
                        Ok(Ordering::Greater)
                    }
                    (Err(_), Err(_)) => {
                        debug!(
                            "  Neither side has a valid number, comparing the values as strings"
                        );
                        Ok(compare_two_strings(
                            self.collator.as_ref(),
                            self.case_insensitive,
                            str1,
                            str2,
                        ))
                    }
                }
            }
            (Some(_), None) => {
                debug!("  Only the left side matches the number regex ");
                Ok(Ordering::Less)
            }
            (None, Some(_)) => {
                debug!("  Only the right side matches the number regex ");
                Ok(Ordering::Greater)
            }
            (None, None) => {
                debug!("  Neither side matches the number regex, comparing the values as strings");
                Ok(compare_two_strings(
                    self.collator.as_ref(),
                    self.case_insensitive,
                    str1,
                    str2,
                ))
            }
        }
    }
}

impl NumberedTextComparer {
    pub(crate) fn new(collator: Option<Collator>, case_insensitive: bool) -> Self {
        Self {
            collator,
            case_insensitive,
        }
    }
}

pub(crate) struct DatetimeTextComparer {
    collator: Option<Collator>,
    case_insensitive: bool,
}

impl Comparer for DatetimeTextComparer {
    fn cmp(&self, str1: &str, str2: &str) -> Result<Ordering> {
        debug!("DatetimeTextComparer comparing `{str1}` <=> `{str2}`");

        let dt1 = Self::datetime_from_str(str1);
        let dt2 = Self::datetime_from_str(str2);

        match (dt1, dt2) {
            (Some(dt1), Some(dt2)) => {
                debug!("Both strings match the datetime regex: `{dt1}` <=> `{dt2}`");
                Ok(dt1.cmp(&dt2))
            }
            (Some(_), None) => {
                debug!("  Only the left side has a valid datetime ");
                Ok(Ordering::Less)
            }
            (None, Some(_)) => {
                debug!("  Only the right side has a valid datetime ");
                Ok(Ordering::Greater)
            }
            (None, None) => {
                debug!("  Neither side has a valid datetime, comparing the values as strings");
                Ok(compare_two_strings(
                    self.collator.as_ref(),
                    self.case_insensitive,
                    str1,
                    str2,
                ))
            }
        }
    }
}

impl DatetimeTextComparer {
    pub(crate) fn new(collator: Option<Collator>, case_insensitive: bool) -> Self {
        Self {
            collator,
            case_insensitive,
        }
    }

    fn datetime_from_str(str: &str) -> Option<DateTime<Utc>> {
        let datetime_text_re = regex!(
            r#"(?x)
            \A
            (?P<datetime>\d\S+)
            (?:\s*|\z)
            \z
        "#
        );
        if let Some(caps) = datetime_text_re.captures(str) {
            if let Some(dt_text) = caps.name("datetime") {
                if let Ok(dt) = dt_text.as_str().parse::<DateTimeUtc>() {
                    return Some(dt.0);
                }
            }
        }

        None
    }
}

#[derive(PartialEq)]
pub(crate) enum PathType {
    Unix,
    Windows,
}

pub(crate) struct PathComparer {
    collator: Option<Collator>,
    case_insensitive: bool,
    path_type: PathType,
}

impl Comparer for PathComparer {
    fn cmp(&self, str1: &str, str2: &str) -> Result<Ordering> {
        match self.path_type {
            PathType::Unix => self.cmp_unix(str1, str2),
            PathType::Windows => self.cmp_windows(str1, str2),
        }
    }
}

impl PathComparer {
    pub(crate) fn new(
        collator: Option<Collator>,
        case_insensitive: bool,
        path_type: PathType,
    ) -> Self {
        Self {
            collator,
            case_insensitive,
            path_type,
        }
    }

    fn cmp_unix(&self, str1: &str, str2: &str) -> Result<Ordering> {
        debug!("PathComparer comparing paths as Unix paths: `{str1}` <=> `{str2}`");

        let path1 = Utf8UnixPath::new(str1);
        let path2 = Utf8UnixPath::new(str2);

        if let Some(o) = self.cmp_absolute(path1, path2) {
            return Ok(o);
        }

        Ok(self.cmp_components(path1, path2))
    }

    fn cmp_windows(&self, str1: &str, str2: &str) -> Result<Ordering> {
        debug!("PathComparer comparing paths as Windows paths: `{str1}` <=> `{str2}`");

        let path1 = Utf8WindowsPath::new(str1);
        let path2 = Utf8WindowsPath::new(str2);

        if let Some(o) = self.cmp_absolute(path1, path2) {
            return Ok(o);
        }

        // We end up calling `components()` again in `cmp_components` but when
        // trying to avoid this by passing the components to `cmp_components`
        // instead of the paths I walked into generic type hell and gave up.
        let path1_first = path1.components().next();
        let path2_first = path2.components().next();

        match (path1_first, path2_first) {
            (
                Some(Utf8WindowsComponent::Prefix(pre1)),
                Some(Utf8WindowsComponent::Prefix(pre2)),
            ) => {
                debug!(
                    "  both sides start with a Windows prefix, comparing `{}` <=> `{}`",
                    pre1.as_str(),
                    pre2.as_str(),
                );
                if pre1 != pre2 {
                    return Ok(pre1.cmp(&pre2));
                }
            }
            (Some(Utf8WindowsComponent::Prefix(_)), _) => {
                debug!("  only the left side starts with a Windows prefix");
                return Ok(Ordering::Less);
            }
            (_, Some(Utf8WindowsComponent::Prefix(_))) => {
                debug!("  only the right side starts with a Windows prefix");
                return Ok(Ordering::Greater);
            }
            _ => {
                debug!("  neither side starts with a Windows prefix");
            }
        }

        Ok(self.cmp_components(path1, path2))
    }

    fn cmp_absolute<T>(&self, path1: &Utf8Path<T>, path2: &Utf8Path<T>) -> Option<Ordering>
    where
        T: for<'enc> Utf8Encoding<'enc>,
    {
        let path1_is_abs = path1.is_absolute();
        let path2_is_abs = path2.is_absolute();

        debug!("  left side is absolute? {path1_is_abs}");
        debug!("  right side is absolute? {path2_is_abs}");

        match (path1_is_abs, path2_is_abs) {
            (true, false) => Some(Ordering::Less),
            (false, true) => Some(Ordering::Greater),
            _ => None,
        }
    }

    fn cmp_components<T>(&self, path1: &Utf8Path<T>, path2: &Utf8Path<T>) -> Ordering
    where
        T: for<'enc> Utf8Encoding<'enc>,
    {
        let elems1 = path1.components().collect::<Vec<_>>();
        let elems2 = path2.components().collect::<Vec<_>>();

        debug!("  left side has {} components", elems1.len());
        debug!("  right side has {} components", elems2.len());

        match (elems1.is_empty(), elems2.is_empty()) {
            (true, true) => {
                debug!("  neither side has components");
                return Ordering::Equal;
            }
            (true, false) => {
                debug!("  only the left side has components");
                return Ordering::Less;
            }
            (false, true) => {
                debug!("  only the right side has components");
                return Ordering::Greater;
            }
            _ => (),
        }

        if elems1.len() != elems2.len() {
            debug!(
                "  the sides differ in numbers of elements: {} <=> {}",
                elems1.len(),
                elems2.len(),
            );
            return elems1.len().cmp(&elems2.len());
        }

        debug!("  comparing each component in turn");
        for i in 0..elems1.len() {
            let elem1_str = elems1[i].as_str();
            let elem2_str = elems2[i].as_str();
            let ord = compare_two_strings(
                self.collator.as_ref(),
                self.case_insensitive,
                elem1_str,
                elem2_str,
            );
            debug!("  {i}: `{elem1_str}` <=> `{elem2_str}`: {ord:?}");
            if ord != Ordering::Equal {
                return ord;
            }
        }

        debug!("  no differences in path found");
        Ordering::Equal
    }
}

pub(crate) struct IpComparer;

impl Comparer for IpComparer {
    fn cmp(&self, str1: &str, str2: &str) -> Result<Ordering> {
        let ip1 = Self::parse_ip_address(str1)?;
        let ip2 = Self::parse_ip_address(str2)?;
        Ok(compare_two_ip_addresses(ip1, ip2))
    }
}

impl IpComparer {
    pub(crate) fn new() -> Self {
        Self
    }

    fn parse_ip_address(addr: &str) -> Result<IpAddr> {
        if addr.contains('.') {
            Ok(IpAddr::V4(addr.parse::<Ipv4Addr>()?))
        } else {
            Ok(IpAddr::V6(addr.parse::<Ipv6Addr>()?))
        }
    }
}

pub(crate) struct NetworkComparer;

impl Comparer for NetworkComparer {
    fn cmp(&self, str1: &str, str2: &str) -> Result<Ordering> {
        let net1 = Self::parse_network(str1)?;
        let net2 = Self::parse_network(str2)?;

        let cmp = compare_two_ip_addresses(net1.addr(), net2.addr());
        if cmp != Ordering::Equal {
            return Ok(cmp);
        }

        Ok(net1.prefix_len().cmp(&net2.prefix_len()))
    }
}

impl NetworkComparer {
    pub(crate) fn new() -> Self {
        Self
    }
    fn parse_network(addr: &str) -> Result<IpNet> {
        Ok(addr.parse::<IpNet>()?)
    }
}

fn compare_two_strings(
    collator: Option<&Collator>,
    case_insensitive: bool,
    str1: &str,
    str2: &str,
) -> Ordering {
    if let Some(c) = collator {
        c.compare(str1, str2)
    } else if case_insensitive {
        str1.to_lowercase().cmp(&str2.to_lowercase())
    } else {
        str1.cmp(str2)
    }
}

fn compare_two_ip_addresses(ip1: IpAddr, ip2: IpAddr) -> Ordering {
    match (ip1, ip2) {
        (IpAddr::V4(_), IpAddr::V6(_)) => return Ordering::Less,
        (IpAddr::V6(_), IpAddr::V4(_)) => return Ordering::Greater,
        _ => (),
    }

    let octets1 = octets_for_ip_address(ip1);
    let octets2 = octets_for_ip_address(ip2);

    for i in 0..octets1.len() {
        if octets1[i] != octets2[i] {
            return octets1[i].cmp(&octets2[i]);
        }
    }

    Ordering::Equal
}

fn octets_for_ip_address(ip: IpAddr) -> [u8; 16] {
    match ip {
        IpAddr::V4(ip) => ip.to_ipv6_compatible().octets(),
        IpAddr::V6(ip) => ip.octets(),
    }
}

#[cfg(test)]
mod test {
    use super::{
        Comparer, DatetimeTextComparer, IpComparer, NetworkComparer, NumberedTextComparer,
        PathComparer, PathType, TextComparer,
    };
    use crate::collation::collator_for_locale;
    use anyhow::Result;

    struct Case {
        name: &'static str,
        input: Vec<&'static str>,
        expect: Vec<&'static str>,
        case_insensitive: bool,
        locale: Option<&'static str>,
    }

    #[test]
    fn text_comparer() -> Result<()> {
        //crate::logging::init(true)?;
        for mut c in cases_from(TEXT_TEST_CASES) {
            println!("# text - {}", c.name);
            let tc = TextComparer {
                collator: c
                    .locale
                    .map(|l| collator_for_locale(l, c.case_insensitive).unwrap()),
                case_insensitive: c.case_insensitive,
            };
            c.input.sort_by(|a, b| tc.cmp(a, b).unwrap());
            assert_eq!(c.input, c.expect);
        }

        Ok(())
    }

    #[test]
    fn numbered_text_comparer() -> Result<()> {
        //crate::logging::init(true)?;
        for mut c in cases_from(TEXT_TEST_CASES)
            .into_iter()
            .chain(cases_from(NUMBERED_TEXT_TEST_CASES))
        {
            println!("# numbered text - {}", c.name);
            let ntc = NumberedTextComparer {
                collator: c
                    .locale
                    .map(|l| collator_for_locale(l, c.case_insensitive).unwrap()),
                case_insensitive: c.case_insensitive,
            };
            c.input.sort_by(|a, b| ntc.cmp(a, b).unwrap());
            assert_eq!(c.input, c.expect);
        }

        Ok(())
    }

    #[test]
    fn datetime_text_comparer() -> Result<()> {
        //crate::logging::init(true)?;
        for mut c in cases_from(TEXT_TEST_CASES)
            .into_iter()
            .chain(cases_from(DATETIME_TEXT_TEST_CASES))
        {
            println!("# datetime text - {}", c.name);
            let dtc = DatetimeTextComparer {
                collator: c
                    .locale
                    .map(|l| collator_for_locale(l, c.case_insensitive).unwrap()),
                case_insensitive: c.case_insensitive,
            };
            c.input.sort_by(|a, b| dtc.cmp(a, b).unwrap());
            assert_eq!(c.input, c.expect);
        }

        Ok(())
    }

    #[test]
    fn path_comparer() -> Result<()> {
        //crate::logging::init(true)?;
        for mut c in cases_from(PATH_TEST_CASES) {
            println!("# path - {}", c.name);
            let pc = PathComparer {
                collator: c
                    .locale
                    .map(|l| collator_for_locale(l, c.case_insensitive).unwrap()),
                case_insensitive: c.case_insensitive,
                path_type: if c.name.contains("Windows") {
                    PathType::Windows
                } else {
                    PathType::Unix
                },
            };
            c.input.sort_by(|a, b| pc.cmp(a, b).unwrap());
            assert_eq!(c.input, c.expect);
        }

        Ok(())
    }

    #[test]
    fn ip_comparer() -> Result<()> {
        //crate::logging::init(true)?;
        for mut c in cases_from(IP_TEST_CASES) {
            println!("# ip - {}", c.name);
            let ic = IpComparer;
            c.input.sort_by(|a, b| ic.cmp(a, b).unwrap());
            assert_eq!(c.input, c.expect);
        }

        Ok(())
    }

    #[test]
    fn network_comparer() -> Result<()> {
        //crate::logging::init(true)?;
        for mut c in cases_from(NETWORK_TEST_CASES) {
            println!("# network - {}", c.name);
            let nc = NetworkComparer;
            c.input.sort_by(|a, b| nc.cmp(a, b).unwrap());
            assert_eq!(c.input, c.expect);
        }

        Ok(())
    }

    fn cases_from(cases_text: &'static str) -> Vec<Case> {
        cases_text
            .split("====\n")
            .map(|case| {
                let mut elts = case.split("----\n");
                let name = elts.next().unwrap().trim();
                let input = elts.next().unwrap().trim().split('\n').collect();
                let expect = elts.next().unwrap().trim().split('\n').collect();
                let case_insensitive = match elts.next() {
                    Some("false\n") | None => false,
                    Some("true\n") => true,
                    Some(ci) => panic!("unknown case-insensitive value: {ci}"),
                };
                let locale = elts.next().map(|l| l.trim());
                Case {
                    name,
                    input,
                    expect,
                    case_insensitive,
                    locale,
                }
            })
            .collect()
    }

    const TEXT_TEST_CASES: &str = r#"
ASCII with no locale
----
go
bears
above
And
all
home
----
And
above
all
bears
go
home
----
false
====
ASCII with no locale, case-insensitive
----
go
bears
above
And
all
home
----
above
all
And
bears
go
home
----
true
====
ASCII with en-US locale
----
go
bears
above
And
all
home
----
above
all
And
bears
go
home
----
false
----
en-US
====
Unicode text with de-DE locale
----
zoo
foo
öoo
----
foo
öoo
zoo
----
false
----
de-DE
====
Unicode text with sv-SE locale
----
zoo
foo
öoo
----
foo
zoo
öoo
----
false
----
sv-SE
"#;

    const NUMBERED_TEXT_TEST_CASES: &str = r#"
numbered ASCII with no locale
----
120001 go
0. bears
15 - above
5. And
1. all
5. act
2. home
----
0. bears
1. all
2. home
5. And
5. act
15 - above
120001 go
----
false
====
numbered ASCII with no locale, case-insensitive
----
120001 go
0. bears
15 - above
5. And
1. all
5. act
2. home
----
0. bears
1. all
2. home
5. act
5. And
15 - above
120001 go
----
true
====
numbered Unicode with de-DE locale
----
3. zoo
1. foo
2. öoo
2. zoo
----
1. foo
2. öoo
2. zoo
3. zoo
----
false
----
de-DE
====
numbered Unicode with sv-Se locale
----
3. zoo
1. foo
2. öoo
2. zoo
----
1. foo
2. zoo
2. öoo
3. zoo
----
false
----
sv-SE
====
mixed numbered and unnumbered
----
10. x
aloe
27. bar
love
1. hello
----
1. hello
10. x
27. bar
aloe
love
----
false
====
numbered text with decimal numbers
----
10.1 - x
27.2314 - bar
1.00 - hello
----
1.00 - hello
10.1 - x
27.2314 - bar
----
false
"#;

    const DATETIME_TEXT_TEST_CASES: &str = r#"
datetime ASCII text with no locale
----
2017-1-12 hello
2014-05-07 foo
2018-12-30 bar
2014-05-07 FUN
----
2014-05-07 FUN
2014-05-07 foo
2017-1-12 hello
2018-12-30 bar
----
false
====
datetime ASCII text with no locale, case-insensitive
----
2017-1-12 hello
2014-05-07 foo
2018-12-30 bar
2014-05-07 FUN
----
2014-05-07 foo
2014-05-07 FUN
2017-1-12 hello
2018-12-30 bar
----
true
====
datetime ASCII text with de-DE locale
----
2017-1-12 hello
2014-05-07 zoo
2018-12-30 bar
2014-05-07 öoo
----
2014-05-07 öoo
2014-05-07 zoo
2017-1-12 hello
2018-12-30 bar
----
false
----
de-DE
====
datetime ASCII text with sv-SE locale
----
2017-1-12 hello
2014-05-07 zoo
2018-12-30 bar
2014-05-07 öoo
----
2014-05-07 zoo
2014-05-07 öoo
2017-1-12 hello
2018-12-30 bar
----
false
----
sv-SE
====
mixed datetime and non-datetime
----
2017-1-12 hello
no dt
also none
1973-01-01 and
----
1973-01-01 and
2017-1-12 hello
also none
no dt
----
false
====
datetime and dates
----
2017-1-12T01:00:37
1991-01-02
2017-1-12T14:01:01
----
1991-01-02
2017-1-12T01:00:37
2017-1-12T14:01:01
----
false
"#;

    const PATH_TEST_CASES: &str = r#"
path with ASCII text
----
/foo
/bar
baz/quux
a/q
C:\
/X
/A
----
/A
/X
/bar
/foo
C:\
a/q
baz/quux
----
false
====
path with ASCII text, case-insensitive
----
/foo
/bar
baz/quux
a/q
C:\
/X
/A
----
/A
/bar
/foo
/X
C:\
a/q
baz/quux
----
true
====
path with ASCII text, depth sorts before path content
----
/zzz
/bbb
/xxx/a
/aaaaaa/q/r
----
/bbb
/zzz
/xxx/a
/aaaaaa/q/r
----
false
====
Windows ASCII path
----
C:\foo
\a\b
\b
C:\bar
E:\a
B:\x
C:\a\b\c
C:\a\b
----
B:\x
C:\bar
C:\foo
C:\a\b
C:\a\b\c
E:\a
\b
\a\b
----
false
====
Unix Unicode path with de-DE locale
----
/foo
/bar
baz/quux
/zoo
a/q
/öoo
C:\\
/X
/A
----
/A
/bar
/foo
/öoo
/X
/zoo
C:\\
a/q
baz/quux
----
false
----
de-DE
====
Unix Unicode path with sv-SE locale
----
/foo
/bar
baz/quux
/zoo
a/q
/öoo
C:\\
/X
/A
----
/A
/bar
/foo
/X
/zoo
/öoo
C:\\
a/q
baz/quux
----
false
----
sv-SE
"#;

    const IP_TEST_CASES: &str = r#"
ip with just IPv4
----
1.1.1.1
0.1.255.255
123.100.125.242
1.255.0.0
----
0.1.255.255
1.1.1.1
1.255.0.0
123.100.125.242
====
ip with just IPv6
----
::1
::0
9876::fe01:1234:457f
1234::
----
::0
::1
1234::
9876::fe01:1234:457f
====
ip with mixed IPv4 and IPv6
----
::1
::0
255.255.255.255
::1234
9876::fe01:1234:457f
1.2.3.4
1234::
----
1.2.3.4
255.255.255.255
::0
::1
::1234
1234::
9876::fe01:1234:457f
"#;

    const NETWORK_TEST_CASES: &str = r#"
network with just IPv4
----
1.1.1.1/32
0.1.255.0/24
123.100.125.0/25
1.255.0.0/17
1.255.0.0/16
----
0.1.255.0/24
1.1.1.1/32
1.255.0.0/16
1.255.0.0/17
123.100.125.0/25
====
network with just IPv6
----
::1/128
::0/127
::0/42
9876::fe01:1234:0/24
1234::/90
----
::0/42
::0/127
::1/128
1234::/90
9876::fe01:1234:0/24
====
network with mixed IPv4 and IPv6
----
::1/128
::0/127
1.2.3.0/18
::0/42
1.2.3.0/16
9876::fe01:1234:0/24
255.255.255.0/25
1234::/90
----
1.2.3.0/16
1.2.3.0/18
255.255.255.0/25
::0/42
::0/127
::1/128
1234::/90
9876::fe01:1234:0/24
"#;
}
