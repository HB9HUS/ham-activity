# ham-activity
This software is a service that collects data from the reverse beacon network
and creates some activity stats for different regions.

## Installation

### docker
pull the docker image. check here for latest version: https://hub.docker.com/r/hb9hus/ham-activity

### build from source
You need a local rust environement.

  git clone https://github.com/HB9HUS/ham-activity
  cd ham-activity
  cargo build --release

## Rest API
Currently the following endpoints are implemented:
* /stats: Statistics of the whole spot database
* /region/REGION: Regional statistics
* /regions: lists all known regions
* /frequency/FREQ-HZ: finds callsigns at that frequency (in Hz). Uses +/- 200Hz

## Simple UI
A simple UI for the region details can be called on http://localhost:8000/ui/?region=EU

## Logging
You can set log level by setting the environement variable RUST\_LOG. p.ex.:
  RUST_LOG=debug cargo run

## Regions
The file containing region to prefix mapping (./data/dxcc.json) is created from
this repository: https://github.com/k0swe/dxcc-json
