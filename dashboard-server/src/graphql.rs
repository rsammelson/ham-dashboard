use diesel::prelude::*;

use crate::contact_data;

pub async fn run_graphql_api(db: crate::database::Database) -> anyhow::Result<()> {
    let schema = async_graphql::Schema::build(
        Query {
            database: db.clone(),
        },
        async_graphql::EmptyMutation,
        Subscription { database: db },
    )
    .finish();

    let app = axum::Router::new()
        .route(
            "/",
            axum::routing::get(graphql_playground).post(graphql_handler),
        )
        .route_service(
            "/ws",
            async_graphql_axum::GraphQLSubscription::new(schema.clone()),
        )
        .layer(axum::Extension(schema))
        .layer(tower_http::cors::CorsLayer::permissive());

    println!("http://localhost:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
    ))
}

async fn graphql_handler(
    schema: axum::extract::Extension<
        async_graphql::Schema<Query, async_graphql::EmptyMutation, Subscription>,
    >,
    req: axum::extract::Json<async_graphql::Request>,
) -> axum::response::Json<async_graphql::Response> {
    schema.execute(req.0).await.into()
}

struct Query {
    database: crate::database::Database,
}

#[async_graphql::Object]
impl Query {
    async fn entries(&self) -> async_graphql::Result<Vec<contact_data::ContactData>> {
        use crate::schema::contacts::dsl::*;
        let mut c = self.database.get().await?;
        Ok(contacts.load(&mut c)?)
    }

    async fn most_recent(
        &self,
        count: Option<u32>,
        is_run: Option<bool>,
        operator: Option<String>,
    ) -> async_graphql::Result<Vec<contact_data::ContactData>> {
        Ok(self.database.most_recent(count, is_run, operator).await?)
    }

    async fn latest(
        &self,
        is_run: Option<bool>,
        operator: Option<String>,
    ) -> async_graphql::Result<Option<contact_data::ContactData>> {
        Ok(self
            .database
            .most_recent(Some(1), is_run, operator)
            .await?
            .pop())
    }
}

struct Subscription {
    database: crate::database::Database,
}

#[async_graphql::Subscription]
impl Subscription {
    async fn latest(
        &self,
    ) -> tokio_stream::wrappers::BroadcastStream<Option<contact_data::ContactData>> {
        log::info!("Subscriber attached");
        tokio_stream::wrappers::BroadcastStream::new(self.database.watch_latest().await)
    }
}
