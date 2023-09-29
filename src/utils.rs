use serde::{
    de::{DeserializeOwned, Deserializer, Error, Visitor},
    forward_to_deserialize_any,
};

pub type StructFields = &'static [&'static str];

// https://github.com/serde-rs/serde/issues/1110#issuecomment-348822979
pub fn struct_fields<T: DeserializeOwned>() -> Result<StructFields, crate::Error> {
    struct StructFieldsDeserializer<'a> {
        fields: &'a mut Option<StructFields>,
        err: &'a mut Option<serde::de::value::Error>,
    }

    impl<'de, 'a> Deserializer<'de> for StructFieldsDeserializer<'a> {
        type Error = serde::de::value::Error;

        fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(Error::custom(""))
        }

        fn deserialize_struct<V>(
            self,
            _: &'static str,
            fields: StructFields,
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.fields = Some(fields);
            self.deserialize_any(visitor)
        }

        fn deserialize_enum<V>(
            self,
            name: &'static str,
            _: &'static [&'static str],
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.err = Some(Error::custom(format!(
                "`{name}` enum struct is not supported for `Query`"
            )));
            Err(Error::custom(""))
        }

        fn deserialize_unit_struct<V>(
            self,
            name: &'static str,
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.err = Some(Error::custom(format!(
                "`{name}` unit struct is not supported for `Query`"
            )));
            Err(Error::custom(""))
        }

        fn deserialize_tuple_struct<V>(
            self,
            name: &'static str,
            _: usize,
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.err = Some(Error::custom(format!(
                "`{name}` tuple struct is not supported for `Query`"
            )));
            Err(Error::custom(""))
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit seq tuple
            map identifier ignored_any
        }

        fn deserialize_newtype_struct<V>(
            self,
            name: &'static str,
            _: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.err = Some(Error::custom(format!(
                "`{name}` tuple struct is not supported for `Query`"
            )));
            Err(Error::custom(""))
        }
    }

    let mut fields = None;
    let mut err = None;
    let _ = T::deserialize(StructFieldsDeserializer {
        fields: &mut fields,
        err: &mut err,
    });

    if let Some(err) = err {
        return Err(err.into());
    }

    Ok(fields.unwrap())
}
