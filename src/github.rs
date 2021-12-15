use serde::{Deserialize, Serialize};

pub struct Credentials {
    pub username: String,
    pub token: String
}

pub struct Github {
    credentials: Credentials,
    client: reqwest::Client
}

#[derive(Debug)]
pub enum GithubError {
    ReqwestError(reqwest::Error),
    DeserializationError(reqwest::Error)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GithubPlan {
    pub name: String,
    pub space: u32,
    pub collaborators: u32,
    pub private_repos: u32
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
    pub login: String,
    pub id: u32,
    pub node_id: String,
    pub avatar_url: String,
    pub gavatar_id: Option<String>,
    pub url: String,
    pub html_url: String,
    pub followers_url: String,
    pub following_url: String,
    pub gists_url: String,
    pub starred_url: String,
    pub subscriptions_url: String,
    pub organizations_url: String,
    pub repos_url: String,
    pub events_url: String,
    pub received_events_url: String,

    #[serde(rename(deserialize = "type", serialize = "type"))]
    pub account_type: String,
    
    pub site_admin: Option<bool>,
    pub name: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub location: Option<String>,
    pub email: Option<String>,
    pub hireable: Option<bool>,
    pub bio: Option<String>,
    pub twitter_username: Option<String>,
    pub public_repos: Option<u32>,
    pub public_gists: Option<u32>,
    pub followers: Option<u32>,
    pub following: Option<u32>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub private_gists: Option<u32>,
    pub total_private_repos: Option<u32>,
    pub owned_private_repos: Option<u32>,
    pub disk_usage: Option<u64>,
    pub collaborators: Option<u32>,
    pub two_factor_authentication: Option<bool>,
    pub plan: Option<GithubPlan>
}
#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct RepositoryPermissions {
    pub maintain: Option<bool>,
    pub triage: Option<bool>,
    pub pull: Option<bool>,
    pub push: Option<bool>,
    pub admin: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct License {
    pub key: String,
    pub name: String,
    pub spdx_id: String,
    pub url: String,
    pub node_id: String
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Repository {
    pub id: u32,
    pub node_id: String,
    pub name: String,
    pub full_name: String,
    pub private: Option<bool>,
    pub owner: User,
    pub html_url: String,
    pub description: Option<String>,
    pub fork: bool,
    pub url: String,
    pub forks_url: String,
    pub keys_url: String,
    pub collaborators_url: String,
    pub hooks_url: String,
    pub issue_events_url: String,
    pub assignees_url: String,
    pub events_url: String,
    pub tags_url: String,
    pub git_tags_url: String,
    pub statuses_url: String,
    pub languages_url: String,
    pub stargazers_url: String,
    pub contributors_url: String,
    pub subscribers_url: String,
    pub commits_url: String,
    pub git_commits_url: String,
    pub comments_url: String,
    pub compare_url: String,
    pub contents_url: String,
    pub merges_url: String,
    pub pulls_url: String,
    pub archive_url: String,
    pub issues_url: String,
    pub milestones_url: String,
    pub downloads_url: String,
    pub deployments_url: String,
    pub labels_url: String,
    pub notifications_url: String,
    pub permissions: RepositoryPermissions,
    pub default_branch: String,
    pub watchers: u32,
    pub open_issues: u32,
    pub forks: u32,
    pub visibility: String,
    pub topics: Vec<String>,
    pub is_template: bool,
    pub has_pages: bool,
    pub allow_forking: bool,
    pub license: Option<License>,
    pub open_issues_count: u32,
    pub disabled: bool,
    pub archived: bool,
    pub mirror_url: Option<String>,
    pub forks_count: u32,
    pub has_wiki: bool,
    pub has_downloads: bool,
    pub language: Option<String>,
    pub stargazers_count: u32,
    pub size: u32,
    pub homepage: Option<String>,
    pub svn_url: String,
    pub ssh_url: String,
    pub clone_url: String,
    pub git_url: String, 
    pub created_at: String,
    pub updated_at: String,
    pub pushed_at: String

}

impl Github {
    pub fn new(credentials: Credentials) -> Self {
        Github { credentials, client: reqwest::Client::new() }
    }

    pub async fn get_user(&self, username: String) -> Result<User, GithubError> {
        let url = &format!("https://api.github.com/users/{}", username);
        let response = self.client.get(url).basic_auth(&self.credentials.username, Some(&self.credentials.token)).send();

        if response.is_err() {
            return Err(GithubError::ReqwestError(response.unwrap_err()));
        }

        let mut response = response.unwrap();

        match response.json() {
            Ok(user_data) => Ok(user_data),
            Err(e) => Err(GithubError::DeserializationError(e))
        }


    }

    pub async fn get_repositories(&self, username: String) -> Result<Vec<Repository>, GithubError> {
        let url = &format!("https://api.github.com/users/{}/repos", username);
        let response = self.client.get(url).basic_auth(&self.credentials.username, Some(&self.credentials.token)).send();

        if response.is_err() {
            return Err(GithubError::ReqwestError(response.unwrap_err()));
        }

        let mut response = response.unwrap();

        match response.json() {
            Ok(repositories) => Ok(repositories),
            Err(e) => Err(GithubError::DeserializationError(e))
        }
    }
}
