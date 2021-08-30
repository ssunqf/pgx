use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

use super::{SqlGraphEntity, SqlGraphIdentifier, ToSql};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryPostgresEnum {
    pub name: &'static str,
    pub file: &'static str,
    pub line: u32,
    pub full_path: &'static str,
    pub module_path: &'static str,
    pub mappings: std::collections::HashSet<super::RustSqlMapping>,
    pub variants: Vec<&'static str>,
}

impl crate::PostgresType for InventoryPostgresEnum {}

impl Hash for InventoryPostgresEnum {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.full_path.hash(state);
    }
}

impl Ord for InventoryPostgresEnum {
    fn cmp(&self, other: &Self) -> Ordering {
        self.file
            .cmp(other.file)
            .then_with(|| self.file.cmp(other.file))
    }
}

impl PartialOrd for InventoryPostgresEnum {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl InventoryPostgresEnum {
    pub fn id_matches(&self, candidate: &core::any::TypeId) -> bool {
        self.mappings.iter().any(|tester| *candidate == tester.id)
    }
}

impl Into<SqlGraphEntity> for InventoryPostgresEnum {
    fn into(self) -> SqlGraphEntity {
        SqlGraphEntity::Enum(self)
    }
}

impl SqlGraphIdentifier for InventoryPostgresEnum {
    fn dot_identifier(&self) -> String {
        format!("enum {}", self.full_path)
    }
    fn rust_identifier(&self) -> String {
        self.full_path.to_string()
    }

    fn file(&self) -> Option<&'static str> {
        Some(self.file)
    }

    fn line(&self) -> Option<u32> {
        Some(self.line)
    }
}

impl ToSql for InventoryPostgresEnum {
    #[tracing::instrument(level = "debug", err, skip(self, context), fields(identifier = %self.rust_identifier()))]
    fn to_sql(&self, context: &super::PgxSql) -> eyre::Result<String> {
        let self_index = context.enums[self];
        let sql = format!(
            "\n\
                    -- {file}:{line}\n\
                    -- {full_path}\n\
                    CREATE TYPE {schema}{name} AS ENUM (\n\
                        {variants}\
                    );\
                ",
            schema = context.schema_prefix_for(&self_index),
            full_path = self.full_path,
            file = self.file,
            line = self.line,
            name = self.name,
            variants = self
                .variants
                .iter()
                .map(|variant| format!("\t'{}'", variant))
                .collect::<Vec<_>>()
                .join(",\n")
                + "\n",
        );
        tracing::debug!(%sql);
        Ok(sql)
    }
}
