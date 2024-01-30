use nom::{
    branch::alt,
    bytes::complete::{is_a, tag},
    character::complete::char,
    combinator::{map, opt, value},
    multi::{many0, many1},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

#[derive(Clone)]
pub enum JavaType<'a> {
    Base(BaseType),
    Reference(ReferenceType<'a>),
}

impl<'a> JavaType<'a> {
    pub(crate) fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(ReferenceType::parse, |x| Self::Reference(x)),
            map(BaseType::parse, |x| Self::Base(x)),
        ))(input)
    }
}

#[derive(Clone)]
pub enum BaseType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
}

impl BaseType {
    fn parse<'a>(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Self::Byte, char('B')),
            value(Self::Char, char('C')),
            value(Self::Double, char('D')),
            value(Self::Float, char('F')),
            value(Self::Int, char('I')),
            value(Self::Long, char('J')),
            value(Self::Short, char('S')),
            value(Self::Boolean, char('Z')),
        ))(input)
    }
}

#[derive(Clone)]
pub enum ReferenceType<'a> {
    JavaString,
    JavaClass,
    ClassType(ClassType<'a>),
    TypeVariable(&'a str),
    ArrayType(Box<JavaType<'a>>),
}

impl<'a> ReferenceType<'a> {
    pub(crate) fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            value(Self::JavaString, tag("Ljava/lang/String;")),
            value(Self::JavaClass, tag("Ljava/lang/Class;")),
            map(ClassType::parse, |x| Self::ClassType(x)),
            map(delimited(char('T'), identifier, char(';')), |x| {
                Self::TypeVariable(x)
            }),
            map(preceded(char('['), JavaType::parse), |x| {
                Self::ArrayType(Box::new(x))
            }),
        ))(input)
    }
}

#[derive(Clone)]
pub enum TypeArgument<'a> {
    Plus(ReferenceType<'a>),
    Minus(ReferenceType<'a>),
    Star,
}

impl<'a> TypeArgument<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(preceded(char('+'), ReferenceType::parse), |x| Self::Plus(x)),
            map(preceded(char('-'), ReferenceType::parse), |x| {
                Self::Minus(x)
            }),
            value(Self::Star, char('*')),
        ))(input)
    }
}

#[derive(Clone)]
pub struct SimpleClassType<'a> {
    pub name: &'a str,
    pub type_arguments: Vec<TypeArgument<'a>>,
}

impl<'a> SimpleClassType<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (name, type_arguments)) = tuple((
            identifier,
            opt(delimited(char('<'), many1(TypeArgument::parse), char('>'))),
        ))(input)?;
        let type_arguments = type_arguments.unwrap_or_default();
        Ok((
            input,
            Self {
                name,
                type_arguments,
            },
        ))
    }
}

#[derive(Clone)]
pub struct ClassType<'a> {
    pub package: Vec<&'a str>,
    pub base: SimpleClassType<'a>,
    pub sub: Vec<SimpleClassType<'a>>,
}

impl<'a> ClassType<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (package, base, sub)) = delimited(
            char('L'),
            tuple((
                many0(terminated(identifier, char('/'))),
                SimpleClassType::parse,
                many0(preceded(char('.'), SimpleClassType::parse)),
            )),
            char(';'),
        )(input)?;
        Ok((input, Self { package, base, sub }))
    }
}

fn identifier<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    is_a("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_")(input)
}

pub struct TypeParameter<'a> {
    pub name: &'a str,
    pub class_bound: Option<ReferenceType<'a>>,
    pub interface_bounds: Vec<ReferenceType<'a>>,
}

impl<'a> TypeParameter<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (name, class_bound, interface_bounds)) = tuple((
            identifier,
            preceded(char(':'), opt(ReferenceType::parse)),
            many0(preceded(char(':'), ReferenceType::parse)),
        ))(input)?;
        Ok((
            input,
            Self {
                name,
                class_bound,
                interface_bounds,
            },
        ))
    }
}

pub struct ClassSignature<'a> {
    pub type_parameters: Vec<TypeParameter<'a>>,
    pub superclass_signature: ClassType<'a>,
    pub superinterface_signatures: Vec<ClassType<'a>>,
}

impl<'a> ClassSignature<'a> {
    pub(crate) fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (type_parameters, superclass_signature, superinterface_signatures)) =
            tuple((
                opt(delimited(char('<'), many1(TypeParameter::parse), char('>'))),
                ClassType::parse,
                many0(ClassType::parse),
            ))(input)?;
        let type_parameters = type_parameters.unwrap_or_default();
        Ok((
            input,
            Self {
                type_parameters,
                superclass_signature,
                superinterface_signatures,
            },
        ))
    }
}

pub enum ThrowsSignature<'a> {
    ClassType(ClassType<'a>),
    TypeVariable(&'a str),
}

impl<'a> ThrowsSignature<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        preceded(
            char('^'),
            alt((
                map(ClassType::parse, |x| Self::ClassType(x)),
                map(delimited(char('T'), identifier, char(';')), |x| {
                    Self::TypeVariable(x)
                }),
            )),
        )(input)
    }
}

pub struct MethodSignature<'a> {
    pub type_parameters: Vec<TypeParameter<'a>>,
    pub parameters: Vec<JavaType<'a>>,
    pub result: Option<JavaType<'a>>,
    pub throws: Vec<ThrowsSignature<'a>>,
}

impl<'a> MethodSignature<'a> {
    pub(crate) fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (type_parameters, parameters, result, throws)) = tuple((
            opt(delimited(char('<'), many1(TypeParameter::parse), char('>'))),
            delimited(char('('), many0(JavaType::parse), char(')')),
            alt((value(None, char('V')), map(JavaType::parse, |x| Some(x)))),
            many0(ThrowsSignature::parse),
        ))(input)?;
        let type_parameters = type_parameters.unwrap_or_default();
        Ok((
            input,
            Self {
                type_parameters,
                parameters,
                result,
                throws,
            },
        ))
    }
}
