# 错误处理

Resolver函数可以返回一个FieldResult类型，以下是FieldResult的定义：

```rust
type FieldResult<T> = std::result::Result<T, FieldError>;
```

任何错误都能够被转换为`FieldError`，并且你还能扩展标准的错误信息。

下面是一个例子，解析一个输入的字符串到整数，当解析失败时返回错误，并且附加额外的错误信息。

```rust
use async_graphql::*;

struct Query;

#[GQLObject]
impl Query {
    #[field]
    async fn parse_with_extensions(&self, input: String) -> FieldResult<i32> {
        Ok("234a"
            .parse()
            .map_err(|err| err.extend_with(|_| json!({"code": 400})))?)
    }
}
```