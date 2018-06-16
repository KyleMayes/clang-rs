// Copyright 2016 Kyle Mayes
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Finding C declarations.

use std::vec;
use std::collections::{HashSet};
use std::str::{FromStr};

use super::{Entity, EntityKind, Type, TypeKind};

//================================================
// Enums
//================================================

// DefinitionValue _______________________________

/// The value of a C preprocessor definition.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DefinitionValue {
    /// An integer.
    Integer(bool, u64),
    /// A floating point number.
    Real(f64),
}

impl DefinitionValue {
    //- Constructors -----------------------------

    fn from_entity(entity: Entity) -> Option<DefinitionValue> {
        let mut tokens = entity.get_range().unwrap().tokenize();
        if tokens.last().map_or(false, |t| t.get_spelling() == "#") {
            tokens.pop();
        }

        let (negated, number) = if tokens.len() == 2 {
            (false, tokens[1].get_spelling())
        } else if tokens.len() == 3 && tokens[1].get_spelling() == "-" {
            (true, tokens[2].get_spelling())
        } else {
            return None;
        };

        if let Ok(integer) = u64::from_str(&number) {
            Some(DefinitionValue::Integer(negated, integer))
        } else if let Ok(real) = f64::from_str(&number) {
            if negated {
                Some(DefinitionValue::Real(-real))
            } else {
                Some(DefinitionValue::Real(real))
            }
        } else {
            None
        }
    }
}

//================================================
// Structs
//================================================

// Declaration ___________________________________

/// A C declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration<'tu> {
    /// The name of the declaration.
    pub name: String,
    /// The entity that describes the declaration (e.g., contains the fields of a struct).
    pub entity: Entity<'tu>,
    /// The entity the declaration originated from if it differs from `entity`.
    pub source: Option<Entity<'tu>>,
}

impl<'tu> Declaration<'tu> {
    //- Constructors -----------------------------

    fn new(name: String, entity: Entity<'tu>, source: Option<Entity<'tu>>) -> Declaration<'tu> {
        Declaration { name, entity, source }
    }
}

// Definition ____________________________________

/// A C preprocessor definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Definition<'tu> {
    /// The name of the definition.
    pub name: String,
    /// The value of the definition.
    pub value: DefinitionValue,
    /// The entity that describes the definition.
    pub entity: Entity<'tu>,
}

impl<'tu> Definition<'tu> {
    //- Constructors -----------------------------

    fn new(name: String, value: DefinitionValue, entity: Entity<'tu>) -> Definition<'tu> {
        Definition { name, value, entity }
    }
}

// Definitions ___________________________________

/// An iterator over preprocessor definition declarations.
#[allow(missing_debug_implementations)]
pub struct Definitions<'tu> {
    entities: vec::IntoIter<Entity<'tu>>,
    seen: HashSet<String>,
}

impl<'tu> Definitions<'tu> {
    //- Constructors -----------------------------

    fn new(entities: vec::IntoIter<Entity<'tu>>) -> Definitions<'tu> {
        Definitions { entities, seen: HashSet::new() }
    }
}

impl<'tu> Iterator for Definitions<'tu> {
    type Item = Definition<'tu>;

    fn next(&mut self) -> Option<Definition<'tu>> {
        for entity in &mut self.entities {
            if entity.get_kind() == EntityKind::MacroDefinition {
                let name = entity.get_name().unwrap();
                if !self.seen.contains(&name) {
                    if let Some(value) = DefinitionValue::from_entity(entity) {
                        self.seen.insert(name.clone());
                        return Some(Definition::new(name, value, entity));
                    }
                }
            }
        }
        None
    }
}

// Enums _________________________________________

/// An iterator over enum declarations.
#[allow(missing_debug_implementations)]
pub struct Enums<'tu> {
    entities: vec::IntoIter<Entity<'tu>>,
    seen: HashSet<String>,
}

impl<'tu> Enums<'tu> {
    //- Constructors -----------------------------

