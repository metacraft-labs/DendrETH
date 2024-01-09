<script>
    import { Button } from 'flowbite-svelte';
  </script>
  
  <div class="bg-black text-white p-24 flex flex-col content-center center">
    <div class="mb-4">
      <button class="text-sm border border-white py-1 px-3">What ?</button>
    </div>
    <div class="container mx-auto flex justify-center">
    
      <h1 class="text-5xl font-bold mb-6 leading-tight mr-8" style="color: #8FD3FE;">
       Proof Of Consensus
      </h1>
      <div>
        <p class="mb-6 max-w-2xl">
          DendrETH generates validity proofs of Consensus specifications using **ZKP circuits**. Proofs can be effectively verified in the targetted chains' execution environments. Using a chain of merkle proofs (or other commitment schemes), the implemented smart contracts can be used to authenticate any portion of the Ethereum consensus or execution layer states.
        </p>
        <button class=" border border-white py-2 px-4 hover:bg-white hover:text-black">Read in our blog</button>
      </div>
    </div>
  </div>
  
  <style>
    .center {
      padding: 30px 10% 100px 10%;
    }
  </style>
  