use utoipa_config::Config;
fn main() {
    Config::new()
        .alias_for(
            "DateTimeWithTimeZone",
            "crate::chrono::DateTime<chrono::FixedOffset>",
        )
        .write_to_file();
}
