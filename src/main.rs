use std::fs;
use clap::Parser;
use reqwest::Client;
use serde_json::Value;

static MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'm', long)]
    minecraft_version: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let web_client = Client::new();

    let response: Value = web_client.get(MANIFEST_URL)
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();

    let versions = response.as_object().unwrap().get("versions").unwrap().as_array().unwrap();

    let mut version: Option<Value> = None;
    for ver in versions {
        let version_object = ver.as_object().unwrap();
        if version_object.get("id").unwrap().as_str().unwrap() == args.minecraft_version {
            version = Some(ver.clone());
            break;
        }
    }

    if version.is_none() {
        println!("version {:?} could not be found", args.minecraft_version);
        return;
    }

    let version = version.unwrap();
    let version_object = version.as_object().unwrap();
    let url = version_object.get("url").unwrap().as_str().unwrap();
    
    let response: Value = web_client.get(url)
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();

    let url = response.as_object().unwrap().get("assetIndex").unwrap().as_object().unwrap().get("url").unwrap().as_str().unwrap();

    let response: Value = web_client.get(url)
        .send()
        .await.unwrap()
        .json()
        .await.unwrap();

    let objects = response.as_object().unwrap().get("objects").unwrap().as_object().unwrap();

    for (path_and_file, object) in objects {
        let (path, file) = match path_and_file.rsplit_once("/") {
            Some((path, file)) => (path, file),
            None => ("", path_and_file.as_str()),
        };
        fs::create_dir_all(format!("{}/{}", args.minecraft_version, path)).unwrap();

        let hash = object.get("hash").unwrap().as_str().unwrap();
        let url = format!("https://resources.download.minecraft.net/{}/{}", hash[0..2].to_string(), hash);
        
        let response: Vec<u8> = web_client.get(url)
            .send()
            .await.unwrap()
            .bytes()
            .await.unwrap()
            .to_vec();

        fs::write(format!("{}/{}/{}", args.minecraft_version, path, file), response).unwrap();
        println!("{} - {}", path, file);
    }
}
