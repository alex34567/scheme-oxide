/*
    Copyright 2018 Alexander Eckhart

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
pub mod pair;
pub use self::pair::SchemePair;
use crate::interperter::FunctionRef;
mod object;
pub use self::object::SchemeObject;

macro_rules! gen_singleton {
    (pub $name:ident) => {
        pub fn $name() -> SchemeObject {
            thread_local! {
                static SINGLETON: SchemeObject = SchemeObject::unique_new()
            }
            SINGLETON.with(|s| s.clone())
        }
    };
}

gen_singleton!(pub get_empty_list);
gen_singleton!(pub get_immutable_pair_type_id);
gen_singleton!(pub get_mutable_pair_type_id);
gen_singleton!(pub get_true);
gen_singleton!(pub get_false);
gen_singleton!(pub get_symbol_type_id);

pub fn new_symbol(name: String) -> SchemeObject {
    thread_local! {
        static NAME_TO_SYM_MAP: RefCell<HashMap<String, SchemeObject>> = RefCell::new(HashMap::new())
    }

    NAME_TO_SYM_MAP.with(|raw_sym_map| {
        let mut sym_map = raw_sym_map.borrow_mut();

        sym_map
            .entry(name.clone())
            .or_insert_with(|| {
                SchemeObject::new(get_symbol_type_id().into(), vec![SchemeType::String(name)])
            })
            .clone()
    })
}

#[derive(Clone, PartialEq, Debug)]
pub enum SchemeType {
    Function(FunctionRef),
    Number(i64),
    String(String),
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

    pub fn into_object(self) -> Result<SchemeObject, CastError> {
        if let SchemeType::Object(obj) = self {
            Ok(obj)
        } else {
            Err(CastError)
        }
    }

    pub fn to_bool(&self) -> bool {
        !(*self == get_false().into())
    }

    pub fn to_function(&self) -> Result<FunctionRef, CastError> {
        Ok(match self {
            SchemeType::Function(func) => func.clone(),
            _ => return Err(CastError),
        })
    }
}

impl From<SchemePair> for SchemeType {
    fn from(pair: SchemePair) -> Self {
        pair.into_object().into()
    }
}

impl From<Option<SchemePair>> for SchemeType {
    fn from(pair: Option<SchemePair>) -> SchemeType {
        match pair {
            Some(x) => x.into(),
            None => get_empty_list().into(),
        }
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

impl From<bool> for SchemeType {
    fn from(is_true: bool) -> Self {
        if is_true {
            get_true().into()
        } else {
            get_false().into()
        }
    }
}
