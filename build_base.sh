#!/bin/bash

set -e
BASE_BUILDER_NAME=coco-trustee-builder
BASE_IMAGE_NAME=coco-trustee-base
HUB=confidentialfilesystems
VERSION=${1:-v1.0.1}

docker build -f ./coco-trustee-builder.dockerfile -t ${HUB}/${BASE_BUILDER_NAME}:${VERSION} .

echo "build time: $(date)"
