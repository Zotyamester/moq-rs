use std::{net, path};
use url::Url;

use clap::Parser;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser, Clone)]
pub struct Config {
	/// Listen on this address
	#[arg(long, default_value = "[::]:4443")]
	pub listen: net::SocketAddr,

	/// Use the certificate file at this path
	#[arg(long)]
	pub cert: path::PathBuf,

	/// Use the private key at this path
	#[arg(long)]
	pub key: path::PathBuf,

	/// Listen on HTTPS and serve /fingerprint, for self-signed certificates
	#[arg(long, action)]
	pub fingerprint: bool,

	/// Optional: Use the moq-api via HTTP to store origin information.
	#[arg(long)]
	pub api: Option<Url>,

	/// Our internal address which we advertise to other origins.
	/// We use QUIC, so the certificate must be valid for this address.
	/// This needs to be prefixed with https:// to use WebTransport
	/// This is only used when --api is set.
	#[arg(long)]
	pub node: Option<Url>,
}
