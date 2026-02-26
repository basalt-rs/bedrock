use basalt_bedrock::roi::{Raw, RawOrImport};
use serde::Deserialize;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn roi_flat() {
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Flat {
        field_a: String,
        field_b: String,
    }

    let flat: RawOrImport<Flat> = toml_edit::de::from_str(
        r#"
        field_a = "foo"
        field_b = "bar"
    "#,
    )
    .unwrap();

    assert_eq!(
        flat.into_inner(),
        Flat {
            field_a: "foo".into(),
            field_b: "bar".into(),
        }
    );
}

#[test]
fn roi_nested_import() {
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Inner {
        field_c: String,
        field_d: String,
    }
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Outer {
        field_a: String,
        field_b: RawOrImport<Inner>,
    }

    let mut f = NamedTempFile::new().unwrap();

    write!(
        f,
        r#"
            field_c = "field c"
            field_d = "field d"
        "#
    )
    .unwrap();

    let flat: RawOrImport<Outer> = toml_edit::de::from_str(&format!(
        r#"
            field_a = "foo"
            field_b = {{ import = {:?} }}
        "#,
        f.path(),
    ))
    .unwrap();

    assert_eq!(
        flat.into_inner(),
        Outer {
            field_a: "foo".into(),
            field_b: Inner {
                field_c: "field c".into(),
                field_d: "field d".into(),
            }
            .into()
        }
    );
}

#[test]
fn roi_nested_inline() {
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Inner {
        field_c: String,
        field_d: String,
    }
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Outer {
        field_a: String,
        field_b: RawOrImport<Inner>,
    }

    let flat: RawOrImport<Outer> = toml_edit::de::from_str(
        r#"
            field_a = "foo"
            field_b = { field_c = "field c", field_d = "field d" }
        "#,
    )
    .unwrap();

    assert_eq!(
        flat.into_inner(),
        Outer {
            field_a: "foo".into(),
            field_b: Inner {
                field_c: "field c".into(),
                field_d: "field d".into(),
            }
            .into()
        }
    );
}

#[test]
fn roi_nested_import_raw() {
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Outer {
        field_a: String,
        field_b: RawOrImport<String, Raw>,
    }

    let mut f = NamedTempFile::new().unwrap();

    let contents = r#"
        I'd just like to interject for a moment. What you're refering to as Linux, is in fact,
        GNU/Linux, or as I've recently taken to calling it, GNU plus Linux. Linux is not an
        operating system unto itself, but rather another free component of a fully functioning
        GNU system made useful by the GNU corelibs, shell utilities and vital system components
        comprising a full OS as defined by POSIX.
    "#;

    f.write_all(contents.as_bytes()).unwrap();

    let flat: RawOrImport<Outer> = toml_edit::de::from_str(&format!(
        r#"
            field_a = "foo"
            field_b = {{ import = {:?} }}
        "#,
        f.path(),
    ))
    .unwrap();

    assert_eq!(
        flat.into_inner(),
        Outer {
            field_a: "foo".into(),
            field_b: contents.to_string().into()
        }
    );
}

#[test]
fn roi_nested_inline_raw() {
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Outer {
        field_a: String,
        field_b: RawOrImport<String, Raw>,
    }

    let mut f = NamedTempFile::new().unwrap();

    let contents = r#"
        I'd just like to interject for a moment. What you're refering to as Linux, is in fact,
        GNU/Linux, or as I've recently taken to calling it, GNU plus Linux. Linux is not an
        operating system unto itself, but rather another free component of a fully functioning
        GNU system made useful by the GNU corelibs, shell utilities and vital system components
        comprising a full OS as defined by POSIX.
    "#;

    f.write_all(contents.as_bytes()).unwrap();

    let flat: RawOrImport<Outer> = toml_edit::de::from_str(&format!(
        r#"
            field_a = "foo"
            field_b = {:?}
        "#,
        contents,
    ))
    .unwrap();

    assert_eq!(
        flat.into_inner(),
        Outer {
            field_a: "foo".into(),
            field_b: contents.to_string().into()
        }
    );
}

#[test]
fn roi_nested_nested_import() {
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct InnerInner {
        inner: String,
    }
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Inner {
        a: RawOrImport<InnerInner>,
        b: RawOrImport<InnerInner>,
    }
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Outer {
        inner: RawOrImport<Inner>,
    }

    let mut inner_a = NamedTempFile::new().unwrap();
    let mut inner_b = NamedTempFile::new().unwrap();

    writeln!(inner_a, r#"inner = "hello""#).unwrap();
    writeln!(inner_b, r#"inner = "world""#).unwrap();

    let mut inner_outer = NamedTempFile::new().unwrap();

    writeln!(
        inner_outer,
        r#"
            a = {{ import = {:?} }}
            b = {{ import = {:?} }}
        "#,
        inner_a.path(),
        inner_b.path()
    )
    .unwrap();

    let flat: RawOrImport<Outer> = toml_edit::de::from_str(&format!(
        r#"
            inner = {{ import = {:?} }}
        "#,
        inner_outer.path(),
    ))
    .unwrap();

    assert_eq!(
        flat.into_inner(),
        Outer {
            inner: Inner {
                a: InnerInner {
                    inner: "hello".into()
                }
                .into(),
                b: InnerInner {
                    inner: "world".into()
                }
                .into(),
            }
            .into()
        }
    );
}

#[test]
fn roi_nested_nested_inline() {
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct InnerInner {
        inner: String,
    }
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Inner {
        a: RawOrImport<InnerInner>,
        b: RawOrImport<InnerInner>,
    }
    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct Outer {
        inner: RawOrImport<Inner>,
    }

    let flat: RawOrImport<Outer> = toml_edit::de::from_str(
        r#"
            [inner.a]
            inner = "hello"

            [inner.b]
            inner = "world"
        "#,
    )
    .unwrap();

    assert_eq!(
        flat.into_inner(),
        Outer {
            inner: Inner {
                a: InnerInner {
                    inner: "hello".into()
                }
                .into(),
                b: InnerInner {
                    inner: "world".into()
                }
                .into(),
            }
            .into()
        }
    );
}
