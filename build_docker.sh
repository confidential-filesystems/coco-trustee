#!/bin/bash

set -e
SERVICE_NAME=coco-trustee
VERSION=v0.8.0-filesystem-d8
HUB=hub.confidentialfilesystems.com:30443
SSH_KEY=${1:-$HOME/.ssh/id_rsa}

docker build --ssh default=${SSH_KEY} -f ./Dockerfile -t ${HUB}/cc/${SERVICE_NAME}:${VERSION} --build-arg TAG=v0.8.0-filesystem-snp .
docker push ${HUB}/cc/${SERVICE_NAME}:${VERSION}

echo "build time: $(date)"