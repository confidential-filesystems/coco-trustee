#!/bin/bash

set -e
BASE_BUILDER_NAME=coco-trustee-builder
BASE_IMAGE_NAME=coco-trustee-base
VERSION=v1.0.0-amd64
HUB=confidentialfilesystems
SSH_KEY=${1:-$HOME/.ssh/id_rsa}

docker build -f ./base.builder.Dockerfile -t ${HUB}/cc/${BASE_BUILDER_NAME}:${VERSION} .
docker build --ssh default=${SSH_KEY} -f ./base.Dockerfile -t ${HUB}/cc/${BASE_IMAGE_NAME}:${VERSION} .

echo "build time: $(date)"
