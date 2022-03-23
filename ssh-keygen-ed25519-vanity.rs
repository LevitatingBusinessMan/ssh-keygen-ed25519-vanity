extern crate rand;
extern crate base64;
extern crate bytebuffer;
extern crate ed25519_dalek;

use std::env::args;
use std::mem::size_of;

use rand::rngs::OsRng;
use base64::encode;
use bytebuffer::{ByteBuffer, Endian::BigEndian};
use ed25519_dalek::{Keypair, PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};

use std::thread;
use std::sync::mpsc;

use std::os::unix::fs::PermissionsExt;
use std::fs::{File, Permissions};
use std::io::Write;

const KEYTYPE: &[u8] = b"ssh-ed25519";
const MAGIC: &[u8] = b"openssh-key-v1\x00";
const NONE: &[u8] = b"none";
const BLOCKSIZE: usize = 8;

fn get_sk(pk: &[u8], keypair: Keypair) -> String {
  let mut buffer = ByteBuffer::new();
  buffer.write_bytes(MAGIC);
  buffer.write_u32(NONE.len() as u32);
  buffer.write_bytes(NONE);                   // cipher
  buffer.write_u32(NONE.len() as u32);
  buffer.write_bytes(NONE);                   // kdfname
  buffer.write_u32(0);                        // no kdfoptions
  buffer.write_u32(1);                        // public keys
  buffer.write_u32(pk.len() as u32);
  buffer.write_bytes(pk);                     // public key

  let mut sk = ByteBuffer::new();
  sk.write_u32(0xf0cacc1a);                   // check bytes
  sk.write_u32(0xf0cacc1a);
  sk.write_bytes(pk);                         // public key (again)
  sk.write_u32((SECRET_KEY_LENGTH + PUBLIC_KEY_LENGTH) as u32);
  sk.write_bytes(&keypair.secret.to_bytes()); // private key
  sk.write_bytes(&keypair.public.to_bytes()); // public part of private key
  sk.write_u32(0);                            // no comments
  for p in 1..=(buffer.len() + sk.len() + size_of::<u32>()) % BLOCKSIZE {
    sk.write_u8(p as u8);                     // padding
  }

  buffer.write_u32(sk.len() as u32);
  buffer.write_bytes(&sk.to_bytes());
  return encode(buffer.to_bytes());
}

fn main() {
  let pattern = args().nth(1).unwrap_or_default();
  let threads = args().nth(2).unwrap_or("1".to_string()).parse::<u8>().unwrap_or(1);
  let path = args().nth(3);

  let (tx, rx) = mpsc::channel();

  eprintln!("Using {} threads", threads);

  for _i in 0..threads {
    let pattern = pattern.clone();
    let tx = tx.clone();
    thread::spawn( move || {
      let mut csprng = OsRng{};
      let mut buffer = ByteBuffer::new();
      buffer.set_endian(BigEndian);
      buffer.write_u32(KEYTYPE.len() as u32);
      buffer.write_bytes(KEYTYPE);
      buffer.write_u32(PUBLIC_KEY_LENGTH as u32);

      loop {
        let keypair = Keypair::generate(&mut csprng);
        buffer.write_bytes(&keypair.public.to_bytes());
        let pk = buffer.to_bytes();
        let pk64 = encode(&pk);
        if pk64.contains(&pattern) {
          println!("ssh-ed25519 {}", pk64);
          let sk64 = get_sk(&pk, keypair);
          tx.send(format!("-----BEGIN OPENSSH PRIVATE KEY-----\n{}\n-----END OPENSSH PRIVATE KEY-----", sk64)).unwrap();
          break;
        }
        buffer.set_wpos(buffer.get_wpos() - PUBLIC_KEY_LENGTH);
      }
    });
  }

  let privkey = rx.recv().unwrap();
  if path.is_some() {
    let mut file = File::create(path.unwrap()).unwrap();
    file.write(format!("{}\n", privkey).as_bytes()).unwrap();
    file.set_permissions(Permissions::from_mode(0o600)).unwrap();
  } else {
    println!("{}", privkey);
  }
  
}
