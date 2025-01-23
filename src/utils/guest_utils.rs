use crate::{
    model::entity::guest::Guest,
    unifi::unifi::{ClientInfo, UnifiController},
};

// Functions
#[allow(unused)]
pub async fn check_and_update_clients_names(
    unifi: &mut UnifiController,
    guests: Vec<Guest>,
    clients: Vec<ClientInfo>,
) {
    for g in guests {
        let c = clients
            .iter()
            .find(|c| if g.mac == c.mac { true } else { false });

        if let Some(client) = c {
            if client.name.is_none() {
                unifi.rename_device_client(
                    client.record_id.clone().unwrap(),
                    g.site,
                    format!("{} (Visitante)", g.full_name),
                );
            }
        } else {
            continue;
        }
    }
}
