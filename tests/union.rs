use async_graphql::*;

#[async_std::test]
pub async fn test_union_simple_object() {
    #[derive(GQLSimpleObject)]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[derive(GQLUnion)]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[GQLObject]
    impl Query {
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    id
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[async_std::test]
pub async fn test_union_simple_object2() {
    #[derive(GQLSimpleObject)]
    struct MyObj {
        id: i32,
        title: String,
    }

    #[derive(GQLUnion)]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[GQLObject]
    impl Query {
        async fn node(&self) -> Node {
            MyObj {
                id: 33,
                title: "haha".to_string(),
            }
            .into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    id
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "node": {
                "id": 33,
            }
        })
    );
}

#[async_std::test]
pub async fn test_multiple_unions() {
    struct MyObj;

    #[GQLObject]
    impl MyObj {
        async fn value_a(&self) -> i32 {
            1
        }

        async fn value_b(&self) -> i32 {
            2
        }

        async fn value_c(&self) -> i32 {
            3
        }
    }

    #[derive(GQLUnion)]
    enum UnionA {
        MyObj(MyObj),
    }

    #[derive(GQLUnion)]
    enum UnionB {
        MyObj(MyObj),
    }

    struct Query;

    #[GQLObject]
    impl Query {
        async fn union_a(&self) -> UnionA {
            MyObj.into()
        }
        async fn union_b(&self) -> UnionB {
            MyObj.into()
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<UnionA>() // `UnionA` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
            unionA {
               ... on MyObj {
                valueA
                valueB
                valueC
              }
            }
            unionB {
                ... on MyObj {
                 valueA
                 valueB
                 valueC
               }
             }
        }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "unionA": {
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            },
            "unionB": {
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }
        })
    );
}

#[async_std::test]
pub async fn test_multiple_objects_in_multiple_unions() {
    struct MyObjOne;

    #[GQLObject]
    impl MyObjOne {
        async fn value_a(&self) -> i32 {
            1
        }

        async fn value_b(&self) -> i32 {
            2
        }

        async fn value_c(&self) -> i32 {
            3
        }
    }

    struct MyObjTwo;

    #[GQLObject]
    impl MyObjTwo {
        async fn value_a(&self) -> i32 {
            1
        }
    }

    #[derive(GQLUnion)]
    enum UnionA {
        MyObjOne(MyObjOne),
        MyObjTwo(MyObjTwo),
    }

    #[derive(GQLUnion)]
    enum UnionB {
        MyObjOne(MyObjOne),
    }

    struct Query;

    #[GQLObject]
    impl Query {
        async fn my_obj(&self) -> Vec<UnionA> {
            vec![MyObjOne.into(), MyObjTwo.into()]
        }
    }

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .register_type::<UnionB>() // `UnionB` is not directly referenced, so manual registration is required.
        .finish();
    let query = r#"{
            myObj {
                ... on MyObjTwo {
                    valueA
                }
                ... on MyObjOne {
                    valueA
                    valueB
                    valueC
                }
            }
         }"#;
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "myObj": [{
                "valueA": 1,
                "valueB": 2,
                "valueC": 3,
            }, {
                "valueA": 1
            }]
        })
    );
}

#[async_std::test]
pub async fn test_union_field_result() {
    struct MyObj;

    #[GQLObject]
    impl MyObj {
        async fn value(&self) -> FieldResult<i32> {
            Ok(10)
        }
    }

    #[derive(GQLUnion)]
    enum Node {
        MyObj(MyObj),
    }

    struct Query;

    #[GQLObject]
    impl Query {
        async fn node(&self) -> Node {
            MyObj.into()
        }
    }

    let query = r#"{
            node {
                ... on MyObj {
                    value
                }
            }
        }"#;
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);
    assert_eq!(
        schema.execute(query).await.into_result().unwrap().data,
        serde_json::json!({
            "node": {
                "value": 10,
            }
        })
    );
}
