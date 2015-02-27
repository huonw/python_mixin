#![feature(plugin)]

#![plugin(python_mixin)]

#[test]
fn smoke() {
    assert_eq!(python_mixin!("print(1 + 2)"), 3);
    assert_eq!(python_mixin!({version = "2.7"} "print 1 + 2"), 3);
}

#[test]
fn can_use_macro() {
    macro_rules! foo {
        () => {
            stringify!(1 + 2)
        }
    }
    let value = python_mixin! {
        concat!("x = ", foo!(), "\nprint(x)")
    };
    assert_eq!(value, 3);
}

python_mixin! {"
for i in range(0, 3):
    print('fn f%d() -> i32 { %d }' % (i, i + 1))
"}

#[test]
fn items() {
    assert_eq!(f0(), 1);
    assert_eq!(f1(), 2);
    assert_eq!(f2(), 3);
}

struct Foo;

impl Foo {
    python_mixin! {"
for i in range(0, 3):
    print('fn f%d(&self) -> i32 { %d }' % (i, i + 1))
    "}
}
#[test]
fn methods() {
    assert_eq!(Foo.f0(), 1);
    assert_eq!(Foo.f1(), 2);
    assert_eq!(Foo.f2(), 3);
}
