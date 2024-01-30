use std::fmt::Debug;

use binrw::BinRead;
use nom::{branch::alt, character::complete::char, combinator::{map, value}, multi::many0, sequence::tuple, IResult};

use crate::{attributes::{Code, Exceptions, Signature}, field::TypeDescriptor, raw::{MethodAccessFlags, MethodRaw}, signature::MethodSignature, ClassFile};

pub struct MethodDescriptor<'a> {
    param_tys: Vec<TypeDescriptor<'a>>,
    return_ty: Option<TypeDescriptor<'a>>,
}

impl<'a> MethodDescriptor<'a> {
    pub fn parameter_types<'b>(&'b self) -> &'b [TypeDescriptor<'a>] {
        &self.param_tys
    }

    pub fn return_type(&self) -> Option<&TypeDescriptor<'a>> {
        self.return_ty.as_ref()
    }

    pub(crate) fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (_, param_tys, _, return_ty)) = tuple((
            char('('),
            many0(TypeDescriptor::parse),
            char(')'),
            alt((
                value(None, char('V')),
                map(TypeDescriptor::parse, |x| Some(x)),
            )),
        ))(input)?;
        Ok((
            input,
            Self {
                param_tys,
                return_ty,
            },
        ))
    }
}

impl<'a> Debug for MethodDescriptor<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MethodDescriptor")
            .field(&self.param_tys)
            .field(&self.return_ty)
            .finish()
    }
}

pub struct Method<'a> {
    pub(crate) class_file: &'a ClassFile,
    pub(crate) method_inner: &'a MethodRaw,
}

impl<'a> Method<'a> {
    pub fn identifier(&self) -> crate::Result<&'a str> {
        self.method_inner.name_index.get_as_string(self.class_file)
    }

    pub fn descriptor(&self) -> crate::Result<MethodDescriptor<'a>> {
        let raw_descriptor = self
            .method_inner
            .descriptor_index
            .get_as_string(self.class_file)?;
        let (_input, ty) = MethodDescriptor::parse(raw_descriptor)?;
        Ok(ty)
    }

    pub fn jni_identifier(&self) -> crate::Result<String> {
        Ok(format!(
            "Java_{}_{}",
            self.class_file
                .this_class()?
                .replace('/', "_")
                .replace('$', "_"),
            self.identifier()?
        ))
    }

    pub fn is_native(&self) -> bool {
        self.method_inner
            .access_flags
            .contains(MethodAccessFlags::NATIVE)
    }

    pub fn code(&self) -> crate::Result<Option<Code<'a>>> {
        match self.method_inner.attributes.0.get("Code") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = Code::read_be_args(&mut buf, (self.class_file,))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn exceptions(&self) -> crate::Result<Option<Exceptions<'a>>> {
        match self.method_inner.attributes.0.get("Exceptions") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = Exceptions::read_be_args(&mut buf, (self.class_file,))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn signature(&self) -> crate::Result<Option<MethodSignature<'a>>> {
        match self.method_inner.attributes.0.get("Signature") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = Signature::read_be_args(&mut buf, (self.class_file,))?;
                Ok(Some(value.get_method()?))
            }
            None => Ok(None),
        }
    }
}

impl<'a> Debug for Method<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Method")
            .field("identifier", &self.identifier())
            .field("desciptor", &self.descriptor())
            .field("code", &self.code().unwrap())
            .finish()
    }
}