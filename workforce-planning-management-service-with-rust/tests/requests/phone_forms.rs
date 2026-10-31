//! A telephone number kept three ways (WPM-D152 question 2, WPM-T382): the migration's SQL backfill and the service's
//! rule agree on the same rows; a change of the place of work re-derives the E.164 form; HR searches by digits, never by
//! the typed text, and the answer names no number.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::rules::phone;

use super::{an_org, seed_worker};

/// Insert a worker's contact details as an old row would be: typed values only, no derived forms.
async fn insert_raw(
    db: &impl ConnectionTrait,
    worker: &str,
    country: Option<&str>,
    mobile: &str,
    home: &str,
) {
    let country = country.map_or_else(|| "NULL".to_string(), |c| format!("'{c}'"));
    db.execute_unprepared(&format!(
        "INSERT INTO worker_contact_details (worker_pid, country, phone_mobile, phone_home, on_behalf)
         VALUES ('{worker}', {country}, '{mobile}', '{home}', false)"
    ))
    .await
    .expect("insert contact details");
}

async fn backfill(db: &impl ConnectionTrait) {
    for sql in migration::m20261031_000114_phone_forms::backfill() {
        db.execute_unprepared(&sql).await.expect("backfill");
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_backfill_and_the_rule_agree_and_a_change_of_place_rederives() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = &uuid::Uuid::new_v4().to_string()[..8];
        let cases: [(Option<&str>, Option<&str>, &str, &str); 6] = [
            (Some("KR"), None, "٠١٠-١٢٣٤-٥٦٧٨", "+44 7700 900123"),
            (None, Some("IN-MH"), "098765 43210", "0044 20 7946 0958"),
            (Some("US"), Some("IN-MH"), "(202) 555-0123", "555-0123"),
            (Some("NP"), Some("IN"), "01 4123456", "＋８１ ９０-１２３４-５６７８"),
            (Some("HU"), None, "06 1 234 5678", "8 495 123-45-67"),
            (Some("IT"), Some("QZ"), "06 1234 5678", "+0 123 4567"),
        ];
        let mut pids = Vec::new();
        for (i, (country, place, mobile, home)) in cases.iter().enumerate() {
            let pid = seed_worker!(&request, org.clone(), format!("PF-{tag}-{i}"), None).await;
            if let Some(place) = place {
                request.put(&format!("/api/workers/{pid}")).json(&json!({ "work_jurisdiction": place })).await.assert_status_ok();
            }
            insert_raw(&ctx.db, &pid, *country, mobile, home).await;
            pids.push(pid);
        }
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO emergency_contacts (pid, worker_pid, name, relationship, phone, alt_phone, priority)
                 VALUES ('{}', '{}', 'Test Contact', 'sibling', '+٩٦٦ ٥٠ ١٢٣ ٤٥٦٧', '01632 960001', 1)",
                uuid::Uuid::new_v4(),
                pids[0]
            ))
            .await
            .expect("insert contact");
        backfill(&ctx.db).await;

        for ((country, place, mobile, home), pid) in cases.iter().zip(&pids) {
            let row = ctx
                .db
                .query_one_raw(sea_orm::Statement::from_string(
                    sea_orm::DbBackend::Postgres,
                    format!(
                        "SELECT phone_mobile_digits, phone_mobile_e164, phone_home_digits, phone_home_e164
                           FROM worker_contact_details WHERE worker_pid = '{pid}'"
                    ),
                ))
                .await
                .expect("read")
                .expect("a row");
            let get = |c: &str| row.try_get::<Option<String>>("", c).expect(c);
            let read_in = phone::country_for(*country, *place);
            let (m, h) = (phone::forms(Some(mobile), read_in), phone::forms(Some(home), read_in));
            assert_eq!((get("phone_mobile_digits"), get("phone_mobile_e164")), (m.digits, m.e164), "{mobile} in {read_in:?}");
            assert_eq!((get("phone_home_digits"), get("phone_home_e164")), (h.digits, h.e164), "{home} in {read_in:?}");
        }
        // Spot checks of what the rule says, so agreement is not agreement on nothing.
        let one: Value = request.get(&format!("/api/workers/{}/contact-details", pids[0])).await.json();
        assert_eq!(one["contact_details"]["phone_mobile"], "٠١٠-١٢٣٤-٥٦٧٨", "as typed, unchanged");
        assert_eq!(one["contact_details"]["phone_mobile_digits"], "01012345678");
        assert_eq!(one["contact_details"]["phone_mobile_e164"], "+821012345678");
        assert_eq!(one["contact_details"]["phone_home_e164"], "+447700900123");
        let two: Value = request.get(&format!("/api/workers/{}/contact-details", pids[1])).await.json();
        assert_eq!(two["contact_details"]["phone_mobile_e164"], "+919876543210", "read in the place of work's country");
        assert_eq!(two["contact_details"]["phone_home_e164"], Value::Null, "an access prefix is not read");
        let four: Value = request.get(&format!("/api/workers/{}/contact-details", pids[3])).await.json();
        assert_eq!(four["contact_details"]["phone_mobile_e164"], Value::Null, "an address country with no plan is not replaced");
        let contacts: Vec<Value> = request.get(&format!("/api/workers/{}/emergency-contacts", pids[0])).await.json();
        assert_eq!(contacts[0]["phone_digits"], "966501234567");
        assert_eq!(contacts[0]["phone_e164"], "+966501234567");
        assert_eq!(contacts[0]["alt_phone_digits"], "01632960001");
        assert_eq!(contacts[0]["alt_phone_e164"], Value::Null, "a contact's number is not read in the worker's country");
        // Running it again changes nothing.
        backfill(&ctx.db).await;
        let again: Value = request.get(&format!("/api/workers/{}/contact-details", pids[0])).await.json();
        assert_eq!(again, one);

        // HR moves the second worker's place of work to a code with no plan: the derived form goes, the typed one stays.
        request.put(&format!("/api/workers/{}", pids[1])).json(&json!({ "work_jurisdiction": "QZ" })).await.assert_status_ok();
        let moved: Value = request.get(&format!("/api/workers/{}/contact-details", pids[1])).await.json();
        assert_eq!(moved["contact_details"]["phone_mobile_e164"], Value::Null);
        assert_eq!(moved["contact_details"]["phone_mobile_digits"], "09876543210");
        request.put(&format!("/api/workers/{}", pids[1])).json(&json!({ "work_jurisdiction": "IN-KA" })).await.assert_status_ok();
        let back: Value = request.get(&format!("/api/workers/{}/contact-details", pids[1])).await.json();
        assert_eq!(back["contact_details"]["phone_mobile_e164"], "+919876543210");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn hr_finds_workers_by_digits_and_the_answer_names_no_number() {
    crate::requests::request_open(|request, ctx| async move {
        let (org, other) = (an_org(), an_org());
        let tag = &uuid::Uuid::new_v4().to_string()[..8];
        let w1 = seed_worker!(&request, org.clone(), format!("PS-{tag}-1"), None).await;
        let w2 = seed_worker!(&request, org.clone(), format!("PS-{tag}-2"), None).await;
        let elsewhere = seed_worker!(&request, other.clone(), format!("PS-{tag}-3"), None).await;
        insert_raw(&ctx.db, &w1, Some("KR"), "٠١٠-١٢٣٤-٥٦٧٨", "02 555 0100").await;
        insert_raw(&ctx.db, &w2, None, "07700 900123", "07700 900124").await;
        insert_raw(
            &ctx.db,
            &elsewhere,
            Some("KR"),
            "010-1234-5678",
            "02 555 0100",
        )
        .await;
        backfill(&ctx.db).await;

        let search = |organization: String, phone: &'static str| {
            let request = &request;
            async move {
                request
                    .get("/api/contact-details/search")
                    .add_query_param("organization", organization)
                    .add_query_param("phone", phone)
                    .await
            }
        };
        for query in [
            "010 1234 5678",
            "+82 10-1234-5678",
            "10 1234 5678",
            "٠١٠١٢٣٤٥٦٧٨",
        ] {
            let found: Value = search(org.clone(), query).await.json();
            assert_eq!(
                found["workers"].as_array().map(Vec::len),
                Some(1),
                "{query}: only this organization's worker"
            );
            assert_eq!(found["workers"][0]["pid"], w1.as_str());
            assert_eq!(found["workers"][0]["matched"], json!(["phone_mobile"]));
            let text = found.to_string();
            assert!(
                !text.contains("5678") && !text.contains("phone_mobile_digits"),
                "no number in the answer"
            );
        }
        let home: Value = search(org.clone(), "07700 900124").await.json();
        assert_eq!(home["workers"][0]["pid"], w2.as_str());
        assert_eq!(home["workers"][0]["matched"], json!(["phone_home"]));
        let none: Value = search(org.clone(), "+44 7700 900123").await.json();
        assert_eq!(
            none["workers"],
            json!([]),
            "a national number with no known country has no E.164 form to match"
        );
        assert_eq!(
            search(org.clone(), "0001234").await.status_code(),
            422,
            "too few digits after the zeros"
        );
        assert_eq!(search(org.clone(), "call me").await.status_code(), 422);
        assert_eq!(
            search("not-an-org".to_string(), "010 1234 5678")
                .await
                .status_code(),
            422
        );
        let audits: Value = request.get(&format!("/api/audits/{w1}")).await.json();
        let text = audits.to_string();
        assert!(text.contains("contact_details_found"));
        assert!(
            !text.contains("01012345678")
                && !text.contains("+821012345678")
                && !text.contains("٠١٠"),
            "the audit holds no number"
        );
    })
    .await;
}
