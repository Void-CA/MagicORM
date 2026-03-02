use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
pub struct ModelJsonMeta {
    pub table: String,
    pub columns: Vec<ColumnJsonMeta>,
    pub foreign_keys: Vec<ForeignKeyJsonMeta>,
}

#[derive(Serialize)]
pub struct ColumnJsonMeta {
    pub name: String,
    pub sql_type: String,
    pub nullable: bool,
    pub primary_key: bool,
}

#[derive(Serialize)]
pub struct ForeignKeyJsonMeta {
    pub field: String,
    pub related_table: String,
    pub related_column: String,
}


pub fn write_model_json(
    table_name: &str,
    columns: Vec<ColumnJsonMeta>,
    foreign_keys: Vec<ForeignKeyJsonMeta>,
) {
    let dir_path = Path::new("target/magicorm");
    fs::create_dir_all(dir_path).expect("Failed to create directory");

    let file_path = dir_path.join(format!("magicorm_{}.json", table_name));
    let file = fs::File::create(&file_path).expect("Failed to create JSON file");

    let model_json = ModelJsonMeta {
        table: table_name.to_string(),
        columns,
        foreign_keys,
    };

    serde_json::to_writer_pretty(file, &model_json).expect("Failed to write JSON");
}