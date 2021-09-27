use anyhow::Result;
use postgres::types::ToSql;
use postgres::{Client, CopyOutReader, IsolationLevel, Row, ToStatement, Transaction};

pub enum QueryWrapper<'a> {
    WithTransaction(Transaction<'a>),
    WithoutTransaction(&'a mut Client),
}

impl<'a> QueryWrapper<'a> {
    pub fn with_iso_level(cl: &'a mut Client, level: Option<IsolationLevel>) -> Result<Self> {
        let wrapper = match level {
            Some(level) => {
                let tr = cl.build_transaction().isolation_level(level).start()?;
                Self::WithTransaction(tr)
            }
            None => Self::WithoutTransaction(cl),
        };
        Ok(wrapper)
    }

    pub fn copy_out<T>(&mut self, query: &T) -> Result<CopyOutReader<'_>, postgres::Error>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            Self::WithTransaction(t) => t.copy_out(query),
            Self::WithoutTransaction(c) => c.copy_out(query),
        }
    }

    pub fn query_one<T>(
        &mut self,
        query: &T,
        params: &[&(dyn ToSql + Sync)],
    ) -> Result<Row, postgres::Error>
    where
        T: ?Sized + ToStatement,
    {
        match self {
            Self::WithTransaction(t) => t.query_one(query, params),
            Self::WithoutTransaction(c) => c.query_one(query, params),
        }
    }
}
