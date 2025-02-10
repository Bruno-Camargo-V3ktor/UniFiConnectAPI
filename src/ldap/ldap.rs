
use crate::configurations::config::LdapConfig;
use ldap3::{LdapConn, Scope, SearchEntry, result::Result};

// Structs
pub struct LdapConnection {
    pub username: String,
    pub password: String,
    pub server: String,
    pub base_dn: String,
}

pub struct LdapUser {
    pub name: String,
    pub email: String,
}

// Impls
impl LdapConnection {
    pub fn new(config: LdapConfig) -> Self {
        let ldap = Self {
            username: config.user_service.clone(),
            password: config.password.clone(),
            server: config.server.clone(),
            base_dn: config.base_dn.clone()
        };

        ldap
    }


    pub fn create_connection(&self) -> Result<LdapConn> {
        let mut conn = LdapConn::new(&self.server)?;
        conn.simple_bind(&self.username, &self.password)?.success()?;

        Ok(conn)
    }

    pub fn get_users_in_group(&self, conn: &mut LdapConn, group_name: &str) -> Result<Vec<LdapUser>> {
        let mut users = vec![];
        let filter = format!("(cn={})", group_name);
        
        
        let (entries, _res) = conn.search(&self.base_dn, Scope::Subtree, &filter, vec!["member"])?.success()?;

        if entries.is_empty() {
            return Ok(users);
        }

        let mut users = Vec::new();

        for entry in entries {
            let search_entry = SearchEntry::construct(entry);
            if let Some(members) = search_entry.attrs.get("member") {
                for member_dn in members {
                    if let Some(user) = self.get_user_details(conn, member_dn)? {
                        users.push(user);
                    }
                }
            }
        }

        Ok(users)
    }

    pub fn get_user_details(&self, ldap: &mut LdapConn, user_dn: &str) -> Result<Option<LdapUser>> {
        let filter = format!("(distinguishedName={})", user_dn);
    
        let (entries, _res) = ldap.search(user_dn, Scope::Base, &filter, vec!["cn", "mail"])?.success()?;
    
        if let Some(entry) = entries.get(0) {
            let search_entry = SearchEntry::construct(entry.clone());
            let name = search_entry.attrs.get("cn").and_then(|v| v.get(0)).cloned().unwrap_or_else(|| "Desconhecido".to_string());
            let email = search_entry.attrs.get("mail").and_then(|v| v.get(0)).cloned().unwrap_or_else(|| "".to_string());
    
            return Ok(Some(LdapUser { name, email }));
        }
    
        Ok(None)
    }

}