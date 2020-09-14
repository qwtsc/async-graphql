mod cache_control;
mod federation;

use crate::parser::types::{BaseType as ParsedBaseType, Type as ParsedType};
use crate::validators::InputValueValidator;
use crate::{model, Value};
use indexmap::map::IndexMap;
use indexmap::set::IndexSet;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub use cache_control::CacheControl;

fn strip_brackets(type_name: &str) -> Option<&str> {
    if let Some(rest) = type_name.strip_prefix('[') {
        Some(&rest[..rest.len() - 1])
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MetaTypeName<'a> {
    List(&'a str),
    NonNull(&'a str),
    Named(&'a str),
}

impl<'a> std::fmt::Display for MetaTypeName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaTypeName::Named(name) => write!(f, "{}", name),
            MetaTypeName::NonNull(name) => write!(f, "{}!", name),
            MetaTypeName::List(name) => write!(f, "[{}]", name),
        }
    }
}

impl<'a> MetaTypeName<'a> {
    pub fn create(type_name: &str) -> MetaTypeName {
        if let Some(type_name) = type_name.strip_suffix('!') {
            MetaTypeName::NonNull(type_name)
        } else if let Some(type_name) = strip_brackets(type_name) {
            MetaTypeName::List(type_name)
        } else {
            MetaTypeName::Named(type_name)
        }
    }

    pub fn concrete_typename(type_name: &str) -> &str {
        match MetaTypeName::create(type_name) {
            MetaTypeName::List(type_name) => Self::concrete_typename(type_name),
            MetaTypeName::NonNull(type_name) => Self::concrete_typename(type_name),
            MetaTypeName::Named(type_name) => type_name,
        }
    }

    pub fn is_non_null(&self) -> bool {
        matches!(self, MetaTypeName::NonNull(_))
    }

    pub fn unwrap_non_null(&self) -> Self {
        match self {
            MetaTypeName::NonNull(ty) => MetaTypeName::create(ty),
            _ => *self,
        }
    }

    pub fn is_subtype(&self, sub: &MetaTypeName<'_>) -> bool {
        match (self, sub) {
            (MetaTypeName::NonNull(super_type), MetaTypeName::NonNull(sub_type))
            | (MetaTypeName::Named(super_type), MetaTypeName::NonNull(sub_type)) => {
                MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
            }
            (MetaTypeName::Named(super_type), MetaTypeName::Named(sub_type)) => {
                super_type == sub_type
            }
            (MetaTypeName::List(super_type), MetaTypeName::List(sub_type)) => {
                MetaTypeName::create(super_type).is_subtype(&MetaTypeName::create(sub_type))
            }
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct MetaInputValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub ty: String,
    pub default_value: Option<String>,
    pub validator: Option<Arc<dyn InputValueValidator>>,
}

#[derive(Clone)]
pub struct MetaField {
    pub name: String,
    pub description: Option<&'static str>,
    pub args: IndexMap<&'static str, MetaInputValue>,
    pub ty: String,
    pub deprecation: Option<&'static str>,
    pub cache_control: CacheControl,
    pub external: bool,
    pub requires: Option<&'static str>,
    pub provides: Option<&'static str>,
}

#[derive(Clone)]
pub struct MetaEnumValue {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub deprecation: Option<&'static str>,
}

pub enum MetaType {
    Scalar {
        name: String,
        description: Option<&'static str>,
        is_valid: fn(value: &Value) -> bool,
    },
    Object {
        name: String,
        description: Option<&'static str>,
        fields: IndexMap<String, MetaField>,
        cache_control: CacheControl,
        extends: bool,
        keys: Option<Vec<String>>,
    },
    Interface {
        name: String,
        description: Option<&'static str>,
        fields: IndexMap<String, MetaField>,
        possible_types: IndexSet<String>,
        extends: bool,
        keys: Option<Vec<String>>,
    },
    Union {
        name: String,
        description: Option<&'static str>,
        possible_types: IndexSet<String>,
    },
    Enum {
        name: String,
        description: Option<&'static str>,
        enum_values: IndexMap<&'static str, MetaEnumValue>,
    },
    InputObject {
        name: String,
        description: Option<&'static str>,
        input_fields: IndexMap<String, MetaInputValue>,
    },
}

impl MetaType {
    pub fn field_by_name(&self, name: &str) -> Option<&MetaField> {
        self.fields().and_then(|fields| fields.get(name))
    }

    pub fn fields(&self) -> Option<&IndexMap<String, MetaField>> {
        match self {
            MetaType::Object { fields, .. } => Some(&fields),
            MetaType::Interface { fields, .. } => Some(&fields),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            MetaType::Scalar { name, .. } => &name,
            MetaType::Object { name, .. } => name,
            MetaType::Interface { name, .. } => name,
            MetaType::Union { name, .. } => name,
            MetaType::Enum { name, .. } => name,
            MetaType::InputObject { name, .. } => name,
        }
    }

    pub fn is_composite(&self) -> bool {
        match self {
            MetaType::Object { .. } => true,
            MetaType::Interface { .. } => true,
            MetaType::Union { .. } => true,
            _ => false,
        }
    }

    pub fn is_abstract(&self) -> bool {
        match self {
            MetaType::Interface { .. } => true,
            MetaType::Union { .. } => true,
            _ => false,
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self {
            MetaType::Enum { .. } => true,
            MetaType::Scalar { .. } => true,
            _ => false,
        }
    }

    pub fn is_input(&self) -> bool {
        match self {
            MetaType::Enum { .. } => true,
            MetaType::Scalar { .. } => true,
            MetaType::InputObject { .. } => true,
            _ => false,
        }
    }

    pub fn is_possible_type(&self, type_name: &str) -> bool {
        match self {
            MetaType::Interface { possible_types, .. } => possible_types.contains(type_name),
            MetaType::Union { possible_types, .. } => possible_types.contains(type_name),
            MetaType::Object { name, .. } => name == type_name,
            _ => false,
        }
    }

    pub fn possible_types(&self) -> Option<&IndexSet<String>> {
        match self {
            MetaType::Interface { possible_types, .. } => Some(possible_types),
            MetaType::Union { possible_types, .. } => Some(possible_types),
            _ => None,
        }
    }

    pub fn type_overlap(&self, ty: &MetaType) -> bool {
        if self as *const MetaType == ty as *const MetaType {
            return true;
        }

        match (self.is_abstract(), ty.is_abstract()) {
            (true, true) => self
                .possible_types()
                .iter()
                .copied()
                .flatten()
                .any(|type_name| ty.is_possible_type(type_name)),
            (true, false) => self.is_possible_type(ty.name()),
            (false, true) => ty.is_possible_type(self.name()),
            (false, false) => false,
        }
    }
}

pub struct MetaDirective {
    pub name: &'static str,
    pub description: Option<&'static str>,
    pub locations: Vec<model::__DirectiveLocation>,
    pub args: IndexMap<&'static str, MetaInputValue>,
}

pub struct Registry {
    pub types: HashMap<String, MetaType>,
    pub directives: HashMap<String, MetaDirective>,
    pub implements: HashMap<String, HashSet<String>>,
    pub query_type: String,
    pub mutation_type: Option<String>,
    pub subscription_type: Option<String>,
}

impl Registry {
    pub fn create_type<T: crate::Type, F: FnMut(&mut Registry) -> MetaType>(
        &mut self,
        mut f: F,
    ) -> String {
        let name = T::type_name();
        if !self.types.contains_key(name.as_ref()) {
            // Inserting a fake type before calling the function allows recursive types to exist.
            self.types.insert(
                name.clone().into_owned(),
                MetaType::Object {
                    name: "".to_string(),
                    description: None,
                    fields: Default::default(),
                    cache_control: Default::default(),
                    extends: false,
                    keys: None,
                },
            );
            let ty = f(self);
            *self.types.get_mut(&*name).unwrap() = ty;
        }
        T::qualified_type_name()
    }

    pub fn add_directive(&mut self, directive: MetaDirective) {
        self.directives
            .insert(directive.name.to_string(), directive);
    }

    pub fn add_implements(&mut self, ty: &str, interface: &str) {
        self.implements
            .entry(ty.to_string())
            .and_modify(|interfaces| {
                interfaces.insert(interface.to_string());
            })
            .or_insert({
                let mut interfaces = HashSet::new();
                interfaces.insert(interface.to_string());
                interfaces
            });
    }

    pub fn add_keys(&mut self, ty: &str, keys: &str) {
        let all_keys = match self.types.get_mut(ty) {
            Some(MetaType::Object { keys: all_keys, .. }) => all_keys,
            Some(MetaType::Interface { keys: all_keys, .. }) => all_keys,
            _ => return,
        };
        if let Some(all_keys) = all_keys {
            all_keys.push(keys.to_string());
        } else {
            *all_keys = Some(vec![keys.to_string()]);
        }
    }

    pub fn concrete_type_by_name(&self, type_name: &str) -> Option<&MetaType> {
        self.types.get(MetaTypeName::concrete_typename(type_name))
    }

    pub fn concrete_type_by_parsed_type(&self, query_type: &ParsedType) -> Option<&MetaType> {
        match &query_type.base {
            ParsedBaseType::Named(name) => self.types.get(name.as_str()),
            ParsedBaseType::List(ty) => self.concrete_type_by_parsed_type(ty),
        }
    }

    pub(crate) fn has_entities(&self) -> bool {
        self.types.values().any(|ty| match ty {
            MetaType::Object {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            MetaType::Interface {
                keys: Some(keys), ..
            } => !keys.is_empty(),
            _ => false,
        })
    }

    fn create_entity_type(&mut self) {
        let possible_types = self
            .types
            .values()
            .filter_map(|ty| match ty {
                MetaType::Object {
                    name,
                    keys: Some(keys),
                    ..
                } if !keys.is_empty() => Some(name.clone()),
                MetaType::Interface {
                    name,
                    keys: Some(keys),
                    ..
                } if !keys.is_empty() => Some(name.clone()),
                _ => None,
            })
            .collect();

        self.types.insert(
            "_Entity".to_string(),
            MetaType::Union {
                name: "_Entity".to_string(),
                description: None,
                possible_types,
            },
        );
    }
}
