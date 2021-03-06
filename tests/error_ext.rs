use async_graphql::*;

#[async_std::test]
pub async fn test_error_extensions() {
    struct Query;

    #[Object]
    impl Query {
        async fn extend_err(&self) -> Result<i32> {
            Err(Error::new("my error").extend_with(|err| {
                serde_json::json!({
                    "msg": err.message,
                    "code": 100
                })
            }))
        }

        async fn extend_result(&self) -> Result<i32> {
            Err(Error::from("my error"))
                .extend_with(|_| {
                    serde_json::json!({
                        "msg": "my error",
                        "code": 100
                    })
                })
                .extend_with(|_| {
                    serde_json::json!({
                        "code2": 20
                    })
                })
        }
    }

    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    assert_eq!(
        serde_json::to_value(&schema.execute("{ extendErr }").await).unwrap(),
        serde_json::json!({
            "data": null,
            "errors": [{
                "message": "my error",
                "locations": [{
                    "column": 3,
                    "line": 1,
                }],
                "path": ["extendErr"],
                "extensions": {
                    "msg": "my error",
                    "code": 100
                }
            }]
        })
    );

    assert_eq!(
        serde_json::to_value(&schema.execute("{ extendResult }").await).unwrap(),
        serde_json::json!({
            "data": null,
            "errors": [{
                "message": "my error",
                "locations": [{
                    "column": 3,
                    "line": 1,
                }],
                "path": ["extendResult"],
                "extensions": {
                    "msg": "my error",
                    "code": 100,
                    "code2": 20
                }
            }]
        })
    );
}
