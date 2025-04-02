use std::{borrow::Cow, collections::HashMap, io::Read};

use xml::{EventReader, attribute::OwnedAttribute, common::Position, reader::XmlEvent};

use crate::{
    FromXml,
    core::{Api, Depends, Deprecation, ExternSync, FromStr, Len, LimitType, ParseAttr, Queue},
    error::{Error, ErrorKind},
    macros::FromXml,
    util::Extract,
};

pub(crate) trait VecAttributesExt {
    fn into_hash_map(self) -> HashMap<String, String>;
}

impl VecAttributesExt for Vec<OwnedAttribute> {
    fn into_hash_map(self) -> HashMap<String, String> {
        HashMap::from_iter(
            self.into_iter()
                .map(|attr| (attr.name.local_name, attr.value)),
        )
    }
}

#[derive(Debug)]
pub enum GenericItem {
    String(String),
    Type(String),    // type
    Name(String),    // name
    Enum(String),    // enum
    Comment(String), // comment
}

impl FromXml for GenericItem {
    fn from_xml<R: Read>(
        reader: &mut EventReader<R>,
        tag: String,
        attrs: HashMap<String, String>,
    ) -> Result<Self, Error> {
        let mut out = String::new();
        let pos = reader.position();
        if !attrs.is_empty() {
            return Err(Error::new(
                pos,
                ErrorKind::UnexpectedTagEnd {
                    parent: "unknown".to_string(),
                    tag,
                },
            ));
        }

        loop {
            match reader.next()? {
                XmlEvent::StartDocument { .. } => {
                    return Err(Error::new(reader.position(), ErrorKind::UnexpectedDocStart));
                }
                XmlEvent::EndDocument => {
                    return Err(Error::new(reader.position(), ErrorKind::UnexpectedDocEnd));
                }

                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::new(
                        reader.position(),
                        ErrorKind::UnexpectedTagStart {
                            parent: tag,
                            tag: name.local_name,
                        },
                    ));
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == tag {
                        break;
                    } else {
                        return Err(Error::new(
                            reader.position(),
                            ErrorKind::UnexpectedTagEnd {
                                parent: tag,
                                tag: name.local_name,
                            },
                        ));
                    }
                }

                XmlEvent::Characters(text) | XmlEvent::Whitespace(text) => out.push_str(&text),

                _ => continue, // TODO: should probably error out
            }
        }

        Ok(match tag.as_str() {
            "type" => Self::Type(out),
            "name" => Self::Name(out),
            "enum" => Self::Enum(out),
            "comment" => Self::Comment(out),
            _ => {
                return Err(Error::new(
                    pos,
                    ErrorKind::BadTaggedVariant {
                        name: "GenericItem".to_string(),
                        tag,
                    },
                ));
            }
        })
    }
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "comment")]
pub struct Comment {
    #[vk(text)]
    pub text: String,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "registry")]
pub struct Registry {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(RegistryItem))]
    pub items: Vec<RegistryItem>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
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

#[derive(Debug, Default, FromXml)]
#[vk(tag = "platforms")]
pub struct Platforms {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(Platform))]
    pub items: Vec<Platform>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "platform")]
pub struct Platform {
    pub name: String,
    pub protect: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "tags")]
pub struct Tags {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(Tag))]
    pub items: Vec<Tag>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "tag")]
pub struct Tag {
    pub name: String,
    pub author: String,
    pub contact: String,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "types")]
pub struct Types {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(TypesItem))]
    pub items: Vec<TypesItem>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum TypesItem {
    Comment(Comment),
    Type(Type),
}

