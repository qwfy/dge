pub(crate) fn gen_string(s: String) -> String {
    format!(r#"String::from("{}")"#, s)
}

pub(crate) fn gen_opt_string(s: Option<String>) -> String {
    match s {
        None => "None".into(),
        Some(s) => format!("Some({})", gen_string(s)),
    }
}

pub(crate) fn gen_opt_str(s: Option<String>) -> String {
    match s {
        None => "None".into(),
        Some(s) => format!(r#"Some("{}")"#, s),
    }
}

pub(crate) fn gen_vec_string(ss: Vec<String>) -> String {
    let middle: String = ss.into_iter().map(gen_string).collect::<Vec<_>>().join(",");
    format!(r#"vec![{}]"#, middle)
}

pub(crate) fn gen_vec_str(ss: Vec<String>) -> String {
    let middle: String = ss
        .iter()
        .map(|s| format!(r#""{}""#, s))
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"vec![{}]"#, middle)
}

pub(crate) fn gen_str(s: String) -> String {
    format!(r#""{}""#, s)
}

pub(crate) fn gen_u32(n: u32) -> String {
    n.to_string()
}

pub(crate) fn gen_ident(s: String) -> String {
    s
}
