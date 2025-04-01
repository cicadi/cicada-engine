use std::{borrow::Cow, collections::HashMap, io::Read, num::NonZeroU32};

use xml::{EventReader, common::Position};

use crate::{
    core::{Api, Depends, Deprecation, FromStr, Len, LimitType, ParseAttr, Queue},
    error::{Error, ErrorKind},
};

pub trait FromXml: Sized {
    fn from_xml<R: Read>(
        reader: &mut EventReader<R>,
        tag: String,
        attrs: HashMap<String, String>,
    ) -> Result<Self, Error>;
}

pub enum GenericItem {
    String(String),
    Type(String),    // type
    Name(String),    // name
    Enum(String),    // enum
    Comment(String), // comment
}

pub struct Comment {
    pub content: String,
}

pub struct Registry {
    pub comment: Option<String>,

    pub items: Vec<RegistryItem>,
}

pub enum RegistryItem {
    Comment(Comment),
    Platforms(Platforms),
    Tags(Tags),
    Types(Types),
    Enums(Enums),
    Commands(Commands),
    Feature(Feature),
    Extensions(Extensions),
    Formats(Formats),
    Sync(Sync),
    VideoCodecs(VideoCodecs),
    SpirvExtensions(SpirvExtensions),
    SpirvCapabilities(SpirvCapabilities),
}

pub struct Platforms {
    pub comment: Option<String>,

    pub items: Vec<Platform>,
}

pub struct Platform {
    pub name: String,
    pub protect: String,
    pub comment: Option<String>,
}

pub struct Tags {
    pub comment: Option<String>,

    pub items: Vec<Tag>,
}

pub struct Tag {
    pub name: String,
    pub author: String,
    pub contact: String,
}

pub struct Types {
    pub comment: Option<String>,

    pub items: Vec<TypesItem>,
}

pub enum TypesItem {
    Comment(Comment),
    Type(Type),
}

pub enum Type {
    Alias(AliasType),
    Explicit(ExplicitType),
}

pub enum Category {
    None,
    Include,
    Define,
    Base,
    Handle,
    Enum,
    Bitmask,
    FnPtr,
    Struct,
    Union,
}

impl ParseAttr for Category {
    type Output = Self;

    type Err = Error;

    fn parse_attr<R: Read>(
        reader: &EventReader<R>,
        tag: &str,
        attr: &str,
        attrs: &mut HashMap<String, String>,
    ) -> Result<Option<Self>, Error> {
        Ok(Some(match attrs.remove(attr) {
            Some(value) => match value.as_str() {
                "include" => Self::Include,
                "define" => Self::Define,
                "basetype" => Self::Base,
                "handle" => Self::Handle,
                "enum" => Self::Enum,
                "bitmask" => Self::Bitmask,
                "funcpointer" => Self::FnPtr,
                "struct" => Self::Struct,
                "union" => Self::Union,
                _ => {
                    return Err(Error::new(
                        reader.position(),
                        ErrorKind::BadAttr {
                            tag: tag.to_string(),
                            attr: attr.to_string(),
                            value,
                        },
                    ));
                }
            },
            None => Self::None,
        }))
    }
}

pub struct AliasType {
    pub name: String,
    pub alias: String,
    pub category: Category,
    pub api: Option<Vec<Api>>,
    pub comment: Option<String>,
}

pub enum ExplicitType {
    Misc(TypeMisc),
    Include(TypeInclude),
    Define(TypeDefine),
    Base(TypeBase),
    Handle(TypeHandle),
    Enum(TypeEnum),
    Bitmask(TypeBitmask),
    FnPtr(TypeFnPtr),
    Struct(TypeStruct),
    Union(TypeUnion),
}

pub struct TypeMisc {
    pub name: String,

    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,
}

pub struct TypeInclude {
    pub name: String,

    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,

    pub items: Option<Vec<GenericItem>>,
}

pub struct TypeDefine {
    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,

    pub items: Vec<GenericItem>,
}

