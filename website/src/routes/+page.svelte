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
	<div class="container flex flex-col items-start h-full px-6 max-w-xl">
		<h1 class="text-start text-5xl font-bold mb-4 leading-tight">
			Main value proposition <br /> & <span style="color: #8FD3FE;">what DendrETH does</span>
		</h1>
		<p class="mb-12 leading-relaxed">
			Maximize your coding efficiency with our compiler-driven software solutions empowering you to
			write better code, achieve faster builds
		</p>
		<div class="flex space-x-4">
			<Button
				href="#"
				class="relative bg-white text-black rounded-bl-none rounded-tr-none text-xl hover:bg-gray-200"
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
				DendrETH Wallet Demo →
			</Button>
		</div>
	</div>
	<img src={main} alt="Descriptive Text" class="z-20 max-w-xs md:max-w-sm" style="opacity: 0" />
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
	/* Vector */

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
