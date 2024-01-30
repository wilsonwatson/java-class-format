use std::fmt::Debug;

use binrw::{binread, BinRead, VecArgs};

use crate::{
    raw::{Attributes, ClassIndex, ConstantPoolItem, NameAndTypeIndex, Utf8Index},
    ClassFile, Error,
};

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
    pub fn int_value(&self) -> crate::Result<i32> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::Integer { value } => Ok(*value),
            x => Err(Error::ConstantPoolError(format!(
                "Expected Integer, instead got {:?}",
                x
            ))),
        }
    }

    pub fn float_value(&self) -> crate::Result<f32> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::Float { value } => Ok(*value),
            x => Err(Error::ConstantPoolError(format!(
                "Expected Float, instead got {:?}",
                x
            ))),
        }
    }

    pub fn long_value(&self) -> crate::Result<i64> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::Long { value } => Ok(*value),
            x => Err(Error::ConstantPoolError(format!(
                "Expected Long, instead got {:?}",
                x
            ))),
        }
    }

    pub fn double_value(&self) -> crate::Result<f64> {
        match &self.class_file.constant_pool.0[self.constantvalue_index as usize - 1] {
            ConstantPoolItem::Double { value } => Ok(*value),
            x => Err(Error::ConstantPoolError(format!(
                "Expected Double, instead got {:?}",
                x
            ))),
        }
    }

    pub fn string_value(&self) -> crate::Result<&'a str> {
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

#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct Exception<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    #[br(map = |x: ClassIndex| { if x.0 == 0 { None } else { Some(x) } } )]
    catch_type: Option<ClassIndex>,
}

impl<'a> Debug for Exception<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Exception")
            .field("start_pc", &self.start_pc)
            .field("end_pc", &self.end_pc)
            .field("handler_pc", &self.handler_pc)
            .field(
                "catch_type",
                &self
                    .catch_type
                    .as_ref()
                    .map(|x| x.get_as_string_impl(&self.class_file.constant_pool)),
            )
            .finish()
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

impl<'a> Code<'a> {
    pub fn stack_map_table(&self) -> crate::Result<Option<StackMapTable<'a>>> {
        match self.attributes.0.get("StackMapTable") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = StackMapTable::read_be_args(&mut buf, (self.class_file,))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub fn line_number_table(&self) -> crate::Result<Option<LineNumberTable<'a>>> {
        match self.attributes.0.get("LineNumberTable") {
            Some(x) => {
                let mut buf = std::io::Cursor::new(&x[..]);
                let value = LineNumberTable::read_be_args(&mut buf, (self.class_file,))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

impl<'a> Debug for Code<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Code")
            .field("max_stack", &self.max_stack)
            .field("max_locals", &self.max_locals)
            .field("code", &"...")
            .field("exception_table", &self.exception_table)
            .field("attributes", &self.attributes)
            .finish()
    }
}

impl<'a> BinRead for Code<'a> {
    type Args<'b> = (&'a ClassFile,);

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        (cf,): Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let class_file = cf;
        let max_stack = u16::read_options(reader, endian, ())?;
        let max_locals = u16::read_options(reader, endian, ())?;
        let code_length = u32::read_options(reader, endian, ())?;
        let mut code = vec![0u8; code_length as usize];
        reader.read_exact(&mut code)?;
        let exception_table_length = u16::read_options(reader, endian, ())?;
        let mut exception_table = Vec::new();
        for _ in 0..exception_table_length {
            let exception = Exception::read_options(reader, endian, (cf,))?;
            exception_table.push(exception);
        }
        let attributes = Attributes::read_options(reader, endian, (&cf.constant_pool,))?;
        Ok(Self {
            class_file,
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        })
    }
}

#[binread]
enum VerificationTypeInfo {
    #[br(magic = 0u8)]
    Top,
    #[br(magic = 1u8)]
    Integer,
    #[br(magic = 2u8)]
    Float,
    #[br(magic = 4u8)]
    Long,
    #[br(magic = 3u8)]
    Double,
    #[br(magic = 5u8)]
    Null,
    #[br(magic = 6u8)]
    UninitializedThis,
    #[br(magic = 7u8)]
    Object { cpool_index: crate::raw::ClassIndex },
    #[br(magic = 8u8)]
    Uninitialized { offset: u16 },
}

