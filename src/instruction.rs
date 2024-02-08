use binrw::{binread, BinRead};

use crate::{
    attributes::BootstrapMethod, field::TypeDescriptor, method::MethodDescriptor, raw::ConstantPoolItem, ClassFile, ClassIndex
};

#[derive(Debug)]
pub struct FieldRef<'a> {
    pub class: &'a str,
    pub name: &'a str,
    pub descriptor: TypeDescriptor<'a>,
}

#[derive(Debug)]
pub struct MethodRef<'a> {
    pub class: &'a str,
    pub name: &'a str,
    pub descriptor: MethodDescriptor<'a>,
}

#[derive(Debug)]
pub struct InterfaceMethodRef<'a> {
    pub class: &'a str,
    pub name: &'a str,
    pub descriptor: MethodDescriptor<'a>,
}

impl<'a> FieldRef<'a> {
    fn from_u16(index: u16, cf: &'a ClassFile) -> super::Result<Self> {
        match &cf.constant_pool.0[index as usize - 1] {
            ConstantPoolItem::Fieldref {
                class_index,
                name_and_type_index,
            } => {
                let class = class_index.get_as_string(cf)?;
                let name = name_and_type_index.get_name(cf)?;
                let descriptor = name_and_type_index.get_descriptor(cf)?;
                let descriptor = TypeDescriptor::parse(descriptor)?.1;
                Ok(Self {
                    class,
                    name,
                    descriptor,
                })
            }
            x => Err(super::Error::ConstantPoolError(format!(
                "expected FieldRef at constant pool index {}. Instead found {:?}.",
                index, x
            ))),
        }
    }
}

impl<'a> MethodRef<'a> {
    fn from_u16(index: u16, cf: &'a ClassFile) -> super::Result<Self> {
        match &cf.constant_pool.0[index as usize - 1] {
            ConstantPoolItem::Methodref {
                class_index,
                name_and_type_index,
            } => {
                let class = class_index.get_as_string(cf)?;
                let name = name_and_type_index.get_name(cf)?;
                let descriptor = name_and_type_index.get_descriptor(cf)?;
                let descriptor = MethodDescriptor::parse(descriptor)?.1;
                Ok(Self {
                    class,
                    name,
                    descriptor,
                })
            }
            x => Err(super::Error::ConstantPoolError(format!(
                "expected MethodRef at constant pool index {}. Instead found {:?}.",
                index, x
            ))),
        }
    }
}

impl<'a> InterfaceMethodRef<'a> {
    fn from_u16(index: u16, cf: &'a ClassFile) -> super::Result<Self> {
        match &cf.constant_pool.0[index as usize - 1] {
            ConstantPoolItem::InterfaceMethodref {
                class_index,
                name_and_type_index,
            } => {
                let class = class_index.get_as_string(cf)?;
                let name = name_and_type_index.get_name(cf)?;
                let descriptor = name_and_type_index.get_descriptor(cf)?;
                let descriptor = MethodDescriptor::parse(descriptor)?.1;
                Ok(Self {
                    class,
                    name,
                    descriptor,
                })
            }
            x => Err(super::Error::ConstantPoolError(format!(
                "expected InterfaceMethodRef at constant pool index {}. Instead found {:?}.",
                index, x
            ))),
        }
    }
}

macro_rules! from_u16_binread {
    ($class:ident) => {
        impl<'a> BinRead for $class<'a> {
            type Args<'b> = (&'a ClassFile,);

            fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
                reader: &mut R,
                endian: binrw::Endian,
                (cf,): Self::Args<'_>,
            ) -> binrw::prelude::BinResult<Self> {
                let pos = reader.stream_position()?;
                let index = u16::read_options(reader, endian, ())?;
                Self::from_u16(index, cf).map_err(|x| binrw::Error::Custom {
                    pos,
                    err: Box::new(x),
                })
            }
        }
    };
}

