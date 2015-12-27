//! Functions for finding declarations in C translation units.

use std::collections::{HashSet};

use super::{Entity, EntityKind, TranslationUnit, TypeKind};

//================================================
// Structs
//================================================

// Enum __________________________________________

/// An enum declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Enum<'tu> {
    name: String,
    entity: Entity<'tu>,
    constants: Vec<(String, i64, u64)>,
}

impl<'tu> Enum<'tu> {
    //- Constructors -----------------------------

    fn new(name: String, entity: Entity<'tu>) -> Enum<'tu> {
        let constants = entity.get_children().into_iter().filter_map(|e| {
            if e.get_kind() == EntityKind::EnumConstantDecl {
                let (signed, unsigned) = e.get_enum_constant_value().unwrap();
                Some((e.get_name().unwrap(), signed, unsigned))
            } else {
                None
            }
        }).collect();

        Enum { name: name, entity: entity, constants: constants }
    }

    //- Accessors --------------------------------

    /// Returns the enum constants in this enum.
    pub fn get_constants(&self) -> &Vec<(String, i64, u64)> {
        &self.constants
    }

    /// Returns the AST entity for this enum.
    pub fn get_entity(&self) -> Entity<'tu> {
        self.entity
    }

    /// Returns the name of this enum.
    pub fn get_name(&self) -> &String {
        &self.name
    }
}

// Struct ________________________________________

/// A struct declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Struct<'tu> {
    name: String,
    entity: Entity<'tu>,
    fields: Vec<Entity<'tu>>,
}

impl<'tu> Struct<'tu> {
    //- Constructors -----------------------------

    fn new(name: String, entity: Entity<'tu>) -> Struct<'tu> {
        let fields = entity.get_children().into_iter().filter_map(|e| {
            if e.get_kind() == EntityKind::FieldDecl {
                Some(e)
            } else {
                None
            }
        }).collect();

        Struct { name: name, entity: entity, fields: fields }
    }

    //- Accessors --------------------------------

    /// Returns the AST entity for this struct.
    pub fn get_entity(&self) -> Entity<'tu> {
        self.entity
    }

    /// Returns the fields in this struct.
    pub fn get_fields(&self) -> &Vec<Entity<'tu>> {
        &self.fields
    }

    /// Returns the name of this struct.
    pub fn get_name(&self) -> &String {
        &self.name
    }
}

// Union _________________________________________

/// A union declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Union<'tu> {
    name: String,
    entity: Entity<'tu>,
    fields: Vec<Entity<'tu>>,
}

impl<'tu> Union<'tu> {
    //- Constructors -----------------------------

    fn new(name: String, entity: Entity<'tu>) -> Union<'tu> {
        let fields = entity.get_children().into_iter().filter_map(|e| {
            if e.get_kind() == EntityKind::FieldDecl {
                Some(e)
            } else {
                None
            }
        }).collect();

        Union { name: name, entity: entity, fields: fields }
    }

    //- Accessors --------------------------------

    /// Returns the AST entity for this union.
    pub fn get_entity(&self) -> Entity<'tu> {
        self.entity
    }

    /// Returns the fields in this union.
    pub fn get_fields(&self) -> &Vec<Entity<'tu>> {
        &self.fields
    }

    /// Returns the name of this union.
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Returns the size of this union in bytes.
    pub fn get_size(&self) -> usize {
        self.entity.get_type().unwrap().get_sizeof().unwrap()
    }
}

//================================================
// Functions
//================================================

/// Returns the enums in the supplied C translation unit.
pub fn find_enums<'tu>(tu: &'tu TranslationUnit<'tu>) -> Vec<Enum<'tu>> {
    let mut seen = HashSet::new();

    tu.get_entity().get_children().into_iter().filter_map(|e| {
        match e.get_kind() {
            EntityKind::EnumDecl => {
                e.get_name().and_then(|n| {
                    if !seen.contains(&n) {
                        seen.insert(n.clone());
                        Some(Enum::new(n, e))
                    } else {
                        None
                    }
                })
            },
            EntityKind::TypedefDecl => {
                let name = e.get_name().unwrap();
                let type_ = e.get_typedef_underlying_type().unwrap().get_canonical_type();

                if type_.get_kind() == TypeKind::Enum && !seen.contains(&name) {
                    seen.insert(name.clone());
                    Some(Enum::new(name, type_.get_declaration().unwrap()))
                } else {
                    None
                }
            },
            _ => None,
        }
    }).collect()
}

/// Returns the functions in the supplied translation unit.
pub fn find_functions<'tu>(tu: &'tu TranslationUnit<'tu>) -> Vec<Entity<'tu>> {
    tu.get_entity().get_children().into_iter().filter(|e| {
        e.get_kind() == EntityKind::FunctionDecl
    }).collect()
}

/// Returns the structs in the supplied C translation unit.
pub fn find_structs<'tu>(tu: &'tu TranslationUnit<'tu>) -> Vec<Struct<'tu>> {
    let mut seen = HashSet::new();

    tu.get_entity().get_children().into_iter().filter_map(|e| {
        match e.get_kind() {
            EntityKind::StructDecl => {
                e.get_name().and_then(|n| {
                    if !seen.contains(&n) {
                        seen.insert(n.clone());
                        Some(Struct::new(n, e))
                    } else {
                        None
                    }
                })
            },
            EntityKind::TypedefDecl => {
                let name = e.get_name().unwrap();
                let type_ = e.get_typedef_underlying_type().unwrap();

                if type_.get_display_name().contains("struct ") && !seen.contains(&name) {
                    seen.insert(name.clone());
                    Some(Struct::new(name, type_.get_declaration().unwrap()))
                } else {
                    None
                }
            },
            _ => None,
        }
    }).collect()
}

/// Returns the typedefs in the supplied translation unit.
pub fn find_typedefs<'tu>(tu: &'tu TranslationUnit<'tu>) -> Vec<Entity<'tu>> {
    tu.get_entity().get_children().into_iter().filter(|e| {
        e.get_kind() == EntityKind::TypedefDecl
    }).collect()
}

/// Returns the unions in the supplied C translation unit.
pub fn find_unions<'tu>(tu: &'tu TranslationUnit<'tu>) -> Vec<Union<'tu>> {
    let mut seen = HashSet::new();

    tu.get_entity().get_children().into_iter().filter_map(|e| {
        match e.get_kind() {
            EntityKind::UnionDecl => {
                e.get_name().and_then(|n| {
                    if !seen.contains(&n) {
                        seen.insert(n.clone());
                        Some(Union::new(n, e))
                    } else {
                        None
                    }
                })
            },
            EntityKind::TypedefDecl => {
                let name = e.get_name().unwrap();
                let type_ = e.get_typedef_underlying_type().unwrap();

                if type_.get_display_name().contains("union ") && !seen.contains(&name) {
                    seen.insert(name.clone());
                    Some(Union::new(name, type_.get_declaration().unwrap()))
                } else {
                    None
                }
            },
            _ => None,
        }
    }).collect()
}
