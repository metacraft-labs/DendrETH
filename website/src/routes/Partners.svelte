<script>
    import { Button } from 'flowbite-svelte';
    import { GlobeOutline, ShieldCheckOutline, UsersOutline, DollarOutline } from 'flowbite-svelte-icons';

	import GnosisChain from '$lib/images/gnosis-chain.svg';
</script>
  
<div class="bg-[#121316] text-white p-24 flex flex-col content-center center items-center">
    <div class="mb-10">
      <button class="text-xs border border-[#8FD3FE] rounded-2xl py-1 px-3">Supported networks and partners</button>
    </div>
    <div class="grid lg:grid-cols-4 md:grid-cols-2 sm:grid-cols-1 flex w-full">
        <div class="flex flex-col items-center justify-center mb-8">
            <img src="{GnosisChain}" class="mb-4" alt="GnosisChainLogo" size="xl">
        </div>
        <div class="flex flex-col items-center justify-center mb-8">
            <img src="{GnosisChain}" class="mb-4" alt="GnosisChainLogo" size="xl">
        </div>
        <div class="flex flex-col items-center justify-center mb-8">
            <img src="{GnosisChain}" class="mb-4" alt="GnosisChainLogo" size="xl">
        </div>
        <div class="flex flex-col items-center justify-center mb-8">
            <img src="{GnosisChain}" class="mb-4" alt="GnosisChainLogo" size="xl">
        </div>
        <div class="flex flex-col items-center justify-center mb-8">
            <img src="{GnosisChain}" class="mb-4" alt="GnosisChainLogo" size="xl">
        </div>
        <div class="flex flex-col items-center justify-center mb-8">
            <img src="{GnosisChain}" class="mb-4" alt="GnosisChainLogo" size="xl">
        </div>
        <div class="flex flex-col items-center justify-center mb-8">
            <img src="{GnosisChain}" class="mb-4" alt="GnosisChainLogo" size="xl">
        </div>
        <div class="flex flex-col items-center justify-center mb-8">
            <img src="{GnosisChain}" class="mb-4" alt="GnosisChainLogo" size="xl">
        </div>
    </div>
    <div class="max-w-md text-center text-gray-500">
        <p>
            Partners worldwide that trust DendrETH to keep their data secure and safe.
        </p>
    </div>
</div>
 
  <style>
    .center {
      padding: 30px 10% 100px 10%;
    }
  </style>
  