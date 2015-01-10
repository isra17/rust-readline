#![crate_type = "lib"]
extern crate libc;

use std::ffi::CString;
use std::ffi::c_str_to_bytes;
//use std::io::fs::File;
use std::io::{IoError, IoResult};
use std::mem;
use std::ptr;
use std::str;
use libc::c_void;

pub type CompletionFunction = extern "C" fn(text: *const i8, start: i32, end: i32) -> *mut *const i8;
pub type CPPFunction = Option<CompletionFunction>;
// rl_compentry_func_t
pub type CompletionEntryFunction = extern "C" fn(text: *const i8, state: i32) -> *const i8;

static mut ENTRIES: *mut *const i8 = 0 as *mut *const i8;
static mut NB_ENTRIES: usize = 0;
static mut MAX_ENTRIES: usize = 0;

fn clear_compentries() {
    unsafe {
        /* freed by readline
        for i in range(0, NB_ENTRIES) {
            libc::free(*ENTRIES.offset(i as isize) as *mut c_void);
        }*/
        NB_ENTRIES = 0;
    }
}
fn alloc_compentries(n: usize) -> *mut *const i8 {
    unsafe {
        if n > MAX_ENTRIES {
            ENTRIES = libc::realloc(ENTRIES as *mut c_void, (n * mem::size_of::<*const i8>()) as u64) as *mut *const i8;
            if ENTRIES.is_null() {
                panic!("Memory allocation failed.");
            }
            MAX_ENTRIES = n;
        }
        NB_ENTRIES = n;
        ENTRIES
    }
}
pub fn set_compentries(entries: Vec<String>) {
    clear_compentries();
    let centries = alloc_compentries(entries.len());
    for i in range(0, entries.len()) {
        entries[i].with_c_str(|entry| {
            unsafe { *centries.offset(i as isize) = ffi::strdup(entry); }
        });
    }
}
pub fn get_compentry(i: usize) -> *const i8 {
    unsafe {
        if i >= NB_ENTRIES {
            clear_compentries();
            ptr::null()
        } else {
            *ENTRIES.offset(i as isize)
        }
    }
}

mod ffi {
    use libc::{c_char, c_int};

    #[repr(C)]
    pub struct HistEntry {
        pub line: *const c_char,
    }

    #[link(name = "readline")]
    extern {
        pub static mut history_base: c_int;
        pub static mut history_length: c_int;
        pub static mut rl_line_buffer: *mut c_char;
        pub static mut rl_point: c_int;
        pub static rl_library_version: *const c_char;
        pub static rl_readline_version: c_int;
        pub static mut rl_readline_name: *const c_char;
        pub static mut rl_attempted_completion_function: super::CPPFunction;
        pub static mut rl_attempted_completion_over: c_int;
        pub static mut rl_completer_word_break_characters: *const c_char;
        //pub static mut rl_completion_append_character: c_int;
        //pub static mut rl_special_prefixes: *const c_char;

        pub fn using_history();
        pub fn add_history(line: *const c_char);
        pub fn history_get(offset: c_int) -> *mut HistEntry;
        pub fn clear_history();
        //pub fn where_history() -> c_int;

        pub fn read_history(filename: *const c_char) -> c_int;
        pub fn write_history(filename: *const c_char) -> c_int;
        pub fn history_truncate_file(filename: *const c_char, nlines: c_int) -> c_int;
        pub fn append_history(nelements: c_int, filename: *const c_char) -> c_int;

        pub fn stifle_history(max : c_int);
        pub fn unstifle_history() -> c_int;
        pub fn history_is_stifled() -> c_int;

        pub fn readline(p: *const c_char) -> *const c_char;
        pub fn rl_initialize() -> c_int;
        pub fn rl_read_init_file(filename: *const c_char) -> c_int;
        pub fn rl_parse_and_bind(line: *const c_char) -> c_int;

        pub fn rl_completion_matches(text: *const c_char, entry_func: super::CompletionEntryFunction) -> *mut *const c_char;
    }
    extern {
        pub fn strdup(s: *const c_char) -> *const c_char;
    }
}

/// Begin a session in which the history functions might be used. This initializes the interactive variables.
///
/// (See [using_history](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX2))
pub fn using_history() {
    unsafe { ffi::using_history() }
}

//static mut PREV_HIST: *const i8 = 0 as *const i8;

