use chrono::NaiveDateTime;
use dotenv::dotenv;
use github::{Credentials, Github};
use rocket::{Config, State};
use storage::JsonStorage;
use std::env;
use crate::{request::Response, github::Repository, cors::CORS};

#[macro_use] extern crate rocket;

pub mod github;
pub mod storage;
pub mod request;
pub mod cors;

const REPOSITORIES_STORAGE_FILENAME: &str = "repositories.json";
const USER_DATA_STORAGE_FILENAME: &str = "user.json";
const USERNAME: &str = "ivanhrabcak"; 

type Repositories = Vec<Repository>;

fn parse_timestamp(timestamp: String) -> Option<NaiveDateTime> {
    let date_part = timestamp.split("T").nth(0).unwrap();
    let time_part = timestamp.split("T").nth(1).unwrap().replace("Z", "");

    let datetime = format!("{} {}", date_part, time_part);
    let result = NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S");

    if result.is_err() {
        None
    }
    else {
        Some(result.unwrap())
    }

}

async fn get_cached_repositories(github: &Github, username: String) -> Vec<Repository> {
    let repositories = JsonStorage::new(REPOSITORIES_STORAGE_FILENAME.to_string()).await.get_stored_data().await;

    if repositories.is_none() {
        let new_repositories = github.get_repositories(username).await;

        if new_repositories.is_err() {
            return Vec::new();
        }

        let new_repositories = new_repositories.unwrap();

        // we want to crash if this fails
        JsonStorage::<Repositories>::new(REPOSITORIES_STORAGE_FILENAME.to_string()).await.store(&new_repositories).await.unwrap();

        return new_repositories;
    }

    repositories.unwrap()
}

#[get("/repositories")]
async fn get_repositories(github: &State<Github>) -> Response<Vec<Repository>> {
    let repositories = get_cached_repositories(github, USERNAME.to_string()).await;

    if repositories.len() == 0 {
        return Response::new(Vec::new(), 500);
    }

    Response::new(repositories, 200)
}

#[get("/repositories/sorted/<n>")]
async fn get_n_starred_repository(github: &State<Github>, n: usize) -> Response<Option<Repository>> {
    let mut repositories = get_cached_repositories(github, USERNAME.to_string()).await;

    if repositories.len() < n {
        return Response::new(None, 400);
    }
    
    repositories.sort_by(|x, y| { y.stargazers_count.cmp(&x.stargazers_count) });
    

    // safe to unwrap, we know there is an item at this index
    let repository = repositories.get(n).unwrap().clone();

    Response::new(Some(repository), 200)
}

#[get("/repositories/last_pushed/<n>")]
async fn get_n_last_pushed_repository(github: &State<Github>, n: usize) -> Response<Option<Repository>> {
    let mut repositories = get_cached_repositories(github, USERNAME.to_string()).await;

    if repositories.len() < n {
        return Response::new(None, 400);
    }

    repositories.sort_by(|x, y| { 
        let last_pushed_x = parse_timestamp(x.pushed_at.clone()).unwrap();
        let last_pushed_y = parse_timestamp(y.pushed_at.clone()).unwrap();
        
        last_pushed_y.cmp(&last_pushed_x)
    });

    let repository = repositories.get(n).unwrap().clone();

    Response::new(Some(repository), 200)
}

#[get("/repositories/size/<n>")]
async fn get_n_largest_repository(github: &State<Github>, n: usize) -> Response<Option<Repository>> {
    let mut repositories = get_cached_repositories(github, USERNAME.to_string()).await;

    if repositories.len() < n {
        return Response::new(None, 400);
    }

    repositories.sort_by(|x, y| {
        let size_x = x.size;
        let size_y = y.size;

        size_y.cmp(&size_x)
    });

    let repository = repositories.get(n).unwrap().clone();

    Response::new(Some(repository), 200)
}

#[get("/stars")]
async fn get_n_of_stars(github: &State<Github>) -> Response<u32> {
    let repositories = get_cached_repositories(github, USERNAME.to_string()).await;

    let mut stars: u32 = 0;
    for repository in repositories {
        stars += repository.stargazers_count; 
    }

    return Response::new(stars, 200);
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();

    let token = env::var("TOKEN").unwrap();
    let username = env::var("USERNAME").unwrap();
    
    let credentials = Credentials { username, token };
    let github = Github::new(credentials);

    // we do not need to create a TemporaryJsonStorage state, as each of its method consumes the object
    // it's not meant to be used as a managed state

    let port: i32 = i32::from_str_radix(&env::var("PORT").unwrap(), 10).unwrap();
    let config = Config::figment().merge(("port", port));
    
    rocket::custom(config)
            .attach(CORS)
            .manage(github)
            .mount("/", routes![get_repositories, get_n_starred_repository, get_n_last_pushed_repository, get_n_largest_repository, get_n_of_stars])
}            