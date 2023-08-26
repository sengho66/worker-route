use serde::{
    de::{self, Deserialize, Deserializer, Visitor},
    forward_to_deserialize_any,
};

// https://github.com/serde-rs/serde/issues/1110#issuecomment-348822979
pub fn struct_fields<'de, T>() -> &'static [&'static str]
where
    T: Deserialize<'de>,
{
    struct StructFieldsDeserializer<'a> {
        fields: &'a mut Option<&'static [&'static str]>,
    }

    impl<'de, 'a> Deserializer<'de> for StructFieldsDeserializer<'a> {
        type Error = serde::de::value::Error;

        fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(de::Error::custom("I'm just here for the fields"))
        }

        fn deserialize_struct<V>(
            self,
            _name: &'static str,
            fields: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            *self.fields = Some(fields);
            self.deserialize_any(visitor)
        }

        forward_to_deserialize_any! {
            bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
            byte_buf option unit unit_struct seq tuple newtype_struct
            tuple_struct map enum identifier ignored_any
        }
    }

    let mut fields = None;

    let _ = T::deserialize(StructFieldsDeserializer {
        fields: &mut fields,
    });
    fields.unwrap()
}
