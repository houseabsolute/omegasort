// @generated
#[clippy::msrv = "1.61"]
mod collator;
#[clippy::msrv = "1.61"]
mod fallback;
#[clippy::msrv = "1.61"]
mod normalizer;
#[clippy::msrv = "1.61"]
use icu_provider::prelude::*;
/// Implement [`DataProvider<M>`] on the given struct using the data
/// hardcoded in this module. This allows the struct to be used with
/// `icu`'s `_unstable` constructors.
///
/// This macro can only be called from its definition-site, i.e. right
/// after `include!`-ing the generated module.
///
/// ```compile_fail
/// struct MyDataProvider;
/// include!("/path/to/generated/mod.rs");
/// impl_data_provider(MyDataProvider);
/// ```
#[allow(unused_macros)]
macro_rules! impl_data_provider {
    ($ provider : path) => {
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu::collator::provider::CollationDataV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu::collator::provider::CollationDataV1Marker>, DataError> {
                collator::data_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu::collator::provider::CollationDataV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu::collator::provider::CollationDiacriticsV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu::collator::provider::CollationDiacriticsV1Marker>, DataError> {
                collator::dia_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu::collator::provider::CollationDiacriticsV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu::collator::provider::CollationJamoV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu::collator::provider::CollationJamoV1Marker>, DataError> {
                collator::jamo_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu::collator::provider::CollationJamoV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu::collator::provider::CollationMetadataV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu::collator::provider::CollationMetadataV1Marker>, DataError> {
                collator::meta_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu::collator::provider::CollationMetadataV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu::collator::provider::CollationReorderingV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu::collator::provider::CollationReorderingV1Marker>, DataError> {
                collator::reord_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu::collator::provider::CollationReorderingV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu::collator::provider::CollationSpecialPrimariesV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu::collator::provider::CollationSpecialPrimariesV1Marker>, DataError> {
                collator::prim_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu::collator::provider::CollationSpecialPrimariesV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu::normalizer::provider::CanonicalDecompositionDataV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu::normalizer::provider::CanonicalDecompositionDataV1Marker>, DataError> {
                normalizer::nfd_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu::normalizer::provider::CanonicalDecompositionDataV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu::normalizer::provider::CanonicalDecompositionTablesV1Marker> for $provider {
            fn load(&self, req: DataRequest) -> Result<DataResponse<::icu::normalizer::provider::CanonicalDecompositionTablesV1Marker>, DataError> {
                normalizer::nfdex_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| DataErrorKind::MissingLocale.with_req(::icu::normalizer::provider::CanonicalDecompositionTablesV1Marker::KEY, req))
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_provider_adapters::fallback::provider::CollationFallbackSupplementV1Marker> for $provider {
            fn load(
                &self,
                req: DataRequest,
            ) -> Result<DataResponse<::icu_provider_adapters::fallback::provider::CollationFallbackSupplementV1Marker>, DataError> {
                fallback::supplement::co_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| {
                        DataErrorKind::MissingLocale.with_req(
                            ::icu_provider_adapters::fallback::provider::CollationFallbackSupplementV1Marker::KEY,
                            req,
                        )
                    })
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_provider_adapters::fallback::provider::LocaleFallbackLikelySubtagsV1Marker> for $provider {
            fn load(
                &self,
                req: DataRequest,
            ) -> Result<DataResponse<::icu_provider_adapters::fallback::provider::LocaleFallbackLikelySubtagsV1Marker>, DataError> {
                fallback::likelysubtags_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| {
                        DataErrorKind::MissingLocale.with_req(
                            ::icu_provider_adapters::fallback::provider::LocaleFallbackLikelySubtagsV1Marker::KEY,
                            req,
                        )
                    })
            }
        }
        #[clippy::msrv = "1.61"]
        impl DataProvider<::icu_provider_adapters::fallback::provider::LocaleFallbackParentsV1Marker> for $provider {
            fn load(
                &self,
                req: DataRequest,
            ) -> Result<DataResponse<::icu_provider_adapters::fallback::provider::LocaleFallbackParentsV1Marker>, DataError> {
                fallback::parents_v1::lookup(&req.locale)
                    .map(zerofrom::ZeroFrom::zero_from)
                    .map(DataPayload::from_owned)
                    .map(|payload| DataResponse {
                        metadata: Default::default(),
                        payload: Some(payload),
                    })
                    .ok_or_else(|| {
                        DataErrorKind::MissingLocale.with_req(::icu_provider_adapters::fallback::provider::LocaleFallbackParentsV1Marker::KEY, req)
                    })
            }
        }
    };
}
/// Implement [`AnyProvider`] on the given struct using the data
/// hardcoded in this module. This allows the struct to be used with
/// `icu`'s `_any` constructors.
///
/// This macro can only be called from its definition-site, i.e. right
/// after `include!`-ing the generated module.
///
/// ```compile_fail
/// struct MyAnyProvider;
/// include!("/path/to/generated/mod.rs");
/// impl_any_provider(MyAnyProvider);
/// ```
#[allow(unused_macros)]
macro_rules! impl_any_provider {
    ($ provider : path) => {
        #[clippy::msrv = "1.61"]
        impl AnyProvider for $provider {
            fn load_any(&self, key: DataKey, req: DataRequest) -> Result<AnyResponse, DataError> {
                const COLLATIONDATAV1MARKER: ::icu_provider::DataKeyHash = ::icu::collator::provider::CollationDataV1Marker::KEY.hashed();
                const COLLATIONDIACRITICSV1MARKER: ::icu_provider::DataKeyHash = ::icu::collator::provider::CollationDiacriticsV1Marker::KEY.hashed();
                const COLLATIONJAMOV1MARKER: ::icu_provider::DataKeyHash = ::icu::collator::provider::CollationJamoV1Marker::KEY.hashed();
                const COLLATIONMETADATAV1MARKER: ::icu_provider::DataKeyHash = ::icu::collator::provider::CollationMetadataV1Marker::KEY.hashed();
                const COLLATIONREORDERINGV1MARKER: ::icu_provider::DataKeyHash = ::icu::collator::provider::CollationReorderingV1Marker::KEY.hashed();
                const COLLATIONSPECIALPRIMARIESV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu::collator::provider::CollationSpecialPrimariesV1Marker::KEY.hashed();
                const CANONICALDECOMPOSITIONDATAV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu::normalizer::provider::CanonicalDecompositionDataV1Marker::KEY.hashed();
                const CANONICALDECOMPOSITIONTABLESV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu::normalizer::provider::CanonicalDecompositionTablesV1Marker::KEY.hashed();
                const COLLATIONFALLBACKSUPPLEMENTV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_provider_adapters::fallback::provider::CollationFallbackSupplementV1Marker::KEY.hashed();
                const LOCALEFALLBACKLIKELYSUBTAGSV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_provider_adapters::fallback::provider::LocaleFallbackLikelySubtagsV1Marker::KEY.hashed();
                const LOCALEFALLBACKPARENTSV1MARKER: ::icu_provider::DataKeyHash =
                    ::icu_provider_adapters::fallback::provider::LocaleFallbackParentsV1Marker::KEY.hashed();
                match key.hashed() {
                    COLLATIONDATAV1MARKER => collator::data_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COLLATIONDIACRITICSV1MARKER => collator::dia_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COLLATIONJAMOV1MARKER => collator::jamo_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COLLATIONMETADATAV1MARKER => collator::meta_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COLLATIONREORDERINGV1MARKER => collator::reord_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COLLATIONSPECIALPRIMARIESV1MARKER => collator::prim_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    CANONICALDECOMPOSITIONDATAV1MARKER => normalizer::nfd_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    CANONICALDECOMPOSITIONTABLESV1MARKER => normalizer::nfdex_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    COLLATIONFALLBACKSUPPLEMENTV1MARKER => fallback::supplement::co_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    LOCALEFALLBACKLIKELYSUBTAGSV1MARKER => fallback::likelysubtags_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    LOCALEFALLBACKPARENTSV1MARKER => fallback::parents_v1::lookup(&req.locale).map(AnyPayload::from_static_ref),
                    _ => return Err(DataErrorKind::MissingDataKey.with_req(key, req)),
                }
                .map(|payload| AnyResponse {
                    payload: Some(payload),
                    metadata: Default::default(),
                })
                .ok_or_else(|| DataErrorKind::MissingLocale.with_req(key, req))
            }
        }
    };
}
#[clippy::msrv = "1.61"]
pub struct BakedDataProvider;
impl_data_provider!(BakedDataProvider);
