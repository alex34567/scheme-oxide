/*
    Copyright 2019 Alexander Eckhart

    This file is part of scheme-oxide.

    Scheme-oxide is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Scheme-oxide is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with scheme-oxide.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::sync::atomic::{AtomicU64, Ordering};

use AstNodeInner::*;
use AstNodeNonList::{Bool, Number, String as SchemeString, Symbol};

use crate::environment;
use crate::types::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CoreSymbol {
    And,
    Begin,
    Or,
    Let,
    LetRec,
    LetStar,
    Lambda,
    If,
    Set,
    Error,
    Quote,
    BeginProgram,
    GenUnspecified,
}

impl CoreSymbol {
    pub fn get_name(self) -> &'static str {
        match self {
            CoreSymbol::And => "and",
            CoreSymbol::Begin => "begin",
            CoreSymbol::Or => "or",
            CoreSymbol::Let => "let",
            CoreSymbol::LetRec => "letrec",
            CoreSymbol::LetStar => "let*",
            CoreSymbol::Lambda => "lambda",
            CoreSymbol::If => "if",
            CoreSymbol::Set => "set",
            CoreSymbol::Error => "error",
            CoreSymbol::Quote => "quote",
            CoreSymbol::BeginProgram => "$begin-program",
            CoreSymbol::GenUnspecified => "$gen_unspecified",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum AstSymbolInner {
    Core(CoreSymbol),
    Temp(u64),
    Defined(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AstSymbol(AstSymbolInner);

impl AstSymbol {
    pub fn new(name: &str) -> AstSymbol {
        AstSymbol(AstSymbolInner::Defined(name.to_string()))
    }

    pub fn gen_temp() -> AstSymbol {
        static TEMP_COUNT: AtomicU64 = AtomicU64::new(0);

        let count = TEMP_COUNT.fetch_add(1, Ordering::Relaxed);

        AstSymbol(AstSymbolInner::Temp(count))
    }

    pub fn get_name(&self) -> String {
        match &self.0 {
            AstSymbolInner::Core(core) => core.get_name().to_string(),
            AstSymbolInner::Temp(id) => format!("$temp$id{}", id),
            AstSymbolInner::Defined(name) => name.clone(),
        }
    }
}

impl From<CoreSymbol> for AstSymbol {
    fn from(core: CoreSymbol) -> AstSymbol {
        AstSymbol(AstSymbolInner::Core(core))
    }
}

#[derive(Clone, Debug, PartialEq)]
enum ListType {
    Proper,
    Improper(AstNodeNonList),
}

impl ListType {
    fn is_proper_list(&self) -> bool {
        !self.is_improper_list()
    }

    fn is_improper_list(&self) -> bool {
        if let ListType::Improper(_) = self {
            true
        } else {
            false
        }
    }

    fn into_node(self) -> AstNode {
        match self {
            ListType::Proper => AstNode(AstNodeInner::List(AstList::none())),
            ListType::Improper(x) => AstNode(AstNodeInner::NonList(x)),
        }
    }

    fn to_datum(&self) -> SchemeType {
        match self {
            ListType::Proper => environment::empty_list(),
            ListType::Improper(node) => AstNode::from_non_list(node.clone()).to_datum(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstList {
    nodes: Vec<AstNode>,
    list_type: ListType,
}

impl AstList {
    pub fn none() -> AstList {
        AstList {
            nodes: Vec::new(),
            list_type: ListType::Proper,
        }
    }

    pub fn one(node: AstNode) -> AstList {
        AstList {
            nodes: vec![node],
            list_type: ListType::Proper,
        }
    }

    pub fn is_proper_list(&self) -> bool {
        self.list_type.is_proper_list()
    }

    pub fn is_empty_list(&self) -> bool {
        self.is_proper_list() && self.nodes.is_empty()
    }

    pub fn is_improper_list(&self) -> bool {
        self.list_type.is_improper_list()
    }

    pub fn as_nodes(&self) -> &[AstNode] {
        &self.nodes
    }

    pub fn into_inner(self) -> (Vec<AstNode>, AstNode) {
        (self.nodes, self.list_type.into_node())
    }
}

impl From<Vec<AstNode>> for AstList {
    fn from(list: Vec<AstNode>) -> AstList {
        AstList {
            nodes: list,
            list_type: ListType::Proper,
        }
    }
}

pub struct AstListBuilder {
    nodes: Vec<AstNode>,
}

impl AstListBuilder {
    pub fn new() -> AstListBuilder {
        AstListBuilder { nodes: Vec::new() }
    }

    pub fn push(&mut self, node: AstNode) {
        self.nodes.push(node)
    }

    fn build_with_type(mut self, list_type: ListType) -> AstList {
        self.nodes.shrink_to_fit();
        AstList {
            nodes: self.nodes,
            list_type,
        }
    }

    pub fn build(self) -> AstList {
        self.build_with_type(ListType::Proper)
    }

    pub fn build_with_tail(mut self, node: AstNode) -> Option<AstList> {
        match node.0 {
            AstNodeInner::List(mut list) => {
                if self.nodes.is_empty() && list.is_improper_list() {
                    return None;
                }

                self.nodes.append(&mut list.nodes);

                Some(self.build_with_type(list.list_type))
            }
            AstNodeInner::NonList(node) => Some(self.build_with_type(ListType::Improper(node))),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum AstNodeNonList {
    Number(i64),
    Symbol(AstSymbol),
    String(String),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq)]
enum AstNodeInner {
    List(AstList),
    NonList(AstNodeNonList),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AstNode(AstNodeInner);

impl AstNode {
    fn from_non_list(non_list: AstNodeNonList) -> AstNode {
        AstNode(NonList(non_list))
    }

    pub fn from_number(number: i64) -> AstNode {
        Self::from_non_list(Number(number))
    }

    pub fn from_string(string: String) -> AstNode {
        Self::from_non_list(SchemeString(string))
    }

    pub fn from_bool(boolean: bool) -> AstNode {
        Self::from_non_list(Bool(boolean))
    }

    pub fn to_datum(&self) -> SchemeType {
        match &self.0 {
            NonList(Number(x)) => SchemeType::Number(*x),
            NonList(Symbol(sym)) => new_symbol(sym.get_name()).into(),
            NonList(SchemeString(stri)) => SchemeType::String(stri.clone().parse().unwrap()),
            List(list) => {
                let mut builder = ListFactory::new(false);

                for node in list.nodes.iter() {
                    builder.push(node.to_datum())
                }

                builder.build_with_tail(list.list_type.to_datum())
            }
            NonList(Bool(is_true)) => (*is_true).into(),
        }
    }

    pub fn as_list(&self) -> Option<&AstList> {
        if let List(list) = &self.0 {
            Some(list)
        } else {
            None
        }
    }

    pub fn as_proper_list(&self) -> Option<&[AstNode]> {
        self.as_list()
            .filter(|x| x.is_proper_list())
            .map(AstList::as_nodes)
    }

    pub fn as_symbol(&self) -> Option<&AstSymbol> {
        if let NonList(Symbol(sym)) = &self.0 {
            Some(sym)
        } else {
            None
        }
    }

    pub fn into_symbol(self) -> Result<AstSymbol, AstNode> {
        if let NonList(Symbol(sym)) = self.0 {
            Ok(sym)
        } else {
            Err(self)
        }
    }

    pub fn into_list(self) -> Result<AstList, AstNode> {
        if let List(list) = self.0 {
            Ok(list)
        } else {
            Err(self)
        }
    }

    pub fn into_proper_list(self) -> Result<Vec<AstNode>, AstNode> {
        let list = self.into_list()?;

        if !list.is_proper_list() {
            return Err(AstNode(List(list)));
        }

        Ok(list.into_inner().0)
    }

    pub fn is_improper_list(&self) -> bool {
        if let Some(list) = self.as_list() {
            list.is_improper_list()
        } else {
            false
        }
    }

    pub fn get_name(&self) -> &'static str {
        match &self.0 {
            NonList(Number(_)) => "number",
            NonList(Symbol(_)) => "symbol",
            NonList(SchemeString(_)) => "string",
            List(list) => {
                if list.is_improper_list() {
                    "improper list"
                } else {
                    "proper list"
                }
            }
            NonList(Bool(_)) => "boolean",
        }
    }
}

impl From<CoreSymbol> for AstNode {
    fn from(sym: CoreSymbol) -> AstNode {
        let ast_sym: AstSymbol = sym.into();
        ast_sym.into()
    }
}

impl From<AstSymbol> for AstNode {
    fn from(sym: AstSymbol) -> AstNode {
        AstNode::from_non_list(Symbol(sym))
    }
}

impl From<AstList> for AstNode {
    fn from(list: AstList) -> AstNode {
        AstNode(AstNodeInner::List(list))
    }
}

impl From<Vec<AstNode>> for AstNode {
    fn from(list: Vec<AstNode>) -> AstNode {
        let list_object: AstList = list.into();
        list_object.into()
    }
}
