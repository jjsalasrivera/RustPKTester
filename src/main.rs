use std::fs::OpenOptions;
use std::io::{Write, BufWriter};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use bitcoin::{Network, PrivateKey, secp256k1::Secp256k1, PublicKey, Address, CompressedPublicKey};
use rand::RngCore;
use rusqlite::{Connection, params};
use log::{info, warn};
use rayon::prelude::*;

const DATABASE: &str = "addresses.sqlite";
const MAX_CHUNK: usize = 5000;
const SECONDS_LOG: u64 = 10;

struct BitcoinChecker {
    checked_addresses: Arc<AtomicUsize>,
}

impl BitcoinChecker {
    fn new() -> Self {
        BitcoinChecker {
            checked_addresses: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn generate_private_key() -> Vec<u8> {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key.to_vec()
    }

    fn check_address_balance(address: &str, conn: &Connection) -> bool {
        let result = conn.query_row(
            "SELECT * FROM addresses WHERE address = ?1 LIMIT 1",
            params![address],
            |_row| Ok(())
        );
        result.is_ok()
    }

    fn log_found_address(
        private_key: &[u8],
        wif: &str,
        address: &str
    ) -> Result<(), std::io::Error> {
        let file_path = Path::new("found.txt");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        let mut writer = BufWriter::new(file);

        writeln!(writer, "ENCONTRADA DIRECCIÓN CON BALANCE!")?;
        writeln!(writer, "Private Key: {}", hex::encode(private_key))?;
        writeln!(writer, "WIF: {}", wif)?;
        writeln!(writer, "Address: {}", address)?;

        Ok(())
    }

    fn process_private_key(private_key: &[u8], conn: &Connection) {
        let secp = Secp256k1::new();

        // Convert private key bytes to Bitcoin PrivateKey
        match PrivateKey::from_slice(&private_key, Network::Bitcoin) {
            Ok(key) => {
                let public_key = PublicKey::from_private_key(&secp, &key);
                let compressed_pk_res = CompressedPublicKey::from_private_key(&secp, &key);
                let addresses_types = if let Ok(compressed_pk) = compressed_pk_res {
                    vec![
                        Address::p2pkh(&public_key, Network::Bitcoin),
                        Address::p2shwpkh(&compressed_pk, Network::Bitcoin),
                        Address::p2wpkh(&compressed_pk, Network::Bitcoin)
                    ]
                } else {
                    vec![
                        Address::p2pkh(&public_key, Network::Bitcoin),
                    ]
                };

                for address in addresses_types.iter() {
                    let address_str = address.to_string();

                    if Self::check_address_balance(&address_str, conn) {
                        info!("\n¡ENCONTRADA DIRECCIÓN CON BALANCE!");
                        info!("Clave Privada: {}", hex::encode(private_key));
                        info!("WIF: {}", key.to_wif());
                        info!("Dirección: {}", address_str);

                        if let Err(e) = Self::log_found_address(
                            private_key,
                            &key.to_wif(),
                            &address_str
                        ) {
                            warn!("Error al escribir en archivo: {}", e);
                        }
                    }
                }
            }
            Err(e) => warn!("Error creating private key: {}", e)
        }
    }

    fn process_keys_batch(&self) {
        let conn = Connection::open(DATABASE)
            .expect("Failed to open database connection");

        let keys: Vec<Vec<u8>> = (0..MAX_CHUNK)
            .map(|_| Self::generate_private_key())
            .collect();

        let len = keys.len();
        for key in keys {
            Self::process_private_key(&key, &conn);
        }

        self.checked_addresses.fetch_add(len, Ordering::SeqCst);
    }

    fn run(&self) {
        let mut last_log = Instant::now();
        let num_cores = num_cpus::get();


        loop {
            (1.. num_cores).into_par_iter().for_each(|_| {
                self.process_keys_batch();
            });

            if last_log.elapsed() >= Duration::from_secs(SECONDS_LOG) {
                info!("Direcciones revisadas: {}",
                    self.checked_addresses.load(Ordering::SeqCst)
                );
                last_log = Instant::now();
            }
        }
    }

    fn main(&self) {
        info!("Iniciando búsqueda de direcciones con balance...");
        self.run();
    }
}

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    let checker = BitcoinChecker::new();
    checker.main();
}
