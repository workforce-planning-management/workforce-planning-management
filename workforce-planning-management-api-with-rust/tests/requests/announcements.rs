//! The announcement feed: live posts for readers, pinned first then newest;
//! scheduled and expired posts only with `include=all`; validation; edit and
//! retire.

use chrono::{Duration, Utc};
use loco_rs::testing::prelude::*;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;

use super::an_org;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_feed_shows_live_posts_pinned_first_and_hides_the_rest() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let today = Utc::now().date_naive();
        let day = |n: i64| (today + Duration::days(n)).to_string();
        let post = |title: &str, extra: Value| {
            let mut body = json!({ "organization_ref": org, "title": title,
                                   "body": "Plain text <script>alert(1)</script> stays text." });
            for (k, v) in extra.as_object().unwrap() {
                body[k] = v.clone();
            }
            body
        };

        // Validation: blank title, empty body, expiry before publish.
        for bad in [
            json!({ "organization_ref": org, "title": " ", "body": "x" }),
            json!({ "organization_ref": org, "title": "x", "body": "" }),
            json!({ "organization_ref": org, "title": "x", "body": "y",
                    "publish_on": day(2), "expires_on": day(1) }),
        ] {
            assert_eq!(request.post("/api/announcements").json(&bad).await.status_code(), 422, "{bad}");
        }

        let older: Value = request
            .post("/api/announcements")
            .json(&post("Older", json!({ "publish_on": day(-3) })))
            .await
            .json();
        let newer: Value = request
            .post("/api/announcements")
            .json(&post("Newer", json!({})))
            .await
            .json();
        let pinned: Value = request
            .post("/api/announcements")
            .json(&post("Pinned", json!({ "pinned": true, "publish_on": day(-10) })))
            .await
            .json();
        request
            .post("/api/announcements")
            .json(&post("Soon", json!({ "publish_on": day(5) })))
            .await
            .assert_status_ok();
        request
            .post("/api/announcements")
            .json(&post("Over", json!({ "publish_on": day(-9), "expires_on": day(-1) })))
            .await
            .assert_status_ok();

        let feed = |extra: &str| format!("/api/announcements?organization={org}{extra}");
        let live: Vec<Value> = request.get(&feed("")).await.json();
        let titles: Vec<&str> = live.iter().map(|a| a["title"].as_str().unwrap()).collect();
        assert_eq!(titles, ["Pinned", "Newer", "Older"], "pinned first, then newest");
        assert!(live.iter().all(|a| a["status"] == "live"));
        // The body is kept verbatim as text; the API does not interpret it.
        assert_eq!(
            live[0]["body"],
            "Plain text <script>alert(1)</script> stays text."
        );

        let all: Vec<Value> = request.get(&feed("&include=all")).await.json();
        let statuses: Vec<&str> = all.iter().map(|a| a["status"].as_str().unwrap()).collect();
        assert_eq!(all.len(), 5);
        assert!(statuses.contains(&"scheduled") && statuses.contains(&"expired"));

        // Paging reports the total.
        let page = request.get(&feed("&limit=2&offset=1")).await;
        assert_eq!(page.header("x-total-count"), "3");
        let rows: Vec<Value> = page.json();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["title"], "Newer");

        // Edit: unpin, and clear a later-set expiry.
        let pinned_pid = pinned["pid"].as_str().unwrap();
        let edited: Value = request
            .put(&format!("/api/announcements/{pinned_pid}"))
            .json(&json!({ "pinned": false, "title": "Was pinned" }))
            .await
            .json();
        assert_eq!(edited["pinned"], false);
        let live: Vec<Value> = request.get(&feed("")).await.json();
        assert_eq!(live[0]["title"], "Newer", "newest now leads");
        assert_eq!(
            request
                .put(&format!("/api/announcements/{pinned_pid}"))
                .json(&json!({ "expires_on": day(-20) }))
                .await
                .status_code(),
            422,
            "an expiry before the publish day is refused"
        );

        // Retire one: gone from the feed and from a direct read.
        let older_pid = older["pid"].as_str().unwrap();
        request.delete(&format!("/api/announcements/{older_pid}")).await.assert_status_ok();
        assert_eq!(
            request.get(&format!("/api/announcements/{older_pid}")).await.status_code(),
            404
        );
        let live: Vec<Value> = request.get(&feed("")).await.json();
        assert_eq!(live.len(), 2);
        assert!(newer["pid"].is_string());
    })
    .await;
}