    fn new(entities: vec::IntoIter<Entity<'tu>>) -> Enums<'tu> {
        Enums { entities, seen: HashSet::new() }
    }
}

impl<'tu> Iterator for Enums<'tu> {
    type Item = Declaration<'tu>;

    fn next(&mut self) -> Option<Declaration<'tu>> {
        next(&mut self.entities, &mut self.seen, EntityKind::EnumDecl, "enum ")
    }
}

// Functions _____________________________________

/// An iterator over function declarations.
#[allow(missing_debug_implementations)]
pub struct Functions<'tu> {
    entities: vec::IntoIter<Entity<'tu>>,
    seen: HashSet<String>,
}

impl<'tu> Functions<'tu> {
    //- Constructors -----------------------------

    fn new(entities: vec::IntoIter<Entity<'tu>>) -> Functions<'tu> {
        Functions { entities, seen: HashSet::new() }
    }
}

impl<'tu> Iterator for Functions<'tu> {
    type Item = Declaration<'tu>;

    fn next(&mut self) -> Option<Declaration<'tu>> {
        for entity in &mut self.entities {
            if entity.get_kind() == EntityKind::FunctionDecl {
                let name = entity.get_name().unwrap();
                if !self.seen.contains(&name) {
                    self.seen.insert(name.clone());
                    return Some(Declaration::new(name, entity, None));
                }
            }
        }
        None
    }
}

// Structs _______________________________________

/// An iterator over struct declarations.
#[allow(missing_debug_implementations)]
pub struct Structs<'tu> {
    entities: vec::IntoIter<Entity<'tu>>,
    seen: HashSet<String>,
}

impl<'tu> Structs<'tu> {
    //- Constructors -----------------------------

    fn new(entities: vec::IntoIter<Entity<'tu>>) -> Structs<'tu> {
        Structs { entities, seen: HashSet::new() }
    }
}

impl<'tu> Iterator for Structs<'tu> {
    type Item = Declaration<'tu>;

    fn next(&mut self) -> Option<Declaration<'tu>> {
        next(&mut self.entities, &mut self.seen, EntityKind::StructDecl, "struct ")
    }
}

// Typedefs ______________________________________

/// An iterator over typedef declarations.
#[allow(missing_debug_implementations)]
pub struct Typedefs<'tu> {
    entities: vec::IntoIter<Entity<'tu>>,
    seen: HashSet<String>,
}

impl<'tu> Typedefs<'tu> {
    //- Constructors -----------------------------

    fn new(entities: vec::IntoIter<Entity<'tu>>) -> Typedefs<'tu> {
        Typedefs { entities, seen: HashSet::new() }
    }
}

impl<'tu> Iterator for Typedefs<'tu> {
    type Item = Declaration<'tu>;

    fn next(&mut self) -> Option<Declaration<'tu>> {
        for entity in &mut self.entities {
            if entity.get_kind() == EntityKind::TypedefDecl {
                let name = entity.get_name().unwrap();
                if !self.seen.contains(&name) {
                    let underlying = entity.get_typedef_underlying_type().unwrap();
                    let display = entity.get_type().unwrap().get_display_name();

                    let typedef = !is_elaborated(underlying) ||
                        underlying.get_result_type().is_some() ||
                        is_alias(underlying, &display);

                    if typedef {
                        self.seen.insert(name.clone());
                        return Some(Declaration::new(name, entity, None));
                    }
                }
            }
        }
        None
    }
}

// Unions ________________________________________

/// An iterator over struct declarations.
#[allow(missing_debug_implementations)]
pub struct Unions<'tu> {
    entities: vec::IntoIter<Entity<'tu>>,
    seen: HashSet<String>,
}

impl<'tu> Unions<'tu> {
    //- Constructors -----------------------------

    fn new(entities: vec::IntoIter<Entity<'tu>>) -> Unions<'tu> {
        Unions { entities, seen: HashSet::new() }
    }
}

impl<'tu> Iterator for Unions<'tu> {
    type Item = Declaration<'tu>;

