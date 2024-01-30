use std::{collections::HashMap, fmt::Debug};

use binrw::{binread, BinRead};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{char, none_of},
    combinator::{map, value},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

bitflags::bitflags! {
    #[derive(Debug)]
    struct ClassAccessFlags: u16 {
        const PUBLIC     = 0x0001;
        const FINAL      = 0x0010;
        const SUPER      = 0x0020;
        const INTERFACE  = 0x0200;
        const ABSTRACT   = 0x0400;
        const SYNTHETIC  = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM       = 0x4000;
        const MODULE     = 0x8000;
    }
}

#[binread]
#[br(big, magic = b"\xca\xfe\xba\xbe")]
pub struct ClassFile {
    _minor_version: u16,
    _major_version: u16,
    constant_pool: ConstantPool,
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
}

impl Debug for ClassFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClassFile")
            .field("minor_version", &self._minor_version)
            .field("major_version", &self._major_version)
            .field("access_flags", &self.access_flags)
            .field("this_class", &self.this_class.debug(&self.constant_pool))
            .field("super_class", &self.super_class.debug(&self.constant_pool))
            .field(
                "interfaces",
                &self
                    .interfaces
                    .iter()
                    .map(|x| x.debug(&self.constant_pool))
                    .collect::<Vec<_>>(),
            )
            .field("fields", &self.fields())
            .field("methods", &self.methods())
            .field("attributes", &self.attributes)
            .finish()
    }
}

#[derive(Debug)]
struct ConstantPool(Vec<ConstantPoolItem>);

impl BinRead for ConstantPool {
    type Args<'a> = ();

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let cpool_count = u16::read_be(reader)?;
        let mut cpool = Vec::new();
        let mut i = 1;
        loop {
            if i >= cpool_count {
                break;
            }
            let pos = reader.stream_position()?;
            let item = ConstantPoolItem::read_be(reader)?;
            let bump = match &item {
                ConstantPoolItem::Long { .. } | ConstantPoolItem::Double { .. } => 2,
                ConstantPoolItem::Skip => {
                    return Err(binrw::Error::AssertFail {
                        pos,
                        message: format!("Invalid Constant Pool Item."),
                    })
                }
                _ => 1,
            };
            cpool.push(item);
            for _ in 1..bump {
                cpool.push(ConstantPoolItem::Skip);
            }

            i += bump;
        }
        Ok(Self(cpool))
    }
}

