use super::sql_type::MsSqlType;
use datanymizer_dumper::Table;

#[derive(Debug)]
pub struct MsSqlRow<T>
where
    T: Table<MsSqlType>,
{
    _table: T,
    _source: String,
}

#[cfg(test)]
mod tests {}
