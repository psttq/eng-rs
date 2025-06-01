pub struct Label{
    pub label: String
}

impl Label {
    pub fn new(label: String) -> Self{
        Self { label }
    }

    pub fn from_str(label: &str) -> Self{
        Self {label: label.to_string()}
    }
}