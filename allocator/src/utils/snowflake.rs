use std::sync::LazyLock;
use snowflake_me::Snowflake;

static SNOWFLAKE: LazyLock<Snowflake> = LazyLock::new(|| {
    Snowflake::new().expect("Failed to initialize Snowflake ID generator")
});

pub fn next_id() -> String {
    SNOWFLAKE.next_id()
        .expect("Failed to generate Snowflake ID")
        .to_string()
}
