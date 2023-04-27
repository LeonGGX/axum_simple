//! tests/quick_dev.rs
//!
//! use by opening two terminals.
//! One for the responses to the client (client terminal)
//! one for the server messages (backend terminal)
//!
//! use cargo watch in both terminals.
//!
//! backend terminal :
//! cargo watch -q -c -w src/ -x run
//! ( -q = quiet, -c = clear, -w = watch followed by directory to watch, -x = , )
//!
//! client terminal :
//! cargo watch -q -c -w tests/ -x "test -q quick_dev -- --nocapture"
//!
#![allow(unused)]

use anyhow::Result;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://127.0.0.1:3000")?;
    //hc.do_get("/").await?.print().await?;
    /*hc.do_post(
        "/auth/login",
        (
            "username=L%C3%A9on&password=123456",
            "application/x-www-form-urlencoded",
        ),
    )
    .await?
    .print()
    .await?;*/
    hc.do_get("/api/welcome").await?.print().await?;

    Ok(())
}
