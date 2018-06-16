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

//! Comments.

#![allow(unused_unsafe)]

use std::fmt;
use std::mem;
use std::marker::{PhantomData};

use clang_sys::*;

use utility;
use super::{TranslationUnit};

//================================================
// Enums
//================================================

// CommentChild __________________________________

/// A child component of a comment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommentChild {
    /// A block command with zero or more arguments and a paragraph as an argument.
    BlockCommand(BlockCommand),
    /// An HTML start tag.
    HtmlStartTag(HtmlStartTag),
    /// An HTML end tag.
    HtmlEndTag(String),
    /// An inline command with word-like arguments.
    InlineCommand(InlineCommand),
    /// A paragraph containing inline content.
    Paragraph(Vec<CommentChild>),
    /// A `\param` command.
    ParamCommand(ParamCommand),
    /// A `\tparam` command.
    TParamCommand(TParamCommand),
    /// Plain text.
    Text(String),
    /// A verbatim command with a closing command.
    VerbatimCommand(Vec<String>),
    /// A verbatim command with a single line and no closing command.
    VerbatimLineCommand(String),
}

impl CommentChild {
    //- Constructors -----------------------------

    fn from_raw(raw: CXComment) -> CommentChild {
        unsafe {
            match clang_Comment_getKind(raw) {
                CXComment_Text =>
                    CommentChild::Text(utility::to_string(clang_TextComment_getText(raw))),
                CXComment_InlineCommand =>
                    CommentChild::InlineCommand(InlineCommand::from_raw(raw)),
                CXComment_HTMLStartTag => CommentChild::HtmlStartTag(HtmlStartTag::from_raw(raw)),
                CXComment_HTMLEndTag => {
                    let name = utility::to_string(clang_HTMLTagComment_getTagName(raw));
                    CommentChild::HtmlEndTag(name)
                },
                CXComment_Paragraph =>
                    CommentChild::Paragraph(Comment::from_raw(raw).get_children()),
                CXComment_BlockCommand => CommentChild::BlockCommand(BlockCommand::from_raw(raw)),
                CXComment_ParamCommand => CommentChild::ParamCommand(ParamCommand::from_raw(raw)),
                CXComment_TParamCommand =>
                    CommentChild::TParamCommand(TParamCommand::from_raw(raw)),
                CXComment_VerbatimBlockCommand => {
                    let lines = iter!(
                        clang_Comment_getNumChildren(raw),
                        clang_Comment_getChild(raw),
                    ).map(|c| {
                        utility::to_string(clang_VerbatimBlockLineComment_getText(c))
                    }).collect();
                    CommentChild::VerbatimCommand(lines)
                },
                CXComment_VerbatimLine => {
                    let line = utility::to_string(clang_VerbatimLineComment_getText(raw));
                    CommentChild::VerbatimLineCommand(line)
                },
                _ => unreachable!(),
            }
        }
    }
}

// ParameterDirection ____________________________

/// Indicates the parameter passing direction for a `\param` command.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum ParameterDirection {
    /// Indicates the parameter is an input parameter.
    In = 0,
    /// Indicates the parameter is an output parameter.
    Out = 1,
    /// Indicates the parameter is both an input and an output parameter.
    InOut = 2,
}

// InlineCommandStyle ____________________________

/// Indicates the appropriate rendering style for an inline command argument.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum InlineCommandStyle {
    /// Indicates the command should be rendered in a bold font.
    Bold = 1,
    /// Indicates the command should be rendered in a monospace font.
    Monospace = 2,
    /// Indicates the command should be rendered emphasized (typically italicized).
    Emphasized = 3,
}

//================================================
// Structs
//================================================

// BlockCommand __________________________________

/// A block command with zero or more arguments and a paragraph as an argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockCommand {
    /// The command.
    pub command: String,
    /// The command arguments.
    pub arguments: Vec<String>,
    /// The children of the paragraph argument.
    pub children: Vec<CommentChild>,
}

impl BlockCommand {
    //- Constructors -----------------------------

    unsafe fn from_raw(raw: CXComment) -> BlockCommand {
        let command = utility::to_string(clang_BlockCommandComment_getCommandName(raw));
        let arguments = iter!(
            clang_BlockCommandComment_getNumArgs(raw),
            clang_BlockCommandComment_getArgText(raw),
        ).map(utility::to_string).collect();
        let paragraph = clang_BlockCommandComment_getParagraph(raw);
        let children = Comment::from_raw(paragraph).get_children();
        BlockCommand { command, arguments, children }
    }
}

// Comment _______________________________________

/// A comment attached to a declaration.
#[derive(Copy, Clone)]
pub struct Comment<'tu> {
    raw: CXComment,
    _marker: PhantomData<&'tu TranslationUnit<'tu>>,
}

impl<'tu> Comment<'tu> {
    //- Constructors -----------------------------

