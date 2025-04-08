use regex::Regex;

use crate::{configurations::config::ClientsConfig, model::entity::client::ClientData};

// Structs
pub struct Validator;

// Impls
impl Validator {
    pub fn validate_client( config: &ClientsConfig, data: &ClientData ) -> bool {
        if let Some(infos) = &config.info {
            
            if let Some(name_validated) = &infos.name_validated {
                let regex_res = Regex::new( &name_validated );

                if let Ok(regex) = regex_res {
                    if regex.is_match( &data.full_name ) == false {return false;}
                }; 
            }
            
            if let Some(email_validated) = &infos.email_validated {
                let regex_res = Regex::new( &email_validated );

                if let Ok(regex) = regex_res {
                    if regex.is_match( &data.email ) == false {return false;}
                }; 
            }
            
            if let Some(phone_validated) = &infos.phone_validated {
                let regex_res = Regex::new( &phone_validated );

                if let Ok(regex) = regex_res {
                    if regex.is_match( &data.phone )  == false {return false;}
                }; 
            }
            
            for (field, validate) in infos.fields.iter() {
                if let Some(value) = data.fields.get(field) {
                    
                    if let Ok(regex) = Regex::new( validate ) {
                        if regex.is_match(value) == false { return false; }
                    }

                }
                else {
                    return false;
                }
            }

        }

        true
    }
}