pub struct TypeBase {
    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,

    pub items: Vec<GenericItem>,
}

pub struct TypeHandle {
    pub parent: Option<String>,
    pub obj_type_enum: String, // objtypeenum

    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,

    pub items: Vec<GenericItem>,
}

pub struct TypeEnum {
    pub name: String,

    pub api: Option<Vec<Api>>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,
}

pub struct TypeBitmask {
    pub bit_values: Option<String>, // bitvalues

    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,

    pub items: Vec<GenericItem>,
}

pub struct TypeFnPtr {
    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,

    pub items: Vec<GenericItem>,
}

pub struct TypeStruct {
    pub name: String,
    pub struct_extends: Option<Vec<String>>, //structextends
    pub returned_only: Option<bool>,         // returndonly
    pub allow_duplicate: Option<bool>,       //allowduplicate

    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,

    pub members: Vec<Member>,
}

pub struct TypeUnion {
    pub name: String,
    pub returned_only: Option<bool>,         // returndonly
    pub struct_extends: Option<Vec<String>>, //structextends

    pub api: Option<Vec<Api>>,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub comment: Option<String>,

    pub members: Vec<Member>,
}

pub struct Member {
    pub values: Option<Vec<String>>,
    pub optional: Option<Vec<bool>>,
    pub extern_sync: Option<bool>,      // externsync
    pub no_auto_validity: Option<bool>, // noautovalidity
    pub object_type: Option<String>,    // objecttype

    pub len: Option<Vec<Len>>,
    pub stride: Option<String>,

    pub selector: Option<String>,
    pub selection: Option<String>,

    pub limit_type: Option<Vec<LimitType>>, // limittype
    pub feature_link: Option<String>,       // featurelink

    pub api: Option<Vec<Api>>,
    pub deprecated: Option<Deprecation>,

    pub items: Vec<GenericItem>,
}

pub struct Enums {
    pub kind: EnumsKind, // type

    pub name: Option<String>,
    pub bitwidth: Option<NonZeroU32>,
    pub comment: Option<String>,

    pub items: Vec<EnumsItem>,
}

pub enum EnumsKind {
    Constants,
    Enum,
    Bitmask,
}

pub enum EnumsItem {
    Comment(Comment),
    Enum(Enum),
    Unused(Unused),
}

pub enum Enum {
    Value(EnumValue),
    Bit(EnumBit),
    Offset(EnumOffset),
    Alias(EnumAlias),
}

pub struct EnumValue {
    pub name: String,
    pub extends: Option<String>,

    pub value: String,
    pub ty: Option<String>, // type

    pub api: Option<Vec<Api>>,
    pub deprecated: Option<Deprecation>,
    pub protect: Option<String>,
    pub comment: Option<String>,
}

pub struct EnumBit {
    pub name: String,
    pub extends: Option<String>,

    pub bitpos: u32,

    pub api: Option<Vec<Api>>,
    pub deprecated: Option<Deprecation>,
    pub protect: Option<String>,
    pub comment: Option<String>,
}

pub struct EnumOffset {
    pub name: String,
    pub extends: String,

    pub ext_number: u32, // extnumber
    pub offset: u32,
    pub dir: Option<Dir>,

    pub api: Option<Vec<Api>>,
    pub deprecated: Option<Deprecation>,
    pub protect: Option<String>,
    pub comment: Option<String>,
}

pub struct EnumAlias {
    pub name: String,
    pub alias: String,
    pub extends: Option<String>,

    pub api: Option<Vec<Api>>,
    pub deprecated: Option<Deprecation>,
    pub protect: Option<String>,
    pub comment: Option<String>,
}

pub struct Unused {
    pub start: String,
    pub end: Option<String>,
    pub vendor: Option<String>,
    pub comment: Option<String>,
}

pub enum Dir {
    Pos, // +
    Neg, // -
}

impl FromStr for Dir {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "+" => Self::Pos,
            "-" => Self::Neg,
            _ => return Err(s.into_owned()),
        })
    }
}

