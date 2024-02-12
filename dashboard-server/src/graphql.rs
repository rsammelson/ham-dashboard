use diesel::prelude::*;

use crate::{activity, contact_data};

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

    axum::Server::bind(&"[::]:8008".parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/dashboard-api/")
            .subscription_endpoint("/dashboard-api/ws"),
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
        Ok(self.database.most_recent(None, None, None).await?)
    }

    async fn contact_count(
        &self,
        is_run: Option<bool>,
        operator: Option<String>,
    ) -> async_graphql::Result<u64> {
        Ok(self.database.count(is_run, operator).await?)
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

    async fn active_minutes(
        &self,
        start: Option<String>,
        end: Option<String>,
        duration: Option<u32>,
    ) -> async_graphql::Result<activity::Activity> {
        use time::format_description::well_known::iso8601;

        let mut act = activity::Activity::from_contacts(self.database.contacts().await?);

        let mut start = start
            .map(|t| {
                time::PrimitiveDateTime::parse(&t, &iso8601::Iso8601::DEFAULT)
                    .map_err(|e| anyhow::anyhow!("Invalid start time: {}", e))
            })
            .transpose()?;

        let mut end = end
            .map(|t| {
                time::PrimitiveDateTime::parse(&t, &iso8601::Iso8601::DEFAULT)
                    .map_err(|e| anyhow::anyhow!("Invalid end time: {}", e))
            })
            .transpose()?;

        match (start, end, duration) {
            (None, None, Some(_)) | (Some(_), Some(_), Some(_)) => {
                return Err(anyhow::anyhow!("Invalid parameter combination").into())
            }
            (Some(s), None, Some(d)) => end = Some(s + time::Duration::minutes(d.into())),
            (None, Some(e), Some(d)) => start = Some(e - time::Duration::minutes(d.into())),
            _ => {}
        }

        if let Some(start) = start {
            if let Some(a) = act.adjust_start(start) {
                act = a;
            } else {
                return Err(anyhow::anyhow!("Could not set start to {}", start).into());
            }
        }

        if let Some(end) = end {
            if let Some(a) = act.adjust_end(end) {
                act = a;
            } else {
                return Err(anyhow::anyhow!("Could not set end to {}", end).into());
            }
        }

        Ok(act)
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
