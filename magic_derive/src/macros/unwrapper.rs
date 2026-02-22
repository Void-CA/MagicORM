macro_rules! unwrap_or_ts {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(err) => return err.to_compile_error().into(),
        }
    };
}