pub struct Commands {
    pub comment: Option<String>,

    pub items: Vec<Command>,
}

pub enum Command {
    Alias(AliasCommand),
    Explicit(ExplicitCommand),
}

pub struct AliasCommand {
    pub name: String,
    pub alias: String,

    pub api: Option<Vec<Api>>,
    pub comment: Option<String>,
}

pub struct ExplicitCommand {
    pub success_codes: Option<Vec<String>>, // successcodes
    pub error_codes: Option<Vec<String>>,   // errorcodes

    pub tasks: Option<Vec<Task>>,
    pub queues: Option<Vec<Queue>>,
    pub render_pass: Option<RenderPass>,   // renderpass
    pub video_coding: Option<VideoCoding>, // videocoding
    pub cmd_buffer_level: Option<CommandBufferLevel>, // cmdbufferlevel

    pub api: Option<Vec<Api>>,
    pub comment: Option<String>,

    pub items: Vec<CommandItem>,
}

pub enum Task {
    Action,
    Synchronization,
    State,
    Indirection,
}

impl FromStr for Task {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "action" => Self::Action,
            "synchronization" => Self::Synchronization,
            "state" => Self::State,
            "indirection" => Self::Indirection,
            _ => return Err(s.into_owned()),
        })
    }
}

pub enum RenderPass {
    Inside,
    Outside,
    Both,
}

impl FromStr for RenderPass {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "inside" => Self::Inside,
            "outside" => Self::Outside,
            "Both" => Self::Both,
            _ => return Err(s.into_owned()),
        })
    }
}

pub enum VideoCoding {
    Inside,
    Outside,
    Both,
}

impl FromStr for VideoCoding {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "inside" => Self::Inside,
            "outside" => Self::Outside,
            "Both" => Self::Both,
            _ => return Err(s.into_owned()),
        })
    }
}

pub enum CommandBufferLevel {
    Primary,
    Secondary,
}

impl FromStr for CommandBufferLevel {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "primary" => Self::Primary,
            "secondary" => Self::Secondary,
            _ => return Err(s.into_owned()),
        })
    }
}
pub enum CommandItem {
    Proto(Proto),
    Param(Param),
    ImplicitExternSyncParams(ImplicitExternSyncParams),
}

pub struct Proto {
    pub items: Vec<GenericItem>,
}

pub struct Param {
    pub optional: Option<Vec<bool>>,
    pub extern_sync: Option<bool>,          // externsync
    pub no_auto_validity: Option<bool>,     // noautovalidity
    pub object_type: Option<String>,        // objecttype
    pub valid_structs: Option<Vec<String>>, // validstructs

    pub len: Option<Vec<Len>>,
    pub stride: Option<String>,

    pub selector: Option<String>,

    pub api: Option<Vec<Api>>,

    pub items: Vec<GenericItem>,
}

pub struct ImplicitExternSyncParams {
    pub items: Vec<ImplictExternSyncParam>,
}

pub struct ImplictExternSyncParam {
    pub content: String,
}

pub struct Feature {
    pub name: String,
    pub api: Vec<Api>,
    pub protect: Option<String>,

    pub depends: Option<Depends>,
    pub sort_order: Option<i32>, // sortorder

    pub number: Option<String>,

    pub comment: Option<String>,
}

pub struct Extensions {
    pub comment: Option<String>,
}

pub struct Require {
    pub api: Option<Vec<Api>>,
    pub profile: Option<String>,
    pub depends: Option<Depends>,

    pub comment: Option<String>,

    pub items: Vec<InterfaceItem>,
}

pub struct Remove {
    pub api: Option<Vec<Api>>,
    pub profile: Option<String>,
    pub reason_link: Option<String>, // reasonlink

    pub comment: Option<String>,

    pub items: Vec<InterfaceItem>,
}

pub enum InterfaceItem {
    Comment(Comment),
    Command(InterfaceCommand),
    Enum(InterfaceEnum),
    Type(InterfaceType),
    Feature(InterfaceFeature),
}

