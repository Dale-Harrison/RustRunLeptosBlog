use actix_web::{test, web, App};
use backend::hello; // We need to export this or move it to lib.rs, but for now let's assume we can access it or redefine a similar test

#[actix_rt::test]
async fn test_hello_endpoint() {
    let mut app = test::init_service(App::new().service(hello)).await;
    let req = test::TestRequest::get().uri("/api/hello").to_request();
    let resp = test::call_service(&mut app, req).await;
    
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["message"], "In the Dusty Clockless Hours");
}
