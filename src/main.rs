// use redis::Commands;
use redis::{AsyncCommands, RedisResult};
use serde::{Deserialize, Serialize};
use actix_web::{web, App, HttpResponse, Responder, HttpServer};
use std::sync::Mutex;
use async_trait::async_trait;


struct AppState {
    client: Mutex<Box<dyn RedisAPI>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = RedisClient::new("redis://127.0.0.1:6379");


    let shared_data = web::Data::new(AppState {
        client: Mutex::new(Box::new(cli)),
    });
    
    HttpServer::new(move || {
        App::new()
        .app_data(shared_data.clone())
        .route("/", web::get().to(get_value))
        .route("/", web::post().to(set_value))
    }).bind("127.0.0.1:8080")?
    .run()
    .await
}


#[derive(Serialize, Deserialize)]
struct GetValueReq {
    key: String,
}
#[derive(Serialize, Deserialize)]
struct GetValueResp {
    key: String,
    value: String,
}

async fn get_value(req: web::Json<GetValueReq>, data: web::Data<AppState>) -> impl Responder {
    let client = data.client.lock().unwrap();
    let val = client.get(&req.key).await.unwrap();

    HttpResponse::Ok().json(GetValueResp{
        key: req.key.clone(),
        value: val
    })
}

#[derive(Serialize, Deserialize)]
struct SetValueReq {
    key: String,
    value: String,
}

async fn set_value(req: web::Json<SetValueReq>, data: web::Data<AppState>) -> impl Responder {
    let client = data.client.lock().unwrap();
    client.set(&req.key, &req.value).await.unwrap();
    HttpResponse::Accepted()
}

pub struct RedisClient {
    client: redis::Client,
}

impl RedisClient {
    pub fn new(redis_url: &str) -> RedisClient {
        RedisClient {
            client: redis::Client::open(redis_url).unwrap(),
        }
    }
}


#[async_trait]
impl RedisAPI for RedisClient {
    async fn get(&self, key: &str) -> RedisResult<String> {
        let mut conn = self.client.get_async_connection().await?;
        let value: String = conn.get(key).await?;
        RedisResult::Ok(value)
    }

    async fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        conn.set(key, value).await?;
        RedisResult::Ok(())
    }
}

#[async_trait]
pub trait RedisAPI: Send + Sync {
  async fn get(&self, key: &str) -> RedisResult<String>;
  async fn set(&self, key: &str, value: &str) -> RedisResult<()>;
}

