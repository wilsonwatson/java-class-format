use std::fmt::Debug;

use attributes::{EnclosingMethod, InnerClasses, Signature, SourceFile};
use binrw::{binread, BinRead};

pub mod error;
pub(crate) mod raw;
pub mod field;
pub mod method;
pub mod attributes;
pub mod signature;

pub use error::{Result, Error};
use field::Field;
use method::Method;
use raw::{Attributes, ClassAccessFlags, ClassIndex, FieldRaw, MethodRaw};
use signature::ClassSignature;

#[binread]
#[br(big, magic = b"\xca\xfe\xba\xbe")]
pub struct ClassFile {
    _minor_version: u16,
    _major_version: u16,
    constant_pool: raw::ConstantPool,
    #[br(map = |x: u16| ClassAccessFlags::from_bits_truncate(x))]
    access_flags: ClassAccessFlags,
    this_class: ClassIndex,
    super_class: ClassIndex,
    #[br(temp)]
    interfaces_count: u16,
    #[br(count = interfaces_count)]
    interfaces: Vec<ClassIndex>,
    #[br(temp)]
    fields_count: u16,
    #[br(args { count: fields_count.into(), inner: (&constant_pool,) })]
    fields: Vec<FieldRaw>,
    #[br(temp)]
    methods_count: u16,
    #[br(args { count: methods_count.into(), inner: (&constant_pool,) })]
    methods: Vec<MethodRaw>,
    #[br(args(&constant_pool,))]
    attributes: Attributes,
}

impl ClassFile {
    pub fn parse<T>(t: T) -> Result<Self>
    where
        std::io::Cursor<T>: std::io::Read + std::io::Seek,
    {
        Ok(Self::read_be(&mut std::io::Cursor::new(t))?)
    }

    pub fn this_class<'a>(&'a self) -> Result<&'a str> {
        self.this_class.get_as_string(&self)
    }

    pub fn super_class<'a>(&'a self) -> Result<&'a str> {
        self.super_class.get_as_string(&self)
    }

    pub fn interfaces<'a>(&'a self) -> Result<Vec<&'a str>> {
        self.interfaces
            .iter()
            .map(|x| x.get_as_string(&self))
            .collect::<Result<Vec<_>>>()
    }

    pub fn methods<'a>(&'a self) -> Vec<Method<'a>> {
        self.methods
            .iter()
            .map(|x| Method {
                class_file: self,
                method_inner: x,
            })
            .collect()
    }

    pub fn fields<'a>(&'a self) -> Vec<Field<'a>> {
        self.fields
            .iter()
            .map(|x| Field {
                class_file: self,
                field_inner: x,
            })
            .collect()
    }

    pub fn inner_classes<'a>(&'a self) -> Result<Option<InnerClasses<'a>>> {
        match self.attributes.0.get("InnerClasses") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = InnerClasses::read_be_args(&mut buf, (self,))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn enclosing_method<'a>(&'a self) -> Result<Option<EnclosingMethod<'a>>> {
        match self.attributes.0.get("EnclosingMethod") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = EnclosingMethod::read_be_args(&mut buf, (self,))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn signature<'a>(&'a self) -> crate::Result<Option<ClassSignature<'a>>> {
        match self.attributes.0.get("Signature") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = Signature::read_be_args(&mut buf, (self,))?;
                Ok(Some(value.get_class()?))
            }
            None => Ok(None),
        }
    }

    pub fn source_file<'a>(&'a self) -> crate::Result<Option<SourceFile<'a>>> {
        match self.attributes.0.get("SourceFile") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = SourceFile::read_be_args(&mut buf, (self,))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

impl Debug for ClassFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassFile")
            .field("minor_version", &self._minor_version)
            .field("major_version", &self._major_version)
            .field("access_flags", &self.access_flags)
            .field("this_class", &self.this_class.get_as_string_impl(&self.constant_pool))
            .field("super_class", &self.super_class.get_as_string_impl(&self.constant_pool))
            .field(
                "interfaces",
                &self
                    .interfaces
                    .iter()
                    .map(|x| x.get_as_string_impl(&self.constant_pool))
                    .collect::<Vec<_>>(),
            )
            .field("fields", &self.fields())
            .field("methods", &self.methods())
            .field("attributes", &self.attributes)
            .finish()
    }
}
