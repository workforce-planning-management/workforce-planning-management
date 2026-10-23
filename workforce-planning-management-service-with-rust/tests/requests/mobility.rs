//! Internal mobility's opportunities list and the API documentation routes: a worker sees open
//! requisitions in their own organization other than their own job title, with a null fit
//! (no evidence, not a poor fit) where no role profile exists; the OpenAPI document and the
//! Swagger page are served and name this service.

use serde_json::{Value, json};
use serial_test::serial;

use super::{an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn opportunities_are_open_requisitions_elsewhere_in_the_organization_with_unknown_fit_left_null()
 {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("OP-{tag}"), None).await;
        let own: Value = request.get(&format!("/api/workers/{worker}/opportunities")).await.json();
        assert_eq!(own["worker_pid"], worker.as_str());
        let own_title = request.get(&format!("/api/workers/{worker}")).await.json::<Value>()["job_title"]
            .as_str()
            .unwrap()
            .to_string();

        let open = |title: String, org: String| {
            let request = &request;
            async move {
                let r: Value = request
                    .post("/api/requisitions")
                    .json(&json!({ "organization_ref": org, "department": "engineering", "job_title": title, "headcount": 1 }))
                    .await
                    .json();
                let pid = r["pid"].as_str().unwrap().to_string();
                request.post(&format!("/api/requisitions/{pid}/status")).json(&json!({ "to": "open" })).await.assert_status_ok();
                pid
            }
        };
        let elsewhere = open(format!("Analyst {tag}"), org.clone()).await;
        let same_title = open(own_title.to_uppercase(), org.clone()).await;
        let other_org = open(format!("Analyst {tag}"), an_org()).await;
        // A draft requisition is not an opportunity.
        let draft: Value = request
            .post("/api/requisitions")
            .json(&json!({ "organization_ref": org, "department": "engineering", "job_title": format!("Draft {tag}"), "headcount": 1 }))
            .await
            .json();
        let draft = draft["pid"].as_str().unwrap().to_string();

        let view: Value = request.get(&format!("/api/workers/{worker}/opportunities")).await.json();
        let listed: Vec<&str> = view["opportunities"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["requisition_pid"].as_str().unwrap())
            .collect();
        assert!(listed.contains(&elsewhere.as_str()));
        assert!(!listed.contains(&same_title.as_str()), "their own job title is not a move");
        assert!(!listed.contains(&other_org.as_str()), "another organization's requisition is not offered");
        assert!(!listed.contains(&draft.as_str()), "only open or interviewing");
        let row = view["opportunities"].as_array().unwrap().iter().find(|o| o["requisition_pid"] == elsewhere.as_str()).unwrap();
        assert_eq!(row["fit"], Value::Null, "no role profile: unknown, not a poor fit");

        assert_eq!(request.get(&format!("/api/workers/{}/opportunities", uuid::Uuid::new_v4())).await.status_code(), 404);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_openapi_document_and_swagger_page_are_served_and_name_this_service() {
    crate::requests::request_open(|request, _ctx| async move {
        let spec: Value = request.get("/api-docs/openapi.json").await.json();
        assert!(spec["openapi"].as_str().unwrap().starts_with('3'));
        assert!(spec["paths"].as_object().unwrap().len() > 50);
        let page = request.get("/swagger-ui").await;
        page.assert_status_ok();
        let html = page.text();
        assert!(html.contains("/api-docs/openapi.json"));
        assert!(html.contains("Workforce Planning Management"));
        assert!(!html.contains("Patient Flow"));
    })
    .await;
}
