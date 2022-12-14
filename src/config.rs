use serde::{Deserialize, Serialize};
use std::{
    net::{TcpStream, ToSocketAddrs},
    path::PathBuf,
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Config {
    pub disable_tls: Option<bool>,
    pub method: Option<String>,
    pub password: Option<String>,
    pub tunnel_path: String,
    #[serde(rename(deserialize = "server_settings", serialize = "server_settings"))]
    pub server: Option<Server>,
    #[serde(rename(deserialize = "client_settings", serialize = "client_settings"))]
    pub client: Option<Client>,
    #[serde(skip)]
    pub test_timeout_secs: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Server {
    pub manage_clients: Option<ManageClients>,
    pub certfile: Option<PathBuf>,
    pub keyfile: Option<PathBuf>,
    pub forward_addr: Option<String>,
    pub listen_host: String,
    pub listen_port: u16,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct ManageClients {
    pub webapi_url: Option<String>,
    pub webapi_token: Option<String>,
    pub node_id: Option<usize>,
    #[serde(rename(deserialize = "api_update_time", serialize = "api_update_time"))]
    pub api_update_interval_secs: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
pub struct Client {
    pub server_host: String,
    pub server_port: u16,
    pub server_domain: Option<String>,
    pub cafile: Option<PathBuf>,
    pub listen_host: String,
    pub listen_port: u16,
    pub client_id: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            disable_tls: None,
            method: None,
            password: None,
            tunnel_path: "/tunnel/".to_string(),
            server: None,
            client: None,
            test_timeout_secs: 5,
        }
    }

    pub fn manage_clients(&self) -> bool {
        let f = |s: &Server| s.manage_clients.is_some();
        self.server.as_ref().map(f).unwrap_or(false)
    }

    pub fn webapi_url(&self) -> Option<String> {
        let f = |s: &Server| s.manage_clients.as_ref().map(|c| c.webapi_url.clone()).unwrap_or(None);
        self.server.as_ref().map(f).unwrap_or(None)
    }

    pub fn webapi_token(&self) -> Option<String> {
        let f = |s: &Server| {
            let f2 = |c: &ManageClients| c.webapi_token.clone();
            s.manage_clients.as_ref().map(f2).unwrap_or(None)
        };
        self.server.as_ref().map(f).unwrap_or(None)
    }

    pub fn node_id(&self) -> Option<usize> {
        let f = |s: &Server| s.manage_clients.as_ref().map(|c| c.node_id).unwrap_or(None);
        self.server.as_ref().map(f).unwrap_or(None)
    }

    pub fn api_update_interval_secs(&self) -> Option<u64> {
        let f = |s: &Server| {
            let f2 = |c: &ManageClients| c.api_update_interval_secs;
            s.manage_clients.as_ref().map(f2).unwrap_or(None)
        };
        self.server.as_ref().map(f).unwrap_or(None)
    }

    pub fn exist_server(&self) -> bool {
        self.server.is_some()
    }

    pub fn exist_client(&self) -> bool {
        self.client.is_some()
    }

    pub fn disable_tls(&self) -> bool {
        self.disable_tls.unwrap_or(false)
    }

    pub fn check_correctness(&mut self, running_server: bool) -> anyhow::Result<()> {
        if running_server {
            self.client = None;
        } else {
            self.server = None;
        }
        if let (None, None) = (&self.server, &self.client) {
            return Err(anyhow::anyhow!("Need server or client settings"));
        }

        if self.test_timeout_secs == 0 {
            self.test_timeout_secs = 5;
        }
        if self.method.is_none() || self.method.as_ref().unwrap_or(&"".to_owned()).is_empty() {
            self.method = Some("none".to_string());
        }
        if self.password.is_none() || self.password.as_ref().unwrap_or(&"".to_owned()).is_empty() {
            self.password = Some("password".to_string());
        }
        if self.tunnel_path.is_empty() {
            self.tunnel_path = "/tunnel/".to_string();
        } else {
            self.tunnel_path = format!("/{}/", self.tunnel_path.trim().trim_matches('/'));
        }

        if let Some(server) = &mut self.server {
            if server.listen_host.is_empty() {
                server.listen_host = "0.0.0.0".to_string();
            }
            if server.listen_port == 0 {
                server.listen_port = 443;
            }
        }
        if let Some(client) = &mut self.client {
            if client.server_host.is_empty() {
                return Err(anyhow::anyhow!("We need server_host in client settings"));
            }
            if client.server_port == 0 {
                client.server_port = 443;
            }
            if client.server_domain.is_none() || client.server_domain.as_ref().unwrap_or(&"".to_string()).is_empty() {
                client.server_domain = Some(client.server_host.clone());
            }
            if client.listen_host.is_empty() {
                client.listen_host = "127.0.0.1".to_string();
            }

            if !running_server {
                let addr = format!("{}:{}", client.server_host, client.server_port);
                let mut addr = addr
                    .to_socket_addrs()
                    .map_err(|e| anyhow::anyhow!("server {addr} error \"{e}\""))?;
                let addr = addr.next().ok_or_else(|| anyhow::anyhow!("address"))?;
                let timeout = std::time::Duration::from_secs(self.test_timeout_secs);
                TcpStream::connect_timeout(&addr, timeout)
                    .map_err(|e| anyhow::anyhow!("server {} error \"{}\"", addr, e))?;
                client.server_host = addr.ip().to_string();
            }
        }
        Ok(())
    }
}
