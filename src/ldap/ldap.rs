
use crate::configurations::config::LdapConfig;
use ldap3::{result::Result, Ldap, LdapConnAsync, Scope, SearchEntry};
use serde::{Deserialize, Serialize};

// Structs
pub struct LdapConnection {
    pub username: String,
    pub password: String,
    pub domain: String,
    pub server: String,
    pub base_dn: String,
    pub attributes: LdapAttributes
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LdapAttributes {
    pub name: String,
    pub username: String,
    pub email: String,
    pub member: String, 
    pub group: String,
    pub login: String,
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
        Self {
            username: config.user_service.clone(),
            password: config.password.clone(),
            domain: config.domain.clone(),
            server: config.server.clone(),
            base_dn: config.base_dn.clone(),
            attributes: config.attrs.clone()
        }
    }

    pub async fn simple_authentication(&self, login: &str, password: &str) -> bool {
        if login.is_empty() || password.is_empty() {
            return false;
        }
    
        // 1. Cria uma conexão LDAP usando um usuário técnico para fazer a busca.
        let (conn, mut ldap) = match LdapConnAsync::new(&self.server).await {
            Ok(pair) => pair,
            Err(_) => return false,
        };
        ldap3::drive!(conn);
    
        // 2. Realiza o bind inicial (usuário técnico) para ter permissão de pesquisar.
        if let Err(_) = ldap
            .simple_bind(&self.username, &self.password)
            .await
            .and_then(|res| res.success())
        {
            return false;
        }
    
        // 3. Monta o filtro de busca usando um atributo de login configurável (ex: "uid" ou "sAMAccountName").
        let filter = format!("({}={})", self.attributes.login, login);
    
        // 4. Pesquisa na base configurada usando o filtro.
        let (results, _) = match ldap
            .search(&self.base_dn, Scope::Subtree, &filter, vec![""])
            .await
            .and_then(|s| s.success())
        {
            Ok(pair) => pair,
            Err(_) => return false,
        };
    
        // 5. Se achar a entrada, extrai o DN do usuário usando SearchEntry::construct()
        let dn = match results.first() {
            Some(entry) => {
                let search_entry = SearchEntry::construct(entry.clone());
                search_entry.dn.clone() // Aqui extraímos o campo `dn` do objeto convertido
            },
            None => return false,
        };
    
        // Libera a conexão técnica antes de proceder com o bind do usuário.
        drop(ldap);
    
        // 6. Cria uma nova conexão para autenticar com o DN encontrado.
        let (conn2, mut user_ldap) = match LdapConnAsync::new(&self.server).await {
            Ok(pair) => pair,
            Err(_) => return false,
        };
        ldap3::drive!(conn2);
    
        // 7. Tenta realizar o simple_bind usando o DN do usuário e a senha fornecida.
        match user_ldap
            .simple_bind(&dn, password)
            .await
            .and_then(|r| r.success())
        {
            Ok(_) => true,
            Err(_) => false,
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
        let filter = format!("({}={})", self.attributes.group.clone(), group_name);
        
        
        let (entries, _res) = conn.search(&self.base_dn, Scope::Subtree, &filter, vec![self.attributes.member.clone()]).await?.success()?;

        if entries.is_empty() {
            return Ok(users);
        }

        for entry in entries {
            let search_entry = SearchEntry::construct(entry);
            if let Some(members) = search_entry.attrs.get( &self.attributes.member ) {
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
        let (entries, _res) = ldap.search(
            user_dn, 
            Scope::Base, 
            "objectClass=*", 
            vec![self.attributes.name.clone(), self.attributes.email.clone(), self.attributes.username.clone()])
            .await?.success()?;
    
        if let Some(entry) = entries.first() {
            let search_entry = SearchEntry::construct(entry.clone());
            let username = search_entry.attrs.get( &self.attributes.username ).and_then(|v| v.first()).cloned().unwrap_or_else(|| "---".to_string());
            let name = search_entry.attrs.get( &self.attributes.name ).and_then(|v| v.first()).cloned().unwrap_or_else(|| "Desconhecido".to_string());
            let email = search_entry.attrs.get( &self.attributes.email ).and_then(|v| v.first()).cloned().unwrap_or_else(|| "".to_string());
    
            return Ok(Some(LdapUser { username, name, email }));
        }
    
        Ok(None)
    }

}
