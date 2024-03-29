#![allow(dead_code, internal_features)]
#![feature(rustc_attrs, iter_map_windows)]
#![allow(dead_code, unused)]

mod activity;
mod adif;
mod contact_data;
mod database;
mod graphql;
mod hamqth;
mod helpers;
mod prefix;
mod rst;
mod schema;
mod udp;
mod xml;

use diesel::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let hamqth_session = hamqth::Session::new(
        std::env::var("HAMQTH_USERNAME")?,
        std::env::var("HAMQTH_PASSWORD")?,
    )
    .await
    .map_err(|e| {
        println!("HamQTH error: {}", e);
        ()
    })
    .ok();

    if hamqth_session.is_none() {
        println!("Could not log in to HamQTH");
    }

    let adif_records = match &std::fs::read_to_string("W9YB.adi") {
        Ok(text) => adif::read_adif(text)?
            .map(|r| r.map(|r| contact_data::ContactData::from(r)))
            .collect::<Result<Vec<_>, _>>()?,
        Err(_) => Vec::new(),
    };
    println!("ADIF record count: {}", adif_records.len());

    let manager = diesel::r2d2::ConnectionManager::<SqliteConnection>::new("db.sql");
    let pool = diesel::r2d2::Pool::builder()
        .test_on_check_out(true)
        .max_size(1)
        .build(manager)?;

    let db = database::Database::new(pool).await?;

    if let Some(ref session) = hamqth_session {
        let mut adif_tasks = tokio::task::JoinSet::new();
        for d in adif_records {
            let db = db.clone();
            let session = session.clone();
            adif_tasks
                .spawn(async move { db.update_and_fetch_location(&session, &d, false).await });
        }
        while let Some(res) = adif_tasks.join_next().await {
            res??;
        }
    }

    let mut tasks = tokio::task::JoinSet::new();
    tasks.spawn(graphql::run_graphql_api(db.clone()));
    tasks.spawn(udp::udp_receiver(db.clone(), hamqth_session));

    tasks.spawn(async move {
        let mut recv = db.watch_latest().await;
        println!("here");
        loop {
            log::info!("Got value from subscription: {:?}", recv.recv().await);
        }
    });

    println!("Finished a task: {:?}", tasks.join_next().await);

    Ok(())
}
