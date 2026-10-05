//! The announcement feed: live posts for readers, pinned first then newest;
//! scheduled and expired posts only with `include=all`; validation; edit and
//! retire.

use chrono::{Duration, Utc};
use loco_rs::testing::prelude::*;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;

use super::{activate, an_org, seed_worker};

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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn announcements_can_target_a_department_carry_links_and_count_reads() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let post = |extra: Value| {
            let mut body = json!({ "organization_ref": org, "title": "T", "body": "B" });
            for (k, v) in extra.as_object().unwrap() {
                body[k] = v.clone();
            }
            body
        };

        // Links: https only, at most three, each labelled.
        let four: Vec<Value> = (1..=4)
            .map(|n| json!({ "label": format!("l{n}"), "url": "https://example.org" }))
            .collect();
        for bad in [
            json!({ "links": [{ "label": "x", "url": "http://example.org" }] }),
            json!({ "links": [{ "label": "x", "url": "javascript:alert(1)" }] }),
            json!({ "links": [{ "label": " ", "url": "https://example.org" }] }),
            json!({ "links": four }),
        ] {
            assert_eq!(
                request.post("/api/announcements").json(&post(bad.clone())).await.status_code(),
                422,
                "{bad}"
            );
        }
        let everyone: Value = request
            .post("/api/announcements")
            .json(&post(json!({ "title": format!("All {tag}"),
                                "links": [{ "label": "Handbook", "url": "https://intranet.example.org/hb" }] })))
            .await
            .json();
        let finance: Value = request
            .post("/api/announcements")
            .json(&post(json!({ "title": format!("Finance {tag}"), "department": "Finance" })))
            .await
            .json();
        let eng: Value = request
            .post("/api/announcements")
            .json(&post(json!({ "title": format!("Eng {tag}"), "department": "engineering" })))
            .await
            .json();

        // The stored links and department come back.
        let feed = |extra: &str| format!("/api/announcements?organization={org}{extra}");
        let all: Vec<Value> = request.get(&feed("")).await.json();
        let everyone_row = all.iter().find(|a| a["pid"] == everyone["pid"]).unwrap();
        assert_eq!(everyone_row["links"][0]["label"], "Handbook");
        assert!(everyone_row["department"].is_null());

        // `?department=` shows the organization-wide posts plus that department's.
        let titles = |rows: &[Value]| -> Vec<String> {
            let mut t: Vec<String> = rows.iter().map(|a| a["title"].as_str().unwrap().to_string()).collect();
            t.sort();
            t
        };
        let for_eng: Vec<Value> = request.get(&feed("&department=ENGINEERING")).await.json();
        assert_eq!(titles(&for_eng), [format!("All {tag}"), format!("Eng {tag}")]);
        let for_fin: Vec<Value> = request.get(&feed("&department=finance")).await.json();
        assert_eq!(titles(&for_fin), [format!("All {tag}"), format!("Finance {tag}")]);

        // Edit: re-aim at everyone, and replace the links.
        let eng_pid = eng["pid"].as_str().unwrap();
        let edited: Value = request
            .put(&format!("/api/announcements/{eng_pid}"))
            .json(&json!({ "clear_department": true,
                           "links": [{ "label": "Doc", "url": "https://docs.example.org/x" }] }))
            .await
            .json();
        assert!(edited["department"].is_null());
        assert_eq!(edited["links"][0]["label"], "Doc");

        // Read receipts: an engineering worker reads the org-wide post (once, idempotently);
        // a Finance-only post is not theirs to read.
        let worker = seed_worker!(&request, &org, format!("AN-{tag}"), None).await;
        activate!(&request, &worker).await;
        let read_url = |pid: &Value| format!("/api/announcements/{}/read", pid.as_str().unwrap());
        for _ in 0..2 {
            request
                .post(&read_url(&everyone["pid"]))
                .json(&json!({ "worker_pid": worker }))
                .await
                .assert_status_ok();
        }
        assert_eq!(
            request
                .post(&read_url(&finance["pid"]))
                .json(&json!({ "worker_pid": worker }))
                .await
                .status_code(),
            404,
            "a post for another department is not theirs"
        );
        let mine: Vec<String> = request
            .get(&format!("/api/workers/{worker}/announcement-reads"))
            .await
            .json();
        assert_eq!(mine, [everyone["pid"].as_str().unwrap()]);

        // Editors see a count — never who.
        let counted: Vec<Value> = request.get(&feed("")).await.json();
        let row = counted.iter().find(|a| a["pid"] == everyone["pid"]).unwrap();
        assert_eq!(row["read_count"], 1);
        assert!(row.get("readers").is_none() && row.get("read_by").is_none());
        let unread = counted.iter().find(|a| a["pid"] == finance["pid"]).unwrap();
        assert_eq!(unread["read_count"], 0);
    })
    .await;
}
