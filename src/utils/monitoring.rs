use crate::{
    model::{
        entity::guest::{Guest, GuestStatus},
        repository::{Repository, guest_repository::GuestRepository},
    },
    unifi::unifi::{ClientInfo, UnifiController},
};
use rocket_db_pools::mongodb::Database;

// Struct
pub struct GuestMonitoring {
    sites: Vec<String>,
    repo: GuestRepository,
    unifi: UnifiController,
}

// Impls
#[allow(unused)]
impl GuestMonitoring {
    pub fn new(sites: Vec<String>, database: Database, unifi: UnifiController) -> Self {
        Self {
            sites,
            repo: GuestRepository {
                database,
                name: String::from("Guests"),
            },
            unifi,
        }
    }

    pub async fn all(&mut self) {
        let mut guests = self.repo.find_all().await;

        let iter = self.sites.clone();
        for site in iter {
            let res = self.unifi.get_guest_clients(site.clone()).await;
            if res.is_err() {
                break;
            }
            let clients = res.unwrap();

            self.check_and_update_guest_fields(&mut guests, &clients);
            self.check_and_update_clients_names(&guests, &clients).await;

            for i in 0..guests.len() {
                let r = self.repo.update(guests.remove(0)).await;
            }
        }
    }

    pub async fn check_and_update_clients_names(
        &mut self,
        guests: &Vec<Guest>,
        clients: &Vec<ClientInfo>,
    ) {
        for g in guests {
            let c = clients
                .iter()
                .find(|c| if g.mac == c.mac { true } else { false });

            if let Some(client) = c {
                if client.name.is_none() {
                    self.unifi
                        .rename_device_client(
                            client.record_id.clone().unwrap(),
                            g.site.clone(),
                            format!("{} (Visitante)", g.full_name),
                        )
                        .await;
                }
            } else {
                continue;
            }
        }
    }

    pub fn check_and_update_guest_fields(
        &self,
        guests: &mut Vec<Guest>,
        clients: &Vec<ClientInfo>,
    ) {
        for g in guests {
            if g.status != GuestStatus::Approved {
                continue;
            }

            let c = clients
                .iter()
                .find(|c| if g.mac == c.mac { true } else { false });

            if let Some(client) = c {
                if client.expired.unwrap_or(true) {
                    g.status = GuestStatus::Expired;
                }

                if client.hostname.is_some() {
                    g.hostname = client.hostname.clone();
                }

                if client.rx_bytes.is_some() {
                    g.rx_bytes = client.rx_bytes.clone();
                }

                if client.tx_bytes.is_some() {
                    g.tx_bytes = client.tx_bytes.clone();
                }
            }
        }
    }
}
