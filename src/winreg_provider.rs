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
use winreg::RegKey;
use winreg::enums::*;
use indexmap::IndexSet;

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
    let mut output = IndexSet::new();
    get_all_with_prefix(root, root_name, &mut output);
    output
}

fn get_all_with_prefix(root: RegKey, prefix: String, set: &mut IndexSet<String>) {
    set.insert(prefix.clone());
    let keys = root.enum_keys();
    let keys: Vec<String> = keys.filter_map(|x| x.ok()).collect();
    for key in &keys {
        let subkey = root.open_subkey(key);
        if subkey.is_err() {
            continue;
        }
        let subkey = subkey.unwrap();
        let path = format!("{}\\{}", prefix, key);
        get_all_with_prefix(subkey, path, set);
    }
}
