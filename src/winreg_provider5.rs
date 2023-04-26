// reggie - fast Windows registry searcher
// Copyright (C) 2023  Donny Johnson
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
use std::sync::Arc;
use std::sync::Mutex;

use string_interner::backend::BufferBackend;
use string_interner::symbol::SymbolU32;
use winreg::RegKey;
use winreg::enums::*;
use indexmap::IndexSet;
use rayon::prelude::*;
use string_interner::StringInterner;

pub fn get_all(hive: isize) -> IndexSet<String> {
    let root = RegKey::predef(hive);
    let root_name = match hive {
        HKEY_CLASSES_ROOT => String::from("HKCR"),
        HKEY_CURRENT_CONFIG => String::from("HKCC"),
        HKEY_CURRENT_USER => String::from("HKCU"),
        HKEY_CURRENT_USER_LOCAL_SETTINGS => String::from("HKCULL"),
        HKEY_DYN_DATA => String::from("HKDD"),
        HKEY_LOCAL_MACHINE => String::from("HKLM"),
        HKEY_PERFORMANCE_DATA => String::from("HKPD"),
        HKEY_PERFORMANCE_NLSTEXT => String::from("HKPL"),
        HKEY_PERFORMANCE_TEXT => String::from("HKPT"),
        HKEY_USERS => String::from("HKU"),
        _ => String::from("(unknown)"),
    };
    let output = Arc::new(Mutex::new(IndexSet::new()));
    let interner: StringInterner<BufferBackend> = StringInterner::new();
    let interner = Arc::new(Mutex::new(interner));
    get_all_with_prefix(root, root_name, Arc::clone(&output), &mut Arc::clone(&interner));
    let interner = Arc::try_unwrap(interner).unwrap().into_inner().unwrap();
    let output = Arc::try_unwrap(output).unwrap().into_inner().unwrap();
    let mut strings = IndexSet::new();
    for symbol in output {
        let string = interner.resolve(symbol).unwrap();
        strings.insert(string.to_string());
    }
    strings.sort();
    strings
}

fn get_all_with_prefix(root: RegKey, prefix: String, set: Arc<Mutex<IndexSet<SymbolU32>>>, interner: &mut Arc<Mutex<StringInterner<BufferBackend>>>) {
    {
        let mut lock_interner = interner.lock().unwrap();
        let s = lock_interner.get_or_intern(&prefix);
        let mut lock = set.lock().unwrap();
        lock.insert(s);
    }
    let keys = root.enum_keys();
    let keys = keys.par_bridge().filter_map(|x| x.ok());
    keys.for_each(|key| {
        let subkey = root.open_subkey(&key);
        if subkey.is_err() {
            return;
        }
        let subkey = subkey.unwrap();
        let path = format!("{}\\{}", prefix, key);
        get_all_with_prefix(subkey, path, Arc::clone(&set), &mut Arc::clone(&interner));
    });
}
