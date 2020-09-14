// Copyright 2020 Daniel Harrison. All Rights Reserved.

#![allow(unused_attributes)]
#![rustfmt::skip::macros(write)]

use capnp;
use capnp::message::ReaderOptions;
use capnp::{serialize, Error, Result};
use heck::{CamelCase, ShoutySnakeCase, SnakeCase};
use std::collections::HashMap;
use std::io;
use std::io::Write;

use crate::schema_capnp::node::{source_info, struct_};
use crate::schema_capnp::{code_generator_request, field, node, type_, value};

struct Field {
  doc_comment: Option<String>,
  name: String,
  type_: FieldTypeEnum,
  offset: u32,
}

impl Field {
  fn name(raw: &str) -> String {
    raw.to_snake_case()
  }

  fn meta_name(&self) -> String {
    self.name.to_shouty_snake_case()
  }

  fn ftype(&self) -> &dyn FieldType {
    self.type_.ftype()
  }
}

// WIP: Rename
enum FieldTypeEnum {
  Primitive(PrimitiveField),
  List(ListField),
  Struct(StructField),
  Union(UnionField),
  Wrapped(WrappedField),
}

impl FieldTypeEnum {
  fn ftype(&self) -> &dyn FieldType {
    match self {
      FieldTypeEnum::Primitive(x) => x,
      FieldTypeEnum::List(x) => x,
      FieldTypeEnum::Struct(x) => x,
      FieldTypeEnum::Union(x) => x,
      FieldTypeEnum::Wrapped(x) => x,
    }
  }
}

trait FieldType {
  fn type_param(&self) -> Option<String>;
  fn type_out(&self) -> String;
  fn type_out_result(&self) -> bool;
  fn type_in(&self) -> String;
  fn type_in_option(&self) -> bool;
  fn type_owned(&self) -> String;
  fn type_meta(&self) -> String;
  fn type_element(&self) -> String;
  fn type_meta_class(&self, field_meta: String) -> String;
}

struct PrimitiveField {
  type_: String,
}

impl FieldType for PrimitiveField {
  fn type_param(&self) -> Option<String> {
    None
  }
  fn type_out(&self) -> String {
    self.type_.clone()
  }
  fn type_out_result(&self) -> bool {
    false
  }
  fn type_in(&self) -> String {
    self.type_.clone()
  }
  fn type_in_option(&self) -> bool {
    false
  }
  fn type_owned(&self) -> String {
    self.type_.clone()
  }
  fn type_meta(&self) -> String {
    self.type_.to_shouty_snake_case()
  }
  fn type_element(&self) -> String {
    format!("ElementType::Primitive(PrimitiveElementType::{})", self.type_.to_shouty_snake_case())
  }
  fn type_meta_class(&self, field_meta: String) -> String {
    format!(
      "FieldMeta::Primitive(PrimitiveFieldMeta::{}({}))",
      self.type_.to_shouty_snake_case(),
      field_meta
    )
  }
}

struct WrappedField {
  wrap_type: String,
  wrapped: Box<FieldTypeEnum>,
}

impl FieldType for WrappedField {
  fn type_param(&self) -> Option<String> {
    None
  }
  fn type_out(&self) -> String {
    self.wrap_type.clone()
  }
  fn type_out_result(&self) -> bool {
    self.wrapped.ftype().type_out_result()
  }
  fn type_in(&self) -> String {
    self.wrap_type.clone()
  }
  fn type_in_option(&self) -> bool {
    self.wrapped.ftype().type_in_option()
  }
  fn type_owned(&self) -> String {
    self.wrap_type.clone()
  }
  fn type_meta(&self) -> String {
    self.wrapped.ftype().type_meta()
  }
  fn type_element(&self) -> String {
    self.wrapped.ftype().type_meta()
  }
  fn type_meta_class(&self, field_meta: String) -> String {
    self.wrapped.ftype().type_meta_class(field_meta)
  }
}

