mod pg_extern;
mod postgres_enum;
mod postgres_hash;
mod postgres_ord;
mod postgres_type;
mod pg_schema;
mod control_file;

pub use pg_extern::{PgExtern, InventoryPgExtern, InventoryPgExternReturn, InventoryPgExternInput, InventoryPgOperator};
pub use postgres_enum::{PostgresEnum, InventoryPostgresEnum};
pub use postgres_hash::{PostgresHash, InventoryPostgresHash};
pub use postgres_ord::{PostgresOrd, InventoryPostgresOrd};
pub use postgres_type::{PostgresType, InventoryPostgresType};
pub use pg_schema::{Schema, InventorySchema};
pub use control_file::{ControlFile, ControlFileError};

// Reexports for the pgx extension inventory builders.
#[doc(hidden)]
pub use inventory;
#[doc(hidden)]
pub use include_dir;
#[doc(hidden)]
pub use impls;
#[doc(hidden)]
pub use once_cell;
#[doc(hidden)]
pub use eyre;
#[doc(hidden)]
pub use color_eyre;
#[doc(hidden)]
pub use tracing;
#[doc(hidden)]
pub use tracing_error;
#[doc(hidden)]
pub use tracing_subscriber;

use tracing::instrument;
use std::collections::HashMap;
use core::{any::TypeId, fmt::Debug};
use crate::ExternArgs;
use eyre::eyre as eyre_err;
use petgraph::{Graph, visit::IntoNodeReferences};

#[derive(Debug, Clone)]
pub struct ExtensionSql {
    pub module_path: &'static str,
    pub full_path: &'static str,
    pub sql: &'static str,
    pub file: &'static str,
    pub line: u32,
}

#[derive(Debug, Clone)]
pub struct PgxSql<'a> {
    pub type_mappings: HashMap<TypeId, String>,
    pub control: ControlFile,
    pub graph: Graph<SqlGraphEntity<'a>, SqlGraphRelationship>,
    pub schemas: HashMap<&'a str, &'a InventorySchema>,
    pub extension_sqls: HashMap<&'a str, &'a ExtensionSql>,
    pub externs: HashMap<&'a str, &'a InventoryPgExtern>,
    pub types: HashMap<&'a str, &'a InventoryPostgresType>,
    pub enums: HashMap<&'a str, &'a InventoryPostgresEnum>,
    pub ords: HashMap<&'a str, &'a InventoryPostgresOrd>,
    pub hashes: HashMap<&'a str, &'a InventoryPostgresHash>,
}

#[derive(Debug, Clone)]
pub enum SqlGraphEntity<'a> {
    Schema(&'a InventorySchema),
    CustomSql(&'a ExtensionSql),
    Function(&'a InventoryPgExtern),
    Type(&'a InventoryPostgresType),
    Enum(&'a InventoryPostgresEnum),
    Ord(&'a InventoryPostgresOrd),
    Hash(&'a InventoryPostgresHash),
}
use SqlGraphEntity::*;

impl<'a> From<&'a InventorySchema> for SqlGraphEntity<'a> {
    fn from(item: &'a InventorySchema) -> Self {
        SqlGraphEntity::Schema(&item)
    }
}

impl<'a> From<&'a ExtensionSql> for SqlGraphEntity<'a> {
    fn from(item: &'a ExtensionSql) -> Self {
        SqlGraphEntity::CustomSql(&item)
    }
}

impl<'a> From<&'a InventoryPgExtern> for SqlGraphEntity<'a> {
    fn from(item: &'a InventoryPgExtern) -> Self {
        SqlGraphEntity::Function(&item)
    }
}

impl<'a> From<&'a InventoryPostgresType> for SqlGraphEntity<'a> {
    fn from(item: &'a InventoryPostgresType) -> Self {
        SqlGraphEntity::Type(&item)
    }
}

impl<'a> From<&'a InventoryPostgresEnum> for SqlGraphEntity<'a> {
    fn from(item: &'a InventoryPostgresEnum) -> Self {
        SqlGraphEntity::Enum(&item)
    }
}

impl<'a> From<&'a InventoryPostgresOrd> for SqlGraphEntity<'a> {
    fn from(item: &'a InventoryPostgresOrd) -> Self {
        SqlGraphEntity::Ord(&item)
    }
}

