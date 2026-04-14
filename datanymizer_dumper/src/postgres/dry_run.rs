use super::{connector, schema_inspector::PgSchemaInspector};
use crate::{SchemaInspector, Table};
use anyhow::Result;
use datanymizer_engine::Settings;

pub fn run(connection: &mut connector::Connection, settings: &mut Settings) -> Result<()> {
    let inspector = PgSchemaInspector {};
    let tables = inspector.get_tables(connection)?;

    for table in &tables {
        settings.register_table_transforms(
            &table.get_full_name(),
            &table.get_name(),
            &table.get_columns_names(),
        );
    }

    let mut matched: Vec<(String, String, Vec<String>)> = Vec::new();
    let mut unmatched: Vec<String> = Vec::new();
    let mut no_cols: Vec<String> = Vec::new();

    for table in &tables {
        let full_name = table.get_full_name();
        let names = table.get_names();
        let actual_columns = table.get_columns_names();

        match settings.dry_run_info(&full_name, &names) {
            Some((rule_label, cols)) => {
                // Filter to columns that actually exist in this table
                let existing: Vec<String> = cols
                    .into_iter()
                    .filter(|c| actual_columns.contains(c))
                    .collect();
                if !existing.is_empty() {
                    matched.push((full_name, rule_label, existing));
                } else {
                    no_cols.push(full_name);
                }
            }
            None => unmatched.push(full_name),
        }
    }

    let matched_name_width = matched
        .iter()
        .map(|(n, _, _)| n.len())
        .max()
        .unwrap_or(0);

    let rule_width = matched
        .iter()
        .map(|(_, r, _)| r.len())
        .max()
        .unwrap_or(0);

    for (full_name, rule_label, cols) in &matched {
        let col_str = cols.join("  ");
        println!("{full_name:<matched_name_width$}  [{rule_label:<rule_width$}]  {col_str}");
    }

    if !unmatched.is_empty() {
        let count = unmatched.len();
        println!("\n── unmatched ({count}) ──");
        for name in &unmatched {
            println!("  {name}");
        }
    }

    if !no_cols.is_empty() {
        let count = no_cols.len();
        eprintln!("\nerror: {count} configured table(s) have no matching columns:");
        for name in &no_cols {
            eprintln!("  {name}");
        }
        anyhow::bail!("config validation failed: no columns matched");
    }

    Ok(())
}