#[binread]
#[derive(Debug)]
enum ConstantPoolItem {
    #[doc = "CONSTANT_Class as defined in §4.4.1"]
    #[br(magic = 7u8)]
    Class { name_index: Utf8Index },
    #[doc = "CONSTANT_Fieldref as defined in §4.4.2"]
    #[br(magic = 9u8)]
    Fieldref {
        class_index: ClassIndex,
        name_and_type_index: NameAndTypeIndex,
    },
    #[doc = "CONSTANT_Methodref as defined in §4.4.2"]
    #[br(magic = 10u8)]
    Methodref {
        class_index: ClassIndex,
        name_and_type_index: NameAndTypeIndex,
    },
    #[doc = "CONSTANT_InterfaceMethodref as defined in §4.4.2"]
    #[br(magic = 11u8)]
    InterfaceMethodref {
        class_index: ClassIndex,
        name_and_type_index: NameAndTypeIndex,
    },
    #[doc = "CONSTANT_String as defined in §4.4.3"]
    #[br(magic = 8u8)]
    String { string_index: Utf8Index },
    #[doc = "CONSTANT_Integer as defined in §4.4.4"]
    #[br(magic = 3u8)]
    Integer { value: i32 },
    #[doc = "CONSTANT_Float as defined in §4.4.4"]
    #[br(magic = 4u8)]
    Float { value: f32 },
    #[doc = "CONSTANT_Long as defined in §4.4.5"]
    #[br(magic = 5u8)]
    Long { value: i64 },
    #[doc = "CONSTANT_Double as defined in §4.4.5"]
    #[br(magic = 6u8)]
    Double { value: f64 },
    #[doc = "CONSTANT_NameAndType as defined in §4.4.6"]
    #[br(magic = 12u8)]
    NameAndType {
        name_index: Utf8Index,
        descriptor_index: Utf8Index,
    },
    #[doc = "CONSTANT_Utf8 as defined in §4.4.7"]
    #[br(magic = 1u8)]
    Utf8 {
        #[br(temp)]
        length: u16,
        #[br(count = length, try_map = |x: Vec<u8>| String::from_utf8(x))]
        value: String,
    },
    #[doc = "CONSTANT_MethodHandle as defined in §4.4.8"]
    #[br(magic = 15u8)]
    MethodHandle(Reference),
    #[doc = "CONSTANT_MethodType as defined in §4.4.9"]
    #[br(magic = 16u8)]
    MethodType { descriptor_index: Utf8Index },
    #[doc = "CONSTANT_Dynamic as defined in §4.4.10"]
    #[br(magic = 17u8)]
    Dynamic {
        bootstrap_method_attr_index: BootstrapMethodAttrInfo,
        name_and_type_index: NameAndTypeIndex,
    },
    #[doc = "CONSTANT_InvokeDynamic as defined in §4.4.10"]
    #[br(magic = 18u8)]
    InvokeDynamic {
        bootstrap_method_attr_index: BootstrapMethodAttrInfo,
        name_and_type_index: NameAndTypeIndex,
    },
    #[doc = "CONSTANT_Module as defined in §4.4.11"]
    #[br(magic = 19u8)]
    Module { name_index: Utf8Index },
    #[doc = "CONSTANT_Package as defined in §4.4.12"]
    #[br(magic = 20u8)]
    Package { name_index: Utf8Index },

    #[doc = "Catch all. Used after Double and Long to give a constant off-by-one for constant pool indexing."]
    Skip,
}

macro_rules! index_ty {
    ($name:ident { $cpool:ident, $($inner:ident),* } => { $($t:tt)* }) => {
        paste::paste! {
            #[binread]
            #[derive(Debug)]
            struct [<$name Index>] (u16);

            impl [<$name Index>] {
                pub fn get_as_string<'a>(&self, class: &'a ClassFile) -> Result<&'a str> {
                    self.get_as_string_impl(&class.constant_pool)
                }

                fn get_as_string_impl<'a>(&self, $cpool: &'a ConstantPool) -> Result<&'a str> {
                    match &$cpool.0[self.0 as usize - 1] {
                        ConstantPoolItem::$name { $($inner),* } => {
                            Ok($($t)*)
                        }
                        x => Err(Error::ConstantPoolError(format!("expected {}, found {:?}", stringify!($name), x)))
                    }
                }

                fn debug(&self, cpool: &ConstantPool) -> String {
                    match self.get_as_string_impl(cpool) {
                        Ok(x) => format!("{}: {}", stringify!($name), x),
                        Err(x) => format!("{:?}", x),
                    }
                }
            }
        }
    };
}

index_ty!(Utf8 { cpool, value } => { value.as_str() });
index_ty!(Class { cpool, name_index } => { name_index.get_as_string_impl(cpool)? });
index_ty!(NameAndType { cpool, name_index, descriptor_index } => { name_index.get_as_string_impl(cpool)? });

#[binread]
#[derive(Debug)]
struct BootstrapMethodAttrInfo(u16);

#[binread]
#[derive(Debug)]
struct Reference {
    _kind: u8,
    _index: u16,
}

bitflags::bitflags! {
    #[derive(Debug)]
    struct FieldAccessFlags: u16 {
        const PUBLIC    = 0x0001;
        const PRIVATE   = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC    = 0x0008;
        const FINAL     = 0x0010;
        const VOLATILE  = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM      = 0x4000;
    }
}

