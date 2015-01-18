#![allow(unstable)]
extern crate "readline" as rl;
extern crate libc;

use std::ffi::c_str_to_bytes;
use std::io::stdio::println;
use std::str;

fn complete(text: String) -> Vec<String> {
    let mut entries: Vec<String> = Vec::new();
    entries.push(text.clone() + "s");
    entries.push(text + "zz");
    return entries;
}

extern fn rl_compentry_func(text: *const i8, state: i32) -> *const i8 {
    if state == 0 {
        let txt = unsafe { c_str_to_bytes(&text) };
        let entries = complete(str::from_utf8(txt).unwrap().to_string());
        rl::set_compentries(entries);
    }
    rl::get_compentry(state as usize)
}

extern fn my_attempted_completion_function(text: *const i8, _start: i32, _end: i32) -> *mut *const i8 {
    return rl::rl_completion_matches(text, rl_compentry_func)
}

pub fn main() {
    rl::rl_initialize().unwrap();
    //println!("{}", rl::rl_readline_version())
    println!("{}", rl::rl_library_version());

    rl::set_rl_attempted_completion_function(Some(my_attempted_completion_function as rl::CompletionFunction));

    loop {
        match rl::readline("> ") {
            Some(line) => {
                let l = line.as_slice();
                rl::add_history(l);
                println(l);
                //println!("{}", rl::history_get(-2));
            },
            _ => {
                println("");
                break
            }
        }
    }
}