struct StructField {
  type_: String,
}

impl FieldType for StructField {
  fn type_param(&self) -> Option<String> {
    None
  }
  fn type_out(&self) -> String {
    format!("{}<'a>", self.type_)
  }
  fn type_out_result(&self) -> bool {
    true
  }
  fn type_in(&self) -> String {
    // TODO: Figure out how to accept a non-shared version. This would require
    // recursively copying in all the data.
    format!("&{}Shared", self.type_)
  }
  fn type_in_option(&self) -> bool {
    true
  }
  fn type_owned(&self) -> String {
    self.type_.clone()
  }
  fn type_meta(&self) -> String {
    "Struct".to_string()
  }
  fn type_element(&self) -> String {
    format!(
      "ElementType::Pointer(PointerElementType::Struct(StructElementType {{meta: &{}::META}}))",
      self.type_
    )
  }
  fn type_meta_class(&self, field_meta: String) -> String {
    format!("FieldMeta::Pointer(PointerFieldMeta::Struct({}))", field_meta)
  }
}

struct ListField {
  wrapped: Box<FieldTypeEnum>,
}

impl FieldType for ListField {
  // let type_param = format!("{}Iter: ExactSizeIterator<Item={}Ref<'a>>", name, name);
  // let type_in = format!("{}Iter", name);
  // let type_out = format!("impl ExactSizeIterator<Item={}Ref<'a>>", name);

  fn type_param(&self) -> Option<String> {
    None
  }
  fn type_out(&self) -> String {
    format!("Vec<{}>", self.wrapped.ftype().type_out())
  }
  fn type_out_result(&self) -> bool {
    true
  }
  fn type_in(&self) -> String {
    format!("&'_ [{}]", self.wrapped.ftype().type_in())
  }
  fn type_in_option(&self) -> bool {
    false
  }
  fn type_owned(&self) -> String {
    format!("Vec<<{}>>", self.wrapped.ftype().type_in())
  }
  fn type_meta(&self) -> String {
    "List".to_string()
  }
  fn type_element(&self) -> String {
    format!(
      "ElementType::Pointer(PointerElementType::List(ListElementType{{values: &{}}}))",
      self.wrapped.ftype().type_element()
    )
  }
  fn type_meta_class(&self, field_meta: String) -> String {
    format!("FieldMeta::Pointer(PointerFieldMeta::List({}))", field_meta)
  }
}

struct UnionField {
  name: String,
  union: Union,
}

impl FieldType for UnionField {
  fn type_param(&self) -> Option<String> {
    None
  }
  fn type_out(&self) -> String {
    format!("{}<'a>", self.name)
  }
  fn type_out_result(&self) -> bool {
    true
  }
  fn type_in(&self) -> String {
    format!("{}Shared", self.name)
  }
  fn type_in_option(&self) -> bool {
    false
  }
  fn type_owned(&self) -> String {
    self.name.clone()
  }
  fn type_meta(&self) -> String {
    "Union".to_string()
  }
  fn type_element(&self) -> String {
    format!("UnionElementType")
  }
  fn type_meta_class(&self, field_meta: String) -> String {
    format!("FieldMeta::Union({})", field_meta)
  }
}

struct Struct {
  name: String,
  doc_comment: Option<String>,
  fields: Vec<Field>,
  data_words: usize,
  pointer_words: usize,
}

impl Struct {
  fn name(raw: &str) -> String {
    raw.to_camel_case()
  }

