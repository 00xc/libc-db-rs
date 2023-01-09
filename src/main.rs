use std::collections::HashMap;
use std::fs;
use std::io::{self, ErrorKind};
use clap::{Parser, Subcommand};
use reqwest::blocking::{Client, RequestBuilder, Response};
use serde::{Serialize, Deserialize};

macro_rules! ioerr {
	($k:ident, $e:expr) => { io::Error::new(ErrorKind::$k, $e) };
}

#[derive(Parser, Debug)]
#[clap(name = "libc-db", author, version, about, long_about = None)]
struct Args {
	#[command(subcommand)]
	command: ApiCommand,
}

#[derive(clap::Args, Debug, Deserialize, Serialize)]
struct Libc {
	/// Lookup by md5 hash
	#[clap(long, value_parser)]
	#[serde(skip_serializing_if = "Option::is_none")]
	md5: Option<String>,

	/// Lookup by sha1 hash
	#[clap(long, value_parser)]
	#[serde(skip_serializing_if = "Option::is_none")]
	sha1: Option<String>,

	/// Lookup by sha256 hash
	#[clap(long, value_parser)]
	#[serde(skip_serializing_if = "Option::is_none")]
	sha256: Option<String>,

	/// Lookup by Build ID
	#[clap(long, value_parser)]
	#[serde(skip_serializing_if = "Option::is_none")]
	buildid: Option<String>,

	/// Lookup by libc ID
	#[clap(long, value_parser)]
	#[serde(skip_serializing_if = "Option::is_none")]
	id: Option<String>,

	/// Lookup by symbol addresses. Specify with a comma-separated list
	/// of colon-separated symbol-address pairs
	/// (e.g. 'strncpy: db0, system: 0x4f4e0')
	#[clap(short, long, value_parser = parse_symbol_map)]
	#[serde(skip_serializing_if = "Option::is_none")]
	symbols: Option<HashMap<String, String>>,

	/// Do not display results and instead download all libcs that
	/// match the query to the current working directory
	#[clap(short, long, value_parser, default_value_t = false)]
	#[serde(skip_deserializing, skip_serializing)]
	download: bool,

	#[clap(skip=String::new())]
	#[serde(skip_serializing)]
	#[allow(dead_code)]
	symbols_url: String,

	#[clap(skip=String::new())]
	#[serde(skip_serializing)]
	#[allow(dead_code)]
	libs_url: String,

	#[clap(skip=String::new())]
	#[serde(skip_serializing)]
	download_url: String,
}

fn parse_symbol_map(st: &str) -> io::Result<HashMap<String, String>> {
	st.split(',')
		.map(|field| field.split_once(':')
			.map(|(a, b)| (a.trim(), b.trim()))
			.filter(|(a, b)| !a.is_empty() && !b.is_empty())
			.map(|(a, b)| (a.to_owned(), b.to_owned()))
			.ok_or_else(|| ioerr!(InvalidData, field)))
		.collect::<Result<HashMap<_, _>, _>>()
}

#[derive(clap::Args, Debug, Serialize)]
struct DumpRequest {
	/// libc ID
	/// (e.g. 'libc6_2.27-3ubuntu1.2_amd64')
	#[clap(required(true))]
	#[serde(skip_serializing)]
	id: String,

	/// Comma-separated list of symbols to dump
	/// (e.g. 'strncat, sprintf')
	#[clap(short, long, value_parser = parse_symbol,
		value_delimiter = ',')]
	symbols: Vec<String>
}

fn parse_symbol(st: &str) -> io::Result<String> {
	let stt = st.trim();
	match stt.is_empty() {
		true => Err(ioerr!(InvalidData, "Empty symbol")),
		false => Ok(stt.to_owned())
	}
}

#[derive(Subcommand, Debug)]
enum ApiCommand {
	/// Look up one or more libcs by various attributes. Several attributes can be specified to form an AND filter.
	Find(Libc),
	/// Dump symbols for a given libc ID
	Dump(DumpRequest),
}

impl ApiCommand {
	fn gen_req(&self, client: &Client) -> RequestBuilder {
		match self {
			Self::Find(fields) => client
				.post("https://libc.rip/api/find")
				.json(&fields),
			Self::Dump(fields) => client
				.post(format!("https://libc.rip/api/libc/{}", fields.id))
				.json(&fields)
		}
	}

	fn display(&self, client: &Client, r: Response) -> reqwest::Result<()> {
		match self {
			Self::Find(f) => match f.download {
				true => Self::download(client, r.json::<Vec<Libc>>()?)?,
				false => println!("{}", r.text()?.trim()),
			},
			_ => println!("{}", r.text()?.trim()),
		};
		Ok(())
	}

	fn download(client: &Client, libcs: Vec<Libc>) -> reqwest::Result<()> {
		for r in libcs.into_iter() {
			let outfile = r.download_url.split('/').last().unwrap();
			println!("{} -> {}", r.download_url, outfile);
			let bytes = client.get(&r.download_url).send()?.bytes()?;
			fs::write(outfile, bytes).unwrap();
		}
		Ok(())
	}
}

fn main() -> reqwest::Result<()> {
	let args = Args::parse();

	let client = reqwest::blocking::Client::new();
	let resp = args.command
		.gen_req(&client)
		.send()?
		.error_for_status()?;

	args.command.display(&client, resp)
}
