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

use super::{Entity, EntityKind, Type, TypeKind};

type Seen = HashSet<String>;

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

fn visit<'tu, F: FnMut(Entity<'tu>)>(
    entities: &[Entity<'tu>], mut f: F, kind: EntityKind, prefix: &str
) -> () {
    let mut seen = Seen::new();

    for entity in entities {
        if entity.get_kind() == kind {
            if let Some(name) = entity.get_name() {
                if !seen.contains(&name) {
                    f(*entity);
                    seen.insert(name);
                }
            }
        } else if entity.get_kind() == EntityKind::TypedefDecl {
            let underlying = entity.get_typedef_underlying_type().unwrap();
            let name = entity.get_name().unwrap();

            if is(underlying, prefix) && !seen.contains(&name) {
                f(*entity);
                seen.insert(name);
            }
        }
    }
}

/// Finds the enums in the supplied entities.
///
/// If an enum is encountered multiple times, only the first instance is collected.
pub fn find_enums<'tu>(entities: &[Entity<'tu>]) -> Vec<Entity<'tu>> {
    let mut enums = vec![];
    visit_enums(entities, |e| enums.push(e));
    enums
}

/// Visits the enums in the supplied entities.
///
/// If an enum is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_enums<'tu, F: FnMut(Entity<'tu>)>(entities: &[Entity<'tu>], f: F) {
    visit(entities, f, EntityKind::EnumDecl, "enum ");
}

/// Finds the functions in the supplied entities.
///
/// If a function is encountered multiple times, only the first instance is collected.
pub fn find_functions<'tu>(entities: &[Entity<'tu>]) -> Vec<Entity<'tu>> {
    let mut functions = vec![];
    visit_functions(entities, |e| functions.push(e));
    functions
}

/// Visits the functions in the supplied entities.
///
/// If a function is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_functions<'tu, F: FnMut(Entity<'tu>)>(entities: &[Entity<'tu>], mut f: F) {
    let mut seen = Seen::new();

    for entity in entities.iter().filter(|e| e.get_kind() == EntityKind::FunctionDecl) {
        let name = entity.get_name().unwrap();

        if !seen.contains(&name) {
            f(*entity);
            seen.insert(name);
        }
    }
}

/// Finds the structs in the supplied entities.
///
/// If a struct is encountered multiple times, only the first instance is collected.
pub fn find_structs<'tu>(entities: &[Entity<'tu>]) -> Vec<Entity<'tu>> {
    let mut structs = vec![];
    visit_structs(entities, |e| structs.push(e));
    structs
}

/// Visits the structs in the supplied entities.
///
/// If a struct is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_structs<'tu, F: FnMut(Entity<'tu>)>(entities: &[Entity<'tu>], f: F) {
    visit(entities, f, EntityKind::StructDecl, "struct ");
}

/// Finds the typedefs in the supplied entities.
///
/// If a typedef is encountered multiple times, only the first instance is collected.
pub fn find_typedefs<'tu>(entities: &[Entity<'tu>]) -> Vec<Entity<'tu>> {
    let mut typedefs = vec![];
    visit_typedefs(entities, |e| typedefs.push(e));
    typedefs
}

/// Visits the typedefs in the supplied entities.
///
/// If a typedef is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_typedefs<'tu, F: FnMut(Entity<'tu>)>(entities: &[Entity<'tu>], mut f: F) {
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
                f(*entity);
                seen.insert(name);
            }
        }
    }
}

/// Finds the unions in the supplied entities.
///
/// If a union is encountered multiple times, only the first instance is collected.
pub fn find_unions<'tu>(entities: &[Entity<'tu>]) -> Vec<Entity<'tu>> {
    let mut unions = vec![];
    visit_unions(entities, |e| unions.push(e));
    unions
}

/// Visits the unions in the supplied entities.
///
/// If a union is encountered multiple times, only the first instance is visited.
#[cfg_attr(feature="clippy", allow(needless_lifetimes))]
pub fn visit_unions<'tu, F: FnMut(Entity<'tu>)>(entities: &[Entity<'tu>], f: F) {
    visit(entities, f, EntityKind::UnionDecl, "union ");
}