#[derive(Debug, FromXml)]
#[vk(var, tag = "type")]
pub enum Type {
    #[vk(attr = "alias")]
    Alias(AliasType),
    #[vk(none)]
    Explicit(ExplicitType),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct AliasType {
    pub name: String,
    pub alias: String,
    pub category: Category,
    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Category {
    #[default]
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

impl Extract for Category {
    type Output = Category;
}

impl ParseAttr for Category {
    type Output = Self;

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

#[derive(Debug, FromXml)]
#[vk(eval, tag = "type", attr = "category")]
pub enum ExplicitType {
    #[vk(value = "include")]
    Include(TypeInclude),
    #[vk(value = "define")]
    Define(TypeDefine),
    #[vk(value = "basetype")]
    Base(TypeBase),
    #[vk(value = "handle")]
    Handle(TypeHandle),
    #[vk(value = "enum")]
    Enum(TypeEnum),
    #[vk(value = "bitmask")]
    Bitmask(TypeBitmask),
    #[vk(value = "funcpointer")]
    FnPtr(TypeFnPtr),
    #[vk(value = "struct")]
    Struct(TypeStruct),
    #[vk(value = "union")]
    Union(TypeUnion),
    #[vk(none)]
    Misc(TypeMisc),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeMisc {
    pub name: String,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeInclude {
    pub name: String,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeDefine {
    #[vk(optional)]
    pub name: Option<String>,
    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeBase {
    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeHandle {
    #[vk(optional)]
    pub parent: Option<String>,
    #[vk(name = "objtypeenum")]
    pub obj_type_enum: String,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeEnum {
    pub name: String,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeBitmask {
    #[vk(optional, name = "bitvalues")]
    pub bit_values: Option<String>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeFnPtr {
    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeStruct {
    pub name: String,
    #[vk(optional, name = "structextends")]
    pub struct_extends: Option<Vec<String>>,
    #[vk(optional, name = "returnedonly")]
    pub returned_only: Option<bool>,
    #[vk(optional, name = "allowduplicate")]
    pub allow_duplicate: Option<bool>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(TypeItem))]
    pub members: Vec<TypeItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct TypeUnion {
    pub name: String,
    #[vk(optional, name = "returnedonly")]
    pub returned_only: Option<bool>,
    #[vk(optional, name = "structextends")]
    pub struct_extends: Option<Vec<String>>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub requires: Option<String>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(TypeItem))]
    pub members: Vec<TypeItem>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum TypeItem {
    Comment(Comment),
    Member(Member),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "member")]
pub struct Member {
    #[vk(optional)]
    pub values: Option<Vec<String>>,
    #[vk(optional)]
    pub optional: Option<Vec<bool>>,
    #[vk(optional, name = "externsync")]
    pub extern_sync: Option<ExternSync>,
    #[vk(optional, name = "noautovalidity")]
    pub no_auto_validity: Option<bool>,
    #[vk(optional, name = "objecttype")]
    pub object_type: Option<String>,

    #[vk(optional)]
    pub len: Option<Vec<Len>>,
    #[vk(optional)]
    pub stride: Option<String>,

    #[vk(optional)]
    pub selector: Option<String>,
    #[vk(optional)]
    pub selection: Option<String>,

    #[vk(optional, name = "limittype")]
    pub limit_type: Option<Vec<LimitType>>,
    #[vk(optional, name = "featurelink")]
    pub feature_link: Option<String>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,

    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enums")]
pub struct Enums {
    #[vk(name = "type")]
    pub ty: EnumsKind,

    #[vk(optional)]
    pub name: Option<String>,
    #[vk(optional)]
    pub bitwidth: Option<u32>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(EnumsItem))]
    pub items: Vec<EnumsItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnumsKind {
    #[default]
    Constants,
    Enum,
    Bitmask,
}

impl Extract for EnumsKind {
    type Output = EnumsKind;
}

impl FromStr for EnumsKind {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "constants" => Self::Constants,
            "enum" => Self::Enum,
            "bitmask" => Self::Bitmask,
            _ => return Err(s.into_owned()),
        })
    }
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum EnumsItem {
    Comment(Comment),
    Enum(Enum),
    Unused(Unused),
}

#[derive(Debug, FromXml)]
#[vk(var, tag = "enum")]
pub enum Enum {
    #[vk(attr = "value")]
    Value(EnumValue),
    #[vk(attr = "bitpos")]
    Bit(EnumBit),
    #[vk(attr = "offset")]
    Offset(EnumOffset),
    #[vk(attr = "alias")]
    Alias(EnumAlias),
    #[vk(none)]
    Ref(RefEnum),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enum")]
pub struct EnumValue {
    pub name: String,
    #[vk(optional)]
    pub extends: Option<String>,

    pub value: String,
    #[vk(optional, name = "type")]
    pub ty: Option<String>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub protect: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enum")]
pub struct EnumBit {
    pub name: String,
    #[vk(optional)]
    pub extends: Option<String>,

    pub bitpos: u32,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub protect: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enum")]
pub struct EnumOffset {
    pub name: String,
    pub extends: String,

    #[vk(optional, name = "extnumber")]
    pub ext_number: Option<u32>,
    pub offset: u32,
    #[vk(optional)]
    pub dir: Option<Dir>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub protect: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enum")]
pub struct EnumAlias {
    pub name: String,
    pub alias: String,
    #[vk(optional)]
    pub extends: Option<String>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub deprecated: Option<Deprecation>,
    #[vk(optional)]
    pub protect: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enum")]
pub struct RefEnum {
    pub name: String,
    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "unused")]
pub struct Unused {
    pub start: String,
    #[vk(optional)]
    pub end: Option<String>,
    #[vk(optional)]
    pub vendor: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Pos, // +
    Neg, // -
}

impl Extract for Dir {
    type Output = Dir;
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

#[derive(Debug, Default, FromXml)]
#[vk(tag = "commands")]
pub struct Commands {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(Command))]
    pub items: Vec<Command>,
}

#[derive(Debug, FromXml)]
#[vk(var, tag = "command")]
pub enum Command {
    #[vk(attr = "alias")]
    Alias(AliasCommand),
    #[vk(none)]
    Explicit(ExplicitCommand),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "command")]
pub struct AliasCommand {
    pub name: String,
    pub alias: String,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "command")]
pub struct ExplicitCommand {
    #[vk(optional, name = "successcodes")]
    pub success_codes: Option<Vec<String>>,
    #[vk(optional, name = "errorcodes")]
    pub error_codes: Option<Vec<String>>,

    #[vk(optional)]
    pub tasks: Option<Vec<Task>>,
    #[vk(optional)]
    pub queues: Option<Vec<Queue>>,
    #[vk(optional, name = "renderpass")]
    pub render_pass: Option<RenderPass>,
    #[vk(optional, name = "videocoding")]
    pub video_coding: Option<VideoCoding>,
    #[vk(optional, name = "cmdbufferlevel")]
    pub cmd_buffer_level: Option<Vec<CommandBufferLevel>>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(CommandItem))]
    pub items: Vec<CommandItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    Action,
    Synchronization,
    State,
    Indirection,
}

impl Extract for Task {
    type Output = Task;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPass {
    Inside,
    Outside,
    Both,
}

impl Extract for RenderPass {
    type Output = RenderPass;
}

impl FromStr for RenderPass {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "inside" => Self::Inside,
            "outside" => Self::Outside,
            "both" => Self::Both,
            _ => return Err(s.into_owned()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCoding {
    Inside,
    Outside,
    Both,
}

impl Extract for VideoCoding {
    type Output = VideoCoding;
}

impl FromStr for VideoCoding {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "inside" => Self::Inside,
            "outside" => Self::Outside,
            "both" => Self::Both,
            _ => return Err(s.into_owned()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandBufferLevel {
    Primary,
    Secondary,
}

impl Extract for CommandBufferLevel {
    type Output = CommandBufferLevel;
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

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum CommandItem {
    Proto(Proto),
    Param(Param),
    ImplicitExternSyncParams(ImplicitExternSyncParams),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "proto")]
pub struct Proto {
    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "param")]
pub struct Param {
    #[vk(optional)]
    pub optional: Option<Vec<bool>>,
    #[vk(optional, name = "externsync")]
    pub extern_sync: Option<ExternSync>,
    #[vk(optional, name = "noautovalidity")]
    pub no_auto_validity: Option<bool>,
    #[vk(optional, name = "objecttype")]
    pub object_type: Option<String>,
    #[vk(optional, name = "validstructs")]
    pub valid_structs: Option<Vec<String>>,

    #[vk(optional)]
    pub len: Option<Vec<Len>>,
    #[vk(optional)]
    pub stride: Option<String>,

    #[vk(optional)]
    pub selector: Option<String>,

    #[vk(optional)]
    pub api: Option<Vec<Api>>,

    #[vk(generic)]
    pub items: Vec<GenericItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "implicitexternsyncparams")]
pub struct ImplicitExternSyncParams {
    #[vk(items(ImplictExternSyncParam))]
    pub items: Vec<ImplictExternSyncParam>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "param")]
pub struct ImplictExternSyncParam {
    #[vk(text)]
    pub text: String,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "feature")]
pub struct Feature {
    pub name: String,
    pub api: Vec<Api>,
    #[vk(optional)]
    pub protect: Option<String>,

    #[vk(optional)]
    pub depends: Option<Depends>,
    #[vk(optional, name = "sortorder")]
    pub sort_order: Option<i32>,

    #[vk(optional)]
    pub number: Option<String>,

    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(FeatureBlock))]
    pub items: Vec<FeatureBlock>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "extensions")]
pub struct Extensions {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(Extension))]
    pub items: Vec<Extension>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "extension")]
pub struct Extension {
    pub name: String,
    pub number: u32,
    #[vk(optional, name = "type")]
    pub ty: Option<ExtensionKind>,
    #[vk(optional)]
    pub depends: Option<Depends>,

    #[vk(optional, name = "sortorder")]
    pub sort_order: Option<u32>,
    #[vk(optional)]
    pub author: Option<String>,
    #[vk(optional)]
    pub contact: Option<String>,
    #[vk(optional)]
    pub protect: Option<String>,
    #[vk(optional)]
    pub platform: Option<String>,

    pub supported: Vec<Api>,
    #[vk(optional)]
    pub ratified: Option<Vec<Api>>,
    #[vk(optional, name = "promotedto")]
    pub promoted_to: Option<String>,
    #[vk(optional, name = "deprecatedby")]
    pub deprecated_by: Option<String>,
    #[vk(optional, name = "obsoletedby")]
    pub obsoleted_by: Option<String>,
    #[vk(optional)]
    pub provisional: Option<bool>,
    #[vk(optional, name = "specialuse")]
    pub special_use: Option<Vec<String>>,
    #[vk(optional, name = "nofeatures")]
    pub no_features: Option<bool>,

    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(FeatureBlock))]
    pub items: Vec<FeatureBlock>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionKind {
    Instance,
    Device,
}

impl Extract for ExtensionKind {
    type Output = ExtensionKind;
}

impl FromStr for ExtensionKind {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "instance" => Self::Instance,
            "device" => Self::Device,
            _ => return Err(s.into_owned()),
        })
    }
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum FeatureBlock {
    Require(Require),
    Remove(Remove),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "require")]
