The Beacon API inteded for light clients is pending specification here:
https://github.com/ethereum/beacon-APIs/pull/181

It's already fully implemented in the Nimbus client. We recommend building the
client [from source](https://nimbus.guide/build.html) and then launching a
Prater node with the following command:

```
./run-prater-node.sh --light-client-data-serve --light-client-data-import-mode=on-demand
```

Wait for the client to fully sync with the network. You'll see an indicator for this
is the status bar at the bottom of your screen - it may take few days and it will require
around 60GB of storage. You'll be then able to query the REST API on the default port (`5052`).

Here is an example query:

```
curl -H http://localhost:5053/eth/v0/beacon/light_client/bootstrap/0x8c59e1ea7215fa02e84ee141be0833ba6e1793281214f3ae4deff6ea019b1f13
```

This requests [block 3,200,000](https://prater.beaconcha.in/block/3200000), the first block in epoch 100,000.
