export interface BalanceVerifier {
  configuration: unknown;
  prepareTasks: (...args: any[]) => unknown;
}
