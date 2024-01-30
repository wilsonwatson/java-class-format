use std::fmt::Debug;

use binrw::BinRead;
use nom::{branch::alt, bytes::complete::{is_not, tag}, character::complete::char, combinator::{map, value}, sequence::{delimited, preceded}, IResult};

use crate::{attributes::{ConstantValue, Signature}, raw::FieldRaw, signature::ReferenceType, ClassFile};

#[derive(Clone, Debug)]
pub enum TypeDescriptor<'a> {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    String,
    Class,
    Array(Box<TypeDescriptor<'a>>),
    ClassName(&'a str),
}

impl<'a> TypeDescriptor<'a> {
    pub(crate) fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Self::Byte, char('B')),
            value(Self::Char, char('C')),
            value(Self::Double, char('D')),
            value(Self::Float, char('F')),
            value(Self::Int, char('I')),
            value(Self::Long, char('J')),
            value(Self::Short, char('S')),
            value(Self::Boolean, char('Z')),
            value(Self::String, tag("Ljava/lang/String;")),
            value(Self::Class, tag("Ljava/lang/Class;")),
            map(delimited(char('L'), is_not(";"), char(';')), |x| {
                Self::ClassName(x)
            }),
            map(preceded(char('['), Self::parse), |x| {
                Self::Array(Box::new(x))
            }),
        ))(input)
    }
}

pub struct Field<'a> {
    pub(crate) class_file: &'a ClassFile,
    pub(crate) field_inner: &'a FieldRaw,
}

impl<'a> Field<'a> {
    pub fn identifier(&self) -> crate::Result<&'a str> {
        self.field_inner.name_index.get_as_string(self.class_file)
    }

    pub fn descriptor(&self) -> crate::Result<TypeDescriptor<'a>> {
        let raw_descriptor = self
            .field_inner
            .descriptor_index
            .get_as_string(self.class_file)?;
        let (_input, ty) = TypeDescriptor::parse(raw_descriptor)?;
        Ok(ty)
    }

    pub fn constant_value(&self) -> crate::Result<Option<ConstantValue<'a>>> {
        match self.field_inner.attributes.0.get("ConstantValue") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = ConstantValue::read_be_args(&mut buf, (self.class_file,))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn signature(&self) -> crate::Result<Option<ReferenceType<'a>>> {
        match self.field_inner.attributes.0.get("Signature") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = Signature::read_be_args(&mut buf, (self.class_file,))?;
                Ok(Some(value.get_field()?))
            }
            None => Ok(None),
        }
    }
}

impl<'a> Debug for Field<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Method")
            .field("identifier", &self.identifier())
            .field("desciptor", &self.descriptor())
            .finish()
    }
}