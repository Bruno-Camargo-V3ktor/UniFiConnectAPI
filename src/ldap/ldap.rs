
use crate::configurations::config::LdapConfig;
use ldap3::{result::Result, Ldap, LdapConnAsync, Scope, SearchEntry};

// Structs
pub struct LdapConnection {
    pub username: String,
    pub password: String,
    pub domain: String,
    pub server: String,
    pub base_dn: String,
}

#[derive(Debug)]
pub struct LdapUser {
    pub username: String,
    pub name: String,
    pub email: String,
}

// Impls
impl LdapConnection {
    pub fn new(config: LdapConfig) -> Self {
        let ldap = Self {
            username: config.user_service.clone(),
            password: config.password.clone(),
            domain: config.domain.clone(),
            server: config.server.clone(),
            base_dn: config.base_dn.clone()
        };

        ldap
    }

    pub async fn simple_authentication(&self, username: &str, password: &str) -> bool {
        if username.is_empty() || password.is_empty() {
            return false;
        }
        
        let user_dn = format!("{}@{}", username, self.domain);

        match LdapConnAsync::new(&self.server).await {
            Ok((conn, mut ldap)) => {
                ldap3::drive!(conn);

                match ldap.simple_bind(&user_dn, &password).await {
                    Ok(res) => {
                        if res.success().is_ok() {
                            return true;
                        }
                        return false;
                    }

                    Err(_) => {return false;}
                }

            }  

            Err(_) => {
                return false
            }
        }

    }

    pub async fn create_connection(&self) -> Result<Ldap> {
        let (conn, mut ldap) = LdapConnAsync::new(&self.server).await?;
        ldap3::drive!(conn);
        ldap.simple_bind(&self.username, &self.password).await?.success()?;

        Ok(ldap)
    }

    pub async fn get_users_in_group(&self, conn: &mut Ldap, group_name: &str) -> Result<Vec<LdapUser>> {
        let mut users = vec![];
        let filter = format!("(cn={})", group_name);
        
        
        let (entries, _res) = conn.search(&self.base_dn, Scope::Subtree, &filter, vec!["member"]).await?.success()?;

        if entries.is_empty() {
            return Ok(users);
        }

        for entry in entries {
            let search_entry = SearchEntry::construct(entry);
            if let Some(members) = search_entry.attrs.get("member") {
                for member_dn in members {
                    if let Some(user) = self.get_user_details(conn, member_dn).await? {
                        users.push(user);
                    }
                }
            }
        }

        Ok(users)
    }

    pub async fn get_user_details(&self, ldap: &mut Ldap, user_dn: &str) -> Result<Option<LdapUser>> {
        let filter = format!("(distinguishedName={})", user_dn);
    
        let (entries, _res) = ldap.search(user_dn, Scope::Base, &filter, vec!["cn", "mail", "sAMAccountName"]).await?.success()?;
    
        if let Some(entry) = entries.get(0) {
            let search_entry = SearchEntry::construct(entry.clone());
            let username = search_entry.attrs.get("sAMAccountName").and_then(|v| v.get(0)).cloned().unwrap_or_else(|| "---".to_string());
            let name = search_entry.attrs.get("cn").and_then(|v| v.get(0)).cloned().unwrap_or_else(|| "Desconhecido".to_string());
            let email = search_entry.attrs.get("mail").and_then(|v| v.get(0)).cloned().unwrap_or_else(|| "".to_string());
    
            return Ok(Some(LdapUser { username, name, email }));
        }
    
        Ok(None)
    }

}