pub struct InterfaceCommand {
    pub name: String,
    pub comment: Option<String>,
}

pub enum InterfaceEnum {
    Ref(RefEnum),
    Ext(Enum),
}

pub struct RefEnum {
    pub name: String,
    pub api: Option<Vec<Api>>,
    pub comment: Option<String>,
}

pub struct InterfaceType {
    pub name: String,
    pub comment: Option<String>,
}

pub struct InterfaceFeature {
    pub name: Vec<String>,
    pub struct_name: String, // struct
    pub comment: Option<String>,
}

pub struct Formats {
    pub comment: Option<String>,

    pub items: Vec<Format>,
}

pub struct Format {
    pub name: String,

    pub class: String,
    pub block_size: u32,              // blockSize
    pub texels_per_block: u32,        // texelsPerBlock
    pub block_extent: Option<Extent>, // blockExtent
    pub packed: Option<u32>,
    pub compressed: Option<String>,
    pub chroma: Option<Chroma>,

    pub comment: Option<String>,

    pub items: Vec<FormatItem>,
}

pub struct Extent {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl FromStr for Extent {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        match s
            .as_ref()
            .split(',')
            .map(|s| u32::from_str(Cow::Borrowed(s)))
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(vec) => match *vec.as_slice() {
                [width, height, depth] => Ok(Self {
                    width,
                    height,
                    depth,
                }),
                _ => Err(s.into_owned()),
            },
            _ => Err(s.into_owned()),
        }
    }
}

pub enum Chroma {
    C420,
    C422,
    C444,
}

impl FromStr for Chroma {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "420" => Self::C420,
            "422" => Self::C422,
            "444" => Self::C444,
            _ => return Err(s.into_owned()),
        })
    }
}

pub enum FormatItem {
    Component(Component),
    Plane(Plane),
    SpirvImageFormat(SpirvImageFormat),
}

pub struct Component {
    pub name: String,
    pub bits: ComponentBits,
    pub numeric_format: NumericFormat, // numericFormat
    pub plane_index: Option<u32>,      // planeIndex

    pub comment: Option<String>,
}

pub enum ComponentBits {
    Bits(u32),
    Compressed,
}

impl FromStr for ComponentBits {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        match s.as_ref() {
            "compressed" => Ok(Self::Compressed),
            _ => u32::from_str(s).map(Self::Bits),
        }
    }
}

pub enum NumericFormat {
    SFixedS,
    SFloat,
    SInt,
    SNorm,
    SRgb,
    SScaled,
    UFloat,
    UInt,
    UNorm,
    UScaled,
}

impl FromStr for NumericFormat {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "SFIXEDS" => Self::SFixedS,
            "SFLOAT" => Self::SFloat,
            "SINT" => Self::SInt,
            "SNORM" => Self::SNorm,
            "SRGB" => Self::SRgb,
            "SSCALED" => Self::SScaled,
            "UFlOAT" => Self::UFloat,
            "UINT" => Self::UInt,
            "UNORM" => Self::UNorm,
            "USCALED" => Self::UScaled,
            _ => return Err(s.into_owned()),
        })
    }
}

pub struct Plane {
    pub index: u32,
    pub width_divisor: u32,  // widthDivisor
    pub height_divisor: u32, // heightDivisor
    pub compatible: String,

    pub comment: Option<String>,
}

pub struct SpirvImageFormat {
    pub name: String,
    pub comment: Option<String>,
}

pub struct Sync {
    pub comment: Option<String>,

    pub items: Vec<SyncItem>,
}

pub enum SyncItem {
    Stage(SyncStage),
    Access(SyncAccess),
    Pipeline(SyncPipeline),
}

pub struct SyncStage {
    pub name: String,
    pub alias: Option<String>,
    pub comment: Option<String>,

    pub items: Vec<SyncStageItem>,
}

