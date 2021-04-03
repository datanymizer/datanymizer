const DB_OBJECT_SIGNATURE: &[u8] = b"/****** Object:  ";
const TABLE_SIGNATURE: &[u8] = b"Table ";

const DB_OBJECT_SIGNATURE_LEN: usize = DB_OBJECT_SIGNATURE.len();
const TABLE_SIGNATURE_LEN: usize = DB_OBJECT_SIGNATURE_LEN + TABLE_SIGNATURE.len();

pub(crate) struct SchemaDump {
    data: Vec<u8>,
    pre_bound: usize,
}

impl SchemaDump {
    pub fn new(data: Vec<u8>) -> Self {
        let pre_bound = Self::end_of_table_section(&data);
        Self { data, pre_bound }
    }

    pub fn pre_data(&self) -> &[u8] {
        &self.data[..self.pre_bound]
    }

    pub fn post_data(&self) -> &[u8] {
        &self.data[self.pre_bound..]
    }

    fn end_of_table_section(data: &[u8]) -> usize {
        let mut index = 0;
        let mut end_of_table_section = 0;
        let mut prev_obj_was_table = false;

        for line in data.split(|&i| i == b'\n') {
            let kind = LineKind::kind_of(line);
            if prev_obj_was_table {
                if let LineKind::OtherObject = kind {
                    prev_obj_was_table = false;
                    end_of_table_section = index;
                }
            } else if let LineKind::Table = kind {
                prev_obj_was_table = true;
            }

            index += line.len() + 1;
        }

        end_of_table_section
    }
}

enum LineKind {
    Table,
    OtherObject,
    Regular,
}

impl LineKind {
    fn kind_of(line: &[u8]) -> Self {
        if line.len() > TABLE_SIGNATURE_LEN && line.starts_with(DB_OBJECT_SIGNATURE) {
            // eprintln!("{}", std::str::from_utf8(line).unwrap());
            if line[DB_OBJECT_SIGNATURE_LEN..TABLE_SIGNATURE_LEN] == *TABLE_SIGNATURE {
                Self::Table
            } else {
                Self::OtherObject
            }
        } else {
            Self::Regular
        }
    }
}