#[binread]
#[br(import(cpool: &ConstantPool))]
#[derive(Debug)]
struct FieldRaw {
    #[br(map = |x: u16| FieldAccessFlags::from_bits_truncate(x))]
    access_flags: FieldAccessFlags,
    name_index: Utf8Index,
    descriptor_index: Utf8Index,
    #[br(args(cpool,))]
    attributes: Attributes,
}

impl FieldRaw {
    fn debug(&self, cpool: &ConstantPool) -> String {
        format!(
            "{}:{} ({:?})",
            self.name_index.get_as_string_impl(cpool).unwrap_or("Err"),
            self.descriptor_index
                .get_as_string_impl(cpool)
                .unwrap_or("Err"),
            self.access_flags
        )
    }
}

bitflags::bitflags! {
    #[derive(Debug)]
    struct MethodAccessFlags: u16 {
        const PUBLIC       = 0x0001;
        const PRIVATE      = 0x0002;
        const PROTECTED    = 0x0004;
        const STATIC       = 0x0008;
        const FINAL        = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE       = 0x0040;
        const VARARGS      = 0x0080;
        const NATIVE       = 0x0100;
        const ABSTRACT     = 0x4000;
        const STRICT       = 0x8000;
        const SYNTHETIC    = 0x1000;
    }
}

#[binread]
#[br(import(cpool: &ConstantPool))]
#[derive(Debug)]
struct MethodRaw {
    #[br(map = |x: u16| MethodAccessFlags::from_bits_truncate(x))]
    access_flags: MethodAccessFlags,
    name_index: Utf8Index,
    descriptor_index: Utf8Index,
    #[br(args(cpool,))]
    attributes: Attributes,
}

impl MethodRaw {
    fn debug(&self, cpool: &ConstantPool) -> String {
        format!(
            "{}:{} ({:?})",
            self.name_index.get_as_string_impl(cpool).unwrap_or("Err"),
            self.descriptor_index
                .get_as_string_impl(cpool)
                .unwrap_or("Err"),
            self.access_flags
        )
    }
}

#[derive(Debug)]
struct Attributes(HashMap<String, Vec<u8>>);

impl BinRead for Attributes {
    type Args<'a> = (&'a ConstantPool,);

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        (cpool,): Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let attributes_count = u16::read_be(reader)?;
        let attributes = (0..attributes_count)
            .map(|_| {
                let attribute_name = Utf8Index::read_be(reader)?
                    .get_as_string_impl(cpool)
                    .unwrap_or("");
                let attribute_len = u32::read_be(reader)?;
                let mut info = vec![0u8; attribute_len as usize];
                reader.read_exact(&mut info)?;
                Ok::<(String, Vec<u8>), binrw::Error>((attribute_name.to_string(), info))
            })
            .collect::<std::result::Result<HashMap<_, _>, _>>()?;
        Ok(Self(attributes))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Constant Pool")]
    ConstantPoolError(String),
    #[error("Binary Parsing Error")]
    BinrwError(#[from] binrw::Error),
    #[error("Text Processing Error")]
    NomError(nom::Err<nom::error::Error<String>>),
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for Error {
    fn from(value: nom::Err<nom::error::Error<&'a str>>) -> Self {
        match value {
            nom::Err::Incomplete(x) => Self::NomError(nom::Err::Incomplete(x)),
            nom::Err::Error(x) => Self::NomError(nom::Err::Error(nom::error::Error::new(
                x.input.to_string(),
                x.code,
            ))),
            nom::Err::Failure(x) => Self::NomError(nom::Err::Failure(nom::error::Error::new(
                x.input.to_string(),
                x.code,
            ))),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

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

    fn parse(input: &'a str) -> IResult<&'a str, Self> {
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
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
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

pub struct Method<'a> {
    class_file: &'a ClassFile,
    method_inner: &'a MethodRaw,
}

impl<'a> Method<'a> {
    pub fn identifier(&self) -> Result<&'a str> {
        self.method_inner.name_index.get_as_string(self.class_file)
    }

    pub fn descriptor(&self) -> Result<MethodDescriptor<'a>> {
        let raw_descriptor = self
            .method_inner
            .descriptor_index
            .get_as_string(self.class_file)?;
        let (_input, ty) = MethodDescriptor::parse(raw_descriptor)?;
        Ok(ty)
    }

    pub fn jni_identifier(&self) -> Result<String> {
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
}

impl<'a> Debug for Method<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Method")
            .field("identifier", &self.identifier())
            .field("desciptor", &self.descriptor())
            .finish()
    }
}

pub struct Field<'a> {
    class_file: &'a ClassFile,
    field_inner: &'a FieldRaw,
}

impl<'a> Field<'a> {
    pub fn identifier(&self) -> Result<&'a str> {
        self.field_inner.name_index.get_as_string(self.class_file)
    }

    pub fn descriptor(&self) -> Result<TypeDescriptor<'a>> {
        let raw_descriptor = self
            .field_inner
            .descriptor_index
            .get_as_string(self.class_file)?;
        let (_input, ty) = TypeDescriptor::parse(raw_descriptor)?;
        Ok(ty)
    }

    pub fn constant_value(&self) -> Result<Option<ConstantValue<'a>>> {
        match self.field_inner.attributes.0.get("ConstantValue") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = ConstantValue::read_be_args(&mut buf, (self.class_file,))?;
                Ok(Some(value))
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