enum StackMapFrame {
    SameFrame {
        offset_delta: u16,
    },
    SameLocals1StackItemFrame {
        offset_delta: u16,
        stack: [VerificationTypeInfo; 1],
    },
    ChopFrame {
        locals_to_remove: u8,
        offset_delta: u16,
    },
    AppendFrame {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    },
    FullFrame {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
        stack: Vec<VerificationTypeInfo>,
    },
}

impl BinRead for StackMapFrame {
    type Args<'a> = ();

    fn read_options<R: std::io::prelude::Read + std::io::prelude::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::prelude::BinResult<Self> {
        let pos = reader.stream_position()?;
        let magic = u8::read_options(reader, endian, ())?;
        match magic {
            0u8..=63u8 => Ok(Self::SameFrame {
                offset_delta: magic as u16,
            }),
            64u8..=127u8 => {
                let stack = <[VerificationTypeInfo; 1]>::read_options(reader, endian, ())?;
                Ok(Self::SameLocals1StackItemFrame {
                    offset_delta: magic as u16 - 64,
                    stack,
                })
            }
            128u8..=246u8 => {
                return Err(binrw::Error::BadMagic {
                    pos,
                    found: Box::new(magic),
                })
            }
            247u8 => {
                let offset_delta = u16::read_options(reader, endian, ())?;
                let stack = <[VerificationTypeInfo; 1]>::read_options(reader, endian, ())?;
                Ok(Self::SameLocals1StackItemFrame {
                    offset_delta,
                    stack,
                })
            }
            248u8..=250u8 => {
                let offset_delta = u16::read_options(reader, endian, ())?;
                Ok(Self::ChopFrame {
                    locals_to_remove: 251u8 - magic,
                    offset_delta,
                })
            }
            251u8 => {
                let offset_delta = u16::read_options(reader, endian, ())?;
                Ok(Self::SameFrame { offset_delta })
            }
            252u8..=254u8 => {
                let k = magic - 251;
                let offset_delta = u16::read_options(reader, endian, ())?;
                let locals = <Vec<VerificationTypeInfo>>::read_options(
                    reader,
                    endian,
                    VecArgs::builder().count(k as usize).finalize(),
                )?;
                Ok(Self::AppendFrame {
                    offset_delta,
                    locals,
                })
            }
            255u8 => {
                let offset_delta = u16::read_options(reader, endian, ())?;
                let num_locals = u16::read_options(reader, endian, ())?;
                let locals = <Vec<VerificationTypeInfo>>::read_options(
                    reader,
                    endian,
                    VecArgs::builder().count(num_locals as usize).finalize(),
                )?;
                let num_stack = u16::read_options(reader, endian, ())?;
                let stack = <Vec<VerificationTypeInfo>>::read_options(
                    reader,
                    endian,
                    VecArgs::builder().count(num_stack as usize).finalize(),
                )?;
                Ok(Self::FullFrame {
                    offset_delta,
                    locals,
                    stack,
                })
            }
        }
    }
}

#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct StackMapTable<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    #[br(temp)]
    number_of_entries: u16,
    #[br(count = number_of_entries)]
    entries: Vec<StackMapFrame>,
}

#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct Exceptions<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    #[br(temp)]
    number_of_exceptions: u16,
    #[br(count = number_of_exceptions)]
    exception_index_table: Vec<ClassIndex>,
}

impl<'a> Exceptions<'a> {
    pub fn class_names(&self) -> crate::Result<Vec<&'a str>> {
        self.exception_index_table
            .iter()
            .map(|x| x.get_as_string(&self.class_file))
            .collect()
    }
}