  fn render_field_meta(w: &mut dyn Write, field: &Field) -> io::Result<()> {
    write!(w, "  const {}_META: {}FieldMeta = {}FieldMeta {{\n", field.meta_name(), field.ftype().type_meta(), field.ftype().type_meta())?;
    write!(w, "    name: \"{}\",\n", field.name)?;
    write!(w, "    offset: NumElements({}),\n", field.offset)?;
    match &field.type_ {
      FieldTypeEnum::Struct(_) => {
        write!(w, "    meta: &{}::META,\n", field.ftype().type_owned())?;
      }
      FieldTypeEnum::List(x) => {
        write!(w, "    meta: &ListMeta {{\n")?;
        write!(w, "      value_type: {}\n", x.wrapped.ftype().type_element())?;
        write!(w, "    }},\n")?;
      }
      FieldTypeEnum::Union(x) => {
        write!(w, "    meta: &{}::META,\n", x.name)?;
      }
      _ => {} // No-op
    }
    write!(w, "  }};\n")
  }

  fn render(&self, w: &mut dyn io::Write) -> io::Result<()> {
    let struct_name = &self.name;

    if let Some(doc_comment) = &self.doc_comment {
      write!(w, "/// {}\n", doc_comment)?;
    }
    write!(w, "#[derive(Clone)]\n")?;
    write!(w, "pub struct {}<'a> {{\n", struct_name)?;
    write!(w, "  data: UntypedStruct<'a>,\n")?;
    write!(w, "}}\n\n")?;

    write!(w, "impl<'a> {}<'a> {{\n", struct_name)?;

    for field in self.fields.iter() {
      Struct::render_field_meta(w, field)?;
    }
    write!(w, "\n")?;

    write!(w, "  const META: StructMeta = StructMeta {{\n")?;
    write!(w, "    name: \"{}\",\n", struct_name)?;
    write!(w, "    data_size: NumWords({}),\n", self.data_words)?;
    write!(w, "    pointer_size: NumWords({}),\n", self.pointer_words)?;
    write!(w, "    fields: || &[\n")?;
    for field in self.fields.iter() {
      write!(w, "      {},\n", field.ftype().type_meta_class(format!("{}::{}_META", struct_name, field.meta_name())))?;
    }
    write!(w, "    ],\n")?;
    write!(w, "  }};\n\n")?;

    for field in self.fields.iter() {
      if let Some(doc_comment) = &field.doc_comment {
        write!(w, "  /// {}\n", doc_comment)?;
      }
      write!(w, "  pub fn {}(&self) -> ", field.name)?;
      if field.ftype().type_out_result() {
        write!(w, "Result<{}, Error>",field.ftype().type_out())?;
      } else {
        write!(w, "{}",field.ftype().type_out())?;
      }
      write!(w, " {{ {}::{}_META.get(&self.data) }}\n",
      struct_name, field.meta_name())?;
    }

    write!(w, "}}\n\n")?;

    write!(w, "impl<'a> TypedStruct<'a> for {}<'a> {{\n", struct_name)?;
    write!(w, "  fn meta() -> &'static StructMeta {{\n")?;
    write!(w, "    &{}::META\n", struct_name)?;
    write!(w, "  }}\n")?;
    write!(w, "  fn from_untyped_struct(data: UntypedStruct<'a>) -> Self {{\n")?;
    write!(w, "    {} {{ data: data }}\n", struct_name)?;
    write!(w, "  }}\n")?;
    write!(w, "  fn as_untyped(&self) -> UntypedStruct<'a> {{\n")?;
    write!(w, "    self.data.clone()\n")?;
    write!(w, "  }}\n")?;
    write!(w, "}}\n\n")?;

    write!(w, "impl<'a> std::fmt::Debug for {}<'a> {{\n", struct_name)?;
    write!(w, "  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n")?;
    write!(w, "    self.as_element().fmt(f)\n")?;
    write!(w, "  }}\n")?;
    write!(w, "}}\n\n")?;

    write!(w, "impl<'a> std::cmp::PartialOrd for {}<'a> {{\n", struct_name)?;
    write!(w, "  fn partial_cmp(&self, other: &{}<'a>) -> Option<std::cmp::Ordering> {{\n", struct_name)?;
    write!(w, "    self.as_element().partial_cmp(&other.as_element())\n")?;
    write!(w, "  }}\n")?;
    write!(w, "}}\n\n")?;

    write!(w, "impl<'a> std::cmp::PartialEq for {}<'a> {{\n", struct_name)?;
    write!(w, "  fn eq(&self, other: &{}<'a>) -> bool {{\n", struct_name)?;
    write!(w, "    self.partial_cmp(&other) == Some(std::cmp::Ordering::Equal)\n")?;
    write!(w, "  }}\n")?;
    write!(w, "}}\n\n")?;

    write!(w, "pub struct {}Shared {{\n", struct_name)?;
    write!(w, "  data: UntypedStructShared,\n")?;
    write!(w, "}}\n\n")?;

    write!(w, "impl {}Shared {{\n", struct_name)?;

    write!(w, "  pub fn new(\n")?;
    for field in self.fields.iter() {
      if field.type_.ftype().type_in_option() {
        write!(w, "    {}: Option<{}>,\n", field.name, field.type_.ftype().type_in())?;
      } else {
        write!(w, "    {}: {},\n", field.name, field.type_.ftype().type_in())?;
      }
    }
    write!(w, "  ) -> {}Shared {{\n", struct_name)?;
    write!(w, "    let mut data = UntypedStructOwned::new_with_root_struct({}::META.data_size, {}::META.pointer_size);\n", struct_name, struct_name)?;
    for field in self.fields.iter() {
      write!(w, "    {}::{}_META.set(&mut data, {});\n", struct_name, field.meta_name(), field.name)?;
    }
    write!(w, "    {}Shared {{ data: data.into_shared() }}\n", struct_name)?;
    write!(w, "  }}\n\n")?;

    write!(w, "  pub fn as_ref<'a>(&'a self) -> {}<'a> {{\n", struct_name)?;
    write!(w, "    {} {{ data: self.data.as_ref() }}\n", struct_name)?;
    write!(w, "  }}\n")?;

    write!(w, "}}\n\n")?;

    write!(w, "impl TypedStructShared for {}Shared {{\n", struct_name)?;
    write!(w, "  fn meta() -> &'static StructMeta {{\n")?;
    write!(w, "    &{}::META\n", struct_name)?;
    write!(w, "  }}\n")?;
    write!(w, "  fn from_untyped_struct(data: UntypedStructShared) -> Self {{\n")?;
    write!(w, "    {}Shared {{ data: data }}\n", struct_name)?;
    write!(w, "  }}\n")?;
    write!(w, "  fn as_untyped(&self) -> UntypedStructShared {{\n")?;
    write!(w, "    self.data.clone()\n")?;
    write!(w, "  }}\n")?;
    write!(w, "}}\n\n")?;

    Ok(())
  }
}

