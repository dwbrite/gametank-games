use std::sync::Arc;
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use itertools::Itertools;
use sqlx_adapter::SqlxAdapter;
use tokio::sync::Mutex;

pub type Casbin = Arc<Mutex<Enforcer>>;

pub async fn init_casbin(database_url: String) -> Casbin {
    let model = DefaultModel::from_str(include_str!("rbac_model.conf")).await.unwrap();
    let adapter = SqlxAdapter::new(database_url, 10).await.unwrap();
    let mut enforcer = Arc::new(Mutex::new(Enforcer::new(model, adapter).await.unwrap()));

    // TODO: default roles
    add_allow_policy(&mut enforcer, "user", "upload", "game").await;

    enforcer
}

pub async fn add_allow_policy(casbin: &mut Casbin, role: &str, action: &str, object: &str) {
    let mut casbin = casbin.lock().await;
    casbin.add_policy(vec![role, action, object, "allow"].into_iter().map_into().collect::<Vec<String>>()).await.expect("could not add policy to casbin");
}