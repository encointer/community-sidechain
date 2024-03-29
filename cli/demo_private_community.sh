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

while getopts ":m:p:A:B:u:W:V:C:" opt; do
    case $opt in
        m)
            READMRENCLAVE=$OPTARG
            ;;
        p)
            NPORT=$OPTARG
            ;;
        A)
            WORKER1PORT=$OPTARG
            ;;
        B)
            WORKER2PORT=$OPTARG
            ;;
        u)
            NODEURL=$OPTARG
            ;;
        V)
            WORKER1URL=$OPTARG
            ;;
        W)
            WORKER2URL=$OPTARG
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

WORKER2PORT=${WORKER2PORT:-3000}
WORKER2URL=${WORKER2URL:-"wss://127.0.0.1"}

CLIENT_BIN=${CLIENT_BIN:-"./../bin/integritee-cli"}
COMMUNITY_IDENTIFIER="sqm1v79dF6b"

echo "Using client binary ${CLIENT_BIN}"
echo "Using node uri ${NODEURL}:${NPORT}"
echo "Using trusted-worker uri ${WORKER1URL}:${WORKER1PORT}"
echo "Using trusted-worker-2 uri ${WORKER2URL}:${WORKER2PORT}"
echo "Using community ${COMMUNITY_IDENTIFIER}"

CLIENTWORKER1="${CLIENT_BIN} -p ${NPORT} -P ${WORKER1PORT} -u ${NODEURL} -U ${WORKER1URL}"
CLIENTWORKER2="${CLIENT_BIN} -p ${NPORT} -P ${WORKER2PORT} -u ${NODEURL} -U ${WORKER2URL}"

echo "* Query on-chain enclave registry:"
${CLIENTWORKER1} list-workers
echo ""

# this will always take the first MRENCLAVE found in the registry !!
read MRENCLAVE <<< $($CLIENTWORKER1 list-workers | awk '/  MRENCLAVE: / { print $2; exit }')
echo "Reading MRENCLAVE from worker list: ${MRENCLAVE}"

[[ -z $MRENCLAVE ]] && { echo "MRENCLAVE is empty. cannot continue" ; exit 1; }
echo ""

echo ""
echo "* Migrating public community ${COMMUNITY_IDENTIFIER} to private"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} make-community-private //Alice ${COMMUNITY_IDENTIFIER}
echo ""

echo "* Waiting 10 seconds"
sleep 10

echo "* Registering participants : "
echo "- Registering 3 bootstrappers : "
echo "  //Alice,"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} register-participant //Alice ${COMMUNITY_IDENTIFIER}
echo "  //Bob"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} register-participant //Bob ${COMMUNITY_IDENTIFIER}
echo "  //Charlie"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} register-participant //Charlie ${COMMUNITY_IDENTIFIER}
echo "- Registering a newbie //Cora "
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} register-participant //Cora ${COMMUNITY_IDENTIFIER}
echo ""

echo "* Listing participants "
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} list-participants //Alice ${COMMUNITY_IDENTIFIER}
echo ""

echo "* Triggering manually next phase: Assigning"
$CLIENTWORKER1 next-phase //Alice
echo ""

echo "* Waiting 10 seconds"
sleep 10

echo ""
echo "* Listing Meetups"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} list-meetups //Alice ${COMMUNITY_IDENTIFIER}
echo ""

echo "* Performing Meetups : "
echo ""

echo "* Triggering manually next phase: Attesting"
$CLIENTWORKER1 next-phase //Alice
echo ""

echo " //Alice attests the other attendees "
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} attest-attendees //Alice ${COMMUNITY_IDENTIFIER} //Bob //Charlie //Cora
echo ""

echo " //Bob attests the other attendees"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} attest-attendees //Bob ${COMMUNITY_IDENTIFIER} //Alice //Charlie //Cora
echo ""

echo " //Charlie attests the other attendees"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} attest-attendees //Charlie ${COMMUNITY_IDENTIFIER} //Alice //Bob //Cora
echo ""

echo " //Cora attests the other attendees"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} attest-attendees //Cora ${COMMUNITY_IDENTIFIER} //Alice //Charlie //Bob
echo ""

echo "* Waiting enough time, such that xt's are processed... 30 seconds"
sleep 30
echo ""

echo "* Listing Attestees"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} list-attestees //Alice ${COMMUNITY_IDENTIFIER}
echo ""