struct UnionVariant {
  name: String,
  field: Field,
  discriminant: u64,
}

struct Union {
  name: String,
  doc_comment: Option<String>,
  variants: Vec<UnionVariant>,
  discriminant_offset: u32,
}

impl Union {
  fn name(raw: &str) -> String {
    raw.to_camel_case()
  }

  fn render(&self, w: &mut dyn Write) -> io::Result<()> {
    if let Some(doc_comment) = &self.doc_comment {
      write!(w, "/// {}\n", doc_comment.trim().replace("\n", " "))?;
    }
    write!(w, "#[derive(Clone)]\n")?;
    write!(w, "pub enum {}<'a> {{\n", &self.name)?;
    for variant in &self.variants {
      write!(w, "  {}({}),\n", variant.name, variant.field.ftype().type_out())?;
    }
    write!(w, "}}\n\n")?;

    write!(w, "impl {}<'_> {{\n", &self.name)?;
    for variant in &self.variants {
      Struct::render_field_meta(w, &variant.field)?;
    }
    write!(w, "  const META: UnionMeta = UnionMeta {{\n")?;
    write!(w, "    name: \"{}\",\n", &self.name)?;
    write!(w, "    variants: &[\n")?;
    for variant in self.variants.iter() {
      write!(w, "      UnionVariantMeta{{\n")?;
      write!(w, "        discriminant: Discriminant({}),\n", variant.discriminant)?;
      write!(w, "        field_meta: {},\n",
        variant.field.ftype().type_meta_class(format!("{}::{}_META", &self.name, variant.field.meta_name())))?;
      write!(w, "      }},\n")?;
    }
    write!(w, "    ],\n")?;
    write!(w, "  }};\n")?;
    write!(w, "}}\n\n")?;

    write!(w, "impl<'a> TypedUnion<'a> for {}<'a> {{\n", &self.name)?;
    write!(w, "  fn meta() -> &'static UnionMeta {{\n")?;
    write!(w, "    &{}::META\n",  &self.name)?;
    write!(w, "  }}\n")?;
    write!(w, "  fn from_untyped_union(untyped: &UntypedUnion<'a>) -> Result<Self, Error> {{\n")?;
    write!(w, "    match untyped.discriminant {{\n")?;
    for variant in self.variants.iter() {
      // TODO: This only works for pointer types.
      write!(w, "      Discriminant({}) => {}::{}_META.get(&untyped.variant_data).map(|x| {}::{}(x)),\n",
        variant.discriminant, &self.name, variant.field.meta_name(), &self.name, variant.name)?;
    }
    write!(w, "      x => Err(Error::from(format!(\"unknown {} discriminant: {{:?}}\", x))),\n", &self.name)?;
    write!(w, "    }}\n")?;
    write!(w, "  }}\n")?;
    write!(w, "}}\n")?;

    if let Some(doc_comment) = &self.doc_comment {
      write!(w, "/// {}\n", doc_comment.trim().replace("\n", " "))?;
    }
    write!(w, "pub enum {}Shared {{\n", &self.name)?;
    for variant in &self.variants {
      write!(w, "  {}({}Shared),\n", variant.name, variant.field.ftype().type_owned())?;
    }
    write!(w, "}}\n\n")?;

    write!(w, "impl {}Shared {{\n", &self.name)?;
    write!(w, "  fn as_ref<'a>(&'a self) -> {}<'a> {{\n", &self.name)?;
    write!(w, "    match self {{\n")?;
    for variant in &self.variants {
      write!(w, "      {}Shared::{}(x) => {}::{}(x.as_ref()),\n", &self.name, variant.name, &self.name, variant.name)?;
    }
    write!(w, "    }}\n")?;
    write!(w, "  }}\n")?;
    write!(w, "}}\n\n")?;

    write!(w, "impl<'a> TypedUnionShared<'a, {}<'a>> for {}Shared {{\n", &self.name, &self.name)?;
    write!(w, "  fn as_ref(&'a self) -> {}<'a> {{\n", &self.name)?;
    write!(w, "    match self {{\n")?;
    for variant in &self.variants {
      write!(w, "      {}Shared::{}(x) => {}::{}(x.as_ref()),\n", &self.name, variant.name, &self.name, variant.name)?;
    }
    write!(w, "    }}\n")?;
    write!(w, "  }}\n")?;
    write!(w, "  fn set(&self, data: &mut UntypedStructOwned, discriminant_offset: NumElements) {{\n")?;
    write!(w, "    match self {{\n")?;
    for variant in &self.variants {
      write!(w, "      {}Shared::{}(x) => {{\n", &self.name, variant.name)?;
      write!(w, "        data.set_discriminant(discriminant_offset, Discriminant({}));\n", variant.discriminant)?;
      write!(w, "        {}::{}_META.set(data, Some(&*x));\n", &self.name, variant.field.meta_name())?;
      write!(w, "      }}\n")?;
    }
    write!(w, "    }}\n")?;
    write!(w, "  }}\n")?;
    write!(w, "}}\n")?;

    Ok(())
  }
}