#[derive(Debug)]
pub enum MaybeInterfaceMethodRef<'a> {
    RegularMethod(MethodRef<'a>),
    InterfaceMethod(InterfaceMethodRef<'a>),
}

impl<'a> MaybeInterfaceMethodRef<'a> {
    fn from_u16(index: u16, cf: &'a ClassFile) -> super::Result<Self> {
        match &cf.constant_pool.0[index as usize - 1] {
            ConstantPoolItem::InterfaceMethodref { .. } => {
                Ok(Self::InterfaceMethod(InterfaceMethodRef::from_u16(index, cf)?))
            }
            ConstantPoolItem::Methodref { .. } => {
                Ok(Self::RegularMethod(MethodRef::from_u16(index, cf)?))
            }
            x => Err(super::Error::ConstantPoolError(format!(
                "expected MethodRef or InterfaceMethodRef at constant pool index {}. Instead found {:?}.",
                index, x
            ))),
        }
    }
}

#[derive(Debug)]
pub enum MethodHandle<'a> {
    GetField(FieldRef<'a>),
    GetStatic(FieldRef<'a>),
    PutField(FieldRef<'a>),
    PutStatic(FieldRef<'a>),
    InvokeVirtual(MethodRef<'a>),
    NewInvokeSpecial(MethodRef<'a>),
    InvokeStatic(MaybeInterfaceMethodRef<'a>),
    InvokeSpecial(MaybeInterfaceMethodRef<'a>),
    InvokeInterface(InterfaceMethodRef<'a>),
}

impl<'a> MethodHandle<'a> {
    pub(crate) fn from_u16(index: u16, cf: &'a ClassFile) -> super::Result<Self> {
        match &cf.constant_pool.0[index as usize - 1] {
            ConstantPoolItem::MethodHandle { reference } => {
                match reference.kind {
                    1 => Ok(Self::GetField(FieldRef::from_u16(reference.index, cf)?)),
                    2 => Ok(Self::GetStatic(FieldRef::from_u16(reference.index, cf)?)),
                    3 => Ok(Self::PutField(FieldRef::from_u16(reference.index, cf)?)),
                    4 => Ok(Self::PutStatic(FieldRef::from_u16(reference.index, cf)?)),
                    5 => Ok(Self::InvokeVirtual(MethodRef::from_u16(reference.index, cf)?)),
                    8 => Ok(Self::NewInvokeSpecial(MethodRef::from_u16(reference.index, cf)?)),
                    6 => Ok(Self::InvokeStatic(MaybeInterfaceMethodRef::from_u16(reference.index, cf)?)),
                    7 => Ok(Self::InvokeSpecial(MaybeInterfaceMethodRef::from_u16(reference.index, cf)?)),
                    9 => Ok(Self::InvokeInterface(InterfaceMethodRef::from_u16(reference.index, cf)?)),
                    x => Err(super::Error::ConstantPoolError(format!("invalid reference_kind {}.", x)))
                }
            }
            x => Err(super::Error::ConstantPoolError(format!(
                "expected MethodRef or InterfaceMethodRef at constant pool index {}. Instead found {:?}.",
                index, x
            ))),
        }
    }
}

#[derive(Debug)]
pub struct DynamicInfo<'a> {
    pub bootstrap_method: BootstrapMethod<'a>,
    pub name: &'a str,
    pub descriptor: MethodDescriptor<'a>,
}

impl<'a> DynamicInfo<'a> {
    fn from_u16(index: u16, cf: &'a ClassFile) -> super::Result<Self> {
        match &cf.constant_pool.0[index as usize - 1] {
            ConstantPoolItem::InvokeDynamic {
                bootstrap_method_attr_index,
                name_and_type_index,
            } => {
                let bootstrap_methods = cf.bootstrap_methods()?.ok_or_else(|| super::Error::NoBootstrapMethods)?;
                let bootstrap_method = bootstrap_methods.get(bootstrap_method_attr_index.0)?.ok_or_else(|| super::Error::InvalidBootstrapIndex(bootstrap_method_attr_index.0))?;

                let name = name_and_type_index.get_name(cf)?;
                let descriptor =
                    MethodDescriptor::parse(name_and_type_index.get_descriptor(cf)?)?.1;
                Ok(Self { bootstrap_method, name, descriptor })
            }
            x => Err(super::Error::ConstantPoolError(format!(
                "expected InvokeDynamic at constant pool index {}. Instead found {:?}.",
                index, x
            ))),
        }
    }
}

from_u16_binread!(FieldRef);
from_u16_binread!(MethodRef);
from_u16_binread!(InterfaceMethodRef);
from_u16_binread!(MaybeInterfaceMethodRef);
from_u16_binread!(MethodHandle);
from_u16_binread!(DynamicInfo);

#[derive(Debug)]
pub struct BytePad;

impl BinRead for BytePad {
    type Args<'a> = ();

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let pos = reader.stream_position()?;
        let d4 = pos % 4;
        if d4 == 0 {
            return Ok(Self);
        }
        let skip = 4 - d4;
        reader.seek(std::io::SeekFrom::Current(skip as i64))?;
        Ok(Self)
    }
}

#[binread]
#[br(import(cf: &'a ClassFile,))]
#[derive(Debug)]
pub enum Instruction<'a> {
    #[br(magic = 0x32u8)]
    Aaload,
    #[br(magic = 0x53u8)]
    Aastore,
    #[br(magic = 0x1u8)]
    AconstNull,
    #[br(magic = 0x19u8)]
    Aload { index: u8 },
    #[br(magic = 0x2au8)]
    Aload0,
    #[br(magic = 0x2bu8)]
    Aload1,
    #[br(magic = 0x2cu8)]
    Aload2,
    #[br(magic = 0x2du8)]
    Aload3,
    #[br(magic = 0xbdu8)]
    Anewarray {
        #[br(try_map = |x: ClassIndex| x.get_as_string(cf))]
        class: &'a str,
    },
    #[br(magic = 0xb0u8)]
    Areturn,
    #[br(magic = 0xbeu8)]
    Arraylength,
    #[br(magic = 0x3au8)]
    Astore { index: u8 },
    #[br(magic = 0x4bu8)]
    Astore0,
    #[br(magic = 0x4cu8)]
    Astore1,
    #[br(magic = 0x4du8)]
    Astore2,
    #[br(magic = 0x4eu8)]
    Astore3,
    #[br(magic = 0xbfu8)]
    Athrow,
    #[br(magic = 0x33u8)]
    Baload,
    #[br(magic = 0x54u8)]
    Bastore,
    #[br(magic = 0x10u8)]
    Bipush { byte: i8 },
    #[br(magic = 0x34u8)]
    Caload,
    #[br(magic = 0x55u8)]
    Castore,
    #[br(magic = 0xc0u8)]
    Checkcast {
        #[br(try_map = |x: ClassIndex| x.get_as_string(cf))]
        class: &'a str,
    },
    #[br(magic = 0x90u8)]
    D2f,
    #[br(magic = 0x8eu8)]
    D2i,
    #[br(magic = 0x8fu8)]
    D2l,
    #[br(magic = 0x63u8)]
    Dadd,
    #[br(magic = 0x31u8)]
    Daload,
    #[br(magic = 0x52u8)]
    Dastore,
    #[br(magic = 0x98u8)]
    Dcmpg,
    #[br(magic = 0x97u8)]
    Dcmpl,
    #[br(magic = 0xeu8)]
    Dconst0,
    #[br(magic = 0xfu8)]
    Dconst1,
    #[br(magic = 0x6fu8)]
    Ddiv,
    #[br(magic = 0x18u8)]
    Dload { index: u8 },
    #[br(magic = 0x26u8)]
    Dload0,
    #[br(magic = 0x27u8)]
    Dload1,
    #[br(magic = 0x28u8)]
    Dload2,
    #[br(magic = 0x29u8)]
    Dload3,
    #[br(magic = 0x6bu8)]
    Dmul,
    #[br(magic = 0x77u8)]
    Dneg,
    #[br(magic = 0x73u8)]
    Drem,
    #[br(magic = 0xafu8)]
    Dreturn,
    #[br(magic = 0x39u8)]
    Dstore { index: u8 },
    #[br(magic = 0x47u8)]
    Dstore0,
    #[br(magic = 0x48u8)]
    Dstore1,
    #[br(magic = 0x49u8)]
    Dstore2,
    #[br(magic = 0x4au8)]
    Dstore3,
    #[br(magic = 0x67u8)]
    Dsub,
    #[br(magic = 0x59u8)]
    Dup,
    #[br(magic = 0x5au8)]
    DupX1,
    #[br(magic = 0x5bu8)]
    DupX2,
    #[br(magic = 0x5cu8)]
    Dup2,
    #[br(magic = 0x5du8)]
    Dup2X1,
    #[br(magic = 0x5eu8)]
    Dup2X2,
    #[br(magic = 0x8du8)]
    F2d,
    #[br(magic = 0x8bu8)]
    F2i,
    #[br(magic = 0x8cu8)]
    F2l,
    #[br(magic = 0x62u8)]
    Fadd,
    #[br(magic = 0x30u8)]
    Faload,
    #[br(magic = 0x51u8)]
    Fastore,
    #[br(magic = 0x96u8)]
    Fcmpg,
    #[br(magic = 0x95u8)]
    Fcmpl,
    #[br(magic = 0xbu8)]
    Fconst0,
    #[br(magic = 0xcu8)]
    Fconst1,
    #[br(magic = 0xdu8)]
    Fconst2,
    #[br(magic = 0x6eu8)]
    Fdiv,
    #[br(magic = 0x17u8)]
    Fload { index: u8 },
    #[br(magic = 0x22u8)]
    Fload0,
    #[br(magic = 0x23u8)]
    Fload1,
    #[br(magic = 0x24u8)]
    Fload2,
    #[br(magic = 0x25u8)]
    Fload3,
    #[br(magic = 0x6au8)]
    Fmul,
    #[br(magic = 0x76u8)]
    Fneg,
    #[br(magic = 0x72u8)]
    Frem,
    #[br(magic = 0xaeu8)]
    Freturn,
    #[br(magic = 0x38u8)]
    Fstore { index: u8 },
    #[br(magic = 0x43u8)]
    Fstore0,
    #[br(magic = 0x44u8)]
    Fstore1,
    #[br(magic = 0x45u8)]
    Fstore2,
    #[br(magic = 0x46u8)]
    Fstore3,
    #[br(magic = 0x66u8)]
    Fsub,
    #[br(magic = 0xb4u8)]
    Getfield {
        #[br(args(cf,))]
        field: FieldRef<'a>,
    },
    #[br(magic = 0xb2u8)]
    Getstatic {
        #[br(args(cf,))]
        field: FieldRef<'a>,
    },
    #[br(magic = 0xa7u8)]
    Goto { offset: i16 },
    #[br(magic = 0xc8u8)]
    GotoW { offset: i32 },
    #[br(magic = 0x91u8)]
    I2b,
    #[br(magic = 0x92u8)]
    I2c,
    #[br(magic = 0x87u8)]
    I2d,
    #[br(magic = 0x86u8)]
    I2f,
    #[br(magic = 0x85u8)]
    I2l,
    #[br(magic = 0x93u8)]
    I2s,
    #[br(magic = 0x60u8)]
    Iadd,
    #[br(magic = 0x2eu8)]
    Iaload,
    #[br(magic = 0x7eu8)]
    Iand,
    #[br(magic = 0x4fu8)]
    Iastore,
    #[br(magic = 0x2u8)]
    IconstM1,
    #[br(magic = 0x3u8)]
    Iconst0,
    #[br(magic = 0x4u8)]
    Iconst1,
    #[br(magic = 0x5u8)]
    Iconst2,
    #[br(magic = 0x6u8)]
    Iconst3,
    #[br(magic = 0x7u8)]
    Iconst4,
    #[br(magic = 0x8u8)]
    Iconst5,
    #[br(magic = 0x6cu8)]
    Idiv,
    #[br(magic = 0xa5u8)]
    IfAcmpeq { offset: i16 },
    #[br(magic = 0xa6u8)]
    IfAcmpne { offset: i16 },
    #[br(magic = 0x9fu8)]
    IfIcmpeq { offset: i16 },
    #[br(magic = 0xa0u8)]
    IfIcmpne { offset: i16 },
    #[br(magic = 0xa1u8)]
    IfIcmplt { offset: i16 },
    #[br(magic = 0xa2u8)]
    IfIcmpge { offset: i16 },
    #[br(magic = 0xa3u8)]
    IfIcmpgt { offset: i16 },
    #[br(magic = 0xa4u8)]
    IfIcmple { offset: i16 },
    #[br(magic = 0x99u8)]
    Ifeq { offset: i16 },
    #[br(magic = 0x9au8)]
    Ifne { offset: i16 },
    #[br(magic = 0x9bu8)]
    Iflt { offset: i16 },
    #[br(magic = 0x9cu8)]
    Ifge { offset: i16 },
    #[br(magic = 0x9du8)]
    Ifgt { offset: i16 },
    #[br(magic = 0x9eu8)]
    Ifle { offset: i16 },
    #[br(magic = 0xc7u8)]
    Ifnonnull { offset: i16 },
    #[br(magic = 0xc6u8)]
    Ifnull { offset: i16 },
    #[br(magic = 0x84u8)]
    Iinc { index: u8, constant: i8 },
    #[br(magic = 0x15u8)]
    Iload { index: u8 },
    #[br(magic = 0x1au8)]
    Iload0,
    #[br(magic = 0x1bu8)]
    Iload1,
    #[br(magic = 0x1cu8)]
    Iload2,
    #[br(magic = 0x1du8)]
    Iload3,
    #[br(magic = 0x68u8)]
    Imul,
    #[br(magic = 0x74u8)]
    Ineg,
    #[br(magic = 0xc1u8)]
    Instanceof {
        #[br(try_map = |x: ClassIndex| x.get_as_string(cf))]
        class: &'a str,
    },
    #[br(magic = 0xbau8)]
    Invokedynamic {
        #[br(args(cf,))]
        index: DynamicInfo<'a>,
        _never_used: u16, // IDK why, but the spec says this is followed by two 0x00 bytes.
    },
    #[br(magic = 0xb9u8)]
    Invokeinterface {
        #[br(args(cf,))]
        index: InterfaceMethodRef<'a>,
        count: u8,
        _never_used: u16, // IDK why, but the spec says this is followed by one 0x00 byte.
    },
    #[br(magic = 0xb7u8)]
    Invokespecial {
        #[br(args(cf,))]
        index: MaybeInterfaceMethodRef<'a>,
    },
    #[br(magic = 0xb8u8)]
    Invokestatic {
        #[br(args(cf,))]
        index: MaybeInterfaceMethodRef<'a>,
    },
    #[br(magic = 0xb6u8)]
    Invokevirtual {
        #[br(args(cf,))]
        index: MethodRef<'a>,
    },
    #[br(magic = 0x80u8)]
    Ior,
    #[br(magic = 0x70u8)]
    Irem,
    #[br(magic = 0xacu8)]
    Ireturn,
    #[br(magic = 0x78u8)]
    Ishl,
    #[br(magic = 0x7au8)]
    Ishr,
    #[br(magic = 0x36u8)]
    Istore { index: u8 },
    #[br(magic = 0x3bu8)]
    Istore0,
    #[br(magic = 0x3cu8)]
    Istore1,
    #[br(magic = 0x3du8)]
    Istore2,
    #[br(magic = 0x3eu8)]
    Istore3,
    #[br(magic = 0x64u8)]
    Isub,
    #[br(magic = 0x7cu8)]
    Iushr,
    #[br(magic = 0x82u8)]
    Ixor,
    #[br(magic = 0xa8u8)]
    Jsr { offset: i16 },
    #[br(magic = 0xc9u8)]
    JsrW { offset: i32 },
    #[br(magic = 0x8au8)]
    L2d,
    #[br(magic = 0x89u8)]
    L2f,
    #[br(magic = 0x88u8)]
    L2i,
    #[br(magic = 0x61u8)]
    Ladd,
    #[br(magic = 0x2fu8)]
    Laload,
    #[br(magic = 0x7fu8)]
    Land,
    #[br(magic = 0x50u8)]
    Lastore,
    #[br(magic = 0x94u8)]
    Lcmp,
    #[br(magic = 0x9u8)]
    Lconst0,
    #[br(magic = 0xau8)]
    Lconst1,
    #[br(magic = 0x12u8)]
    Ldc { index: u8 },
    #[br(magic = 0x13u8)]
    LdcW { index: u16 },
    #[br(magic = 0x14u8)]
    Ldc2W { index: u16 },
    #[br(magic = 0x6du8)]
    Ldiv,
    #[br(magic = 0x16u8)]
    Lload { index: u8 },
    #[br(magic = 0x1eu8)]
    Lload0,
    #[br(magic = 0x1fu8)]
    Lload1,
    #[br(magic = 0x20u8)]
    Lload2,
    #[br(magic = 0x21u8)]
    Lload3,
    #[br(magic = 0x69u8)]
    Lmul,
    #[br(magic = 0x75u8)]
    Lneg,
    #[br(magic = 0xabu8)]
    Lookupswitch {
        _padding: BytePad,
        default: i32,
        #[br(temp)]
        npairs: u32,
        #[br(count = npairs)]
        pairs: Vec<(i32, i32)>,
    },
    #[br(magic = 0x81u8)]
    Lor,
    #[br(magic = 0x71u8)]
    Lrem,
    #[br(magic = 0xadu8)]
    Lreturn,
    #[br(magic = 0x79u8)]
    Lshl,
    #[br(magic = 0x7bu8)]
    Lshr,
    #[br(magic = 0x37u8)]
    Lstore { index: u8 },
    #[br(magic = 0x3fu8)]
    Lstore0,
    #[br(magic = 0x40u8)]
    Lstore1,
    #[br(magic = 0x41u8)]
    Lstore2,
    #[br(magic = 0x42u8)]
    Lstore3,
    #[br(magic = 0x65u8)]
    Lsub,
    #[br(magic = 0x7du8)]
    Lushr,
    #[br(magic = 0x83u8)]
    Lxor,
    #[br(magic = 0xc2u8)]
    Monitorenter,
    #[br(magic = 0xc3u8)]
    Monitorexit,
    #[br(magic = 0xc5u8)]
    Multianewarray {
        #[br(try_map = |x: ClassIndex| x.get_as_string(cf))]
        class: &'a str,
        dimensions: u8,
    },
    #[br(magic = 0xbbu8)]
    New {
        #[br(try_map = |x: ClassIndex| x.get_as_string(cf))]
        class: &'a str,
    },
    #[br(magic = 0xbcu8)]
    Newarray { atype: u8 },
    #[br(magic = 0x0u8)]
    Nop,
    #[br(magic = 0x57u8)]
    Pop,
    #[br(magic = 0x58u8)]
    Pop2,
    #[br(magic = 0xb5u8)]
    Putfield {
        #[br(args(cf,))]
        field: FieldRef<'a>,
    },
    #[br(magic = 0xb3u8)]
    Putstatic {
        #[br(args(cf,))]
        field: FieldRef<'a>,
    },
    #[br(magic = 0xa9u8)]
    Ret { index: u8 },
    #[br(magic = 0xb1u8)]
    Return,
    #[br(magic = 0x35u8)]
    Saload,
    #[br(magic = 0x56u8)]
    Sastore,
    #[br(magic = 0x11u8)]
    Sipush {
        #[br(map = |x: u16| x as i32)]
        value: i32,
    },
    #[br(magic = 0x5fu8)]
    Swap,
    #[br(magic = 0xaau8)]
    Tableswitch {
        _padding: BytePad,
        default: i32,
        low: i32,
        high: i32,
        #[br(count = high - low + 1)]
        jump_offsets: Vec<i32>,
    },
    #[br(magic = 0xc4u8)]
    Wide {
        opcode: u8,
        index: u16,
        #[br(if(opcode == 0x84u8))]
        constant: u16,
    },
}
