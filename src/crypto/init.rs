use crate::add_to_file;
use home::home_dir;
use openssl::rsa::Rsa;
use openssl::symm::Cipher;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Config {
    username: String,
    repository: String,
    password_protected: bool,
}

pub async fn init_data() -> Result<(), std::io::Error> {
    let (path, passphrase, github, username, repository, password_protected) = get_init_data()?;
    // create folder if doesnt exists
    if !Path::new(path.trim()).exists() {
        std::fs::create_dir(path.trim())?;
    }
    let rsa = Rsa::generate(4096).unwrap();
    let private_key: Vec<u8> = rsa
        .private_key_to_pem_passphrase(Cipher::aes_256_cbc(), passphrase.as_bytes())
        .unwrap();
    let public_key: Vec<u8> = rsa.public_key_to_pem().unwrap();
    let private_key_save = String::from_utf8(private_key).unwrap();
    let public_key_save = String::from_utf8(public_key).unwrap();
    let mut private_key_file = File::create(format!("{}rustykey", path))?;
    let mut public_key_file = File::create(format!("{}rustykey.pem", path))?;
    let mut github_api = File::create(format!("{}github", path))?;
    let mut user_config = File::create(format!("{}config.json", path))?;

    private_key_file.write_all(private_key_save.as_bytes())?;
    public_key_file.write_all(public_key_save.as_bytes())?;
    github_api.write_all(github.as_bytes())?;
    let config = Config {
        username,
        repository,
        password_protected,
    };
    let config_json = serde_json::to_string(&config)?;
    user_config.write_all(config_json.as_bytes())?;

    Ok(add_to_file(true, "default", "welcome").await?)
}

fn get_init_data() -> Result<(String, String, String, String, String, bool), std::io::Error> {
    let path = if let Some(home_path) = home_dir() {
        String::from(format!("{}/.rustyvault/", home_path.to_string_lossy()))
    } else {
        String::from("/")
    };
    if Path::new(format!("{}rustykey", path).trim()).exists()
        && Path::new(format!("{}rustykey.pem", path).trim()).exists()
        && Path::new(format!("{}github", path).trim()).exists()
    {
        print!("The current key value for the program will be overriten, do you want to proceed? (y/N)");
        std::io::stdout().flush().unwrap();
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer).unwrap();
        buffer.pop().unwrap();
        if buffer != "y" && buffer != "Y" {
            panic!("exited")
        }
    }
    let pass = rpassword::prompt_password_stdout(
        "Enter a password for the keys (leave blank for no password): ",
    )
    .unwrap();
    let pass_confirm = rpassword::prompt_password_stdout(
        "Enter again the password (leave blank for no password): ",
    )
    .unwrap();
    let github_api_key = rpassword::prompt_password_stdout("Enter your github api key: ").unwrap();
    let github_api_key_confirm =
        rpassword::prompt_password_stdout("Enter again your github api key: ").unwrap();
    let username = rpassword::prompt_password_stdout("Enter your GitHub username: ").unwrap();
    let repository = rpassword::prompt_password_stdout(
        "Enter the repository name you want to use as your vault: ",
    )
    .unwrap();
    if pass == pass_confirm && github_api_key == github_api_key_confirm {
        println!("Creating the key pair in the folder ~/.rustyvault");
        println!("Creating the file github with your api key in the folder ~/.rustyvault");
        let password_protected = if pass.len() != 0 { true } else { false };
        return Ok((
            path,
            pass,
            github_api_key,
            username,
            repository,
            password_protected,
        ));
    }
    panic!("There was an error")
}
