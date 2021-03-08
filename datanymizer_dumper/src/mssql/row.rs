use crate::{mssql::MsSqlType, Table};

#[derive(Debug)]
pub struct MsSqlRow<T>
where
    T: Table<MsSqlType>,
{
    table: T,
    source: String,
}

#[cfg(test)]
mod tests {
    use super::*;
}