/// Place `line` at the end of the history list.
///
/// Blank lines and consecutive duplicates are discarded.
/// (See [add_history](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX5))
pub fn add_history(line: &str) {
    if line.len() == 0 || line.char_at(0).is_whitespace() { // HISTCONTROL=ignorespace
        return;
    }
    // HISTCONTROL=ignoredups
    if history_get(-1).map_or(false, |prev| prev.as_slice() == line) {
        return;
    }
    line.with_c_str(|line| {
        unsafe {
            // HISTCONTROL=ignoredups
            //if PREV_HIST.is_null() || libc::strcmp(PREV_HIST, line) != 0 {
                ffi::add_history(line);
            //}
            //libc::free(PREV_HIST as *mut c_void);
            //PREV_HIST = ffi::strdup(line);
        }
    })
}

/// Return the history entry at position `index`, starting from 0.
///
/// If there is no entry there, or if `index` is greater than the history length, return `None`.
/// (See [history_get](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX17))
pub fn history_get(mut index: i32) -> Option<String> {
    if index < 0 {
        index += history_length();
    }
    index += history_base(); // TODO validate
    let entry = unsafe { ffi::history_get(index) };
    if entry.is_null() {
        None
    } else {
        let line = unsafe { c_str_to_bytes(&(*entry).line); };
        str::from_utf8(line).unwrap()
    }
}

/// Add the content of `filename` to the history list, a line at a time.
///
/// If `filename` is `None`, then read from '~/.history'.
/// (See [read_history](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX27))
pub fn read_history(filename: Option<&Path>) -> IoResult<()> {
    let errno = match filename {
        Some(filename) => filename.with_c_str(|filename| {
            unsafe { ffi::read_history(filename) }
        }),
        None => unsafe { ffi::read_history(ptr::null()) }
    };
    match errno {
        0 => Ok(()),
        errno => Err(IoError::from_errno(errno as usize, true))
    }
}

/// Write the current history to `filename`, overwriting `filename` if necessary.
///
/// If `filename` is `None`, then write the history list to `~/.history'.
/// (See [write_history](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX29))
pub fn write_history(filename: Option<&Path>) -> IoResult<()> {
    if history_length() == 0 {
        return Ok(());
    }
    let errno = match filename {
        Some(filename) => filename.with_c_str(|filename| {
            unsafe { ffi::write_history(filename) }
        }),
        None => unsafe { ffi::write_history(ptr::null()) }
    };
    match errno {
        0 => Ok(()),
        errno => Err(IoError::from_errno(errno as usize, true))
    }
}

/// Truncate the history file `filename`, leaving only the last `nlines` lines.
///
/// If `filename` is `None`, then `~/.history' is truncated.
/// (See [history_truncate_file](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX31))
pub fn history_truncate_file(filename: Option<&Path>, nlines: i32) -> IoResult<()> {
    let errno = match filename {
        Some(filename) => filename.with_c_str(|filename| {
                unsafe { ffi::history_truncate_file(filename, nlines) }
        }),
        None => unsafe { ffi::history_truncate_file(ptr::null(), nlines) }
    };
    match errno {
        0 => Ok(()),
        errno => Err(IoError::from_errno(errno as usize, true))
    }
}

/// Append the last `nelements` of the history list to `filename`.
///
/// If `filename` is `None`, then `~/.history' is truncated.
/// (See [append_history](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX30))
pub fn append_history(nelements: i32, filename: Option<&Path>) -> IoResult<()> {
    if history_length() == 0 {
        return Ok(());
    }
    let errno = match filename {
        Some(filename) => {
            /*if !filename.exists() {
                File::create(filename);
            }*/
            filename.with_c_str(|filename| {
                unsafe { ffi::append_history(nelements, filename) }
            })
        },
        None => unsafe { ffi::append_history(nelements, ptr::null()) }
    };
    match errno {
        0 => Ok(()),
        errno => Err(IoError::from_errno(errno as usize, true))
    }
}

/// Clear the history list by deleting all the entries.
///
/// (See [clear_history](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX10))
pub fn clear_history() {
    unsafe {
        ffi::clear_history();
        //PREV_HIST = ptr::null();
    }
}

/// Cut off the history list, remembering only the last `max` entries.
///
/// (See [stifle_history](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX11))
pub fn stifle_history(max: i32) {
    unsafe { ffi::stifle_history(max) }
}

/// Stop stifling the history.
///
/// This returns the previously-set maximum number of history entries (as set by `stifle_history()`).
/// The value is positive if the history was stifled, negative if it wasn't.
/// (See [unstifle_history](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX12))
pub fn unstifle_history() -> i32 {
    unsafe { ffi::unstifle_history() }
}

/// Say if the history is stifled.
///
/// (See [history_is_stifled](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX13))
pub fn history_is_stifled() -> bool {
    unsafe { ffi::history_is_stifled() != 0 }
}

