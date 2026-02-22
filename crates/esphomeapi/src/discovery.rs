use std::{collections::HashSet, net::Ipv4Addr, time::Duration};

use mdns_sd::{ServiceDaemon, ServiceEvent};
use tracing::{error, info};

use crate::Result;

const SERVICE_NAME: &str = "_esphomelib._tcp.local.";

#[derive(Clone, Debug)]
pub struct ServiceInfo {
  pub ty_domain: String, // <service>.<domain>

  /// See RFC6763 section 7.1 about "Subtypes":
  /// <https://datatracker.ietf.org/doc/html/rfc6763#section-7.1>
  pub sub_domain: Option<String>, // <subservice>._sub.<service>.<domain>

  pub fullname: String, // <instance>.<service>.<domain>
  pub server: String,   // fully qualified name for service host
  pub addresses: HashSet<Ipv4Addr>,
  pub port: u16,
}

pub async fn discover(seconds: u32) -> Result<Vec<ServiceInfo>> {
  let mdns = ServiceDaemon::new()?;
  let receiver = mdns.browse(SERVICE_NAME)?;

  let mut found_services = std::collections::HashMap::new();

  info!("starting discovery");

  let sleep = tokio::time::sleep(Duration::from_secs(seconds as u64));
  tokio::pin!(sleep);

  loop {
    tokio::select! {
      _ = &mut sleep => break,
      result = receiver.recv_async() => {
        match result {
          Ok(ServiceEvent::ServiceResolved(info)) => {
            found_services.insert(
              info.get_fullname().to_owned(),
              ServiceInfo {
                ty_domain: info.ty_domain.clone(),
                sub_domain: info.get_subtype().to_owned(),
                fullname: info.get_fullname().to_owned(),
                server: info.get_hostname().to_owned(),
                addresses: info.get_addresses_v4().clone(),
                port: info.get_port()
              },
            );
          }
          Ok(_) => {}
          Err(err) => {
            error!(error = ?err, "failed to receive service event");
          }
        }
      }
    }
  }

  // Drop receiver first so shutdown doesn't try to send events to it
  drop(receiver);
  if let Err(err) = mdns.shutdown() {
    error!(error = ?err, "mdns shutdown failed");
  }

  let services = found_services.values().cloned().collect();
  info!(services = ?services, "discovery finished");
  Ok(services)
}
