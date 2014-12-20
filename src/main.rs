extern crate "readline" as rl;
extern crate libc;

use std::io::stdio::println;
use std::ptr;

static mut ENTRIES: *mut *const i8 = 0 as *mut *const i8;
static mut NENTRIES: i32 = 0;

extern fn rl_compentry_func(text: *const i8, state: i32) -> *const i8 {
	if state == 0 {
		unsafe {
			for i in range(0, NENTRIES) {
				libc::free(*ENTRIES.offset(i as int) as *mut libc::c_void)
			}
		}
	}
	unsafe {
		if state >= NENTRIES {
			ptr::null()
		} else {
			*ENTRIES.offset(state as int)
		}
	}
}

extern fn my_attempted_completion_function(text: *const i8, _start: i32, _end: i32) -> *mut *const i8 {
	return rl::rl_completion_matches(text, rl_compentry_func)
}

pub fn main() {
	rl::rl_initialize().unwrap();
	//println!("{}", rl::rl_readline_version())
	println!("{}", rl::rl_library_version().unwrap());

	rl::set_rl_attempted_completion_function(Some(my_attempted_completion_function));

	loop {
		match rl::readline("> ") {
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