macro_rules! get_lang_string {
    ($template:expr, $($key:ident = $value:expr),*) => {
        {
            let mut result = String::from($template);

            $(
                let var_name = stringify!($key).trim_matches('"');
                let holder = format!("{{{var_name}}}");
                result = result.replace(&holder, &format!("{}", $value));
            )*

            result
        }
    };
}
