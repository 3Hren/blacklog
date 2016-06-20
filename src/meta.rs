
pub enum Value<'a> {
    // Nil,
    // Bool(bool),
    // Signed(i64),
    // Unsigned(u64),
    // Float(f64),
    String(&'a str),
    // Func(&'a Fn(&mut Write) -> Result<(), ::std::io::Error>),
}

impl<'a> Into<Value<'a>> for &'a str {
    fn into(self) -> Value<'a> {
        Value::String(&self)
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