    #[doc(hidden)]
    pub fn from_raw(raw: CXComment) -> Comment<'tu> {
        Comment { raw, _marker: PhantomData }
    }

    //- Accessors --------------------------------

    /// Returns the children of this comment.
    pub fn get_children(&self) -> Vec<CommentChild> {
        iter!(
            clang_Comment_getNumChildren(self.raw),
            clang_Comment_getChild(self.raw),
        ).map(CommentChild::from_raw).collect()
    }

    /// Returns this comment as an HTML string.
    pub fn as_html(&self) -> String {
        unsafe { utility::to_string(clang_FullComment_getAsHTML(self.raw)) }
    }

    /// Returns this comment as an XML string.
    pub fn as_xml(&self) -> String {
        unsafe { utility::to_string(clang_FullComment_getAsXML(self.raw)) }
    }
}

impl<'tu> fmt::Debug for Comment<'tu> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.get_children())
    }
}

// HtmlStartTag __________________________________

/// An HTML start tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HtmlStartTag {
    /// The tag name.
    pub name: String,
    /// The attributes associated with the tag, if any.
    pub attributes: Vec<(String,  String)>,
    /// Whether the tag is self-closing.
    pub closing: bool,
}

impl HtmlStartTag {
    //- Constructors -----------------------------

    unsafe fn from_raw(raw: CXComment) -> HtmlStartTag {
        let name = utility::to_string(clang_HTMLTagComment_getTagName(raw));
        let attributes = iter!(
            clang_HTMLStartTag_getNumAttrs(raw),
            clang_HTMLStartTag_getAttrName(raw),
            clang_HTMLStartTag_getAttrValue(raw),
        ).map(|(n, v)| (utility::to_string(n), utility::to_string(v))).collect();
        let closing = clang_HTMLStartTagComment_isSelfClosing(raw) != 0;
        HtmlStartTag { name, attributes, closing }
    }
}

// InlineCommand _________________________________

/// An inline command with word-like arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InlineCommand {
    /// The command.
    pub command: String,
    /// The command arguments.
    pub arguments: Vec<String>,
    /// The style with which to render the command arguments, if any.
    pub style: Option<InlineCommandStyle>,
}

impl InlineCommand {
    //- Constructors -----------------------------

    unsafe fn from_raw(raw: CXComment) -> InlineCommand {
        let command = utility::to_string(clang_InlineCommandComment_getCommandName(raw));
        let arguments = iter!(
            clang_InlineCommandComment_getNumArgs(raw),
            clang_InlineCommandComment_getArgText(raw),
        ).map(utility::to_string).collect();
        let style = match clang_InlineCommandComment_getRenderKind(raw) {
            CXCommentInlineCommandRenderKind_Normal => None,
            other => Some(mem::transmute(other)),
        };
        InlineCommand { command, arguments, style }
    }
}

// ParamCommand __________________________________

/// A `\param` command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParamCommand {
    /// The index of the parameter, if this command refers to a valid parameter.
    pub index: Option<usize>,
    /// The parameter.
    pub parameter: String,
    /// The parameter direction, if specified.
    pub direction: Option<ParameterDirection>,
    /// The children of this parameter.
    pub children: Vec<CommentChild>
}

impl ParamCommand {
    //- Constructors -----------------------------

    unsafe fn from_raw(raw: CXComment) -> ParamCommand {
        let index = if clang_ParamCommandComment_isParamIndexValid(raw) != 0 {
            Some(clang_ParamCommandComment_getParamIndex(raw) as usize)
        } else {
            None
        };
        let parameter = utility::to_string(clang_ParamCommandComment_getParamName(raw));
        let direction = if clang_ParamCommandComment_isDirectionExplicit(raw) != 0 {
            Some(mem::transmute(clang_ParamCommandComment_getDirection(raw)))
        } else {
            None
        };
        let paragraph = clang_BlockCommandComment_getParagraph(raw);
        let children = Comment::from_raw(paragraph).get_children();
        ParamCommand { index, parameter, direction, children }
    }
}

// TParamCommand _________________________________

/// A `\tparam` command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TParamCommand {
    /// The nesting depth and the index of the template parameter, if this command refers to a
    /// valid template parameter.
    pub position: Option<(usize, usize)>,
    /// The template parameter.
    pub parameter: String,
    /// The children of this type parameter.
    pub children: Vec<CommentChild>
}

impl TParamCommand {
    //- Constructors -----------------------------

    unsafe fn from_raw(raw: CXComment) -> TParamCommand {
        let position = if clang_TParamCommandComment_isParamPositionValid(raw) != 0 {
            let depth = clang_TParamCommandComment_getDepth(raw);
            let index = clang_TParamCommandComment_getIndex(raw, depth) as usize;
            Some((depth as usize, index))
        } else {
            None
        };
        let parameter = utility::to_string(clang_TParamCommandComment_getParamName(raw));
        let paragraph = clang_BlockCommandComment_getParagraph(raw);
        let children = Comment::from_raw(paragraph).get_children();
        TParamCommand { position, parameter, children }
    }
}