pub enum SyncStageItem {
    Comment(Comment),
    Support(SyncStageSupport),
    Equivalent(SyncStageEquivalent),
}

pub struct SyncStageSupport {
    pub queues: Vec<Queue>,
    pub comment: Option<String>,
}

pub struct SyncStageEquivalent {
    pub stage: Vec<String>,
    pub comment: Option<String>,
}

pub struct SyncAccess {
    pub name: String,
    pub alias: Option<String>,
    pub comment: Option<String>,

    pub items: Vec<SyncAccessItem>,
}

pub enum SyncAccessItem {
    Comment(Comment),
    Support(SyncAccessSupport),
    Equivalent(SyncAccessEquivalent),
}

pub struct SyncAccessSupport {
    pub stage: Vec<String>,
    pub comment: Option<String>,
}

pub struct SyncAccessEquivalent {
    pub access: Vec<String>,
    pub comment: Option<String>,
}

pub struct SyncPipeline {
    pub name: String,
    pub depends: Option<Depends>,
    pub comment: Option<String>,

    pub items: Vec<SyncPipelineStage>,
}

pub struct SyncPipelineStage {
    pub name: String, // from content
    pub comment: Option<String>,

    pub order: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
}

pub struct VideoCodecs {
    pub comment: Option<String>,

    pub items: Vec<VideoCodec>,
}

pub struct VideoCodec {
    pub name: String,
    pub extend: Option<String>,
    pub value: Option<String>,
    pub comment: Option<String>,

    pub items: Vec<VideoCodecItem>,
}

pub enum VideoCodecItem {
    Profiles(VideoProfiles),
    Capabilities(VideoCapabilities),
    Format(VideoFormat),
}

pub struct VideoProfiles {
    pub struct_name: String, // struct
    pub comment: Option<String>,

    pub items: Vec<VideoProfileMember>,
}

pub struct VideoProfileMember {
    pub name: String,
    pub comment: Option<String>,

    pub items: Vec<VideoProfile>,
}

pub struct VideoProfile {
    pub name: String,
    pub value: String,
    pub comment: Option<String>,
}

pub struct VideoCapabilities {
    pub struct_name: String, // struct
    pub comment: Option<String>,
}

pub struct VideoFormat {
    pub name: Option<String>,
    pub usage: Option<String>,
    pub extends: Option<String>,
    pub comment: Option<String>,

    pub items: Vec<VideoFormatItems>,
}

pub enum VideoFormatItems {
    RequireCapabilities(VideoRequireCapabilities),
    FormatProperties(VideoFormatProperties),
}

pub struct VideoRequireCapabilities {
    pub struct_name: String, // struct
    pub member: String,
    pub value: String,
    pub comment: Option<String>,
}

pub struct VideoFormatProperties {
    pub struct_name: String, // struct
    pub comment: Option<String>,
}

pub struct SpirvExtensions {
    pub comment: Option<String>,

    pub items: Vec<SpirvExtension>,
}

pub struct SpirvExtension {
    pub name: String,
    pub comment: Option<String>,

    pub items: Vec<Enable>,
}

pub struct SpirvCapabilities {
    pub comment: Option<String>,

    pub items: Vec<SprivCapability>,
}

pub struct SprivCapability {
    pub name: String,
    pub comment: Option<String>,

    pub items: Vec<Enable>,
}

pub enum Enable {
    Version(EnableVersion),
    Extension(EnableExtension),
    Feature(EnableFeature),
    Property(EnableProperty),
}

pub struct EnableVersion {
    pub version: String,
    pub comment: Option<String>,
}

pub struct EnableExtension {
    pub extension: String,
    pub comment: Option<String>,
}

pub struct EnableFeature {
    pub struct_name: String, // struct
    pub feature: String,
    pub requires: String,
    pub alias: Option<String>,

    pub comment: Option<String>,
}

pub struct EnableProperty {
    pub property: String,
    pub member: String,
    pub value: String,
    pub requires: Option<String>,

    pub comment: Option<String>,
}
