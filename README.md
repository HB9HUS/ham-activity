# ham-activity
This software is a service that collects data from the reverse beacon network
and creates some activity stats for different regions.

## Rest API
Currently the following endpoints are implemented:
/stats: Statistics of the whole spot database
/region/REGION: Regional statistics

## Regions
The file containing region to prefix mapping (./data/dxcc.json) is created from
this repository: https://github.com/k0swe/dxcc-json
