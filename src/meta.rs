use std::borrow::Cow;
use std::convert::From;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'a> {
    Nil,
    Bool(bool),
    Signed(i64),
    // Unsigned(u64),
    // Float(f64),
    String(Cow<'a, str>),
    // Func(&'a Fn(&mut Write) -> Result<(), ::std::io::Error>),
}

impl<'a> From<bool> for Value<'a> {
    fn from(val: bool) -> Value<'a> {
        Value::Bool(val)
    }
}

impl<'a> From<i8> for Value<'a> {
    fn from(val: i8) -> Value<'a> {
        Value::Signed(val as i64)
    }
}

impl<'a> From<i16> for Value<'a> {
    fn from(val: i16) -> Value<'a> {
        Value::Signed(val as i64)
    }
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(val: &'a str) -> Value<'a> {
        Value::String(Cow::Borrowed(val))
    }
}

impl<'a> From<&'a String> for Value<'a> {
    fn from(val: &'a String) -> Value<'a> {
        Value::String(Cow::Borrowed(val))
    }
}

impl<'a> From<String> for Value<'a> {
    fn from(val: String) -> Value<'a> {
        Value::String(Cow::Owned(val))
    }
}

impl<'a, T> From<Option<T>> for Value<'a>
    where T: Into<Value<'a>>
{
    fn from(val: Option<T>) -> Value<'a> {
        unimplemented!();
    }
}

pub struct Meta<'a> {
    name: &'a str,
    value: Value<'a>,
}

impl<'a> Meta<'a> {
    pub fn new<V>(name: &'a str, value: V) -> Meta<'a>
        where V: Into<Value<'a>>
    {
        Meta {
            name: name,
            value: value.into(),
        }
    }
}

pub struct MetaList<'a> {
    prev: Option<&'a MetaList<'a>>,
    meta: &'a [Meta<'a>],
}

impl<'a> MetaList<'a> {
    pub fn new(meta: &'a [Meta<'a>]) -> MetaList<'a> {
        MetaList::next(meta, None)
    }

    pub fn next(meta: &'a [Meta<'a>], prev: Option<&'a MetaList<'a>>) -> MetaList<'a> {
        MetaList {
            prev: prev,
            meta: meta,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::Value;

    #[test]
    fn bool_into_value() {
        assert_eq!(Value::Bool(true), true.into());
        assert_eq!(Value::Bool(false), false.into());
    }

    #[test]
    fn i8_into_value() {
        assert_eq!(Value::Signed(42), 42i8.into());
    }

    #[test]
    fn i16_into_value() {
        assert_eq!(Value::Signed(4200), 4200i16.into());
    }

    #[test]
    fn str_into_value() {
        assert_eq!(Value::String(Cow::Borrowed("le message")), "le message".into());
    }

    #[test]
    fn from_string_by_ref() {
        let string = "le message".to_owned();
        assert_eq!(Value::String(Cow::Borrowed("le message")), From::from(&string));
    }

    #[test]
    fn from_string() {
        let string = "le message".to_owned();
        assert_eq!(Value::String(Cow::Owned("le message".into())), From::from(string));
    }

    #[test]
    fn from_none() {
        let val: Option<bool> = None;
        assert_eq!(Value::Nil, From::from(val));
    }
}
