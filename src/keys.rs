use super::hydrogen;
use failure::{Error, ResultExt};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;

const KEYID_SZ: usize = 32;

#[derive(Serialize, Deserialize, Debug)]
pub struct MasterKey {
    id: [u8; KEYID_SZ],
    hash_key1: [u8; hydrogen::HASH_KEYBYTES],
    data_pk: [u8; hydrogen::KX_PUBLICKEYBYTES],
    data_sk: [u8; hydrogen::KX_SECRETKEYBYTES],
    data_psk: [u8; hydrogen::KX_PSKBYTES],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientKey {
    id: [u8; KEYID_SZ],
    master_key_id: [u8; KEYID_SZ],
    master_data_pk: [u8; hydrogen::KX_PUBLICKEYBYTES],
    hash_key1: [u8; hydrogen::HASH_KEYBYTES],
    hash_key2: [u8; hydrogen::HASH_KEYBYTES],
    data_psk: [u8; hydrogen::KX_PSKBYTES],
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Key {
    MasterKeyV1(MasterKey),
    ClientKeyV1(ClientKey),
}

impl Key {
    pub fn write_to_file(&self, path: &str) -> Result<(), Error> {
        let mut file = OpenOptions::new()
            .mode(0o600)
            .write(true)
            .create_new(true)
            .open(path)
            .with_context(|e| format!("error opening {}: {}", path, e))?; // Give read/write for owner and read for others.
        let j = serde_json::to_string(self)?;
        file.write_all(j.as_bytes())
            .with_context(|e| format!("writing key file failed: {}", e))?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Key, Error> {
        let mut file = OpenOptions::new()
            .mode(0o600)
            .read(true)
            .open(path)
            .with_context(|e| format!("error opening {}: {}", path, e))?; // Give read/write for owner and read for others.
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|e| format!("reading key file failed: {}", e))?;
        let k: Key = serde_json::from_str(&contents)?;
        Ok(k)
    }
}

fn keyid_gen() -> [u8; KEYID_SZ] {
    let mut id = [0; KEYID_SZ];
    hydrogen::random_buf(&mut id[..]);
    id
}

impl MasterKey {
    pub fn gen() -> MasterKey {
        let id = keyid_gen();
        let hash_key1 = hydrogen::hash_keygen();
        let data_psk = hydrogen::kx_psk_keygen();
        let (data_pk, data_sk) = hydrogen::kx_keygen();

        MasterKey {
            id,
            hash_key1,
            data_psk,
            data_pk,
            data_sk,
        }
    }
}

impl ClientKey {
    pub fn gen(mk: &MasterKey) -> ClientKey {
        ClientKey {
            id: keyid_gen(),
            master_key_id: mk.id,
            hash_key1: mk.hash_key1,
            hash_key2: hydrogen::hash_keygen(),
            data_psk: mk.data_psk,
            master_data_pk: mk.data_pk,
        }
    }
}