pub struct Generator<'a> {
  nodes: HashMap<u64, node::Reader<'a>>,
  source_infos: HashMap<u64, source_info::Reader<'a>>,
  names: HashMap<u64, String>,
}

#[rustfmt::skip::macros(write)]
impl<'a> Generator<'a> {
  pub fn generate(r: &mut dyn io::Read, w: &mut dyn io::Write) -> Result<()> {
    let req_reader = serialize::read_message(r, ReaderOptions::new())?;
    let req = req_reader.get_root::<code_generator_request::Reader>()?;
    let mut g =
      Generator { nodes: HashMap::new(), source_infos: HashMap::new(), names: HashMap::new() };
    g.req(w, req)
  }

  fn req(&mut self, w: &mut dyn io::Write, req: code_generator_request::Reader<'a>) -> Result<()> {
    for source_info in req.get_source_info()?.iter() {
      self.source_infos.insert(source_info.get_id(), source_info);
    }
    for node in req.get_nodes()?.iter() {
      self.nodes.insert(node.get_id(), node);
      for nested_node in node.get_nested_nodes()? {
        self.names.insert(nested_node.get_id(), nested_node.get_name()?.to_string());
      }
      match node.which()? {
        node::Which::Struct(struct_) => {
          for field in struct_.get_fields()? {
            match field.which()? {
              field::Which::Group(group) => {
                self.names.insert(group.get_type_id(), field.get_name()?.to_string());
              }
              _ => {}
            }
          }
        }
        _ => {}
      }
    }

    Generator::render_preamble(w)?;
    for node in req.get_nodes()?.iter() {
      match node.which()? {
        node::Which::Struct(struct_) => {
          if struct_.get_discriminant_count() > 0 {
            self.union(node, struct_)?.render(w)?;
          } else {
            self.struct_(node, struct_)?.render(w)?;
          }
        }
        _ => {}
      }
    }
    Ok(())
  }

