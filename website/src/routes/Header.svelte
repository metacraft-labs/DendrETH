<script>
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import {
		Navbar,
		Button
	} from 'flowbite-svelte';

	let isOpen = false;

	function toggleMenu() {
		isOpen = !isOpen;
	}

	function handleOutsideClick(event) {
		const dropdownMenu = document.querySelector('.dropdown-menu');

		if (isOpen && dropdownMenu && !dropdownMenu.contains(event.target)) {
			closeMenu();
		}
	}

	onMount(() => {
		window.addEventListener('click', handleOutsideClick);
	});

	function closeMenu() {
		isOpen = false;
	}
</script>

<header>
	<div>
		<Navbar class="z-20 bg-gradient-to-b from-[#0B061C] to-[#1A1739] p-4" style="position: fixed;">
			<div><a href="/" class="text-white text-2xl">DendrETH</a></div>
			<div class="relative dropdown-menu">
				<button on:click={toggleMenu} class="p-2 focus:outline-none md:hidden">
					<div class="w-6 h-0.5 bg-white" />
					<div class="w-6 h-0.5 mt-1 bg-white" />
					<div class="w-6 h-0.5 mt-1 bg-white" />
				</button>

				{#if isOpen}
					<div
						class="absolute border border-white top-0 right-10 mt-2 p-2 bg-[#1A1739] shadow-md rounded-lg w-auto"
					>
						<a on:click={closeMenu} href="#" class="block px-4 py-2 text-white hover:text-gray-300"
							>Docs</a
						>
						<a
							on:click={closeMenu}
							href="/explorer"
							class="block px-4 py-2 text-white hover:text-gray-300">Explorer</a
						>
						<a on:click={closeMenu} href="#" class="block px-4 py-2 text-white hover:text-gray-300"
							>Career</a
						>
						<a on:click={closeMenu} href="#" class="block px-4 py-2 text-white hover:text-gray-300"
							>Blog</a
						>
						<a on:click={closeMenu} href="/demo" class="block px-4 py-2 text-white hover:text-gray-300"
							>Demo</a
						>
					</div>
				{/if}
			</div>
			<div class="md:flex space-x-12 hidden" style="align-items: center;">
				<a href="#" class="text-white hover:text-gray-300">Docs</a>
				<a href="/explorer" class="text-white hover:text-gray-300">Explorer</a>
				<a href="#" class="text-white hover:text-gray-300">Career</a>
				<a href="#" class="text-white hover:text-gray-300">Blog</a>
				<div>
					<Button
						class="bg-transparent rounded-bl-none rounded-tr-none border border-white hover:bg-white hover:text-black"
						href="/demo">DendrETH Wallet Demo</Button
					>
				</div>
			</div>
		</Navbar>
	</div>
</header>

<style>
</style>
