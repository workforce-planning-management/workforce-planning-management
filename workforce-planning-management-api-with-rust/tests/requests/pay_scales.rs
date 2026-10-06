//! Pay scales: the Agenda for Change (Wales) scale is served from the pay
//! circular, and the lookup places a salary and answers progression — refusing
//! what it cannot answer rather than guessing.

use loco_rs::testing::prelude::*;
use serde_json::Value;
use serial_test::serial;
use workforce_planning_management_service::app::App;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_wales_scale_is_served_and_salaries_are_placed_on_it() {
    request::<App, _, _>(|request, _ctx| async move {
        // The list names the scale, its source and its bands.
        let list: Value = request.get("/api/pay-scales").await.json();
        let wales = &list.as_array().unwrap()[0];
        assert_eq!(wales["id"], "afc-wales-2026-27");
        assert_eq!(wales["nation"], "wales");
        assert_eq!(wales["effective_from"], "2026-04-01");
        assert!(wales["source"].as_str().unwrap().contains("AfC(W) 02/2026"));
        assert_eq!(wales["bands"].as_array().unwrap().len(), 12);

        // The full scale: band 5 has three steps with years to progression.
        let scale: Value = request
            .get("/api/pay-scales/afc-wales-2026-27")
            .await
            .json();
        let band5 = scale["bands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["code"] == "5")
            .unwrap();
        assert_eq!(band5["steps"][0]["annual_minor"], 3_255_700);
        assert_eq!(band5["steps"][0]["years_to_next"], 2);
        assert_eq!(band5["steps"][2]["years_to_next"], Value::Null);
        assert_eq!(scale["allowances"].as_array().unwrap().len(), 3);
        assert_eq!(
            request
                .get("/api/pay-scales/afc-england-2026-27")
                .await
                .status_code(),
            404
        );

        // On a step, between steps, below entry, above the top.
        let at = |q: &'static str| {
            let request = &request;
            async move {
                let r = request
                    .get(&format!("/api/pay-scales/afc-wales-2026-27/position?{q}"))
                    .await;
                (r.status_code(), r.json::<Value>())
            }
        };
        let (status, v) = at("band=5&salary_minor=3511400").await;
        assert_eq!(status, 200);
        assert_eq!(
            v["position"],
            serde_json::json!({ "kind": "on_step", "step": 2 })
        );
        let (_, v) = at("band=5&salary_minor=3400000").await;
        assert_eq!(v["position"]["kind"], "between_steps");
        assert_eq!(v["position"]["below_step"], 1);
        let (_, v) = at("band=5&salary_minor=3000000").await;
        assert_eq!(v["position"]["kind"], "below_entry");
        assert_eq!(v["position"]["shortfall_minor"], 255_700);
        let (_, v) = at("band=5&salary_minor=4000000").await;
        assert_eq!(v["position"]["kind"], "above_top");
        // Band is case-insensitive and band 1 is flagged closed.
        let (_, v) = at("band=8A&salary_minor=5837900").await;
        assert_eq!(v["position"]["step"], 1);
        let (_, v) = at("band=1&salary_minor=2630000").await;
        assert_eq!(v["closed_to_new_entrants"], true);

        // Progression: not yet (one month short) then due, then nowhere to go.
        let (_, v) = at("band=6&step=1&months_on_step=23").await;
        assert_eq!(v["progression"]["kind"], "not_yet");
        assert_eq!(v["progression"]["months_remaining"], 1);
        assert_eq!(v["position"], Value::Null, "no salary asked, none placed");
        let (_, v) = at("band=6&step=1&months_on_step=24").await;
        assert_eq!(v["progression"]["kind"], "due");
        assert_eq!(v["progression"]["next_annual_minor"], 4_280_500);
        let (_, v) = at("band=6&step=3&months_on_step=240").await;
        assert_eq!(v["progression"]["kind"], "at_top");

        // What it cannot answer is refused (422), not guessed.
        for q in [
            "salary_minor=3511400",           // no band
            "band=11&salary_minor=3511400",   // unknown band
            "band=5",                         // nothing asked
            "band=5&salary_minor=-1",         // negative
            "band=5&step=1",                  // step without months
            "band=5&months_on_step=3",        // months without step
            "band=5&step=4&months_on_step=3", // no fourth step
            "band=5&step=0&months_on_step=3", // steps are 1-based
        ] {
            assert_eq!(at(q).await.0, 422, "{q}");
        }
        assert_eq!(
            request
                .get("/api/pay-scales/nope/position?band=5&salary_minor=1")
                .await
                .status_code(),
            404
        );
    })
    .await;
}