pub struct ConstantValue<'a> {
    class_file: &'a ClassFile,
    constantvalue_index: u16,
}

impl<'a> BinRead for ConstantValue<'a> {
    type Args<'b> = (&'a ClassFile,);

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        (class_file,): Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let constantvalue_index = u16::read_options(reader, endian, ())?;
        Ok(Self {
            class_file,
            constantvalue_index,
        })
    }
}

impl<'a> ConstantValue<'a> {
    pub fn int_value(&self) -> Result<i32> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::Integer { value } => Ok(*value),
            x => Err(Error::ConstantPoolError(format!(
                "Expected Integer, instead got {:?}",
                x
            ))),
        }
    }

    pub fn float_value(&self) -> Result<f32> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::Float { value } => Ok(*value),
            x => Err(Error::ConstantPoolError(format!(
                "Expected Float, instead got {:?}",
                x
            ))),
        }
    }

    pub fn long_value(&self) -> Result<i64> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::Long { value } => Ok(*value),
            x => Err(Error::ConstantPoolError(format!(
                "Expected Long, instead got {:?}",
                x
            ))),
        }
    }

    pub fn double_value(&self) -> Result<f64> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::Double { value } => Ok(*value),
            x => Err(Error::ConstantPoolError(format!(
                "Expected Double, instead got {:?}",
                x
            ))),
        }
    }

    pub fn string_value(&self) -> Result<&'a str> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::String { string_index } => {
                string_index.get_as_string(self.class_file)
            }
            x => Err(Error::ConstantPoolError(format!(
                "Expected String, instead got {:?}",
                x
            ))),
        }
    }
}

pub struct Exception<'a> {
    class_file: &'a ClassFile,
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: Option<ClassIndex>,
}

impl<'a> BinRead for Exception<'a> {
    type Args<'b> = (&'a ClassFile,);

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        (class_file,): Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let start_pc = u16::read_options(reader, endian, ())?;
        let end_pc = u16::read_options(reader, endian, ())?;
        let handler_pc = u16::read_options(reader, endian, ())?;
        let catch_type = u16::read_options(reader, endian, ())?;
        let catch_type = if catch_type == 0 {
            None
        } else {
            Some(ClassIndex(catch_type))
        };
        Ok(Self {
            class_file,
            start_pc,
            end_pc,
            handler_pc,
            catch_type
        })
    }
}

pub struct Code<'a> {
    class_file: &'a ClassFile,
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
    exception_table: Vec<Exception<'a>>,
    attributes: Attributes,
}
