To run tests

```
npx hardhat test
```

which will run test on the circuits. Have in mind that `aggregate-bitmask.test.ts` will take a few minutes.

In scripts folder you have the circuits with main components. And you can build them with the scripts. As well get example input from the JS scripts. For bigger circuits you may need up to hundreds of GB of RAM.


Also you have previous versions of this current branch in sync_implementation. Where most of the circuits are implemented in circom-pairing fork in sync_protcol branch.
Circuits are executed with commands from the scripts and data is feeded from JS scripts there.
For bigger circuits you may need up to hundreds of GB of RAM.

Diagrams

![](light_client.drawio.png)
![](zero_knowledge_diagram.drawio.png)
