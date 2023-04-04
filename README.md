# Encointer Community Sidechain
This repository is a detached fork of the [integritee-worker](https://github.com/integritee-network/worker).

Encointer community-sidechain for Encointer [node](https://github.com/encointer/encointer-node) or [parachain](https://github.com/encointer/encointer-parachain)

This is part of [Encointer](https://github.com/encointer)

The Encointer community-sidechain is a service that provides privacy to Encointer. Privacy-sensitive pallets (encointer_balances and encointer_ceremonies pallets) are executed on the sidechain, inside a trusted execution environment. 

##Private Community Demo
Meetups of a private community are held confidentially, they are performed on the sidechain. The location and participants of a meetup are not publicly leaked. Balances and transactions are also on the sidechain, so users' balances and consumption profiles remain confidential.

The demo can be run in the docker. See [docker/README.md](docker/README.md)

The demo ```demo-private-community.sh``` shows the execution of a private community ceremony. Alice (sudo) is the community manager.

- A private community (on the sidechain), with id : sqm1v79dF6b, is created from a public community (on the encointer node).
- We are in the registration phase. 3 bootstrappers an 1 newbie register for the next meetup on the sidechain with a trusted call. 
  Bob trusted call is 
  
  ```encointer-client trusted register-participant //Bob --mrenclave Jtpuqp6iA98JmhUYwhbcV8mvEgF9uFbksWaAeyALZQA sqm1v79dF6b```
- The community manager Alice checks the list of participants. Only she has the right to do so. There are 3 registered bootstrappers and 1 newbie.
- The next phase ```Assigning``` is triggered manually. The participants are automatically assigned to a meetup.
- The community manager Alice checks the meetups. Only she has the right to do so. There is only one meeting with the 4 participants at a particular location and time.
- The Meetup can be performed. The next phase ```Attesting``` is triggered manually: 
  
  All participants attest and claim the presence of the other participants with a trusted call.
  Ex: Bobs call is :
  
  ```encointer-client trusted --mrenclave Jtpuqp6iA98JmhUYwhbcV8mvEgF9uFbksWaAeyALZQA attest-attendees //Bob  sqm1v79dF6b //Alice //Charlie //Cora```
- The community manager Alice checks the attestees. Only she has the right to do so. All 4 particpants were attested by the others.
- Bob claims the early rewards for all participants in his meetup with a trusted call. 
  Ex: Bob claims the early reward with : 
  
  ```encointer-client trusted --mrenclave Jtpuqp6iA98JmhUYwhbcV8mvEgF9uFbksWaAeyALZQA claim-rewards //Bob sqm1v79dF6b```
- The community manager Alice checks the community infos.
- Participants got their community incomes: 
  
  Ex: Bob's' balance in Community currency increases by the reward Community income. But his balance in native currency is still 0.
- After some time, Bob's balance decreases because of demurrage.  
 


## Handle Upstream Updates
There is an `upstream/master` branch, which is used to track the upstream repository.

1.  Add upstream remote.
```bash
git remote add upstream git@github.com:integritee-network/worker.git
```

2. Sync `upstream/master` branch
```bash
git checkout upstream/master
git pull upstream master
git push
```
3. Create PR to master

Alternatively, we can also track the release branches of `integritee-worker` - to be discussed.


## Build and Run
Please see our [Integritee Book](https://docs.integritee.network/4-development/4.4-sdk) to learn how to build and run this.

To start multiple worker and a node with one simple command: Check out [this README](local-setup/README.md).

## Docker
See [docker/README.md](docker/README.md).

## Tests

There are 3 types of tests:
- cargo tests
- enclave tests
- integration tests

### Cargo Tests
Run
```
cargo test
```

### Enclave Tests
Run

```
make
./bin/integritee-service test --all
```

### Integration Tests
See [docker/README.md](docker/README.md)

## Direct calls scalability

For direct calls, a worker runs a web-socket server inside the enclave. An important factor for scalability is the transaction throughput of a single worker instance, which is in part defined by the maximum number of concurrent socket connections possible. On Linux by default, a process can have a maximum of `1024` concurrent file descriptors (show by `ulimit -n`).
If the web-socket server hits that limit, incoming connections will be declined until one of the established connections is closed. Permanently changing the `ulimit -n` value can be done in the `/etc/security/limits.conf` configuration file. See [this](https://linuxhint.com/permanently_set_ulimit_value/) guide for more information.
