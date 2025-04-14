use std::collections::HashMap;
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::json;

use crate::model::entity::client::Client;


// STRUCTS
pub struct GLPI {
    base_url: String,
    app_token: String,
    authorization: String,
    ids: HashMap<String, usize>,
    client: reqwest::Client
}

#[derive(Deserialize)]
struct TicketCreateResponse {
    pub id: usize,
    pub message: String,
}

// IMPLS
impl GLPI {
    pub fn new(base_url: String, app_token: String, authorization: String) -> Self {
        Self {
            base_url,
            app_token,
            authorization,
            ids: HashMap::new(),
            client: reqwest::Client::builder().danger_accept_invalid_certs(true).cookie_store(true).build().unwrap()
        }
    }

    pub async fn init_session(&self) -> String {
        let mut headers = HeaderMap::new();
        headers.insert("App-Token", self.app_token.clone().parse().unwrap());
        headers.insert("Authorization", self.authorization.clone().parse().unwrap());

        let res = self.client.get( format!("{}/apirest.php/initSession", self.base_url) )
            .headers( headers )
            .send()
            .await;
        
        match res {
            Ok( content ) => {
                let body: HashMap<String, String> = content.json().await.unwrap();
                
                if let Some(value) = body.get( "session_token" ) { value.clone() } 
                else { String::new() }
            }

            Err(_) => {
                String::new()
            }
        }
    }
    
    pub async fn create_ticket(&mut self, client: Client, title: String, content_ticket: String, status: usize, priority: usize, user_id: usize, category_id: usize ) {
        let mut headers = HeaderMap::new();
        headers.insert( "Content-Type", "application/json".parse().unwrap() );
        headers.insert( "App-Token", self.app_token.clone().parse().unwrap() );
        headers.insert( "Session-Token", self.init_session().await.parse().unwrap() );
       
        let mut content_formated = content_ticket
            .replace("{name}", &client.full_name)
            .replace("{email}", &client.email)
            .replace("{phone}", &client.phone);
        
        for (key, value) in &client.fields {
            content_formated = content_formated.replace( format!("{{{key}}}").as_str() , value);
        }

        let body = json!(
            {
                "input": {
                    "name": title,
                    "content": content_formated,
                    "status": status,
                    "urgency": 2,
                    "impact": 3,
                    "priority": priority,
                    "_users_id_requester": user_id,
                    "itilcategories_id": category_id
                }
            }
        );

        let res = self.client.post( format!("{}/apirest.php/Ticket", self.base_url) )
            .headers( headers )
            .json( &body )
            .send()
            .await;
        
        if let Ok(response) = res {
            if response.status().is_success() {
                let body: TicketCreateResponse = response.json().await.unwrap();
                self.ids.insert(client.id, body.id);
            }
        }
    }
    
    pub async fn finish_ticket(&mut self, client_id: String, message: String, status: usize, template_id: usize) {
        let mut headers = HeaderMap::new();
        headers.insert( "Content-Type", "application/json".parse().unwrap() );
        headers.insert( "App-Token", self.app_token.clone().parse().unwrap() );
        headers.insert( "Authorization", self.authorization.parse().unwrap() );
        
        let session_token = self.init_session().await;
        let id = self.ids.remove(&client_id).unwrap_or(0);

        let body = json!(
            {
                "input": {
                    "status": status,
                    "solution": message,
                    "solutiontypes_id": 3,
                    "solutiontemplates_id": template_id
                }
            }
        );

        let _ = self.client.put( format!("{}/apirest.php/Ticket/{}?session_token={}", self.base_url, id, session_token) )
            .headers( headers )
            .json( &body )
            .send()
            .await;
    }
}
