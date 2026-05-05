use bip39::{Language, Mnemonic};
use bip32::{DerivationPath, XPrv, Seed};
use k256::ecdsa::SigningKey;
use rayon::prelude::*;
use tiny_keccak::{Hasher, Keccak};

use std::convert::TryInto;
use std::fs;
use std::time::{Duration, Instant};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, AtomicBool, Ordering}
};
use std::thread;
use std::io::{self, Write};

fn eth_address_from_prv(prv: &SigningKey) -> String {
    let pubkey = prv.verifying_key().to_encoded_point(false);
    let pub_uncompressed = pubkey.as_bytes();
    let mut keccak = Keccak::v256();
    let mut out = [0u8; 32];
    keccak.update(&pub_uncompressed[1..]);
    keccak.finalize(&mut out);
    let addr = &out[12..];
    format!("0x{}", hex::encode(addr))
}

fn derive_eth_address_from_seed(seed: &Seed, path: &DerivationPath) -> String {
    let xprv = XPrv::derive_from_path(seed.clone(), path).expect("derive_from_path failed");
    let sk_bytes = xprv.private_key().to_bytes();
    let signing_key = SigningKey::from_bytes(&sk_bytes).expect("to signing key");
    eth_address_from_prv(&signing_key)
}

#[derive(Clone)]
struct Task {
    known_words: Vec<String>,
    blank_idx: Vec<usize>,
    target_address: String,
    path: DerivationPath,
    candidate_words: Vec<String>,
}

fn main() {
    // ===== Baca file =====
    let known_words: Vec<String> = fs::read_to_string("known.txt")
        .expect("gagal baca known.txt")
        .lines()
        .map(|s| s.trim().to_string())
        .collect();

    let candidate_words: Vec<String> = fs::read_to_string("wordlist.txt")
        .expect("gagal baca wordlist.txt")
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let target_address = fs::read_to_string("target.txt")
        .expect("gagal baca target.txt")
        .trim()
        .to_lowercase();

    assert_eq!(known_words.len(), 12, "known.txt harus 12 baris");

    let blank_idx: Vec<usize> = known_words.iter().enumerate()
        .filter(|(_, w)| w == &"__")
        .map(|(i, _)| i)
        .collect();

    let k = blank_idx.len();
    assert!(k >= 1, "Harus ada minimal 1 slot kosong '__' di known.txt");

    let path: DerivationPath = "m/44'/60'/0'/0/0".parse().expect("path salah");

    let task = Task {
        known_words,
        blank_idx,
        target_address,
        path,
        candidate_words,
    };

    let w = task.candidate_words.len() as u128;
    let total_u128 = w.pow(k as u32);
    if total_u128 > (usize::MAX as u128) {
        eprintln!("Total kombinasi terlalu besar untuk platform ini.");
        return;
    }
    let total = total_u128 as usize;

    println!("Slot kosong: {} | Kandidat/slot: {} | Total kombinasi: {}", k, w, total);

    // Cache pangkat W untuk decode indeks
    let mut pow_cache: Vec<u128> = Vec::with_capacity(k);
    let mut acc: u128 = 1;
    for _ in 0..k {
        pow_cache.push(acc);
        acc = acc.saturating_mul(w);
    }

    let processed = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));
    let found_flag = Arc::new(AtomicBool::new(false));
    let start = Instant::now();

    // Thread progress
    {
        let processed = processed.clone();
        let done = done.clone();
        let found_flag = found_flag.clone();
        thread::spawn(move || {
            loop {
                if done.load(Ordering::Relaxed) || found_flag.load(Ordering::Relaxed) {
                    break;
                }
                let p = processed.load(Ordering::Relaxed);
                let elapsed = start.elapsed().as_secs_f64().max(1e-9);
                let rate = p as f64 / elapsed;
                let remain = if rate > 0.0 {
                    (total.saturating_sub(p)) as f64 / rate
                } else { 0.0 };
                print!(
                    "\rProgress: {}/{} ({:.2}%)  |  {:.0} cand/s  |  ETA {:.1}s   ",
                    p, total, (p as f64 / total as f64) * 100.0, rate, remain
                );
                let _ = io::stdout().flush();
                thread::sleep(Duration::from_secs(1));
            }
            println!();
        });
    }

    // Enumerasi paralel dengan early stop
    let found: Option<String> = (0..total).into_par_iter()
        .find_any(|&idx_usize| {
            if found_flag.load(Ordering::Relaxed) { return false; }

            let mut trial = task.known_words.clone();
            let mut n = idx_usize as u128;
            for (pos, &bidx) in task.blank_idx.iter().enumerate() {
                let digit = (n / pow_cache[pos]) % w;
                trial[bidx] = task.candidate_words[digit as usize].clone();
            }
            let phrase = trial.join(" ");

            let mnemonic = match Mnemonic::parse_in(Language::English, &phrase) {
                Ok(m) => m,
                Err(_) => { processed.fetch_add(1, Ordering::Relaxed); return false; }
            };
            let raw_seed_vec = mnemonic.to_seed("");
            let raw_seed_arr: [u8; 64] = match raw_seed_vec.as_slice().try_into() {
                Ok(arr) => arr,
                Err(_) => { processed.fetch_add(1, Ordering::Relaxed); return false; }
            };
            let seed = Seed::new(raw_seed_arr);
            let addr = derive_eth_address_from_seed(&seed, &task.path).to_lowercase();
            processed.fetch_add(1, Ordering::Relaxed);

            if addr == task.target_address {
                found_flag.store(true, Ordering::Relaxed);
                println!("\nMATCH FOUND:\n{}\nADDR : {}", phrase, addr);
                true
            } else {
                false
            }
        })
        .map(|_| "match found".to_string());

    done.store(true, Ordering::Relaxed);

    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "Selesai dalam {:.2} detik | {:.0} kandidat/detik",
        elapsed,
        processed.load(Ordering::Relaxed) as f64 / elapsed
    );

    if found.is_none() {
        println!("Tidak ada kandidat yang cocok.");
    }
}

