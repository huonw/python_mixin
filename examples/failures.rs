#![feature(plugin)]
#![plugin(python_mixin)]

#[cfg(feature = "compile_error")]
fn foo() {
    python_mixin! {
        { bad = options are bad, another_bad_one, }
        "print('()')"
    }
    // option double up
    python_mixin! {
        { version = "2", version = "3" }
        "print('()')"
    }
    // bad syntax (hopefully the python errors point to the bad line)
    python_mixin! {"
invalid invalid
    "}

    // successful run, but output on stderr
    python_mixin! {
        { version = "3" }
        r#"
import sys
print('()')
print("this is on stderr", file = sys.stderr)
    "#}

    // uncaught exception
    python_mixin! {"raise ValueError()"}
    // segfault!
    python_mixin! {r#"
import ctypes
ctypes.string_at(0)
    "#}
}

fn main() {}