pub struct Require {
    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub profile: Option<String>,
    #[vk(optional)]
    pub depends: Option<Depends>,

    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(InterfaceItem))]
    pub items: Vec<InterfaceItem>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "remove")]
pub struct Remove {
    #[vk(optional)]
    pub api: Option<Vec<Api>>,
    #[vk(optional)]
    pub profile: Option<String>,
    #[vk(optional, name = "reasonlink")]
    pub reason_link: Option<String>,

    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(InterfaceItem))]
    pub items: Vec<InterfaceItem>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum InterfaceItem {
    Comment(Comment),
    Command(InterfaceCommand),
    Enum(Enum),
    Type(InterfaceType),
    Feature(InterfaceFeature),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "command")]
pub struct InterfaceCommand {
    pub name: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "type")]
pub struct InterfaceType {
    pub name: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "feature")]
pub struct InterfaceFeature {
    pub name: Vec<String>,
    #[vk(name = "struct")]
    pub struct_name: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "formats")]
pub struct Formats {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(Format))]
    pub items: Vec<Format>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "format")]
pub struct Format {
    pub name: String,

    pub class: String,
    #[vk(name = "blockSize")]
    pub block_size: u32,
    #[vk(name = "texelsPerBlock")]
    pub texels_per_block: u32,
    #[vk(optional, name = "blockExtent")]
    pub block_extent: Option<Extent>,
    #[vk(optional)]
    pub packed: Option<u32>,
    #[vk(optional)]
    pub compressed: Option<String>,
    #[vk(optional)]
    pub chroma: Option<Chroma>,

    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(FormatItem))]
    pub items: Vec<FormatItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Extent {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Extract for Extent {
    type Output = Extent;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Chroma {
    C420,
    C422,
    C444,
}

impl Extract for Chroma {
    type Output = Chroma;
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

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum FormatItem {
    Component(Component),
    Plane(Plane),
    SpirvImageFormat(SpirvImageFormat),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "component")]
pub struct Component {
    pub name: String,
    pub bits: ComponentBits,
    #[vk(name = "numericFormat")]
    pub numeric_format: NumericFormat,
    #[vk(optional, name = "planeIndex")]
    pub plane_index: Option<u32>,

    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentBits {
    Bits(u32),
    Compressed,
}

impl Default for ComponentBits {
    fn default() -> Self {
        Self::Bits(32)
    }
}

impl Extract for ComponentBits {
    type Output = ComponentBits;
}

impl FromStr for ComponentBits {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        match s.as_ref() {
            "compressed" => Ok(Self::Compressed),
            _ => u32::from_str(s).map(Self::Bits),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NumericFormat {
    SFixedS,
    SFloat,
    #[default]
    SInt,
    SNorm,
    SRgb,
    SScaled,
    UFloat,
    UInt,
    UNorm,
    UScaled,
}

impl Extract for NumericFormat {
    type Output = NumericFormat;
}

impl FromStr for NumericFormat {
    fn from_str(s: Cow<'_, str>) -> Result<Self, String> {
        Ok(match s.as_ref() {
            "SFIXED5" => Self::SFixedS,
            "SFLOAT" => Self::SFloat,
            "SINT" => Self::SInt,
            "SNORM" => Self::SNorm,
            "SRGB" => Self::SRgb,
            "SSCALED" => Self::SScaled,
            "UFLOAT" => Self::UFloat,
            "UINT" => Self::UInt,
            "UNORM" => Self::UNorm,
            "USCALED" => Self::UScaled,
            _ => return Err(s.into_owned()),
        })
    }
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "plane")]
pub struct Plane {
    pub index: u32,
    #[vk(name = "widthDivisor")]
    pub width_divisor: u32,
    #[vk(name = "heightDivisor")]
    pub height_divisor: u32,
    pub compatible: String,

    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "spirvimageformat")]
pub struct SpirvImageFormat {
    pub name: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "sync")]
pub struct Sync {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(SyncItem))]
    pub items: Vec<SyncItem>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum SyncItem {
    Stage(SyncStage),
    Access(SyncAccess),
    Pipeline(SyncPipeline),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "syncstage")]
