#!/bin/bash

set -e
SERVICE_NAME=coco-trustee
HUB=hub.confidentialfilesystems.com:30443
VERSION=${1:-v0.8.0-filesystem-d9}
SSH_KEY=${2:-$HOME/.ssh/id_rsa}

docker build --ssh default=${SSH_KEY} -f ./coco-trustee.dockerfile -t ${HUB}/cc/${SERVICE_NAME}:${VERSION} .
docker push ${HUB}/cc/${SERVICE_NAME}:${VERSION}

echo "build time: $(date)"