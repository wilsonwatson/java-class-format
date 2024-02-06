use std::{collections::HashMap, fmt::Debug};

use binrw::{binread, BinRead};

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct ClassAccessFlags: u16 {
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

#[derive(Debug)]
pub struct ConstantPool(pub Vec<ConstantPoolItem>);

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
pub enum ConstantPoolItem {
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
    MethodHandle { reference: Reference },
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
            pub struct [<$name Index>] (pub u16);

            impl [<$name Index>] {
                pub fn get_as_string<'a>(&self, class: &'a super::ClassFile) -> super::Result<&'a str> {
                    self.get_as_string_impl(&class.constant_pool)
                }

                pub(crate) fn get_as_string_impl<'a>(&self, $cpool: &'a ConstantPool) -> super::Result<&'a str> {
                    match &$cpool.0[self.0 as usize - 1] {
                        ConstantPoolItem::$name { $($inner),* } => {
                            Ok($($t)*)
                        }
                        x => Err(super::Error::ConstantPoolError(format!("expected {}, found {:?}", stringify!($name), x)))
                    }
                }
            }
        }
    };
}

index_ty!(Utf8 { cpool, value } => { value.as_str() });
index_ty!(Class { cpool, name_index } => { name_index.get_as_string_impl(cpool)? });
index_ty!(NameAndType { cpool, name_index, descriptor_index } => { name_index.get_as_string_impl(cpool)? });
index_ty!(MethodHandle { cpool, reference } => { "" });

#[binread]
#[derive(Debug)]
pub struct BootstrapMethodAttrInfo(u16);

#[binread]
#[derive(Debug)]
pub struct Reference {
    pub kind: u8,
    pub index: u16,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct FieldAccessFlags: u16 {
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
pub struct FieldRaw {
    #[br(map = |x: u16| FieldAccessFlags::from_bits_truncate(x))]
    pub access_flags: FieldAccessFlags,
    pub name_index: Utf8Index,
    pub descriptor_index: Utf8Index,
    #[br(args(cpool,))]
    pub attributes: Attributes,
}

bitflags::bitflags! {
    #[derive(Debug)]
    pub struct MethodAccessFlags: u16 {
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
pub struct MethodRaw {
    #[br(map = |x: u16| MethodAccessFlags::from_bits_truncate(x))]
    pub access_flags: MethodAccessFlags,
    pub name_index: Utf8Index,
    pub descriptor_index: Utf8Index,
    #[br(args(cpool,))]
    pub attributes: Attributes,
}

pub struct Attributes(pub(crate) HashMap<String, Vec<u8>>);

impl Debug for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Attributes").field(&self.0.keys().collect::<Vec<_>>()).finish()
    }
}

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