pub struct SyncStage {
    pub name: String,
    #[vk(optional)]
    pub alias: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(SyncStageItem))]
    pub items: Vec<SyncStageItem>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum SyncStageItem {
    Comment(Comment),
    Support(SyncStageSupport),
    Equivalent(SyncStageEquivalent),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "syncsupport")]
pub struct SyncStageSupport {
    pub queues: Vec<Queue>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "syncequivalent")]
pub struct SyncStageEquivalent {
    pub stage: Vec<String>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "syncaccess")]
pub struct SyncAccess {
    pub name: String,
    #[vk(optional)]
    pub alias: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(SyncAccessItem))]
    pub items: Vec<SyncAccessItem>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum SyncAccessItem {
    Comment(Comment),
    Support(SyncAccessSupport),
    Equivalent(SyncAccessEquivalent),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "syncsupport")]
pub struct SyncAccessSupport {
    pub stage: Vec<String>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "syncequivalent")]
pub struct SyncAccessEquivalent {
    pub access: Vec<String>,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "syncpipeline")]
pub struct SyncPipeline {
    pub name: String,
    #[vk(optional)]
    pub depends: Option<Depends>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(SyncPipelineStage))]
    pub items: Vec<SyncPipelineStage>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "syncpipelinestage")]
pub struct SyncPipelineStage {
    #[vk(text)]
    pub name: String,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(optional)]
    pub order: Option<String>,
    #[vk(optional)]
    pub before: Option<String>,
    #[vk(optional)]
    pub after: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videocodecs")]
