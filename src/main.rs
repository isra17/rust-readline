extern crate "readline" as rl;
extern crate libc;

use std::io::stdio::println;
use std::ptr;
use libc::{c_char, c_int};

extern fn rl_compentry_func(text: *const c_char, state: c_int) -> *mut c_char {
	ptr::null_mut() // TODO
}

extern fn my_attempted_completion_function(text: *const c_char, start: c_int, end: c_int) -> *mut *mut c_char {
	return rl::rl_completion_matches(text, rl_compentry_func)
}

pub fn main() {
	rl::rl_initialize().unwrap();
	//println!("{}", rl::rl_readline_version())
	println!("{}", rl::rl_library_version().unwrap())

	rl::set_rl_attempted_completion_function(Some(my_attempted_completion_function));

	loop {
		match rl::readline(Some("> ")) {
			Some(line) => {
				let l = line.as_slice();
				rl::add_history(l);
				println(l);
				//println!("{}", rl::history_get(-2));
			},
			_ => break
		}
	}
}