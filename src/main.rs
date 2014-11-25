extern crate "readline" as rl;

use std::io::stdio::println;
use std::ptr;

//static mut entries: Option<Vec<String>> = None;

extern fn rl_compentry_func(text: *const i8, i: i32) -> *const i8 {
	if i == 0 {
		// entries = ...
	}
	/*
	match entries {
		Some(entries) => {
			if i >= entries.len() {
				ptr::null()
			} else {
		    entries[i].with_c_str(|entry| {
		        unsafe { ffi::strdup(entry) }
		    });
			}
		}
		_ => ptr::null()
	}
	*/
	ptr::null() // TODO
}

extern fn my_attempted_completion_function(text: *const i8, _start: i32, _end: i32) -> *mut *const i8 {
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