use crate::parser::types::{ExecutableDocumentData, Field, Selection, SelectionSet};

/// A selection performed by a query.
pub struct Lookahead<'a> {
    document: &'a ExecutableDocumentData,
    field: Option<&'a Field>,
}

impl<'a> Lookahead<'a> {
    pub(crate) fn new(document: &'a ExecutableDocumentData, field: &'a Field) -> Self {
        Self {
            document,
            field: Some(field),
        }
    }

    /// Get the first subfield of the selection set with the specified name. This will ignore
    /// aliases.
    ///
    /// For example, calling `.field("a")` on `{ a { b } }` will return a lookahead that
    /// represents `{ b }`.
    pub fn field(&self, name: &str) -> Self {
        Self {
            document: self.document,
            field: self
                .field
                .and_then(|field| find(self.document, &field.selection_set.node, name)),
        }
    }

    /// Returns true if field exists otherwise return false.
    #[inline]
    pub fn exists(&self) -> bool {
        self.field.is_some()
    }
}

fn find<'a>(
    document: &'a ExecutableDocumentData,
    selection_set: &'a SelectionSet,
    name: &str,
) -> Option<&'a Field> {
    selection_set
        .items
        .iter()
        .find_map(|item| match &item.node {
            Selection::Field(field) => {
                if field.node.name.node == name {
                    Some(&field.node)
                } else {
                    None
                }
            }
            Selection::InlineFragment(fragment) => {
                find(document, &fragment.node.selection_set.node, name)
            }
            Selection::FragmentSpread(spread) => document
                .fragments
                .get(&spread.node.fragment_name.node)
                .and_then(|fragment| find(document, &fragment.node.selection_set.node, name)),
        })
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[async_std::test]
    async fn test_look_ahead() {
        #[derive(GQLSimpleObject)]
        #[graphql(internal)]
        struct Detail {
            c: i32,
            d: i32,
        }

        #[derive(GQLSimpleObject)]
        #[graphql(internal)]
        struct MyObj {
            a: i32,
            b: i32,
            detail: Detail,
        }

        struct Query;

        #[GQLObject(internal)]
        impl Query {
            async fn obj(&self, ctx: &Context<'_>, n: i32) -> MyObj {
                if ctx.look_ahead().field("a").exists() {
                    // This is a query like `obj { a }`
                    assert_eq!(n, 1);
                } else if ctx.look_ahead().field("detail").field("c").exists() {
                    // This is a query like `obj { detail { c } }`
                    assert_eq!(n, 2);
                } else {
                    // This query doesn't have `a`
                    assert_eq!(n, 3);
                }
                MyObj {
                    a: 0,
                    b: 0,
                    detail: Detail { c: 0, d: 0 },
                }
            }
        }

        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

        assert!(!schema
            .execute(
                r#"{
            obj(n: 1) {
                a
            }
        }"#,
            )
            .await
            .is_err());

        assert!(!schema
            .execute(
                r#"{
            obj(n: 1) {
                k:a
            }
        }"#,
            )
            .await
            .is_err());

        assert!(!schema
            .execute(
                r#"{
            obj(n: 2) {
                detail {
                    c
                }
            }
        }"#,
            )
            .await
            .is_err());

        assert!(!schema
            .execute(
                r#"{
            obj(n: 3) {
                b
            }
        }"#,
            )
            .await
            .is_err());

        assert!(!schema
            .execute(
                r#"{
            obj(n: 1) {
                ... {
                    a
                }
            }
        }"#,
            )
            .await
            .is_err());

        assert!(!schema
            .execute(
                r#"{
            obj(n: 2) {
                ... {
                    detail {
                        c
                    }
                }
            }
        }"#,
            )
            .await
            .is_err());

        assert!(!schema
            .execute(
                r#"{
            obj(n: 1) {
                ... A
            }
        }
        
        fragment A on MyObj {
            a
        }"#,
            )
            .await
            .is_err());

        assert!(!schema
            .execute(
                r#"{
            obj(n: 2) {
                ... A
            }
        }
        
        fragment A on MyObj {
            detail {
                c
            }
        }"#,
            )
            .await
            .is_err());
    }
}
