// ---------------------------------------------------------------------------
// BindArg — valor tipado que puede bindearse a un sqlx::Query.
// ---------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub enum BindArg {
    Null,
    I64(i64),
    F64(f64),
    Text(String),
    Bool(bool),
    Uuid(uuid::Uuid),
    Blob(Vec<u8>),
}

// Conversiones desde tipos comunes
impl From<i64> for BindArg {
    fn from(v: i64) -> Self {
        BindArg::I64(v)
    }
}
impl From<i32> for BindArg {
    fn from(v: i32) -> Self {
        BindArg::I64(v as i64)
    }
}
impl From<f64> for BindArg {
    fn from(v: f64) -> Self {
        BindArg::F64(v)
    }
}
impl From<String> for BindArg {
    fn from(v: String) -> Self {
        BindArg::Text(v)
    }
}
impl From<&str> for BindArg {
    fn from(v: &str) -> Self {
        BindArg::Text(v.to_string())
    }
}
impl From<bool> for BindArg {
    fn from(v: bool) -> Self {
        BindArg::Bool(v)
    }
}

// Conversiones desde referencias (para el CRUD generado)
impl From<&i64> for BindArg {
    fn from(v: &i64) -> Self {
        BindArg::I64(*v)
    }
}
impl From<&i32> for BindArg {
    fn from(v: &i32) -> Self {
        BindArg::I64(*v as i64)
    }
}
impl From<&f64> for BindArg {
    fn from(v: &f64) -> Self {
        BindArg::F64(*v)
    }
}
impl From<&String> for BindArg {
    fn from(v: &String) -> Self {
        BindArg::Text(v.clone())
    }
}
impl From<&bool> for BindArg {
    fn from(v: &bool) -> Self {
        BindArg::Bool(*v)
    }
}
impl From<uuid::Uuid> for BindArg {
    fn from(v: uuid::Uuid) -> Self {
        BindArg::Uuid(v)
    }
}
impl From<&uuid::Uuid> for BindArg {
    fn from(v: &uuid::Uuid) -> Self {
        BindArg::Uuid(*v)
    }
}
impl From<Vec<u8>> for BindArg {
    fn from(v: Vec<u8>) -> Self {
        BindArg::Blob(v)
    }
}
impl From<&Vec<u8>> for BindArg {
    fn from(v: &Vec<u8>) -> Self {
        BindArg::Blob(v.clone())
    }
}

// Conversiones desde Option<T> (nullable)
impl From<Option<i64>> for BindArg {
    fn from(v: Option<i64>) -> Self {
        v.map_or(BindArg::Null, BindArg::I64)
    }
}
impl From<Option<i32>> for BindArg {
    fn from(v: Option<i32>) -> Self {
        v.map_or(BindArg::Null, |v| BindArg::I64(v as i64))
    }
}
impl From<Option<f64>> for BindArg {
    fn from(v: Option<f64>) -> Self {
        v.map_or(BindArg::Null, BindArg::F64)
    }
}
impl From<Option<String>> for BindArg {
    fn from(v: Option<String>) -> Self {
        v.map_or(BindArg::Null, BindArg::Text)
    }
}
impl From<Option<bool>> for BindArg {
    fn from(v: Option<bool>) -> Self {
        v.map_or(BindArg::Null, BindArg::Bool)
    }
}
impl From<Option<uuid::Uuid>> for BindArg {
    fn from(v: Option<uuid::Uuid>) -> Self {
        v.map_or(BindArg::Null, BindArg::Uuid)
    }
}
impl From<Option<Vec<u8>>> for BindArg {
    fn from(v: Option<Vec<u8>>) -> Self {
        v.map_or(BindArg::Null, BindArg::Blob)
    }
}

// Conversiones desde &Option<T> (para el CRUD generado)
impl From<&Option<i64>> for BindArg {
    fn from(v: &Option<i64>) -> Self {
        v.as_ref().map_or(BindArg::Null, |v| BindArg::I64(*v))
    }
}
impl From<&Option<i32>> for BindArg {
    fn from(v: &Option<i32>) -> Self {
        v.as_ref()
            .map_or(BindArg::Null, |v| BindArg::I64(*v as i64))
    }
}
impl From<&Option<f64>> for BindArg {
    fn from(v: &Option<f64>) -> Self {
        v.as_ref().map_or(BindArg::Null, |v| BindArg::F64(*v))
    }
}
impl From<&Option<String>> for BindArg {
    fn from(v: &Option<String>) -> Self {
        v.as_ref()
            .map_or(BindArg::Null, |v| BindArg::Text(v.clone()))
    }
}
impl From<&Option<bool>> for BindArg {
    fn from(v: &Option<bool>) -> Self {
        v.as_ref().map_or(BindArg::Null, |v| BindArg::Bool(*v))
    }
}
impl From<&Option<uuid::Uuid>> for BindArg {
    fn from(v: &Option<uuid::Uuid>) -> Self {
        v.as_ref().map_or(BindArg::Null, |v| BindArg::Uuid(*v))
    }
}
impl From<&Option<Vec<u8>>> for BindArg {
    fn from(v: &Option<Vec<u8>>) -> Self {
        v.as_ref()
            .map_or(BindArg::Null, |v| BindArg::Blob(v.clone()))
    }
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
