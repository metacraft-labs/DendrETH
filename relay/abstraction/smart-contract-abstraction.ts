export interface ISmartContract {
  optimisticHeaderRoot(): Promise<string>;

  postUpdateOnChain(update: {
    attested_header_root: string;
    finalized_header_root: string;
    finalized_execution_state_root: string;
    a: string[];
    b: string[][];
    c: string[];
  }): Promise<any>;
}
