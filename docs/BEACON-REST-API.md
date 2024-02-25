The Beacon API intended for light clients is pending specification here:
https://github.com/ethereum/beacon-APIs/pull/181

It's already fully implemented in the Nimbus client. We recommend building the
client [from source](https://nimbus.guide/build.html) and then launching a
Prater node with the following command:

```
./run-prater-node.sh --light-client-data-serve --light-client-data-import-mode=on-demand --light-client-data-max-periods=999999
```

Wait for the client to fully sync with the network. You'll see an indicator for this
is the status bar at the bottom of your screen - it may take few days and it will require
around 60GB of storage. You'll be then able to query the REST API on the default port (`5052`).

Here is an example query:

```
curl http://localhost:5052/eth/v0/beacon/light_client/bootstrap/0x8c59e1ea7215fa02e84ee141be0833ba6e1793281214f3ae4deff6ea019b1f13
```

This requests [block 3,200,000](https://prater.beaconcha.in/block/3200000), the first block in epoch 100,000.

You can also launch additional nodes by specifying a unique `NODE_ID` environment variable when launching the script above. For example, to launch a separate Mainnet node while your Prater node is still running, use the following command:

```
NODE_ID=1 ./run-mainnet-node.sh --light-client-data-serve --light-client-data-import-mode=on-demand --light-client-data-max-periods=999999
```

The `NODE_ID` value determines the data dir of the node in `nimbus-eth2/build/data` and the REST port being used (5052 + `NODE_ID`). In other words, the Mainnet node launched above will store its data in `build/data/shared_mainnet_1` and it will listen on port 5053. You'll be then able to request the snapshot matching the [first Altair block](https://beaconcha.in/block/2375680) with the following command:

```
curl http://localhost:5053/eth/v0/beacon/light_client/bootstrap/0x4df61a042151aa94fe5412063bdc7357e7a0266348745fc741ea669487ce6553
```

Similarly, the [first Altair block for Prater](https://prater.beaconcha.in/block/1173120) can still be obtained on port 5052 with the following command:

```
curl 'http://localhost:5052/eth/v0/beacon/light_client/bootstrap/0x24ad5b7d941e147b80edb0aa34c1c454e6e467e68e83142e51e9d21b9226eb79'
```
