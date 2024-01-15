<script>
	import Counter from './Counter.svelte';
	import { AdvancedRating, Button } from 'flowbite-svelte';

	import Explainer from './Explainer.svelte';
	import Advantages from './Advantages.svelte';
	import UseCases from './UseCases.svelte';
	import Protocol from './Protocol.svelte';
	import Partners from './Partners.svelte';
	import Blog from './Blog.svelte';

	import main from '$lib/images/main.png';
	import vector1 from '$lib/images/vector1.svg';
	import vector2 from '$lib/images/vector2.svg';
</script>

<svelte:head>
	<title>Dendreth</title>
	<meta name="description" content="Svelte demo app" />
</svelte:head>

<section
	class="bg-gradient-to-b from-[#1A1739] via-[#363571] to-[#A8C5EB] text-white flex justify-between items-center center"
>
	<div class="container flex flex-col md:items-start items-center h-full md:px-6 md:max-w-3xl">
		<h1 class="text-start md:text-5xl font-bold mb-4 leading-tight">
			Scaling Trust,<br> <span class="spanh1" style="color: #8FD3FE;">the Interchain Interoperability Layer</span>
		</h1>
		<p class="mb-12 leading-relaxed">
			Remove your dependence on centralized infrastructure by leveraging a truly 
			<span style="font-family: 'Inter-Bold'">trust-minimized, censorship resistant, decentralized,</span> and highly secure smart contract solution.
		</p>
		<img src={main} alt="Descriptive Text" class="max-w-xs md:max-w-sm pb-10 md:hidden block" style="opacity: 50; z-inde: 1;" />	
		<div class="flex md:flex-row flex-col space-x-4 md:w-auto w-full">
			<Button
				href="#"
				class="relative bg-white text-black rounded-bl-none mb-10 md:mb-0 rounded-tr-none text-xl hover:bg-gray-200"
			>
				Documentation →
				<span
					class="absolute rounded-br-lg rounded-tl-lg inset-0 ring-1 ring-white hover:ring-gray-200"
					style="transform: translate(2px, 2px);"
				/>
			</Button>
			<Button
				href="#"
				class="bg-transparent text-white text-xl hover:text-grey-300 hover:bg-transparent"
			>
				Wallet Demo →
			</Button>
		</div>
	</div>
	<img src={main} alt="Descriptive Text" class="max-w-xs md:max-w-sm md:block hidden" style="opacity: 50; z-index: 1;" />
	<img src={vector1} alt="Descriptive Text" class="vector1" />
	<img src={vector2} alt="Descriptive Text" class="vector2" />
</section>

<Explainer />
<UseCases />
<Protocol />
<Advantages />
<Partners />
<Blog />

<style>
	.center {
		padding: 30px 10% 100px 10%;
	}
	@media (max-width: 786px) {
		.center {
			padding: 30px 10% 30px 10%;
		}
	}

	h1 {
		font-family: 'ChakraPetch';
		font-size: 4rem;
		line-height: 120%;
		letter-spacing: -0.60px;
	}
	@media (max-width: 768px) {
		h1 {
			font-size: 2.5rem;
		}
	}

	.spanh1 {
		font-family: 'BonaNova-Bold';
	}

	.vector1 {
		opacity: 0.2;
		position: absolute;
		right: 0;
		top: 320px;
		bottom: 50%;
		height: auto;
		width: 700px;
	}

	.vector2 {
		opacity: 0.5;
		position: absolute;
		right: 0;
		top: 70px;
		height: auto;
		width: 550px;
	}
</style>