    fn next(&mut self) -> Option<Declaration<'tu>> {
        next(&mut self.entities, &mut self.seen, EntityKind::UnionDecl, "union ")
    }
}

//================================================
// Functions
//================================================

fn is(type_: Type, prefix: &str) -> bool {
    is_elaborated(type_) && type_.get_display_name().starts_with(prefix)
}

fn is_alias(type_: Type, name: &str) -> bool {
    for prefix in &["enum ", "struct ", "union "] {
        let display = type_.get_display_name();

        if display.starts_with(prefix) && &display[prefix.len()..] != name {
            return true;
        }
    }

    false
}

fn is_elaborated(type_: Type) -> bool {
    type_.is_elaborated().unwrap_or(type_.get_kind() == TypeKind::Unexposed)
}

fn next<'tu>(
    entities: &mut vec::IntoIter<Entity<'tu>>,
    seen: &mut HashSet<String>,
    kind: EntityKind,
    prefix: &str,
) -> Option<Declaration<'tu>> {
    for entity in entities {
        if entity.get_kind() == kind {
            if let Some(name) = entity.get_name() {
                if !seen.contains(&name) {
                    seen.insert(name);
                    if entity.get_child(0).is_some() {
                        return Some(Declaration::new(entity.get_name().unwrap(), entity, None));
                    }
                }
            }
        } else if entity.get_kind() == EntityKind::TypedefDecl {
            let underlying = entity.get_typedef_underlying_type().unwrap();
            let name = entity.get_name().unwrap();

            if is(underlying, prefix) && !seen.contains(&name) {
                let declaration = underlying.get_declaration().unwrap();

                let complete = declaration.get_type().map_or(false, |t| t.get_sizeof().is_ok());
                let anonymous = declaration.get_display_name().is_none();
                let same = entity.get_display_name() == declaration.get_display_name();

                seen.insert(name);
                if complete && (anonymous || same) {
                    let name = entity.get_name().unwrap();
                    return Some(Declaration::new(name, declaration, Some(entity)));
                }
            }
        }
    }
    None
}

/// Returns an iterator over the simple preprocessor definitions in the supplied entities.
///
/// Simple preprocessor definitions are those that consist only of a single integer or floating
/// point literal, optionally negated.
///
/// If a preprocessor definition is encountered multiple times, only the first instance is included.
pub fn find_definitions<'tu, E: Into<Vec<Entity<'tu>>>>(entities: E) -> Definitions<'tu> {
    Definitions::new(entities.into().into_iter())
}

/// Returns an iterator over the enums in the supplied entities.
///
/// If an enum is encountered multiple times, only the first instance is included.
pub fn find_enums<'tu, E: Into<Vec<Entity<'tu>>>>(entities: E) -> Enums<'tu> {
    Enums::new(entities.into().into_iter())
}

/// Returns an iterator over the functions in the supplied entities.
///
/// If a function is encountered multiple times, only the first instance is included.
pub fn find_functions<'tu, E: Into<Vec<Entity<'tu>>>>(entities: E) -> Functions<'tu> {
    Functions::new(entities.into().into_iter())
}

/// Returns an iterator over the structs in the supplied entities.
///
/// If a struct is encountered multiple times, only the first instance is included.
pub fn find_structs<'tu, E: Into<Vec<Entity<'tu>>>>(entities: E) -> Structs<'tu> {
    Structs::new(entities.into().into_iter())
}

/// Returns an iterator over the typedefs in the supplied entities.
///
/// If a typedef is encountered multiple times, only the first instance is included.
pub fn find_typedefs<'tu, E: Into<Vec<Entity<'tu>>>>(entities: E) -> Typedefs<'tu> {
    Typedefs::new(entities.into().into_iter())
}

/// Returns an iterator over the unions in the supplied entities.
///
/// If a union is encountered multiple times, only the first instance is included.
pub fn find_unions<'tu, E: Into<Vec<Entity<'tu>>>>(entities: E) -> Unions<'tu> {
    Unions::new(entities.into().into_iter())
}