pub struct VideoCodecs {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(VideoCodec))]
    pub items: Vec<VideoCodec>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videocodec")]
pub struct VideoCodec {
    pub name: String,
    #[vk(optional)]
    pub extend: Option<String>,
    #[vk(optional)]
    pub value: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(VideoCodecItem))]
    pub items: Vec<VideoCodecItem>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum VideoCodecItem {
    Profiles(VideoProfiles),
    Capabilities(VideoCapabilities),
    Format(VideoFormat),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videoprofiles")]
pub struct VideoProfiles {
    #[vk(name = "struct")]
    pub struct_name: String,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(VideoProfileMember))]
    pub items: Vec<VideoProfileMember>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videoprofilemember")]
pub struct VideoProfileMember {
    pub name: String,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(VideoProfile))]
    pub items: Vec<VideoProfile>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videoprofile")]
pub struct VideoProfile {
    pub name: String,
    pub value: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videocapabilities")]
pub struct VideoCapabilities {
    #[vk(name = "struct")]
    pub struct_name: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videoformat")]
pub struct VideoFormat {
    #[vk(optional)]
    pub name: Option<String>,
    #[vk(optional)]
    pub usage: Option<String>,
    #[vk(optional)]
    pub extend: Option<String>,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(VideoFormatItems))]
    pub items: Vec<VideoFormatItems>,
}

