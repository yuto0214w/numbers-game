// 書いた人: @kagesakura

macro_rules! dcwt {
    {
        target: $target:ty,
        tag: $tag:expr,
        content: $content:expr,
        with_no_content: [$($no_content_variants:ident = $ncv_init:expr),+],
        with_single_content: [$to_vwc:ty => $($with_single_content:ident ( $_unused:ident ) = $vwc_init:expr),+],
        with_tuplelike_content: [$to_vwtc:ty => $($with_tuplelike_content:ident ( $($tmp_p:ident),+ ) = $vwtc_init:expr),+]
    } => {
        const _: () = {
            extern crate serde as _serde;

            use _serde::__private::de::{ContentDeserializer, TagContentOtherField, TagContentOtherFieldVisitor, TagOrContentField};
            use _serde::de::{Visitor, Error as SerdeDeError, IgnoredAny, MapAccess, Unexpected};
            use _serde::Deserializer;
            use ::core::option::Option::{self, None, Some};
            use ::core::result::Result::{self, Ok, Err};
            use ::core::fmt::{Formatter, Result as FmtResult};
            use ::std::string::String;
            use ::core::primitive::*;

            const TAG: &'static str = $tag;
            const CONTENT: &'static str = $content;

            #[inline(always)]
            fn find_tag_or_content<'a, A: MapAccess<'a>>(
                tag: &'static str,
                content: &'static str,
                map: &mut A,
            ) -> Result<Option<TagOrContentField>, A::Error> {
                while let Some(k) = map.next_key_seed(TagContentOtherFieldVisitor { tag, content })? {
                    match k {
                        TagContentOtherField::Other => {
                            let _: IgnoredAny = MapAccess::next_value(map)?;
                        }
                        TagContentOtherField::Tag => return Ok(Some(TagOrContentField::Tag)),
                        TagContentOtherField::Content => return Ok(Some(TagOrContentField::Content)),
                    }
                }
                Ok(None)
            }

            struct EnumVisitor;

            impl<'de> Visitor<'de> for EnumVisitor {
                type Value = $target;

                #[inline(always)]
                fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                    formatter.write_str("adjacently tagged enum")
                }

                #[inline(always)]
                fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                    macro_rules! field_error {
                        (duplicated, $name:expr) => {
                            Err(SerdeDeError::duplicate_field($name))
                        };
                        (missing, $name:expr) => {
                            Err(SerdeDeError::missing_field($name))
                        };
                    }
                    match find_tag_or_content(TAG, CONTENT, &mut map)? {
                        Some(TagOrContentField::Tag) => {
                            let tag: _Variant = map.next_value()?;
                            match find_tag_or_content(TAG, CONTENT, &mut map)? {
                                Some(TagOrContentField::Tag) => field_error!(duplicated, TAG),
                                Some(TagOrContentField::Content) => {
                                    let ret = deserialize_content_with_tag(
                                        tag,
                                        ContentDeserializer::new(map.next_value()?),
                                    )?;
                                    match find_tag_or_content(TAG, CONTENT, &mut map)? {
                                        Some(TagOrContentField::Tag) => field_error!(duplicated, TAG),
                                        Some(TagOrContentField::Content) => {
                                            field_error!(duplicated, CONTENT)
                                        }
                                        None => Ok(ret),
                                    }
                                }
                                None => deserialize_no_content_tag(tag)
                                    .ok_or(SerdeDeError::missing_field(CONTENT)),
                            }
                        }
                        Some(TagOrContentField::Content) => {
                            let content = MapAccess::next_value(&mut map)?;
                            match find_tag_or_content(TAG, CONTENT, &mut map)? {
                                Some(TagOrContentField::Tag) => {
                                    let ret = deserialize_content_with_tag(
                                        map.next_value()?,
                                        ContentDeserializer::new(content),
                                    )?;
                                    match find_tag_or_content(TAG, CONTENT, &mut map)? {
                                        Some(TagOrContentField::Tag) => field_error!(duplicated, TAG),
                                        Some(TagOrContentField::Content) => {
                                            field_error!(duplicated, CONTENT)
                                        }
                                        None => Ok(ret),
                                    }
                                }
                                Some(TagOrContentField::Content) => field_error!(duplicated, CONTENT),
                                None => field_error!(missing, TAG),
                            }
                        }
                        None => field_error!(missing, TAG),
                    }
                }
            }

            #[repr(u64)]
            enum _Variant {
                $($no_content_variants = $ncv_init),+,
                $($with_single_content = $vwc_init),+,
                $($with_tuplelike_content = $vwtc_init),+
            }

            struct _TagVisitor;

            impl<'de> Visitor<'de> for _TagVisitor {
                type Value = _Variant;

                #[inline(always)]
                fn expecting(&self, formatter: &mut Formatter) -> FmtResult {
                    formatter.write_str("valid variant")
                }

                #[inline(always)]
                fn visit_u64<E: SerdeDeError>(self, value: u64) -> Result<Self::Value, E> {
                    $(if _Variant::$no_content_variants as u64 == value {
                        return Ok(_Variant::$no_content_variants)
                    })+
                    $(if _Variant::$with_single_content as u64 == value {
                        return Ok(_Variant::$with_single_content)
                    })+
                    $(if _Variant::$with_tuplelike_content as u64 == value {
                        return Ok(_Variant::$with_tuplelike_content)
                    })+
                    Err(SerdeDeError::invalid_value(
                        Unexpected::Unsigned(value),
                        &"valid variant index",
                    ))
                }

                #[inline(always)]
                fn visit_str<E: SerdeDeError>(self, value: &str) -> Result<Self::Value, E> {
                    $(if stringify!($no_content_variants) == value {
                        return Ok(_Variant::$no_content_variants)
                    })+
                    $(if stringify!($with_single_content) == value {
                        return Ok(_Variant::$with_single_content)
                    })+
                    $(if stringify!($with_tuplelike_content) == value {
                        return Ok(_Variant::$with_tuplelike_content)
                    })+
                    Err(SerdeDeError::unknown_variant(value, &[$(
                        stringify!($no_content_variants)
                    ),+, $(
                        stringify!($with_single_content)
                    ),+, $(
                        stringify!($with_tuplelike_content)
                    ),+]))
                }

                #[inline(always)]
                fn visit_bytes<E: SerdeDeError>(self, value: &[u8]) -> Result<Self::Value, E> {
                    $(if stringify!($no_content_variants).as_bytes() == value {
                        return Ok(_Variant::$no_content_variants)
                    })+
                    $(if stringify!($with_single_content).as_bytes() == value {
                        return Ok(_Variant::$with_single_content)
                    })+
                    $(if stringify!($with_tuplelike_content).as_bytes() == value {
                        return Ok(_Variant::$with_tuplelike_content)
                    })+
                    let value = &String::from_utf8_lossy(value);
                    Err(SerdeDeError::unknown_variant(value, &[$(
                        stringify!($no_content_variants)
                    ),+, $(
                        stringify!($with_single_content)
                    ),+, $(
                        stringify!($with_tuplelike_content)
                    ),+]))
                }
            }

            impl<'de> _serde::Deserialize<'de> for _Variant {
                #[inline(always)]
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    deserializer.deserialize_any(_TagVisitor)
                }
            }

            #[inline(always)]
            fn deserialize_no_content_tag(tag: _Variant) -> Option<$target> {
                match tag {
                    $(_Variant::$no_content_variants => Some(<$target>::$no_content_variants)),+,
                    _ => None,
                }
            }

            #[inline(always)]
            fn deserialize_content_with_tag<E: SerdeDeError>(
                tag: _Variant,
                content_deserializer: ContentDeserializer<E>,
            ) -> Result<PlayerAction, E> {
                match tag {
                    $(_Variant::$no_content_variants => <() as _serde::Deserialize>::deserialize(content_deserializer).map(|_| <$target>::$no_content_variants)),+,
                    $(_Variant::$with_single_content => <$to_vwc as _serde::Deserialize>::deserialize(content_deserializer).map(<$target>::$with_single_content)),+,
                    $(_Variant::$with_tuplelike_content => <$to_vwtc as _serde::Deserialize>::deserialize(content_deserializer).map(|( $($tmp_p),+ )| <$target>::$with_tuplelike_content( $($tmp_p),+ ))),+,
                }
            }

            impl<'de> _serde::Deserialize<'de> for $target {
                #[inline(always)]
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    deserializer.deserialize_struct(
                        stringify!($target),
                        &[TAG, CONTENT],
                        EnumVisitor,
                    )
                }
            }
        };
    };
}

pub(crate) use dcwt;
