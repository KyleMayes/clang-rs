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

use std::collections::{HashSet};
use std::str::{FromStr};

use super::{Entity, EntityKind, Type, TypeKind};

type Seen = HashSet<String>;

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
        Declaration { name: name, entity: entity, source: source }
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
        Definition { name: name, value: value, entity: entity }
    }
}

//================================================
// Functions
//================================================

fn is(type_: Type, prefix: &str) -> bool {
    type_.get_display_name().starts_with(prefix) && !is_function(type_) && !is_pointer(type_)
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

fn is_function(type_: Type) -> bool {
    let kind = type_.get_kind();
    kind == TypeKind::FunctionPrototype || kind == TypeKind::FunctionNoPrototype
}

fn is_pointer(type_: Type) -> bool {
    type_.get_kind() == TypeKind::Pointer
}

fn visit<'tu, F: FnMut(Declaration<'tu>)>(
    entities: &[Entity<'tu>], mut f: F, kind: EntityKind, prefix: &str
) -> () {
    let mut seen = Seen::new();

    for entity in entities {
        if entity.get_kind() == kind {
            if let Some(name) = entity.get_name() {
                if !seen.contains(&name) {
                    f(Declaration::new(entity.get_name().unwrap(), *entity, None));
                    seen.insert(name);
                }
            }
        } else if entity.get_kind() == EntityKind::TypedefDecl {
            let underlying = entity.get_typedef_underlying_type().unwrap();
            let name = entity.get_name().unwrap();
            let declaration = underlying.get_declaration().unwrap();

            if is(underlying, prefix) && !seen.contains(&name) {
                f(Declaration::new(entity.get_name().unwrap(), declaration, Some(*entity)));
                seen.insert(name);
            }
        }
    }
}

/// Visits the simple preprocessor definitions in the supplied entities.
///
/// Simple preprocessor definitions are those that consist only of a single integer or floating
/// point literal, optionally negated.
///
/// If a preprocessor definition is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_definitions<'tu, F: FnMut(Definition<'tu>)>(entities: &[Entity<'tu>], mut f: F) {
    let mut seen = Seen::new();

    for entity in entities.iter().filter(|e| e.get_kind() == EntityKind::MacroDefinition) {
        let name = entity.get_name().unwrap();
        let range = entity.get_range().unwrap();

        if !seen.contains(&name) && !range.get_start().is_in_system_header() {
            if let Some(value) = DefinitionValue::from_entity(*entity) {
                f(Definition::new(name.clone(), value, *entity));
                seen.insert(name);
            }
        }
    }
}

/// Finds the simple preprocessor definitions in the supplied entities.
///
/// See `visit_definitions` for more information.
pub fn find_definitions<'tu>(entities: &[Entity<'tu>]) -> Vec<Definition<'tu>> {
    let mut definitions = vec![];
    visit_definitions(entities, |d| definitions.push(d));
    definitions
}

/// Returns the enums in the supplied entities.
///
/// If an enum is encountered multiple times, only the first instance is collected.
pub fn find_enums<'tu>(entities: &[Entity<'tu>]) -> Vec<Declaration<'tu>> {
    let mut enums = vec![];
    visit_enums(entities, |e| enums.push(e));
    enums
}

/// Visits the enums in the supplied entities.
///
/// If an enum is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_enums<'tu, F: FnMut(Declaration<'tu>)>(entities: &[Entity<'tu>], f: F) {
    visit(entities, f, EntityKind::EnumDecl, "enum ");
}

/// Returns the functions in the supplied entities.
///
/// If a function is encountered multiple times, only the first instance is collected.
pub fn find_functions<'tu>(entities: &[Entity<'tu>]) -> Vec<Declaration<'tu>> {
    let mut functions = vec![];
    visit_functions(entities, |e| functions.push(e));
    functions
}

/// Visits the functions in the supplied entities.
///
/// If a function is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_functions<'tu, F: FnMut(Declaration<'tu>)>(entities: &[Entity<'tu>], mut f: F) {
    let mut seen = Seen::new();

    for entity in entities.iter().filter(|e| e.get_kind() == EntityKind::FunctionDecl) {
        let name = entity.get_name().unwrap();

        if !seen.contains(&name) {
            f(Declaration::new(name.clone(), *entity, None));
            seen.insert(name);
        }
    }
}

/// Returns the structs in the supplied entities.
///
/// If a struct is encountered multiple times, only the first instance is collected.
pub fn find_structs<'tu>(entities: &[Entity<'tu>]) -> Vec<Declaration<'tu>> {
    let mut structs = vec![];
    visit_structs(entities, |e| structs.push(e));
    structs
}

/// Visits the structs in the supplied entities.
///
/// If a struct is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_structs<'tu, F: FnMut(Declaration<'tu>)>(entities: &[Entity<'tu>], f: F) {
    visit(entities, f, EntityKind::StructDecl, "struct ");
}

/// Returns the typedefs in the supplied entities.
///
/// If a typedef is encountered multiple times, only the first instance is collected.
pub fn find_typedefs<'tu>(entities: &[Entity<'tu>]) -> Vec<Declaration<'tu>> {
    let mut typedefs = vec![];
    visit_typedefs(entities, |e| typedefs.push(e));
    typedefs
}

/// Visits the typedefs in the supplied entities.
///
/// If a typedef is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_typedefs<'tu, F: FnMut(Declaration<'tu>)>(entities: &[Entity<'tu>], mut f: F) {
    let mut seen = Seen::new();

    for entity in entities.iter().filter(|e| e.get_kind() == EntityKind::TypedefDecl) {
        let name = entity.get_name().unwrap();

        if !seen.contains(&name) {
            let underlying = entity.get_typedef_underlying_type().unwrap();
            let display = entity.get_type().unwrap().get_display_name();

            let typedef = underlying.get_kind() != TypeKind::Unexposed ||
                underlying.get_result_type().is_some() ||
                is_alias(underlying, &display);

            if typedef {
                f(Declaration::new(name.clone(), *entity, None));
                seen.insert(name);
            }
        }
    }
}

/// Returns the unions in the supplied entities.
///
/// If a union is encountered multiple times, only the first instance is collected.
pub fn find_unions<'tu>(entities: &[Entity<'tu>]) -> Vec<Declaration<'tu>> {
    let mut unions = vec![];
    visit_unions(entities, |e| unions.push(e));
    unions
}

/// Visits the unions in the supplied entities.
///
/// If a union is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_unions<'tu, F: FnMut(Declaration<'tu>)>(entities: &[Entity<'tu>], f: F) {
    visit(entities, f, EntityKind::UnionDecl, "union ");
}
