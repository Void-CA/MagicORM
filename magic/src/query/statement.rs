// ---------------------------------------------------------------------------
// BindArg — valor tipado que puede bindearse a un sqlx::Query.
// ---------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub enum BindArg {
    I64(i64),
    F64(f64),
    Text(String),
    Bool(bool),
}

// Conversiones desde tipos comunes
impl From<i64> for BindArg {
    fn from(v: i64) -> Self { BindArg::I64(v) }
}
impl From<i32> for BindArg {
    fn from(v: i32) -> Self { BindArg::I64(v as i64) }
}
impl From<f64> for BindArg {
    fn from(v: f64) -> Self { BindArg::F64(v) }
}
impl From<String> for BindArg {
    fn from(v: String) -> Self { BindArg::Text(v) }
}
impl From<&str> for BindArg {
    fn from(v: &str) -> Self { BindArg::Text(v.to_string()) }
}
impl From<bool> for BindArg {
    fn from(v: bool) -> Self { BindArg::Bool(v) }
}

// Conversiones desde referencias (para el CRUD generado)
impl From<&i64> for BindArg {
    fn from(v: &i64) -> Self { BindArg::I64(*v) }
}
impl From<&i32> for BindArg {
    fn from(v: &i32) -> Self { BindArg::I64(*v as i64) }
}
impl From<&f64> for BindArg {
    fn from(v: &f64) -> Self { BindArg::F64(*v) }
}
impl From<&String> for BindArg {
    fn from(v: &String) -> Self { BindArg::Text(v.clone()) }
}
impl From<&bool> for BindArg {
    fn from(v: &bool) -> Self { BindArg::Bool(*v) }
}

// ---------------------------------------------------------------------------
// Statement — snapshot del SQL generado + bindings pendientes.
// Sirve para testear y para ejecutar a través de helpers.
// ---------------------------------------------------------------------------
pub struct Statement {
    pub sql: String,
    pub values: Vec<BindArg>,
}

impl Statement {
    pub fn new(sql: String, values: Vec<BindArg>) -> Self {
        Self { sql, values }
    }
}
