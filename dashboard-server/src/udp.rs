use std::sync::{Arc, Mutex};

use crate::{contact_data, database, hamqth, xml};

pub async fn udp_receiver(
    db: database::Database,
    hamqth_session: Option<hamqth::Session>,
) -> anyhow::Result<()> {
    let socket = Arc::new(Mutex::new(std::net::UdpSocket::bind("0.0.0.0:12063")?));
    let buffer = Arc::new(tokio::sync::Mutex::new([0u8; 4096]));
    loop {
        match receive_udp(&db, hamqth_session.as_ref(), &socket, &buffer).await {
            Ok(_) => log::trace!("Successfully received UDP packet"),
            Err(e) => log::warn!("Error receiving UDP packet: {}", e),
        }
    }
}

async fn receive_udp<const N: usize>(
    db: &database::Database,
    hamqth_session: Option<&hamqth::Session>,
    socket: &Arc<Mutex<std::net::UdpSocket>>,
    buffer: &Arc<tokio::sync::Mutex<[u8; N]>>,
) -> anyhow::Result<()> {
    let len = {
        let socket = socket.clone();
        let buffer_ref = buffer.clone();
        tokio::task::spawn_blocking(move || {
            socket
                .lock()
                .unwrap()
                .recv(&mut *buffer_ref.blocking_lock())
        })
        .await??
    };
    anyhow::ensure!(len < N);

    let buffer = buffer.lock().await;

    let xml = std::str::from_utf8(&buffer[0..len])?;
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
