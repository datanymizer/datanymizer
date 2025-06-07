use super::key::Key;
use std::{collections::HashMap, rc::Rc};

pub(crate) type KeyColMap = HashMap<String, Rc<Box<dyn Key>>>;

pub struct GenTable {
    pub row_count: usize,
    keys: KeyColMap,
}

impl GenTable {
    pub fn new(row_count: usize, keys: KeyColMap) -> Self {
        Self { row_count, keys }
    }
}

pub struct GenTableIter {
    i: usize,
    len: usize,
    row_base: Vec<Option<Rc<Box<dyn Key>>>>,
}

impl GenTableIter {
    pub fn new(table: &GenTable, column_indexes: HashMap<String, usize>) -> Self {
        let i = 0;
        let len = table.row_count;
        let col_count = column_indexes.len();
        let mut row_base: Vec<Option<Rc<Box<dyn Key>>>> = Vec::with_capacity(col_count);
        row_base.resize_with(col_count, || None);
        for (name, key) in &table.keys {
            let i = *column_indexes.get(name).expect("Missing key");
            row_base[i] = Some(key.clone());
        }

        Self { i, len, row_base }
    }
}

impl Iterator for GenTableIter {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.len {
            self.i += 1;
            Some(
                self.row_base
                    .iter()
                    .map(|col| {
                        col.as_ref()
                            .map_or(String::new(), |key| key.index(self.i - 1).to_string())
                    })
                    .collect(),
            )
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::{
        super::{foreign_key::MonotonicFKey, key::MonotonicKey},
        *,
    };
    use std::rc::Rc;

    fn assert_row(t: &mut GenTableIter, vals: Vec<&str>) {
        assert_eq!(
            t.next(),
            Some(vals.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        );
    }

    #[test]
    fn monotonic_fk_iteration() {
        let mut all_keys = KeyColMap::new();

        let tbl1_len = 6;
        all_keys.insert(
            "t1.id".to_string(),
            Rc::new(Box::new(MonotonicKey::new(1, tbl1_len))),
        );
        all_keys.insert(
            "t2.id".to_string(),
            Rc::new(Box::new(MonotonicKey::new(1, 3))),
        );
        let id = all_keys["t2.id"].clone();
        all_keys.insert(
            "t1.fk1".to_string(),
            Rc::new(Box::new(MonotonicFKey::new(id, tbl1_len))),
        );

        let mut keys = KeyColMap::new();
        keys.insert("t1.id".to_string(), all_keys["t1.id"].clone());
        keys.insert("t1.fk1".to_string(), all_keys["t1.fk1"].clone());
        let gen_table = GenTable::new(tbl1_len, keys);

        let mut col_indexes = HashMap::new();
        col_indexes.insert("t1.id".to_string(), 0);
        col_indexes.insert("t1.fk1".to_string(), 1);
        col_indexes.insert("t1.name".to_string(), 2);

        let mut gen_iter = GenTableIter::new(&gen_table, col_indexes);
        assert_row(&mut gen_iter, vec!["1", "1", ""]);
        assert_row(&mut gen_iter, vec!["2", "1", ""]);
        assert_row(&mut gen_iter, vec!["3", "2", ""]);
        assert_row(&mut gen_iter, vec!["4", "2", ""]);
        assert_row(&mut gen_iter, vec!["5", "3", ""]);
        assert_row(&mut gen_iter, vec!["6", "3", ""]);
        assert_eq!(gen_iter.next(), None);
    }
}
