use actix_web::{get, patch, post, web, Error, HttpResponse};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::app_config::MappedAppConfig;
use crate::storage::MySqlPool;

use crate::models::config::{NewConfig, UpdateConfig, Config};

#[get("/configs")]
async fn show_configs(
    auth: BearerAuth,
    app_config: web::Data<MappedAppConfig>,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, Error> {
    let token = String::from(auth.token());

    let mtoken = token.clone();
    let mpool = pool.clone();
    let mut configs = web::block(move || {
        let mut conn = mpool.get()?;
        Config::get_all_config_by_user(&mut conn, &mtoken)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    for shared_config in &app_config.users.get(&token).unwrap().shared_configs {
        let mpool = pool.clone();
        let id = shared_config.to_owned();

        let config = web::block(move || {
            let mut conn = mpool.get()?;
            Config::get_shared_config_without_content_by_id(&mut conn, id)
        })
        .await?
        .map_err(actix_web::error::ErrorInternalServerError)?
        .unwrap();

        configs.push(config);
    }

    Ok(HttpResponse::Ok().json(configs))
}

#[post("/configs")] // create a new config
async fn new_config(
    auth: BearerAuth,
    pool: web::Data<MySqlPool>,
    json: web::Json<NewConfig>,
) -> Result<HttpResponse, Error> {
    let token = auth.token();
    let new_user_config = json
        .into_inner()
        .into_new_user_config_with_user(String::from(token));

    web::block(move || {
        let mut conn = pool.get()?;
        Config::insert_new_user_config(&mut conn, new_user_config)
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

#[get("/configs/{id}")]
async fn get_config(
    auth: BearerAuth,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let token = String::from(auth.token());
    let id = path.into_inner();

    match web::block(move || {
        let mut conn = pool.get()?;
        if (1..crate::models::config::MAX_SHARED_CONFIG_ID).contains(&id) { // SHARED CONFIG
            Config::get_user_shared_config_by_id(&mut conn, id, &token)
        } else { // USER CONFIG
            Config::get_config_by_id_and_user(&mut conn, id, &token)
        }
    })
    .await?
    .map_err(actix_web::error::ErrorInternalServerError)?
    {
        Some(config) => Ok(HttpResponse::Ok().json(config)), // TODO: remove user from config
        None => Ok(HttpResponse::Unauthorized().finish()),
    }

}

#[patch("/configs/{id}")]
async fn update_config(
    auth: BearerAuth,
    pool: web::Data<MySqlPool>,
    path: web::Path<i32>,
    json: web::Json<UpdateConfig>,
) -> Result<HttpResponse, Error> {
    let token = String::from(auth.token());
    let id = path.into_inner();
    let updated_config = json.into_inner();

    if (1..crate::models::config::MAX_SHARED_CONFIG_ID).contains(&id) {
        // TODO: SHARED CONFIG
        Ok(HttpResponse::InternalServerError().finish())
    } else {
        // CHECK IF THE USER POSSESS THIS CONFIG ID

        let t = token.clone();
        let p = pool.clone();
        let config = web::block(move || {
            let mut conn = p.get()?;
            Config::get_config_by_id_and_user(&mut conn, id, &t)
        })
        .await?
        .map_err(actix_web::error::ErrorInternalServerError)?;

        if config.is_none() {
            return Ok(HttpResponse::Unauthorized().finish());
        }

        let config = config.unwrap();
        web::block(move || {
            // update config content
            let mut conn = pool.get()?;
            Config::update_user_config_content(&mut conn, config, &updated_config.content)
        })
        .await?
        .map_err(actix_web::error::ErrorInternalServerError)?;

        Ok(HttpResponse::Ok().finish())
    }
}

pub fn config_route_config(cfg: &mut web::ServiceConfig) {
    cfg.service(show_configs)
        .service(new_config)
        .service(get_config)
        .service(update_config);
}
