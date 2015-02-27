# `python_mixin!`

Write Python code to emit Rust code right in your crate.

```rust
#![feature(plugin)]
#![plugin(python_mixin)]

python_mixin! {"
x = 1 + 2
print('fn get_x() -> u64 { %d }' % x)
"}

fn main() {
    let value = get_x();
    assert_eq!(value, 3);
}
```

## Should I actually use it?

Probably not, this is mainly me experimenting with
[more](https://github.com/huonw/brainfuck_macros) language
[plugins](https://github.com/huonw/fractran_macros). A more
portable/usable way to do this sort of code-generation is via
[a `Cargo` build script](http://doc.crates.io/build-script.html) plus
the `include!` macro.

Some downsides (not exhaustive):

- `python_mixin!` relies on having correctly-named Python binaries in
  the user's path, and, e.g. "`python`" is sometimes Python 2 and
  sometimes Python 3 and it's mean to require users to have installed
  Python on Windows. (Build scripts only need a Cargo and a Rust
  compiler, which the user is guaranteed to have if they're trying to
  build your Rust code.)

- Errors in the generated code are hard to debug, although the macro
  does try to give as useful error messages as possible e.g. file/line
  numbers emitted by Python point as closely as possible to the
  relevant part of the original string containing the source
  (including working with editors' jump-to-error facilities). The
  parsed Rust doesn't actually appear anywhere on disk or otherwise,
  so you cannot easily see the full context when the compiler
  complains (in contrast, a build script just generates a normal file
  right in your file-system).

## Installation

[Available on crates.io](https://crates.io/crates/python_mixin), so
you can just add

```toml
[dependencies]
python_mixin = "*"
```

to your `Cargo.toml`.

## Documentation

The `python_mixin!` macro consumes a single string, passes it to a
Python interpreter and then parses the output of that as Rust code. It
behaves like a `macro_rules!` macro, in that it can be used in any AST
position: expression, item etc.

The string argument to `python_mixin!` can be a macro invocation
itself, it is expanded before passing to Python.

Options can be specified (comma-separated) in an optional `{ ... }`
block before the Python string. See [Options](#options) for the
possible options.

### Examples

Compute the Unix time that the program was built at, by calling
Python's `time.time` function.

```rust
#![feature(plugin)]
#![plugin(python_mixin)]

fn main() {
    let time_of_build = python_mixin! {"
import time
print(time.time())
    "};
    println!("this was compiled at {}", time_of_build);
}
```

Use Python 2's naked print statement and Python 3's division
semantics:

```rust
#![feature(plugin)]
#![plugin(python_mixin)]

fn main() {
    let value2 = python_mixin! {
        { version = "2" }
        "print 1 / 2"
    };
    let value3 = python_mixin! {
        { version = "3" }
        "print(1 / 2)"
    };

    assert_eq!(value2, 0);
    assert_eq!(value3, 0.5);
}
```

Compute Fibonacci numbers in the *best* way possible, by making Python
print a function to compute each number:

```rust
#![feature(plugin)]
#![plugin(python_mixin)]

// create fib_0, fib_1, ..., fib_N functions that return the
// respective fibonacci number.
python_mixin! { r#"
print("fn fib_0() -> u64 { 0 }")
print("fn fib_1() -> u64 { 1 }")

def make_function(n):
    print("fn fib_%d() -> u64 { fib_%d() + fib_%d() }" % (n, n - 1, n - 2))

for i in range(2, 30 + 1):
    make_function(i)
"#}

fn main() {
    println!("the 30th fibonacci number is {}", fib_30());
}
```

### Options

| name      | type   | default |     |
|-----------|--------|---------|-----|
| `version` | string | `""`   | controls the version of Python used: `python_mixin!` tries to execute the `python{version}` binary. |

(Maybe this table will get longer? Who knows. Tables are cool.)