  fn doc_comment(&self, id: u64) -> Result<(Option<String>, Vec<Option<String>>)> {
    let si = self.source_infos.get(&id).ok_or_else(wip_err)?;
    let doc_comment =
      if si.has_doc_comment() { Some(si.get_doc_comment()?.trim().to_string()) } else { None };
    let mut field_doc_comments = Vec::new();
    for fsi in si.get_members()?.iter() {
      field_doc_comments.push(if fsi.has_doc_comment() {
        Some(fsi.get_doc_comment()?.trim().to_string())
      } else {
        None
      });
    }
    Ok((doc_comment, field_doc_comments))
  }

  fn struct_(&self, node: node::Reader<'a>, struct_: struct_::Reader<'a>) -> Result<Struct> {
    let name = self.names.get(&node.get_id()).ok_or_else(wip_err)?;
    let name = Struct::name(name);
    let (doc_comment, field_doc_comments) = self.doc_comment(node.get_id())?;
    let raw_fields = struct_.get_fields()?.iter();
    if raw_fields.len() != field_doc_comments.len() {
      return Err(wip_err());
    }
    let mut fields = Vec::new();
    for (field, doc_comment) in raw_fields.zip(field_doc_comments) {
      let field_opt = self.field(struct_, field, doc_comment)?;
      if let Some(field) = field_opt {
        fields.push(field);
      }
    }

    Ok(Struct {
      name: name,
      doc_comment: doc_comment,
      fields: fields,
      data_words: struct_.get_data_word_count() as usize,
      pointer_words: struct_.get_pointer_count() as usize,
    })
  }

  fn union(&self, node: node::Reader<'a>, struct_: struct_::Reader<'a>) -> Result<Union> {
    let name = self.names.get(&node.get_id()).ok_or_else(wip_err)?;
    let name = Union::name(name);

    let (doc_comment, field_doc_comments) = self.doc_comment(node.get_id())?;
    let fields = struct_.get_fields()?.iter();
    if fields.len() != field_doc_comments.len() {
      return Err(wip_err());
    }
    let mut variants = Vec::new();
    for (field, doc_comment) in fields.zip(field_doc_comments) {
      let discriminant = field.get_discriminant_value() as u64;
      let field_opt = self.field(struct_, field, doc_comment)?;
      if let Some(field) = field_opt {
        variants.push(UnionVariant {
          name: field.name.to_camel_case(),
          field: field,
          discriminant: discriminant,
        });
      }
    }

    Ok(Union {
      name: name,
      doc_comment: doc_comment,
      variants: variants,
      discriminant_offset: struct_.get_discriminant_offset(),
    })
  }

