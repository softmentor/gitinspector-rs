use super::Report;

pub fn format(report: &Report) -> String {
    quick_xml::se::to_string(report).unwrap_or_else(|e| format!("<error>{}</error>", e))
}
