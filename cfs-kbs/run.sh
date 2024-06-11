#!/bin/bash

CurrDir=$(cd "$(dirname "$0")"; pwd)

#
echo "" && echo "" && echo ""
mkdir -p /opt/confidential-containers/kbs/repository

export AA_EMULATE_ATTESTER=yes
export CFS_EMULATED_MODE=true

export LD_LIBRARY_PATH=/cfs-kbs/lib
/cfs-kbs/kbs --config-file /cfs-kbs/kbs-config-docker.toml

#
echo "" && echo "" && echo ""
echo "run kbs error -> sleep 36000 ..."
sleep 36000

#
exit 0
#end.
