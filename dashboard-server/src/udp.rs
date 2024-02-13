use std::sync::{Arc, Mutex};

use crate::{contact_data, database, hamqth, xml};

pub async fn udp_receiver(
    db: database::Database,
    hamqth_session: Option<hamqth::Session>,
) -> anyhow::Result<()> {
    let db = Arc::new(db);
    let hamqth_session = hamqth_session.map(|s| Arc::new(s));
    let socket = Arc::new(Mutex::new(std::net::UdpSocket::bind("0.0.0.0:12063")?));
    loop {
        match receive_udp(db.clone(), hamqth_session.clone(), socket.clone()).await {
            Ok(_) => log::trace!("Successfully received UDP packet"),
            Err(e) => log::warn!("Error receiving UDP packet: {}", e),
        }
    }
}

async fn receive_udp(
    db: Arc<database::Database>,
    hamqth_session: Option<Arc<hamqth::Session>>,
    socket: Arc<Mutex<std::net::UdpSocket>>,
) -> anyhow::Result<()> {
    const N: usize = 8192;

    let (data, len) = {
        tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            let mut data = Vec::new();
            data.resize(N, 0);
            let len = socket.try_lock().unwrap().recv(&mut data)?;
            anyhow::ensure!(len < N);
            data.resize(len, 0);
            Ok((data, len))
        })
        .await??
    };

    tokio::spawn(async move {
        match process_udp(&db, hamqth_session.as_deref(), data).await {
            Ok(_) => log::trace!("Successfully processed UDP packet"),
            Err(e) => log::warn!("Error processing UDP packet: {}", e),
        }
    });

    Ok(())
}

async fn process_udp(
    db: &database::Database,
    hamqth_session: Option<&hamqth::Session>,
    data: Vec<u8>,
) -> anyhow::Result<()> {
    let xml = std::str::from_utf8(&data)?;
    log::debug!("Got UDP packet: {}", xml);

    let data: xml::UdpData = quick_xml::de::from_str(xml)?;
    match data {
        xml::UdpData::ContactInfo(info) | xml::UdpData::ContactReplace(info) => {
            log::info!("Got new contact {:?}", info);
            let cd = contact_data::ContactData::from(info);
            if let Some(session) = hamqth_session {
                db.update_and_fetch_location(session, &cd, true).await?;
            }
        }
        xml::UdpData::ContactDelete(data) => {
            log::info!("Got contact delete {:?}", data);
            db.delete_by_id(data.id).await?;
        }
        other => log::info!("Other UDP data: {:?}", other),
    }

    Ok(())
}
