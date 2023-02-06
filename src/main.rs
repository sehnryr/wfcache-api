mod args;
mod common;
mod utils;

use crate::args::Args;
use crate::common::ls;
use clap::Parser;
use log::{error, info, trace};
use lotus_lib::{
    cache_pair::{CachePair, CachePairReader},
    package::{PackageCollection, PackageTrioType},
};

fn main() {
    env_logger::init();

    let args = Args::parse();
    trace!("Args: {:?}", args);

    if !args.ls && args.extract.is_none() {
        error!("You must specify either --ls or --extract");
        std::process::exit(1);
    }

    trace!("Initializing PackageCollection");
    let collection = PackageCollection::<CachePairReader>::new(args.directory, true);
    trace!("PackageCollection initialized");
    info!(
        "Loaded {} packages: {:?}",
        collection.packages().len(),
        collection.packages().keys().collect::<Vec<_>>()
    );

    trace!("Getting package: {}", args.package);
    let package = collection.get_package(args.package.as_str());
    if package.is_none() {
        error!("Package not found: {}", args.package);
        std::process::exit(1);
    }
    let package = package.unwrap();
    trace!("Package found: {}", args.package);

    trace!("Getting header: H.{}.toc", args.package);
    let header = package.get(&PackageTrioType::H);
    if header.is_none() {
        error!("Package has no header: {}", args.package);
        std::process::exit(1);
    }
    let header = header.unwrap();
    trace!("Header found: H.{}.toc", args.package);

    trace!("Loading header");
    header.read_toc();
    trace!("Header loaded");

    let files = header.files();
    let dirs = header.dirs();

    info!("Loaded {} files, {} directories", files.len(), dirs.len());

    if args.ls {
        ls(header, args.lotus_path.as_str());
    } else if args.extract.is_some() {
        todo!("Extracting is not implemented yet");
    }
}