impl<'a> From<&'a InventoryPostgresHash> for SqlGraphEntity<'a> {
    fn from(item: &'a InventoryPostgresHash) -> Self {
        SqlGraphEntity::Hash(&item)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum SqlGraphRelationship {
    DependsOn,
}


impl<'a> SqlGraphEntity<'a> {
    fn to_sql(&self, context: &PgxSql<'a>) -> eyre::Result<String> {
        let sql = match self {
            Schema(item) => context.inventory_schema_to_sql(item),
            CustomSql(item) => context.inventory_extension_sql_to_sql(item),
            Function(item) => context.inventory_extern_to_sql(item)?,
            Type(item) => context.inventory_type_to_sql(item)?,
            Enum(item) => context.inventory_enums_to_sql(item),
            Ord(item) => context.inventory_ord_to_sql(item),
            Hash(item) => context.inventory_hash_to_sql(item),
        };
        Ok(sql)
    }
}

impl<'a> PgxSql<'a> {
    pub fn build(
        control: ControlFile,
        type_mappings: HashMap<TypeId, String>,
        schemas: HashMap<&'a str, &'a InventorySchema>,
        extension_sqls: HashMap<&'a str, &'a ExtensionSql>,
        externs: HashMap<&'a str, &'a InventoryPgExtern>,
        types: HashMap<&'a str, &'a InventoryPostgresType>,
        enums: HashMap<&'a str, &'a InventoryPostgresEnum>,
        ords: HashMap<&'a str, &'a InventoryPostgresOrd>,
        hashes: HashMap<&'a str, &'a InventoryPostgresHash>,
    ) -> Self {
        let mut graph = Graph::new();
        for (_, &item) in &schemas {
            graph.add_node(item.into());
        }
        for (_, &item) in &extension_sqls {
            graph.add_node(item.into());
        }
        for (_, &item) in &externs {
            graph.add_node(item.into());
        }
        for (_, &item) in &types {
            graph.add_node(item.into());
        }
        for (_, &item) in &enums {
            graph.add_node(item.into());
        }
        for (_, &item) in &ords {
            graph.add_node(item.into());
        }
        for (_, &item) in &hashes {
            graph.add_node(item.into());
        }
        let mut this = Self {
            type_mappings,
            control,
            schemas,
            extension_sqls,
            externs,
            types,
            enums,
            ords,
            hashes,
            graph,
        };
        this.register_types();
        this
    }

    #[instrument(level = "info", err, skip(self))]
    pub fn to_file(self, file: impl AsRef<str> + Debug) -> eyre::Result<()> {
        use std::{fs::{File, create_dir_all}, path::Path, io::Write};
        let generated = self.to_sql()?;
        let path = Path::new(file.as_ref());

        let parent = path.parent();
        if let Some(parent) = parent {
            create_dir_all(parent)?;
        }
        let mut out = File::create(path)?;
        write!(out, "{}", generated)?;
        Ok(())
    }

    #[instrument(level = "info", skip(self))]
    pub fn schema_alias_of(&self, module_path: impl AsRef<str> + Debug) -> Option<String> {
        let mut needle = None;
        for (_full_path, item) in &self.schemas {
            if item.module_path.starts_with(module_path.as_ref()) {
                needle = Some(item.name.to_string());
                break;
            }
        }
        needle
    }

    pub fn schema_prefix_for(&self, module_path: impl AsRef<str> + Debug) -> String {
        self.schema_alias_of(module_path.as_ref())
            .map(|v| (v + ".").to_string()).unwrap_or_else(|| "".to_string())
    }

    #[instrument(level = "info", err, skip(self))]
    pub fn to_sql(self) -> eyre::Result<String> {
        let sql = format!("\
                /* \
                   This file is auto generated by pgx.\n\
                   \n\
                   The ordering of items is not stable, it is driven by a dependency graph.\n\
                */
                {sql}\n\
            ",
            sql = {
                let mut sql = String::new();
                for step in self.walk() {
                    let item_sql = step.to_sql(&self)?;
                    sql.push_str(&item_sql)
                }
                sql
            },
        );
        Ok(sql)
    }

    pub fn walk(&self) -> impl Iterator<Item=&SqlGraphEntity>  + '_ {
        self.graph.node_references().map(|x| x.1)
    }

    #[instrument(level = "info", skip(self, item), fields(item = item.full_path))]
    fn inventory_extension_sql_to_sql(&self, item: &ExtensionSql) -> String {
        let sql = format!("\
                -- {file}:{line}\n\
                {sql}\
                ",
                file = item.file,
                line = item.line,
                sql = item.sql,
        );
        tracing::debug!(%sql);
        sql
    }

    #[instrument(level = "info", skip(self))]
    fn schemas(&self) -> String {
        let mut buf = String::new();
        if let Some(schema) = &self.control.schema {
            buf.push_str(&format!("CREATE SCHEMA IF NOT EXISTS {};\n", schema));
        }
        for (_full_path, item) in &self.schemas {
            match item.name {
                "pg_catalog" | "public" =>  (),
                _ => buf.push_str(&self.inventory_schema_to_sql(item)),
            };
        }
        buf
    }

    #[instrument(level = "info", skip(self, item), fields(item = item.module_path))]
    fn inventory_schema_to_sql(&self, item: &InventorySchema) -> String {
        let sql = format!("\
                    -- {file}:{line}\n\
                    CREATE SCHEMA IF NOT EXISTS {name}; /* {module_path} */\n\
                ",
                name = item.name,
                file = item.file,
                line = item.line,
                module_path = item.module_path,
        );
        tracing::debug!(%sql);
        sql
    }

    #[instrument(level = "info", skip(self, item), fields(item = item.full_path))]
    fn inventory_enums_to_sql(&self, item: &InventoryPostgresEnum) -> String {
        let sql = format!("\
                    -- {file}:{line}\n\
                    -- {full_path}\n\
                    CREATE TYPE {schema}{name} AS ENUM (\n\
                        {variants}\
                    );\n\
                ",
            schema = self.schema_prefix_for(item.module_path),
            full_path = item.full_path,
            file = item.file,
            line = item.line,
            name = item.name,
            variants = item.variants.iter().map(|variant| format!("\t'{}'\n", variant)).collect::<Vec<_>>().join(","),
        );
        tracing::debug!(%sql);
        sql
    }

    #[instrument(level = "info", err, skip(self, item), fields(item = item.full_path))]
    fn inventory_extern_to_sql(&self, item: &InventoryPgExtern) -> eyre::Result<String> {
        let mut extern_attrs = item.extern_attrs.clone();
        let mut strict_upgrade = true;
        if !extern_attrs.iter().any(|i| i == &ExternArgs::Strict) {
            for arg in &item.fn_args {
                if arg.is_optional {
                    strict_upgrade = false;
                }
            }
        }
        tracing::trace!(?extern_attrs, strict_upgrade);

        if strict_upgrade {
            extern_attrs.push(ExternArgs::Strict);
        }

        let fn_sql = format!("\
                                CREATE OR REPLACE FUNCTION {schema}\"{name}\"({arguments}) {returns}\n\
                                {extern_attrs}\
                                {search_path}\
                                LANGUAGE c /* Rust */\n\
                                AS 'MODULE_PATHNAME', '{name}_wrapper';\
                            ",
                             schema = self.schema_prefix_for(item.module_path),
                             name = item.name,
                             arguments = if !item.fn_args.is_empty() {
                                 let mut args = Vec::new();
                                 for (idx, arg) in item.fn_args.iter().enumerate() {
                                     let needs_comma = idx < (item.fn_args.len() - 1);
                                     let buf = format!("\
                                            \t\"{pattern}\" {schema_prefix}{sql_type}{default}{maybe_comma}/* {ty_name} */\
                                        ",
                                                       pattern = arg.pattern,
                                                       schema_prefix = self.schema_prefix_for(arg.module_path.clone()),
                                                       sql_type = self.type_id_to_sql_type(arg.ty_id).ok_or_else(|| eyre_err!("Failed to map argument `{}` type `{}` to SQL type while building function `{}`.", arg.pattern, arg.ty_name, item.name))?,
                                                       default = if let Some(def) = arg.default { format!(" DEFAULT {}", def) } else { String::from("") },
                                                       maybe_comma = if needs_comma { ", " } else { " " },
                                                       ty_name = arg.ty_name,
                                     );
                                     args.push(buf);
                                 };
                                 String::from("\n") + &args.join("\n") + "\n"
                             } else { Default::default() },
                             returns = match &item.fn_return {
                                 InventoryPgExternReturn::None => String::from("RETURNS void"),
                                 InventoryPgExternReturn::Type { id, name, module_path } => {
                                     format!("RETURNS {schema_prefix}{sql_type} /* {name} */",
                                             sql_type = self.type_id_to_sql_type(*id).ok_or_else(|| eyre_err!("Failed to map return type `{}` to SQL type while building function `{}`.", name, item.name))?,
                                             schema_prefix = self.schema_prefix_for(module_path.clone()),
                                             name = name
                                     )
                                 },
                                 InventoryPgExternReturn::SetOf { id, name, module_path } => {
                                     format!("RETURNS SETOF {schema_prefix}{sql_type} /* {name} */",
                                             sql_type = self.type_id_to_sql_type(*id).ok_or_else(|| eyre_err!("Failed to map return type `{}` to SQL type while building function `{}`.", name, item.name))?,
                                             schema_prefix = self.schema_prefix_for(module_path.clone()),
                                             name = name
                                     )
                                 },
                                 InventoryPgExternReturn::Iterated(table_items) => {
                                     let mut items = String::new();
                                     for (idx, (id, ty_name, module_path, col_name)) in table_items.iter().enumerate() {
                                         let needs_comma = idx < (table_items.len() - 1);
                                         let item = format!("\n\t{col_name} {schema_prefix}{ty_resolved}{needs_comma} /* {ty_name} */",
                                                            col_name = col_name.unwrap(),
                                                            schema_prefix = self.schema_prefix_for(module_path.clone()),
                                                            ty_resolved = self.type_id_to_sql_type(*id).ok_or_else(|| eyre_err!("Failed to map return type `{}` to SQL type while building function `{}`.", ty_name, item.name))?,
                                                            needs_comma = if needs_comma { ", " } else { " " },
                                                            ty_name = ty_name
                                         );
                                         items.push_str(&item);
                                     }
                                     format!("RETURNS TABLE ({}\n)", items)
                                 },
                                 InventoryPgExternReturn::Trigger => String::from("RETURNS trigger"),
                             },
                             search_path = if let Some(search_path) = &item.search_path {
                                 let retval = format!("SET search_path TO {}", search_path.join(", "));
                                 retval + "\n"
                             } else { Default::default() },
                             extern_attrs = if extern_attrs.is_empty() {
                                 String::default()
                             } else {
                                 let mut retval = extern_attrs.iter().map(|attr| format!("{}", attr).to_uppercase()).collect::<Vec<_>>().join(" ");
                                 retval.push('\n');
                                 retval
                             },
        );

        let ext_sql = format!("\n\
                                -- {file}:{line}\n\
                                -- {module_path}::{name}\n\
                                {fn_sql}\n\
                                {overridden}\
                            ",
                              name = item.name,
                              module_path = item.module_path,
                              file = item.file,
                              line = item.line,
                              fn_sql = if item.overridden.is_some() {
                                  let mut inner = fn_sql.lines().map(|f| format!("-- {}", f)).collect::<Vec<_>>().join("\n");
                                  inner.push_str("\n--\n-- Overridden as (due to a `//` comment with a `sql` code block):");
                                  inner
                              } else {
                                  fn_sql
                              },
                              overridden = item.overridden.map(|f| f.to_owned() + "\n").unwrap_or_default(),
        );
        tracing::debug!(sql = %ext_sql);

        let rendered = match (item.overridden, &item.operator) {
            (None, Some(op)) => {
                let mut optionals = vec![];
                if let Some(it) = op.commutator {
                    optionals.push(format!("\tCOMMUTATOR = {}", it));
                };
                if let Some(it) = op.negator {
                    optionals.push(format!("\tNEGATOR = {}", it));
                };
                if let Some(it) = op.restrict {
                    optionals.push(format!("\tRESTRICT = {}", it));
                };
                if let Some(it) = op.join {
                    optionals.push(format!("\tJOIN = {}", it));
                };
                if op.hashes {
                    optionals.push(String::from("\tHASHES"));
                };
                if op.merges {
                    optionals.push(String::from("\tMERGES"));
                };

                let left_arg = item.fn_args.get(0).ok_or_else(|| eyre_err!("Did not find `left_arg` for operator `{}`.", item.name))?;
                let right_arg = item.fn_args.get(1).ok_or_else(|| eyre_err!("Did not find `left_arg` for operator `{}`.", item.name))?;

                let operator_sql = format!("\n\
                                        -- {file}:{line}\n\
                                        -- {module_path}::{name}\n\
                                        CREATE OPERATOR {opname} (\n\
                                            \tPROCEDURE=\"{name}\",\n\
                                            \tLEFTARG={schema_prefix_left}{left_arg}, /* {left_name} */\n\
                                            \tRIGHTARG={schema_prefix_right}{right_arg}{maybe_comma} /* {right_name} */\n\
                                            {optionals}\
                                        );
                                    ",
                                           opname = op.opname.unwrap(),
                                           file = item.file,
                                           line = item.line,
                                           name = item.name,
                                           module_path = item.module_path,
                                           left_name = left_arg.ty_name,
                                           right_name = right_arg.ty_name,
                                           schema_prefix_left = self.schema_prefix_for(left_arg.module_path.clone()),
                                           left_arg = self.type_id_to_sql_type(left_arg.ty_id).ok_or_else(|| eyre_err!("Failed to map argument `{}` type `{}` to SQL type while building operator `{}`.", left_arg.pattern, left_arg.ty_name, item.name))?,
                                           schema_prefix_right = self.schema_prefix_for(right_arg.module_path.clone()),
                                           right_arg = self.type_id_to_sql_type(right_arg.ty_id).ok_or_else(|| eyre_err!("Failed to map argument `{}` type `{}` to SQL type while building operator `{}`.", right_arg.pattern, right_arg.ty_name, item.name))?,
                                           maybe_comma = if optionals.len() >= 1 { "," } else { "" },
                                           optionals = optionals.join(",\n") + "\n"
                );
                tracing::debug!(sql = %operator_sql);
                ext_sql + &operator_sql
            },
            (None, None) | (Some(_), Some(_)) | (Some(_), None) => ext_sql,
        };
        Ok(rendered)
    }

    #[instrument(level = "info", err, skip(self, item), fields(item = item.full_path))]
    fn inventory_type_to_sql(&self, item: &InventoryPostgresType) -> eyre::Result<String> {
        // The `in_fn`/`out_fn` need to be present in a certain order:
        // - CREATE TYPE;
        // - CREATE FUNCTION _in;
        // - CREATE FUNCTION _out;
        // - CREATE TYPE (...);

        let in_fn_module_path = if !item.in_fn_module_path.is_empty() {
            item.in_fn_module_path.clone()
        } else {
            item.module_path.to_string() // Presume a local
        };
        let in_fn_path = format!("{module_path}{maybe_colons}{in_fn}",
                                  module_path = in_fn_module_path,
                                  maybe_colons = if !in_fn_module_path.is_empty() { "::" } else { "" },
                                  in_fn = item.in_fn,
        );
        let (_, in_fn) = self.externs.iter().find(|(k, _v)| {
            tracing::trace!(%k, %in_fn_path, "Checked");
            **k == in_fn_path.as_str()
        }).ok_or_else(|| eyre::eyre!("Did not find `in_fn: {}`.", in_fn_path))?;
        tracing::trace!(in_fn = ?in_fn_path, "Found matching `in_fn`");
        let in_fn_sql = self.inventory_extern_to_sql(in_fn)?;
        tracing::trace!(%in_fn_sql);

        let out_fn_module_path = if !item.out_fn_module_path.is_empty() {
            item.out_fn_module_path.clone()
        } else {
            item.module_path.to_string() // Presume a local
        };
        let out_fn_path = format!("{module_path}{maybe_colons}{out_fn}",
                                  module_path = out_fn_module_path,
                                  maybe_colons = if !out_fn_module_path.is_empty() { "::" } else { "" },
                                  out_fn = item.out_fn,
        );
        let (_, out_fn) = self.externs.iter().find(|(k, _v)| {
            tracing::trace!(%k, %out_fn_path, "Checked");
            **k == out_fn_path.as_str()
        }).ok_or_else(|| eyre::eyre!("Did not find `out_fn: {}`.", out_fn_path))?;
        tracing::trace!(out_fn = ?out_fn_path, "Found matching `out_fn`");
        let out_fn_sql = self.inventory_extern_to_sql(out_fn)?;
        tracing::trace!(%out_fn_sql);

        let shell_type = format!("\n\
                                -- {file}:{line}\n\
                                -- {full_path}\n\
                                CREATE TYPE {schema}{name};\n\
                            ",
                                 schema = self.schema_prefix_for(item.module_path),
                                 full_path = item.full_path,
                                 file = item.file,
                                 line = item.line,
                                 name = item.name,
        );
        tracing::debug!(sql = %shell_type);

        let materialized_type = format!("\n\
                                -- {file}:{line}\n\
                                -- {full_path} - {id:?}\n\
                                CREATE TYPE {schema}{name} (\n\
                                    \tINTERNALLENGTH = variable,\n\
                                    \tINPUT = {schema_prefix_in_fn}{in_fn}, /* {in_fn_path} */\n\
                                    \tOUTPUT = {schema_prefix_out_fn}{out_fn}, /* {out_fn_path} */\n\
                                    \tSTORAGE = extended\n\
                                );
                            ",
                                        full_path = item.full_path,
                                        file = item.file,
                                        line = item.line,
                                        schema = self.schema_prefix_for(item.module_path),
                                        id = item.id,
                                        name = item.name,
                                        schema_prefix_in_fn = self.schema_prefix_for(in_fn_module_path.clone()),
                                        in_fn = item.in_fn,
                                        in_fn_path = in_fn_path,
                                        schema_prefix_out_fn = self.schema_prefix_for(out_fn_module_path.clone()),
                                        out_fn = item.out_fn,
                                        out_fn_path = out_fn_path,
        );
        tracing::debug!(sql = %materialized_type);

        Ok(shell_type + &in_fn_sql + &out_fn_sql + &materialized_type)
    }

    #[instrument(level = "info", skip(self, item), fields(item = item.full_path))]
    fn inventory_hash_to_sql(&self, item: &InventoryPostgresHash) -> String {
        let sql = format!("\n\
                            -- {file}:{line}\n\
                            -- {full_path}\n\
                            -- {id:?}\n\
                            CREATE OPERATOR FAMILY {name}_hash_ops USING hash;\n\
                            CREATE OPERATOR CLASS {name}_hash_ops DEFAULT FOR TYPE {name} USING hash FAMILY {name}_hash_ops AS\n\
                                \tOPERATOR    1   =  ({name}, {name}),\n\
                                \tFUNCTION    1   {name}_hash({name});\
                            ",
                          name = item.name,
                          full_path = item.full_path,
                          file = item.file,
                          line = item.line,
                          id = item.id,
        );
        tracing::debug!(%sql);
        sql
    }

    #[instrument(level = "info", skip(self, item), fields(item = item.full_path))]
    fn inventory_ord_to_sql(&self, item: &InventoryPostgresOrd) -> String {
        let sql = format!("\n\
                            -- {file}:{line}\n\
                            -- {full_path}\n\
                            -- {id:?}\n\
                            CREATE OPERATOR FAMILY {name}_btree_ops USING btree;\n\
                            CREATE OPERATOR CLASS {name}_btree_ops DEFAULT FOR TYPE {name} USING btree FAMILY {name}_btree_ops AS\n\
                                  \tOPERATOR 1 <,\n\
                                  \tOPERATOR 2 <=,\n\
                                  \tOPERATOR 3 =,\n\
                                  \tOPERATOR 4 >=,\n\
                                  \tOPERATOR 5 >,\n\
                                  \tFUNCTION 1 {name}_cmp({name}, {name});\n\
                            ",
                          name = item.name,
                          full_path = item.full_path,
                          file = item.file,
                          line = item.line,
                          id = item.id,
        );
        tracing::debug!(%sql);
        sql
    }


    #[instrument(level = "info", skip(self))]
    pub fn register_types(&mut self) {
        for (_full_path, item) in self.enums.clone() {
            self.map_type_id_to_sql_type(item.id, item.name);
            self.map_type_id_to_sql_type(item.option_id, item.name);
            self.map_type_id_to_sql_type(item.vec_id, format!("{}[]", item.name));
            if let Some(val) = item.varlena_id {
                self.map_type_id_to_sql_type(val, item.name);
            }
            if let Some(val) = item.array_id {
                self.map_type_id_to_sql_type(val, format!("{}[]", item.name));
            }
            if let Some(val) = item.option_array_id {
                self.map_type_id_to_sql_type(val, format!("{}[]", item.name));
            }
        }
        for (_full_path, item) in self.types.clone() {
            self.map_type_id_to_sql_type(item.id, item.name);
            self.map_type_id_to_sql_type(item.option_id, item.name);
            self.map_type_id_to_sql_type(item.vec_id, format!("{}[]", item.name));
            self.map_type_id_to_sql_type(item.vec_option_id, format!("{}[]", item.name));
            if let Some(val) = item.varlena_id {
                self.map_type_id_to_sql_type(val, item.name);
            }
            if let Some(val) = item.array_id {
                self.map_type_id_to_sql_type(val, format!("{}[]", item.name));
            }
            if let Some(val) = item.option_array_id {
                self.map_type_id_to_sql_type(val, format!("{}[]", item.name));
            }
        }
    }

    #[instrument(level = "debug")]
    pub fn type_id_to_sql_type(&self, id: TypeId) -> Option<String> {
        self.type_mappings
            .get(&id)
            .map(|f| f.clone())
    }

    #[instrument(level = "debug")]
    pub fn map_type_to_sql_type<T: 'static>(&mut self, sql: impl AsRef<str> + Debug) {
        let sql = sql.as_ref().to_string();
        self.type_mappings
            .insert(TypeId::of::<T>(), sql.clone());
    }

    #[instrument(level = "debug")]
    pub fn map_type_id_to_sql_type(&mut self, id: TypeId, sql: impl AsRef<str> + Debug) {
        let sql = sql.as_ref().to_string();
        self.type_mappings.insert(id, sql);
    }
}
