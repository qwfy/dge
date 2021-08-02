pub(crate) fn gen_string(s: String) -> String {
    format!(r#"String::from("{}")"#, s)
}

pub(crate) fn gen_opt_string(s: Option<String>) -> String {
    match s {
        None => "None".into(),
        Some(s) => format!("Some({})", gen_string(s)),
    }
}

pub(crate) fn gen_u32(n: u32) -> String {
    n.to_string()
}
