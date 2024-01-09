<script>
	import Counter from './Counter.svelte';
	import { AdvancedRating, Button } from 'flowbite-svelte';
	import main from '$lib/images/main.png';
	import Explainer from './Explainer.svelte';
	import Advantages from './Advantages.svelte';
	import UseCases from './UseCases.svelte';
	import Protocol from './Protocol.svelte';
	import Partners from './Partners.svelte';
</script>

<svelte:head>
	<title>Home</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

	<section class="bg-gradient-to-b from-[#2E2C5F] to-[#7684C3] text-white flex justify-between items-center center">
		<div class="container mx-auto flex flex-col items-start justify-center h-full px-6 max-w-md">
		  <h1 class="text-start text-4xl font-bold mb-4 leading-tight">
			Scaling Trust <br> the Interchain Interoperability Layer
		  </h1>
		  <p class="mb-8 leading-relaxed">
			Remove your dependence on centralized infrastructure by leveraging a truly trust-minimized, censorship resistant, decentralized, and highly secure smart contract solution.
		  </p>
		  <div class="flex space-x-4">
			<Button href="#"  class="bg-white text-[#667eea] hover:bg-gray-200">
			  Documentation
			</Button>
			<Button href="#" class="bg-white text-[#667eea] hover:bg-gray-200">
			  Demo
			</Button>
		  </div>
		</div>
		<img src={main} alt="Descriptive Text" class="max-w-xs md:max-w-sm" />
	  </section>

	  <Explainer/>
	  <UseCases/>
	  <Protocol/>
	  <Advantages/>
	  <Partners/>
<style>
	 .center {
      padding: 30px 10% 100px 10%;
    }
</style>
