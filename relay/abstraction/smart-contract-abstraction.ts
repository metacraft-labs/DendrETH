export interface ISmartContract {
  optimisticHeaderRoot(): Promise<string>;

  postUpdateOnChain(update: {
    attestedHeaderRoot: string;
    attestedHeaderSlot: number;
    finalizedHeaderRoot: string;
    finalizedExecutionStateRoot: string;
    a: string[];
    b: string[][];
    c: string[];
  }): Promise<any>;
}
