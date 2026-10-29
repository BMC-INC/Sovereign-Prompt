//! Runs only against a disposable, explicitly named test database.
use sovereign_prompt::{
    db::Database,
    types::{AuditLogEntry, LearningSignal, PromptRecord},
};

#[tokio::test]
#[ignore = "requires disposable SOVEREIGN_TEST_POSTGRES_URL with database sovereign_test"]
async fn postgres_crud_reports_and_tenant_insights() {
    let url = std::env::var("SOVEREIGN_TEST_POSTGRES_URL").expect("dedicated test database URL");
    assert!(
        url.ends_with("/sovereign_test"),
        "refusing to use a non-test database"
    );
    let db = Database::new(&url).await.unwrap();
    db.migrate().await.unwrap();
    db.migrate().await.unwrap();
    let user = format!("postgres-test-{}", uuid::Uuid::new_v4());
    let record = PromptRecord::new(
        user.clone(),
        "original".into(),
        100,
        "refined".into(),
        60,
        serde_json::json!([]),
    );
    let id = record.id.clone();
    db.insert_prompt(&record).await.unwrap();
    // Existing 0.1 PostgreSQL tables use REAL; the new reader must preserve them.
    let pool = sqlx_core::any::AnyPoolOptions::new()
        .connect(&url)
        .await
        .unwrap();
    sqlx_core::query::query("ALTER TABLE prompts ALTER COLUMN savings_percentage TYPE REAL")
        .execute(&pool)
        .await
        .unwrap();
    pool.close().await;

    assert_eq!(
        db.get_prompt_by_id(&id).await.unwrap().unwrap().user_id,
        user
    );
    assert_eq!(db.get_recent_prompts(&user, 5).await.unwrap().len(), 1);
    db.update_output(&id, "response", 5).await.unwrap();
    db.update_output_hash(&id, "fixture-hash").await.unwrap();
    db.update_approval_status(&id, "approved").await.unwrap();
    db.update_signature(&id, "fixture-signature", &chrono::Utc::now())
        .await
        .unwrap();
    db.insert_audit_log(&AuditLogEntry {
        id: uuid::Uuid::new_v4().to_string(),
        prompt_id: id.clone(),
        action: "test".into(),
        actor: user.clone(),
        detail: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    })
    .await
    .unwrap();
    db.insert_learning_signal(&LearningSignal {
        id: uuid::Uuid::new_v4().to_string(),
        prompt_id: id.clone(),
        signal: "positive".into(),
        comment: None,
        actor: user.clone(),
        created_at: chrono::Utc::now(),
    })
    .await
    .unwrap();
    assert_eq!(db.get_audit_trail(&id).await.unwrap().len(), 1);
    assert_eq!(db.get_user_stats(&user).await.unwrap().total_prompts, 1);
    assert_eq!(
        db.get_savings_report(&user, "all", None)
            .await
            .unwrap()
            .total_prompts,
        1
    );
    assert_eq!(
        db.get_team_report(std::slice::from_ref(&user), "all", None)
            .await
            .unwrap()
            .total_prompts,
        1
    );
    assert_eq!(
        db.get_learning_insights(Some(&user))
            .await
            .unwrap()
            .total_ratings,
        1
    );
    assert_eq!(
        db.get_learning_insights(Some("absent-user"))
            .await
            .unwrap()
            .total_ratings,
        0
    );
    assert!(db.get_learning_insights(None).await.unwrap().total_ratings >= 1);
    assert!(
        db.get_team_report(&[], "all", None)
            .await
            .unwrap()
            .total_prompts
            >= 1
    );
}
