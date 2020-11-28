pub struct StringValue {
    pub table_name: String,
    pub field_name: String,
    pub value: String,
}

impl StringValue {
    pub fn update(&mut self, new_value: String) {
        self.value = new_value;
    }
}
