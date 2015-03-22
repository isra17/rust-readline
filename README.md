rust-readline
=============

Simple wrapper around readline for the Rust language

Exposes:
 - `add_history(line: &str)`
 - `readline(prompt: &str) -> Option<String>`
 - `set_rl_attempted_completion_function(f: Option<CompletionFunction>)`

```sh
$ cargo run --example simple
```