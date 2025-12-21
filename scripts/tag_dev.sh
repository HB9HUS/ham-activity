#!/bin/bash

prefix="dev"
if [ -n "$1" ]; then
  prefix=$1
fi

# gen dev version
dt=$(date "+%Y%m%d")
version=$prefix-$dt-$(git rev-parse --short HEAD)

git tag $version
git push
git push --tags

