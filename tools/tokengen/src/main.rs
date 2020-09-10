use clap::Clap;
use humantime::parse_duration;
use picky::{
    jose::{
        jwe::{Jwe, JweAlg, JweEnc},
        jws::JwsAlg,
        jwt::JwtSig,
    },
    key::{PrivateKey, PublicKey},
    pem::Pem,
};
use serde::Serialize;
use std::{error::Error, path::PathBuf, time::SystemTime};

#[derive(Clap)]
struct App {
    #[clap(long)]
    prx_usr: String,
    #[clap(long)]
    prx_pwd: String,
    #[clap(long)]
    dst_usr: String,
    #[clap(long)]
    dst_pwd: String,
    #[clap(long)]
    dst_hst: String,
    #[clap(long, default_value = "15m")]
    validity_duration: String,
    #[clap(long)]
    provider_private_key: PathBuf,
    #[clap(long)]
    jet_public_key: PathBuf,
}

#[derive(Serialize)]
struct RoutingClaims {
    exp: i64,
    nbf: i64,
    prx_usr: String,
    prx_pwd: String,
    dst_usr: String,
    dst_pwd: String,
    dst_hst: String,
    jet_cm: String,
    jet_ap: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let app = App::parse();

    let private_key_str = std::fs::read_to_string(&app.provider_private_key)?;
    let private_key_pem = private_key_str.parse::<Pem>()?;
    let private_key = PrivateKey::from_pem(&private_key_pem)?;

    let public_key_str = std::fs::read_to_string(&app.jet_public_key)?;
    let public_key_pem = public_key_str.parse::<Pem>()?;
    let public_key = PublicKey::from_pem(&public_key_pem)?;

    let validity_duration = parse_duration(&app.validity_duration)?;
    let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;

    let exp = (now + validity_duration).as_secs();

    let claims = RoutingClaims {
        exp: exp as i64,
        nbf: now.as_secs() as i64,
        prx_usr: app.prx_usr,
        prx_pwd: app.prx_pwd,
        dst_usr: app.dst_usr,
        dst_pwd: app.dst_pwd,
        dst_hst: app.dst_hst,
        jet_cm: "fwd".to_owned(),
        jet_ap: "rdp".to_owned(),
    };

    let signed = JwtSig::new(JwsAlg::RS256, claims).encode(&private_key)?;
    let encrypted = Jwe::new(JweAlg::RsaOaep256, JweEnc::Aes256Gcm, signed.into_bytes()).encode(&public_key)?;

    println!("{}", encrypted);

    Ok(())
}
