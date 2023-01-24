#!/bin/bash
set -euo pipefail

trap "echo The demo is terminated (SIGINT); exit 1" SIGINT
trap "echo The demo is terminated (SIGTERM); exit 1" SIGTERM

# Demo of private communities: ceremonies and balances are confidential
# An Encointer community is first created on chain and then migrated to private
#

# setup:
# run all on localhost:
#   encointer-node-notee purge-chain --dev
#   encointer-node-notee --dev --enable-offchain-indexing true --rpc-methods unsafe
#   encointer-node/client/bootstrap_demo_community.py --port <NODEPORT>
#   integritee-service --clean-reset run -p ${NPORT} (--skip-ra --dev)
#
# then run this script

# usage:
#   demo_private_community.sh -p <NODEPORT> -P <WORKERPORT> -u <NODE_URL> -V <WORKER_URL> -C <CLIENT_BINARY_PATH>

while getopts ":p:P:d:i:u:V:C:" opt; do
    case $opt in
        p)
            NPORT=$OPTARG
            ;;
        P)
            WORKER1PORT=$OPTARG
            ;;
        d)
            DURATION=$OPTARG
            ;;
        i)
            INTERVAL=$OPTARG
            ;;
        u)
            NODEURL=$OPTARG
            ;;
        V)
            WORKER1URL=$OPTARG
            ;;
        C)
            CLIENT_BIN=$OPTARG
            ;;
    esac
done

# using default port if none given as arguments
NPORT=${NPORT:-9944}
NODEURL=${NODEURL:-"ws://127.0.0.1"}

WORKER1PORT=${WORKER1PORT:-2000}
WORKER1URL=${WORKER1URL:-"wss://127.0.0.1"}

CLIENT_BIN=${CLIENT_BIN:-"./../bin/integritee-cli"}
COMMUNITY_IDENTIFIER="sqm1v79dF6b"

echo "Using client binary ${CLIENT_BIN}"
echo "Using node uri ${NODEURL}:${NPORT}"
echo "Using trusted-worker uri ${WORKER1URL}:${WORKER1PORT}"
echo "Using community ${COMMUNITY_IDENTIFIER}"

CLIENT="${CLIENT_BIN} -p ${NPORT} -P ${WORKER1PORT} -u ${NODEURL} -U ${WORKER1URL}"

echo "* Query on-chain enclave registry:"
${CLIENT} list-workers
echo ""

# this will always take the first MRENCLAVE found in the registry !!
read MRENCLAVE <<< $($CLIENT list-workers | awk '/  MRENCLAVE: / { print $2; exit }')
echo "Reading MRENCLAVE from worker list: ${MRENCLAVE}"

[[ -z $MRENCLAVE ]] && { echo "MRENCLAVE is empty. cannot continue" ; exit 1; }
echo ""

echo ""
echo "* Try to register //Alice, but community is not private! "
$CLIENT trusted --mrenclave ${MRENCLAVE} register-participant //Alice ${COMMUNITY_IDENTIFIER}
echo ""
echo "* Listing participants: There is no participants! "
$CLIENT trusted --mrenclave ${MRENCLAVE} list-participants //Alice ${COMMUNITY_IDENTIFIER}
echo ""

echo ""
echo "* Migrating community ${COMMUNITY_IDENTIFIER} to private"
$CLIENT trusted --mrenclave ${MRENCLAVE} make-community-private //Alice ${COMMUNITY_IDENTIFIER}
echo ""

echo ""
echo "* Registering 3 bootstrapper : "
echo "  //Alice,"
$CLIENT trusted --mrenclave ${MRENCLAVE} register-participant //Alice ${COMMUNITY_IDENTIFIER}
echo "  //Bob"
$CLIENT trusted --mrenclave ${MRENCLAVE} register-participant //Bob ${COMMUNITY_IDENTIFIER}
echo "  //Charlie"
$CLIENT trusted --mrenclave ${MRENCLAVE} register-participant //Charlie ${COMMUNITY_IDENTIFIER}
echo ""
echo "* Registering a newbie //Cora "
$CLIENT trusted --mrenclave ${MRENCLAVE} register-participant //Cora ${COMMUNITY_IDENTIFIER}
echo ""

echo ""
echo "* Listing participants "
$CLIENT trusted --mrenclave ${MRENCLAVE} list-participants //Alice ${COMMUNITY_IDENTIFIER}
echo ""

echo ""
echo "* Triggering manually next phase: Assigning"
$CLIENT next-phase //Alice
echo ""

echo "* Waiting 10 seconds"
sleep 10

echo ""
echo "* Listing Meetups"
$CLIENT trusted --mrenclave ${MRENCLAVE} list-meetups //Alice ${COMMUNITY_IDENTIFIER}
echo ""