/// Return the logical offset of the first entry in the history list.
///
/// (See [history_base](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX36))
pub fn history_base() -> i32 {
    unsafe { ffi::history_base }
}

/// Return the number of entries currently stored in the history list.
///
/// (See [history_length](http://cnswww.cns.cwru.edu/php/chet/readline/history.html#IDX37))
pub fn history_length() -> i32 {
    unsafe { ffi::history_length }
}

/// Print a `prompt` and then read and return a single line of text from the user.
///
/// If readline encounters an EOF while reading the line, and the line is empty at that point, then `None` is returned.
/// Otherwise, the line is ended just as if a newline had been typed.
/// (See [readline](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX190))
pub fn readline(prompt: &str) -> Option<String> {
    let line = prompt.with_c_str(|prompt| unsafe { ffi::readline(prompt) });
    if line.is_null() {  // user pressed Ctrl-D
        None
    } else {
        unsafe { CString::new(line, true).as_str().map(|line| line.to_string()) }
    }
}

/// Return the line gathered so far.
///
/// (See [rl_line_buffer](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX192))
pub fn rl_line_buffer() -> *mut i8 {
    unsafe { ffi::rl_line_buffer }
}

/// Return the offset of the current cursor position in `rl_line_buffer` (the point).
///
/// (See [rl_point](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX192))
pub fn rl_point() -> i32 {
    unsafe { ffi::rl_point }
}

/// Initialize or re-initialize Readline's internal state. It's not strictly necessary to call this; `readline()` calls it before reading any input.
///
/// (See [rl_initialize](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX316))
pub fn rl_initialize() -> IoResult<()> {
    let errno = unsafe { ffi::rl_initialize() };
    match errno {
        0 => Ok(()),
        errno => Err(IoError::from_errno(errno as usize, true))
    }
}

/// Return the version number of this revision of the library.
///
/// (See [rl_library_version](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX214))
pub fn rl_library_version() -> Option<String> {
    let version = unsafe { c_str_to_bytes(&ffi::rl_library_version); };
    str::from_utf8(version).unwrap()
}

/// Returns an integer encoding the current version of the library.
///
/// (See [rl_readline_version](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX214))
pub fn rl_readline_version() -> i32 {
    ffi::rl_readline_version
}

/// Name is set to a unique name by each application using Readline. The value allows conditional parsing of the inputrc file.
///
/// (See [rl_readline_name](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX218))
pub fn rl_readline_name() -> Option<String> {
    unsafe {
        let name = ffi::rl_readline_name;
        if name.is_null() {
            None
        } else {
            let slice = unsafe { c_str_to_bytes(&name); };
            str::from_utf8(slice).unwrap()
        }
    }
}

/// Set to a unique name by each application using Readline. The value allows conditional parsing of the inputrc file.
///
/// (See [rl_readline_name](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX218))
pub fn set_rl_readline_name(name: &str) {
    // The memory will never be freed.
    /*unsafe {
        libc::free(ffi::rl_readline_name as *mut c_void);
    }*/
    name.with_c_str(|name| {
        unsafe { ffi::rl_readline_name = ffi::strdup(name) }
    });
}

/// Read keybindings and variable assignments from `filename`.
///
/// (See [rl_read_init_file](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX267))
pub fn rl_read_init_file(filename: &Path) -> IoResult<()> {
    let errno = filename.with_c_str(|filename| {
        unsafe { ffi::rl_read_init_file(filename) }
    });
    match errno {
        0 => Ok(()),
        errno => Err(IoError::from_errno(errno as usize, true))
    }
}

/// Parse line as if it had been read from the inputrc file and performs any key bindings and variable assignments found
///
/// (See [rl_parse_and_bind](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX266))
pub fn rl_parse_and_bind(line: &str) -> IoResult<()> {
    let errno = line.with_c_str(|line| {
        unsafe { ffi::rl_parse_and_bind(line) }
    });
    match errno {
        0 => Ok(()),
        errno => Err(IoError::from_errno(errno as usize, true))
    }
}

pub fn rl_attempted_completion_over(b: bool) {
    unsafe { ffi::rl_attempted_completion_over = b as i32; }
}

/// Return the list of characters that signal a break between words for completion.
/// The default list is " \t\n\"\\'`@$><=;|&{(".
///
/// (See [rl_completer_word_break_characters](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX354))
pub fn rl_completer_word_break_characters() -> Option<String> {
    unsafe {
        let wbc = ffi::rl_completer_word_break_characters;
        if wbc.is_null() {
            None
        } else {
            let slice = unsafe { c_str_to_bytes(&wbc); };
            str::from_utf8(slice).unwrap()
        }
    }
}