#[derive(Debug, FromXml)]
#[vk(tagged)]
pub enum VideoFormatItems {
    RequireCapabilities(VideoRequireCapabilities),
    FormatProperties(VideoFormatProperties),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videorequirecapabilities")]
pub struct VideoRequireCapabilities {
    #[vk(name = "struct")]
    pub struct_name: String,
    pub member: String,
    pub value: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "videoformatproperties")]
pub struct VideoFormatProperties {
    #[vk(name = "struct")]
    pub struct_name: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "spirvextensions")]
pub struct SpirvExtensions {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(SpirvExtension))]
    pub items: Vec<SpirvExtension>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "spirvextension")]
pub struct SpirvExtension {
    pub name: String,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(Enable))]
    pub items: Vec<Enable>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "spirvcapabilities")]
pub struct SpirvCapabilities {
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(SprivCapability))]
    pub items: Vec<SprivCapability>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "spirvcapability")]
pub struct SprivCapability {
    pub name: String,
    #[vk(optional)]
    pub comment: Option<String>,

    #[vk(items(Enable))]
    pub items: Vec<Enable>,
}

#[derive(Debug, FromXml)]
#[vk(var, tag = "enable")]
pub enum Enable {
    #[vk(attr = "version")]
    Version(EnableVersion),
    #[vk(attr = "extension")]
    Extension(EnableExtension),
    #[vk(attr = "feature")]
    Feature(EnableFeature),
    #[vk(attr = "property")]
    Property(EnableProperty),
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enable")]
pub struct EnableVersion {
    pub version: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enable")]
pub struct EnableExtension {
    pub extension: String,
    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enable")]
pub struct EnableFeature {
    #[vk(name = "struct")]
    pub struct_name: String,
    pub feature: String,
    pub requires: String,
    #[vk(optional)]
    pub alias: Option<String>,

    #[vk(optional)]
    pub comment: Option<String>,
}

#[derive(Debug, Default, FromXml)]
#[vk(tag = "enable")]
pub struct EnableProperty {
    pub property: String,
    pub member: String,
    pub value: String,
    #[vk(optional)]
    pub requires: Option<String>,

    #[vk(optional)]
    pub comment: Option<String>,
}
