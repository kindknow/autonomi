// Copyright 2024 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::wallet::load_wallet;
use autonomi::Multiaddr;
use color_eyre::eyre::Context;
use color_eyre::eyre::Result;
use color_eyre::Section;

pub async fn cost(peers: Vec<Multiaddr>) -> Result<()> {
    let client = crate::actions::connect_to_network(peers).await?;
    let vault_sk = crate::keys::get_vault_secret_key()?;

    println!("Getting cost to create a new vault...");
    let total_cost = client.vault_cost(&vault_sk).await?;

    if total_cost.is_zero() {
        println!("Vault already exists, modifying an existing vault is free");
    } else {
        println!("Cost to create a new vault: {total_cost} AttoTokens");
    }
    Ok(())
}

pub async fn create(peers: Vec<Multiaddr>) -> Result<()> {
    let client = crate::actions::connect_to_network(peers).await?;
    let wallet = load_wallet()?;
    let vault_sk = crate::keys::get_vault_secret_key()?;

    println!("Retrieving local user data...");
    let local_user_data = crate::user_data::get_local_user_data()?;
    let file_archives_len = local_user_data.file_archives.len();
    let private_file_archives_len = local_user_data.private_file_archives.len();

    println!("Pushing to network vault...");
    let total_cost = client
        .put_user_data_to_vault(&vault_sk, wallet.into(), local_user_data)
        .await?;

    if total_cost.is_zero() {
        println!("✅ Successfully pushed user data to existing vault");
    } else {
        println!("✅ Successfully created new vault containing local user data");
    }

    println!("Total cost: {total_cost} AttoTokens");
    println!("Vault contains:");
    println!("{file_archives_len} public file archive(s)");
    println!("{private_file_archives_len} private file archive(s)");
    Ok(())
}

pub async fn sync(peers: Vec<Multiaddr>, force: bool) -> Result<()> {
    let client = crate::actions::connect_to_network(peers).await?;
    let vault_sk = crate::keys::get_vault_secret_key()?;
    let wallet = load_wallet()?;

    println!("Fetching vault from network...");
    let net_user_data = client
        .get_user_data_from_vault(&vault_sk)
        .await
        .wrap_err("Failed to fetch vault from network")
        .with_suggestion(|| "Make sure you have already created a vault on the network")?;

    if force {
        println!("The force flag was provided, overwriting user data in the vault with local user data...");
    } else {
        println!("Syncing vault with local user data...");
        crate::user_data::write_local_user_data(&net_user_data)?;
    }

    println!("Pushing local user data to network vault...");
    let local_user_data = crate::user_data::get_local_user_data()?;
    let file_archives_len = local_user_data.file_archives.len();
    let private_file_archives_len = local_user_data.private_file_archives.len();
    client
        .put_user_data_to_vault(&vault_sk, wallet.into(), local_user_data)
        .await?;

    println!("✅ Successfully synced vault");
    println!("Vault contains:");
    println!("{file_archives_len} public file archive(s)");
    println!("{private_file_archives_len} private file archive(s)");
    Ok(())
}

pub async fn load(peers: Vec<Multiaddr>) -> Result<()> {
    let client = crate::actions::connect_to_network(peers).await?;
    let vault_sk = crate::keys::get_vault_secret_key()?;

    println!("Retrieving vault from network...");
    let user_data = client.get_user_data_from_vault(&vault_sk).await?;
    println!("Writing user data to disk...");
    crate::user_data::write_local_user_data(&user_data)?;

    println!("✅ Successfully loaded vault with:");
    println!("{} public file archive(s)", user_data.file_archives.len());
    println!(
        "{} private file archive(s)",
        user_data.private_file_archives.len()
    );
    Ok(())
}