/// Set the list of characters that signal a break between words for completion.
/// The default list is " \t\n\"\\'`@$><=;|&{(".
///
/// (See [rl_completer_word_break_characters](http://cnswww.cns.cwru.edu/php/chet/readline/readline.html#IDX354))
pub fn set_rl_completer_word_break_characters(wbc: &str) {
    // The memory will never be freed.
    /*unsafe {
        libc::free(ffi::rl_completer_word_break_characters as *mut c_void);
    }*/
    wbc.with_c_str(|wbc| {
        unsafe { ffi::rl_completer_word_break_characters = ffi::strdup(wbc) }
    });
}


pub fn set_rl_attempted_completion_function(f: CPPFunction) {
    unsafe { ffi::rl_attempted_completion_function = f }
}

pub fn rl_completion_matches(text: *const i8, entry_func: CompletionEntryFunction) -> *mut *const i8 {
    unsafe {
        ffi::rl_completion_matches(text, entry_func)
    }
}

#[cfg(test)]
mod history_tests {
    use std::io::TempDir;
    use std::sync::{Once, ONCE_INIT};
    static START: Once = ONCE_INIT;

    #[test]
    fn clear() {
        START.doit(|| {
            super::rl_initialize().unwrap();
            super::using_history();
        });
        super::clear_history();
        super::add_history("entry1");
        super::clear_history();
        assert_eq!(super::history_length(), 0);
    }

    #[test]
    fn add_history() {
        START.doit(|| {
            super::rl_initialize().unwrap();
            super::using_history();
        });
        super::clear_history();

        assert_eq!(super::history_length(), 0);
        super::add_history(""); // empty line ignored
        assert_eq!(super::history_length(), 0);
        super::add_history(" \t"); // blank line ignored
        assert_eq!(super::history_length(), 0);
        super::add_history(" entry1"); // line starting with space ignored
        assert_eq!(super::history_length(), 0);

        super::add_history("entry1");
        assert_eq!(super::history_length(), 1);
        super::add_history("entry2");
        assert_eq!(super::history_length(), 2);

        super::add_history("entry2"); // consecutive duplicates ignored
        assert_eq!(super::history_length(), 2);

        super::clear_history();
    }

    #[test]
    fn stifle_history() {
        START.doit(|| {
            super::rl_initialize().unwrap();
            super::using_history();
        });
        super::clear_history();
        super::add_history("entry1");
        super::add_history("entry2");
        assert!(!super::history_is_stifled(), "history is not expected to be stifled by default");

        super::stifle_history(1);
        assert!(super::history_is_stifled(), "history has not been stifled");

        super::add_history("entry2");
        super::add_history("entry3");
        assert_eq!(super::history_length(), 1);

        assert_eq!(1, super::unstifle_history());
        assert!(!super::history_is_stifled(), "history has not been unstifled");

        super::clear_history();
    }

    #[test]
    fn read_history() {
        START.doit(|| {
            super::rl_initialize().unwrap();
            super::using_history();
        });
        super::clear_history();
        let td = TempDir::new_in(&Path::new("."), "histo").unwrap();
        let history = td.path().join(".history");

        super::add_history("entry1");
        super::add_history("entry2");

        super::write_history(Some(&history)).unwrap();
        super::clear_history();

        super::read_history(Some(&history)).unwrap();
        assert_eq!(super::history_length(), 2);
        assert_eq!(super::history_get(-1), Some("entry2".to_string()));
        super::clear_history();

        td.close().unwrap();
    }

    #[test]
    fn history_base() {
        START.doit(|| {
            super::rl_initialize().unwrap();
            super::using_history();
        });
        super::clear_history();
        assert_eq!(super::history_base(), 1);
    }
}

#[cfg(test)]
mod rl_tests {
    use std::sync::{Once, ONCE_INIT};
    static START: Once = ONCE_INIT;

    #[test]
    fn rl_parse_and_bind() {
        START.doit(|| {
            super::rl_initialize().unwrap();
        });
        super::rl_parse_and_bind("bind \\t rl_complete").unwrap();
    }

    #[test]
    fn rl_readline_name() {
        //assert_eq!(super::rl_readline_name(), Some("".to_string()));
        super::set_rl_readline_name("rust");
        assert_eq!(super::rl_readline_name(), Some("rust".to_string()));
    }

    #[test]
    fn rl_completer_word_break_characters() {
        //assert_eq!(super::rl_completer_word_break_characters(), None);
        super::set_rl_completer_word_break_characters(" \t\n\"\\'`@$><=;|&{(");
        assert_eq!(super::rl_completer_word_break_characters(), Some(" \t\n\"\\'`@$><=;|&{(".to_string()));
    }
}
