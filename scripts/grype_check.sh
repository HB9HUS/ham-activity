#!/bin/sh
# This script is used to check the vulnerabilities of all packages

grype --add-cpes-if-none . > grype_check.txt

cat grype_check.txt

# get max level from arg if present
max_level=$1
if [ -z "$1" ]; then
    max_level="Low"
fi

for severity in Critical High Medium Low; do
    if grep -q "$severity" grype_check.txt; then
        echo "There are $severity vulnerabilities"
        exit 1
    fi
    if [ "$severity" = "$max_level" ]; then
        break
    fi
done
rm grype_check.txt
echo "There are no vulnerabilities with level of at least $max_level"