  fn field(
    &self,
    struct_: struct_::Reader<'a>,
    field: field::Reader<'a>,
    doc_comment: Option<String>,
  ) -> Result<Option<Field>> {
    let name = Field::name(field.get_name()?);
    let mut _offset_extra: usize = 0; // WIP hacks
    let ret = match field.which()? {
      field::Which::Slot(slot) => {
        let field_type: FieldTypeEnum = match slot.get_type()?.which()? {
          type_::Which::Uint64(_) => {
            FieldTypeEnum::Primitive(PrimitiveField { type_: "u64".to_string() })
          }
          type_::Which::Data(_) => {
            _offset_extra += struct_.get_data_word_count() as usize * 8;
            FieldTypeEnum::List(ListField {
              wrapped: Box::new(FieldTypeEnum::Primitive(PrimitiveField {
                type_: "u8".to_string(),
              })),
            })
          }
          type_::Which::Struct(substruct_) => {
            _offset_extra += struct_.get_data_word_count() as usize * 8;
            let type_name = self.names.get(&substruct_.get_type_id()).ok_or_else(wip_err)?;
            FieldTypeEnum::Struct(StructField { type_: Struct::name(type_name) })
          }
          type_::Which::List(item) => match item.get_element_type()?.which()? {
            type_::Which::Struct(substruct_) => {
              _offset_extra += struct_.get_data_word_count() as usize * 8;
              let type_name = self.names.get(&substruct_.get_type_id()).ok_or_else(wip_err)?;
              FieldTypeEnum::List(ListField {
                wrapped: Box::new(FieldTypeEnum::Struct(StructField {
                  type_: Struct::name(type_name),
                })),
              })
            }
            _ => return Ok(None), // WIP
          },
          _ => return Ok(None), // WIP
        };

        let custom_type_id = 17086713920921479830;
        let custom_type = field.get_annotations()?.iter().find(|a| a.get_id() == custom_type_id);
        let custom_type = custom_type.map(|annotation| {
          let ret = match annotation.get_value().expect("WIP").which().expect("WIP") {
            value::Which::Text(text) => text.expect("WIP").to_string(),
            _ => panic!("WIP"),
          };
          ret
        });
        let field_type: FieldTypeEnum = match custom_type {
          Some(custom_type) => FieldTypeEnum::Wrapped(WrappedField {
            wrap_type: custom_type,
            wrapped: Box::new(field_type),
          }),
          None => field_type,
        };

        Field { name: name, doc_comment: doc_comment, type_: field_type, offset: slot.get_offset() }
      }
      field::Which::Group(group) => {
        let type_name = self.names.get(&group.get_type_id()).expect("WIP");
        let node = self.nodes.get(&group.get_type_id()).expect("WIP").clone();
        let struct_ = match node.which()? {
          node::Which::Struct(struct_) => struct_,
          _ => return Err(wip_err()),
        };
        let union = self.union(node, struct_)?;

        let offset = union.discriminant_offset;
        let type_ =
          FieldTypeEnum::Union(UnionField { name: type_name.to_camel_case(), union: union });
        Field { name: name, doc_comment: doc_comment, type_: type_, offset: offset }
      }
    };

    // WIP hacks

    Ok(Some(ret))
  }

  fn render_preamble(w: &mut dyn Write) -> io::Result<()> {
    write!(w, "use capnp_runtime::prelude::*;\n\n")?;
    Ok(())
  }
}

fn wip_err() -> Error {
  todo!()
  // Error::failed("WIP".to_string())
}