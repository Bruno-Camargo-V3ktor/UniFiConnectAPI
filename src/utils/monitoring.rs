use crate::{
    model::{
        entity::client::{Client, ClientStatus},
        repository::{Repository, mongo_repository::MongoRepository},
    },
    unifi::unifi::{DeviceInfo, UnifiController},
};
use rocket_db_pools::mongodb::Database;

// Struct
pub struct ClientsMonitoring {
    repo: MongoRepository<Client>,
    unifi: UnifiController,
}

// Impls
#[allow(unused)]
impl ClientsMonitoring {
    pub fn new(database: Database, unifi: UnifiController) -> Self {
        Self {
            repo: MongoRepository::new(database),
            unifi,
        }
    }

    pub async fn all(&mut self) {
        let mut sites: Vec<String> = vec![];
        let mut clients = self.repo.find_all().await;

        for c in clients.iter() {
            if !sites.contains(&c.site) {
                sites.push(c.site.clone());
            }
        }

        for site in sites.iter() {
            let res = self.unifi.get_guest_devices(site.clone()).await;
            if res.is_err() {
                break;
            }
            let devices = res.unwrap();

            self.check_and_update_client_fields(&mut clients, &devices);
            self.check_and_update_clients_names(&clients, &devices)
                .await;

            for i in 0..clients.len() {
                let r = self.repo.update(clients.remove(0)).await;
            }
        }
    }

    pub async fn check_and_update_clients_names(
        &mut self,
        clients: &Vec<Client>,
        devices: &Vec<DeviceInfo>,
    ) {
        for c in clients {
            let d = devices
                .iter()
                .find(|d| if c.mac == d.mac { true } else { false });

            if let Some(device) = d {
                if device.name.is_none() {
                    self.unifi
                        .rename_device(
                            device.record_id.clone().unwrap(),
                            c.site.clone(),
                            format!("{} (Visitante)", c.full_name),
                        )
                        .await;
                }
            } else {
                continue;
            }
        }
    }

    pub fn check_and_update_client_fields(
        &self,
        clients: &mut Vec<Client>,
        devices: &Vec<DeviceInfo>,
    ) {
        for c in clients {
            if c.status != ClientStatus::Approved {
                continue;
            }

            let d = devices
                .iter()
                .find(|d| if c.mac == d.mac { true } else { false });

            if let Some(device) = d {
                if device.expired.unwrap_or(true) {
                    c.status = ClientStatus::Expired;
                }

                if device.hostname.is_some() {
                    c.hostname = device.hostname.clone();
                }

                if device.rx_bytes.is_some() {
                    c.rx_bytes = device.rx_bytes.clone();
                }

                if device.tx_bytes.is_some() {
                    c.tx_bytes = device.tx_bytes.clone();
                }
            }
        }
    }
}
