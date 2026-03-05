use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;

/// Representa un modelo de tabla
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ModelJson {
    pub table: String,
    pub columns: Vec<ColumnJson>,
    pub foreign_keys: Vec<ForeignKeyJson>,
}

/// Representa una columna
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ColumnJson {
    pub name: String,
    pub sql_type: String,
    pub nullable: bool,
    pub primary_key: bool,
}

/// Representa una clave foránea
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ForeignKeyJson {
    pub field: String,
    pub related_table: String,
    pub related_column: String,
}

/// Carga todos los modelos desde target/magicorm/*.json
pub fn load_models(target_dir: &str) -> Result<HashMap<String, ModelJson>> {
    let mut models = HashMap::new();
    let dir = Path::new(target_dir);

    if !dir.exists() {
        anyhow::bail!("El directorio '{}' no existe. Compila primero los modelos.", target_dir);
    }

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let data = fs::read_to_string(&path)?;
            let model: ModelJson = serde_json::from_str(&data)?;
            models.insert(model.table.clone(), model);
        }
    }

    Ok(models)
}

/// Carga el último estado guardado de modelos para comparar cambios
pub fn load_last_state(path: &str) -> Result<HashMap<String, ModelJson>> {
    if !Path::new(path).exists() {
        return Ok(HashMap::new());
    }
    let data = fs::read_to_string(path)?;
    let state: HashMap<String, ModelJson> = serde_json::from_str(&data)?;
    Ok(state)
}

/// Guarda el estado actual de los modelos para futuras comparaciones
pub fn save_last_state(path: &str, state: &HashMap<String, ModelJson>) -> Result<()> {
    let serialized = serde_json::to_string_pretty(state)?;
    fs::write(path, serialized)?;
    Ok(())
}