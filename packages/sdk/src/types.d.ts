// Type stubs for packages that ship no @types declarations.
declare module "circomlibjs" {
  export interface PoseidonFn {
    (inputs: bigint[]): Uint8Array;
    F: {
      toObject(buf: Uint8Array): bigint;
    };
  }
  export function buildPoseidon(): Promise<PoseidonFn>;
  export function buildBabyjub(): Promise<unknown>;
}

declare module "snarkjs" {
  export interface Groth16Proof {
    pi_a: string[];
    pi_b: string[][];
    pi_c: string[];
    protocol: string;
    curve: string;
  }
  export const groth16: {
    fullProve(
      input: Record<string, string>,
      wasmFile: string,
      zkeyFile: string
    ): Promise<{ proof: Groth16Proof; publicSignals: string[] }>;
    verify(
      vkey: unknown,
      publicSignals: string[],
      proof: Groth16Proof
    ): Promise<boolean>;
  };
}
