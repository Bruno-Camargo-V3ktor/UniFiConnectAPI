use crate::{
    configurations::config::{ApproversConfig, LdapConfig, UsersConfig}, ldap::ldap::LdapConnection, model::{
        entity::{admin::Admin, approver::Approver, client::{Client, ClientStatus}, user::User},
        repository::{mongo_repository::MongoRepository, Repository},
    }, unifi::unifi::{DeviceInfo, UnifiController}
};
use bcrypt::{hash, DEFAULT_COST};
use ldap3::Ldap;
use rocket_db_pools::mongodb::Database;

use super::generator;

// Struct
pub struct ClientsMonitoring {
    repo: MongoRepository<Client>,
    unifi: UnifiController,
}

pub struct LdapMonitoring {
    config: LdapConfig,
    users_repo: MongoRepository<User>,
    approvers_repo: MongoRepository<Approver>,
    admins_repo: MongoRepository<Admin>
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

            for i in 0..clients.len() {
                let r = self.repo.update(clients.remove(0)).await;
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

#[allow(unused)]
impl LdapMonitoring {
    pub fn new(database: Database, config: LdapConfig) -> Self {
        Self {
            config,
            users_repo: MongoRepository::new(database.clone()),
            approvers_repo: MongoRepository::new(database.clone()),
            admins_repo: MongoRepository::new(database.clone()),
        }
    }

    pub async fn scan_approvers(&self, conn: &mut Ldap, ldap: &LdapConnection, config: &ApproversConfig) {
        let mut approvers: Vec<_> = self.approvers_repo.find_all().await.into_iter().filter(|a| a.password.is_empty()).collect();

        for group in &self.config.approvers_search {
            if let Ok(entitys) = ldap.get_users_in_group(conn, group).await {
                for e in &entitys {
                    let op = approvers.iter().position( |a| a.username == e.username );
                    if let Some(index) = op { 
                        let _ = approvers.remove(index);
                        continue; 
                    }

                    let mut approver = Approver::new_wiht_ldap_user(e);
                    let new_code = generator::generator_code(config.code_size, config.just_numbers);
                    approver.secrete_code = hash(new_code.clone(), DEFAULT_COST).unwrap();
                    approver.create_validity(config.validity_days_code.clone() as i64);
                    approver.approved_types.push(config.default_group.clone());

                    let _ = self.approvers_repo.save(approver).await;
                }
            }  
        }
        
        for a in approvers {
            self.approvers_repo.delete(a).await;
        }

    }

    pub async fn scan_users(&self, conn: &mut Ldap, ldap: &LdapConnection, config: &UsersConfig) {
        let mut users: Vec<_> = self.users_repo.find_all().await.into_iter().filter(|u| u.password.is_empty()).collect();

        for group in &self.config.users_search {
            if let Ok(entitys) = ldap.get_users_in_group(conn, group).await {
                for e in &entitys {
                    let op = users.iter().position( |a| a.username == e.username );
                    if let Some(index) = op { 
                        let _ = users.remove(index);
                        continue; 
                    }

                    let mut user = User::new_with_ldap_user(e);
                    user.data.client_type = config.default_group.clone();

                    let _ = self.users_repo.save(user).await;
                }
            }  
        }

        for u in users {
            self.users_repo.delete(u).await;
        }

    }

    pub async fn scan_admins(&self, conn: &mut Ldap, ldap: &LdapConnection) {
        let mut admins: Vec<_> = self.admins_repo.find_all().await.into_iter().filter(|u| u.password.is_none()).collect();

        for group in &self.config.admins_search {
            if let Ok(entitys) = ldap.get_users_in_group(conn, group).await {
                for e in &entitys {
                    let op = admins.iter().position( |a| a.username == e.username );
                    if let Some(index) = op { 
                        let _ = admins.remove(index);
                        continue; 
                    }

                    let mut admin = Admin::new_with_ldap_user(e);
                    let _ = self.admins_repo.save(admin).await;
                }
            }  
        }

        for a in admins {
            self.admins_repo.delete(a).await;
        }

    }
}