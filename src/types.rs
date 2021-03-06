/*
    Copyright 2018-2019 Alexander Eckhart

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

use std::cell::RefCell;
use std::collections::HashMap;

use crate::environment;
use crate::interpreter::FunctionRef;

pub use self::object::SchemeObject;
pub use self::string::SchemeString;
pub use self::string::StringSetError;

mod object;
mod string;

pub fn new_symbol(name: String) -> SchemeObject {
    thread_local! {
        static NAME_TO_SYM_MAP: RefCell<HashMap<String, SchemeObject>> = RefCell::new(HashMap::new())
    }

    NAME_TO_SYM_MAP.with(|raw_sym_map| {
        let mut sym_map = raw_sym_map.borrow_mut();

        sym_map
            .entry(name.clone())
            .or_insert_with(|| {
                SchemeObject::new(
                    environment::symbol_type_id(),
                    vec![SchemeType::String(name.parse().unwrap())],
                )
            })
            .clone()
    })
}

#[derive(Clone, Debug)]
pub struct ListFactory {
    push_fn: FunctionRef,
    res_fn: FunctionRef,
}

impl ListFactory {
    pub fn new(mutable: bool) -> Self {
        let list_factory = environment::make_list_factory(mutable.into()).unwrap();
        let push_fn = environment::car(list_factory.clone())
            .unwrap()
            .to_function()
            .unwrap();
        let res_fn = environment::cdr(list_factory)
            .unwrap()
            .to_function()
            .unwrap();

        Self { push_fn, res_fn }
    }

    pub fn push(&mut self, object: SchemeType) {
        self.push_fn.clone().call(vec![object]).unwrap();
    }

    pub fn build(self) -> SchemeType {
        self.build_with_tail(environment::empty_list())
    }

    pub fn build_with_tail(self, object: SchemeType) -> SchemeType {
        self.res_fn.call(vec![object]).unwrap()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum SchemeType {
    Function(FunctionRef),
    Number(i64),
    Char(char),
    String(SchemeString),
    Object(SchemeObject),
}

#[derive(Clone, Debug)]
pub struct CastError;

impl SchemeType {
    pub fn to_number(&self) -> Result<i64, CastError> {
        if let SchemeType::Number(num) = self {
            Ok(*num)
        } else {
            Err(CastError)
        }
    }

    pub fn to_index(&self) -> Result<usize, CastError> {
        let raw_num = self.to_number()?;
        //Indexes need to be positive
        if raw_num < 0 {
            return Err(CastError);
        }
        let num = raw_num as u64;

        //On 32-bit platforms make sure that the index does not overflow.
        //Should be optimized to a no-op on 64-bit platforms.
        if num > (usize::max_value() as u64) {
            Err(CastError)
        } else {
            Ok(num as usize)
        }
    }

    pub fn to_char(&self) -> Result<char, CastError> {
        if let SchemeType::Char(c) = self {
            Ok(*c)
        } else {
            Err(CastError)
        }
    }

    pub fn into_object(self) -> Result<SchemeObject, CastError> {
        if let SchemeType::Object(obj) = self {
            Ok(obj)
        } else {
            Err(CastError)
        }
    }

    pub fn into_string(self) -> Result<SchemeString, CastError> {
        if let SchemeType::String(stri) = self {
            Ok(stri)
        } else {
            Err(CastError)
        }
    }

    pub fn to_bool(&self) -> bool {
        *self != environment::s_false()
    }

    pub fn to_function(&self) -> Result<FunctionRef, CastError> {
        Ok(match self {
            SchemeType::Function(func) => func.clone(),
            _ => return Err(CastError),
        })
    }
}

impl From<FunctionRef> for SchemeType {
    fn from(func: FunctionRef) -> Self {
        SchemeType::Function(func)
    }
}

impl From<SchemeObject> for SchemeType {
    fn from(object: SchemeObject) -> Self {
        SchemeType::Object(object)
    }
}

impl From<SchemeString> for SchemeType {
    fn from(string: SchemeString) -> Self {
        SchemeType::String(string)
    }
}

impl From<bool> for SchemeType {
    fn from(is_true: bool) -> Self {
        if is_true {
            environment::s_true()
        } else {
            environment::s_false()
        }
    }
}

impl From<usize> for SchemeType {
    fn from(index: usize) -> SchemeType {
        if (index as u64) > (i64::max_value() as u64) {
            panic!("Overflow")
        }

        SchemeType::Number(index as i64)
    }
}