bitflags::bitflags! {
    #[derive(Debug)]
    struct InnerClassAccessFlags: u16 {
        #[doc = "Marked or implicitly public in source."]
        const PUBLIC = 0x0001;
        #[doc = "Marked private in source."]
        const PRIVATE = 0x0002;
        #[doc = "Marked protected in source."]
        const PROTECTED = 0x0004;
        #[doc = "Marked or implicitly static in source."]
        const STATIC = 0x0008;
        #[doc = "Marked or implicitly final in source."]
        const FINAL = 0x0010;
        #[doc = "Was an interface in source."]
        const INTERFACE = 0x0200;
        #[doc = "Marked or implicitly abstract in source."]
        const ABSTRACT = 0x0400;
        #[doc = "Declared synthetic; not present in the source code."]
        const SYNTHETIC = 0x1000;
        #[doc = "Declared as an annotation interface."]
        const ANNOTATION = 0x2000;
        #[doc = "Declared as an enum class. "]
        const ENUM = 0x4000;
    }
}

#[binread]
struct InnerClass {
    inner_class_info: ClassIndex,
    #[br(map = |x: ClassIndex| { if x.0 == 0 { None } else { Some(x) } } )]
    outer_class_info: Option<ClassIndex>,
    #[br(map = |x: Utf8Index| { if x.0 == 0 { None } else { Some(x) } } )]
    inner_name: Option<Utf8Index>,
    #[br(map = |x: u16| InnerClassAccessFlags::from_bits_truncate(x))]
    inner_class_access_flags: InnerClassAccessFlags,
}

#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct InnerClasses<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    #[br(temp)]
    number_of_classes: u16,
    #[br(count = number_of_classes)]
    classes: Vec<InnerClass>,
}

#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct EnclosingMethod<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    class_index: ClassIndex,
    method_index: NameAndTypeIndex,
}



#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct Signature<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    signature_index: Utf8Index,
}

impl<'a> Signature<'a> {
    pub(crate) fn get_class(&self) -> crate::Result<crate::signature::ClassSignature<'a>> {
        Ok(crate::signature::ClassSignature::parse(self.signature_index.get_as_string(self.class_file)?)?.1)
    }

    pub(crate) fn get_method(&self) -> crate::Result<crate::signature::MethodSignature<'a>> {
        Ok(crate::signature::MethodSignature::parse(self.signature_index.get_as_string(self.class_file)?)?.1)
    }

    pub(crate) fn get_field(&self) -> crate::Result<crate::signature::ReferenceType<'a>> {
        Ok(crate::signature::ReferenceType::parse(self.signature_index.get_as_string(self.class_file)?)?.1)
    }
}

#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct SourceFile<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    sourcefile_index: Utf8Index,
}

impl<'a> SourceFile<'a> {
    pub fn get(&self) -> crate::Result<&'a str> {
        self.sourcefile_index.get_as_string(self.class_file)
    }
}

#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct LineNumberTable<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    #[br(temp)]
    line_number_table_length: u16,
    #[br(count = line_number_table_length)]
    line_number_table: Vec<(u16, u16)>,
}

impl<'a> LineNumberTable<'a> {
    // TODO interact with code instructions.
}

pub struct LocalVariable<'a> {
    pub start_pc: u16,
    pub length: u16,
    pub name: &'a str,
    pub descriptor: crate::field::TypeDescriptor<'a>,
    pub index: u16,
}

#[binread]
#[br(import(cf: &'a ClassFile,))]
pub struct LocalVariableTable<'a> {
    #[br(calc = cf)]
    class_file: &'a ClassFile,
    #[br(temp)]
    local_variable_table_length: u16,
    #[br(count = local_variable_table_length)]
    local_variable_table: Vec<(u16, u16, Utf8Index, Utf8Index, u16)>,
}

impl<'a> LocalVariableTable<'a> {
    pub fn get_variables(&self) -> crate::Result<Vec<LocalVariable<'a>>> {
        self.local_variable_table.iter().map(|(start_pc, length, name, descriptor, index)| Ok(LocalVariable {
            start_pc: *start_pc,
            length: *length,
            name: name.get_as_string(self.class_file)?,
            descriptor: crate::field::TypeDescriptor::parse(descriptor.get_as_string(self.class_file)?)?.1,
            index: *index,
        })).collect()
    }
}

// TODO 4.7.14. The LocalVariableTypeTable Attribute 