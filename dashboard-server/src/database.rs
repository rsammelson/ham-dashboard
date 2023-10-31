use std::sync::Arc;
use tokio::sync::broadcast;

use diesel::{prelude::*, r2d2};

use crate::{
    contact_data::{self, ContactData},
    hamqth, prefix,
};

#[derive(Clone)]
pub struct Database {
    pool: r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    last: Arc<tokio::sync::RwLock<LastData>>,
}

struct LastData {
    sender: broadcast::Sender<Option<ContactData>>,
    value: Option<ContactData>,
}

impl Database {
    pub async fn new(
        pool: r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            pool,
            last: Arc::new(tokio::sync::RwLock::new(LastData {
                sender: broadcast::Sender::new(8),
                value: None,
            })),
        })
    }

    pub async fn get(
        &self,
    ) -> anyhow::Result<r2d2::PooledConnection<r2d2::ConnectionManager<SqliteConnection>>> {
        Ok(self.pool.get()?)
    }

    pub async fn insert_rows(&self, data: &[ContactData]) -> anyhow::Result<()> {
        use crate::schema::contacts::dsl::*;
        let inserted_count = diesel::insert_into(contacts)
            .values(data)
            .execute(&mut self.pool.get()?)?;
        log::info!("Inserted {} rows", inserted_count);
        self.maybe_publish(None, true, false).await
    }

    pub async fn delete_by_id(&self, delete_id: &str) -> anyhow::Result<()> {
        use crate::schema::contacts::dsl::*;

        let delete = diesel::delete(contacts.filter(id.eq(delete_id)));
        let count = delete.execute(&mut self.pool.get()?)?;
        log::info!("Deleted {} rows", count);

        if count > 0 {
            self.maybe_publish(Some(delete_id), true, true).await?;
        }

        Ok(())
    }

    pub async fn update(
        &self,
        data: &ContactData,
        publish: bool,
    ) -> anyhow::Result<(Option<contact_data::LocationSource>)> {
        use crate::schema::contacts::dsl::*;
        let mut conn = self.pool.get()?;

        let old_recv_call: Option<(String, contact_data::LocationSource)> = contacts
            .select((recv_callsign, location_source))
            .filter(id.eq(data.id()))
            .load(&mut conn)?
            .pop();

        let location_source_info = match old_recv_call {
            Some((call, source)) => {
                if call == data.recv_callsign {
                    Some(source)
                } else {
                    None
                }
            }
            None => None,
        };

        let count = if location_source_info.is_some() {
            diesel::update(contacts.filter(id.eq(data.id())))
                .set((
                    recv_callsign.eq(&data.recv_callsign),
                    sent_callsign.eq(&data.sent_callsign),
                    recv_signal_report.eq(&data.recv_signal_report),
                    sent_signal_report.eq(&data.sent_signal_report),
                    timestamp.eq(data.timestamp),
                    mode.eq(&data.mode),
                    freq_rx.eq(data.freq_rx),
                    freq_tx.eq(data.freq_tx),
                    exchange1.eq(&data.exchange1),
                    section.eq(&data.section),
                    prefix_wpx.eq(&data.prefix_wpx),
                    operator.eq(&data.operator),
                    contest_name.eq(&data.contest_name),
                    is_mult_1.eq(data.is_mult_1),
                    is_mult_2.eq(data.is_mult_2),
                    is_mult_3.eq(data.is_mult_3),
                    is_run_qso.eq(data.is_run_qso),
                    is_claimed_qso.eq(data.is_claimed_qso),
                    points.eq(data.points),
                ))
                .execute(&mut conn)?
        } else {
            diesel::replace_into(contacts)
                .values(data)
                .execute(&mut conn)?
        };
        drop(conn);

        if publish {
            log::info!("Updated {} rows", count);
        }

        if publish && count > 0 {
            self.maybe_publish(data.id(), true, true).await?;
        }

        Ok(location_source_info)
    }

    pub async fn add_location(
        &self,
        update_id: &str,
        source: contact_data::LocationSource,
        lat: f32,
        lng: f32,
    ) -> anyhow::Result<()> {
        use crate::schema::contacts::dsl::*;
        let update = diesel::update(contacts.filter(id.eq(update_id))).set((
            location_source.eq(source),
            latitude.eq(lat),
            longitude.eq(lng),
        ));
        update.execute(&mut self.pool.get()?)?;
        log::info!("Added location to {}", update_id);
        self.maybe_publish(Some(update_id), false, true).await
    }

    pub async fn update_and_fetch_location(
        &self,
        hamqth_session: &hamqth::Session,
        data: &ContactData,
        publish: bool,
    ) -> anyhow::Result<()> {
        match self.update(data, false).await? {
            Some(contact_data::LocationSource::NoLocation) => {
                self.get_location_from_prefix(data).await
            }
            Some(contact_data::LocationSource::Prefix) => Ok(()),
            Some(contact_data::LocationSource::HamQTH) => Ok(()),
            None => self.get_location(data, hamqth_session).await,
        }
    }

    async fn get_location(
        &self,
        data: &ContactData,
        hamqth_session: &hamqth::Session,
    ) -> anyhow::Result<()> {
        if !self
            .get_location_from_hamqth(data, hamqth_session)
            .await
            .is_ok_and(|v| v)
        {
            self.get_location_from_prefix(data).await
        } else {
            Ok(())
        }
    }

    async fn get_location_from_hamqth(
        &self,
        data: &ContactData,
        hamqth_session: &hamqth::Session,
    ) -> anyhow::Result<bool> {
        log::debug!("Fetching location for ADIF entry: {}", data.recv_callsign);
        let location = hamqth_session.query(&data.recv_callsign).await?;
        match location {
            Some(l) => {
                log::info!("Location {:?} for {}", l, data.recv_callsign);
                self.add_location(
                    data.id().unwrap(),
                    contact_data::LocationSource::HamQTH,
                    l.latitude,
                    l.longitude,
                )
                .await?;
                Ok(true)
            }
            None => {
                log::info!("No location for {}", data.recv_callsign);
                Ok(false)
            }
        }
    }

    async fn get_location_from_prefix(&self, data: &ContactData) -> anyhow::Result<()> {
        let location = prefix::get_location_for_callsign(&data.recv_callsign);
        if let Some(l) = location {
            self.add_location(
                data.id().unwrap(),
                contact_data::LocationSource::Prefix,
                l.0,
                l.1,
            )
            .await
        } else {
            Ok(())
        }
    }

    pub async fn most_recent(
        &self,
        count: Option<u32>,
        is_run: Option<bool>,
        op: Option<String>,
    ) -> anyhow::Result<Vec<ContactData>> {
        use crate::schema::contacts::dsl::*;
        let mut expr = contacts.into_boxed();

        if let Some(is_run) = is_run {
            expr = expr.filter(is_run_qso.eq(is_run));
        }

        if let Some(op) = op {
            expr = expr.filter(operator.eq(op));
        }

        expr = expr.order(timestamp.desc());

        if let Some(c) = count {
            expr = expr.limit(c.into());
        }

        Ok(expr.load(&mut self.pool.get()?)?)
    }

    pub async fn watch_latest(&self) -> tokio::sync::broadcast::Receiver<Option<ContactData>> {
        self.last.read().await.sender.subscribe()
    }

    async fn maybe_publish(
        &self,
        updated_id: Option<&str>,
        could_change_first: bool,
        could_update_value: bool,
    ) -> anyhow::Result<()> {
        let last = self.last.write().await;

        if !could_change_first {
            match (updated_id, last.value.as_ref()) {
                (Some(s), Some(l)) => {
                    if l.id().is_some_and(|l| s != l) {
                        // just an update to something not at the front
                        return Ok(());
                    }
                }
                _ => {}
            }
        }

        let most_recent = self.most_recent(Some(1), None, None).await?.pop();

        match (last.value.as_ref(), &most_recent) {
            (None, None) => {
                return Ok(());
            }
            (Some(last_contact), Some(most_recent)) => {
                if !could_update_value && last_contact.id() == most_recent.id() {
                    return Ok(());
                }
            }
            _ => {}
        }

        log::info!(
            "Sending update to {} subscribers: {:?}",
            last.sender.receiver_count(),
            most_recent.as_ref().map(|d| (&d.recv_callsign, d.id()))
        );
        let _ = last.sender.send(most_recent);

        Ok(())
    }
}