# Debug Bob initial balances
#echo "Get balance (native currency) of Bob's incognito account"
INIT_BOB_NATIVE=$(${CLIENTWORKER1} trusted --mrenclave ${MRENCLAVE} balance //Bob | xargs)
#echo $INIT_BOB_NATIVE
#echo ""
#echo "Get balance (community currency) of Bob's account"
INIT_BOB_COMMUNITY_CURRENCY=$(${CLIENTWORKER1} trusted --mrenclave ${MRENCLAVE} balance //Bob ${COMMUNITY_IDENTIFIER} | xargs)
#echo $INIT_BOB_COMMUNITY_CURRENCY
#echo ""

echo "* Claiming early rewards for all participants in Bob's meetup"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} claim-rewards //Bob ${COMMUNITY_IDENTIFIER}
echo ""

echo "* Waiting enough time, such that xt's are processed... 30 seconds"
sleep 30
echo ""

echo "* Info :  "
echo ""
echo "Community infos :"
$CLIENTWORKER1 trusted --mrenclave ${MRENCLAVE} community-infos //Alice ${COMMUNITY_IDENTIFIER}
echo ""

echo "Bob's balances:"
echo "Initial balances"
echo "  in native currency: $INIT_BOB_NATIVE"
echo "  in community currency: $INIT_BOB_COMMUNITY_CURRENCY"
echo "After rewards"
REWARDED_BOB_NATIVE=$(${CLIENTWORKER1} trusted --mrenclave ${MRENCLAVE} balance //Bob | xargs)
echo "  in native currency: $REWARDED_BOB_NATIVE"
REWARDED_BOB_COMMUNITY_CURRENCY=$(${CLIENTWORKER1} trusted --mrenclave ${MRENCLAVE} balance //Bob ${COMMUNITY_IDENTIFIER} | xargs)
echo "  in community currency: $REWARDED_BOB_COMMUNITY_CURRENCY"
echo ""

sleep 30
echo "After some time (demurrage)"
DEMURRAGE_BOB_NATIVE=$(${CLIENTWORKER1} trusted --mrenclave ${MRENCLAVE} balance //Bob | xargs)
echo "  in native currency: $DEMURRAGE_BOB_NATIVE"
DEMURRAGE_BOB_COMMUNITY_CURRENCY=$(${CLIENTWORKER1} trusted --mrenclave ${MRENCLAVE} balance //Bob ${COMMUNITY_IDENTIFIER} | xargs)
echo "  in community currency: $DEMURRAGE_BOB_COMMUNITY_CURRENCY"
echo ""

# The following tests are for automated CI.
echo "* Verifying Bob's balance in community currency"
echo ""
echo "1) Reward: "
if [ bc <<< "$REWARDED_BOB_COMMUNITY_CURRENCY > $INIT_BOB_COMMUNITY_CURRENCY" ]; then
  echo "Bob's balance in community community has increased ($REWARDED_BOB_COMMUNITY_CURRENCY)."
else
  echo "test failed: Bob has not received the rewards. His balance in community currency should be greater than $INIT_BOB_COMMUNITY_CURRENCY"
  exit 1
fi

echo "2) Demurrage:"
if [ bc <<< "$REWARDED_BOB_COMMUNITY_CURRENCY > $DEMURRAGE_BOB_COMMUNITY_CURRENCY" ]; then
  echo "Bob's balance in community currency got devalued ($DEMURRAGE_BOB_COMMUNITY_CURRENCY)"
else
  echo "test failed: It seems that no demurrage was applied on Bob's balances in community currency: $REWARDED_BOB_COMMUNITY_CURRENCY should be greater than $DEMURRAGE_BOB_COMMUNITY_CURRENCY"
  exit 1
fi
echo ""

echo "* Verifying Bob's balance in native currency :"
echo "1) No reward:"
if [ bc <<< "$REWARDED_BOB_NATIVE = $INIT_BOB_NATIVE" ]; then
  echo "Bob's balance in native currency has not changed ($REWARDED_BOB_NATIVE)"
else
    echo "test failed: Bob's balance in native currency has changed: $REWARDED_BOB_NATIVE should be $INIT_BOB_NATIVE"
    exit 1
fi

echo "2) No demurrage:"
if [ bc <<< "$DEMURRAGE_BOB_NATIVE = $INIT_BOB_NATIVE" ]; then
  echo "Bob's balance in native currency has not changed ($DEMURRAGE_BOB_NATIVE)"
else
    echo "test failed: Bob's balance in native currency has changed: $REWARDED_BOB_NATIVE should be $INIT_BOB_NATIVE"
    exit 1
fi
echo "Test passed"
echo ""

exit 0
