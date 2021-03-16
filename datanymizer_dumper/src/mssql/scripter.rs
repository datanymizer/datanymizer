const DB_OBJECT_SIGNATURE: &[u8] = b"/****** Object:  ";
const TABLE_SIGNATURE: &[u8] = b"Table ";

const DB_OBJECT_SIGNATURE_LEN: usize = DB_OBJECT_SIGNATURE.len();
const TABLE_SIGNATURE_LEN: usize = DB_OBJECT_SIGNATURE_LEN + TABLE_SIGNATURE.len();

pub fn pre_data(data: &Vec<u8>) -> &[u8] {
    &data[..end_of_table_section(&data)]
}

pub fn post_data(data: &Vec<u8>) -> &[u8] {
    &data[end_of_table_section(&data)..]
}

fn end_of_table_section(data: &Vec<u8>) -> usize {
    let mut index = 0;
    let mut table_section_started = false;
    let mut iter = data.split(|&i| i == b'\n');

    while let Some(line) = iter.next() {
        let kind = LineKind::kind_of(line);
        if table_section_started {
            if let LineKind::OtherObject = kind {
                break;
            }
        } else if let LineKind::Table = kind {
            table_section_started = true;
        }

        index += line.len() + 1;
    }

    if index > data.len() {
        index = data.len();
    }

    index
}

enum LineKind {
    Table,
    OtherObject,
    Regular,
}

impl LineKind {
    fn kind_of(line: &[u8]) -> Self {
        if line.len() > TABLE_SIGNATURE_LEN && line.starts_with(DB_OBJECT_SIGNATURE) {
            if line[DB_OBJECT_SIGNATURE_LEN..TABLE_SIGNATURE_LEN] == *TABLE_SIGNATURE {
                LineKind::Table
            } else {
                LineKind::OtherObject
            }
        } else {
            LineKind::Regular
        }
    }
}
