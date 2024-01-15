<script>
	import { onMount, onDestroy } from 'svelte';
	import { page } from '$app/stores';
	import logo from '$lib/images/svelte-logo.svg';
	import github from '$lib/images/github.svg';
	import {
		Navbar,
		NavBrand,
		NavLi,
		NavUl,
		NavHamburger,
		ImagePlaceholder,
		Skeleton,
		TextPlaceholder,
		Button
	} from 'flowbite-svelte';

	$: activeUrl = $page.url.pathname;
	let activeClass =
		'text-white bg-green-700 md:bg-transparent md:text-green-700 md:dark:text-white dark:bg-green-600 md:dark:bg-transparent';
	let nonActiveClass =
		'text-gray-700 hover:bg-gray-100 md:hover:bg-transparent md:border-0 md:hover:text-green-700 dark:text-gray-400 md:dark:hover:text-white dark:hover:bg-gray-700 dark:hover:text-white md:dark:hover:bg-transparent';

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
		<!-- <Navbar navClass="px-2 sm:px-4 py-2.5 absolute w-full z-20 top-0 left-0 border-b" let:hidden let:toggle>
		  <NavBrand href="/">
			<img src="/images/flowbite-svelte-icon-logo.svg" class="mr-3 h-6 sm:h-9" alt="DendrETH Logo" />
			<span class="self-center whitespace-nowrap text-xl font-semibold dark:text-white">DendrETH</span>
		  </NavBrand>
		  <div class="flex md:order-2">
			<Button size="sm">Get started</Button>
			<NavHamburger on:click={toggle} />
		  </div>
		  <NavUl {hidden}>
			<NavLi href="/" active={true}>Home</NavLi>
			<NavLi href="/docs">Docs</NavLi>
			<NavLi href="/explorer">Explorer</NavLi>
			<NavLi href="/demo">Demo</NavLi>
			<NavLi href="/careers">Careers</NavLi>
		  </NavUl>
		</Navbar> -->

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
						<a on:click={closeMenu} href="#" class="block px-4 py-2 text-white hover:text-gray-300"
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
						href="#">DendrETH Wallet Demo</Button
					>
				</div>
			</div>
		</Navbar>

		<!-- <nav class="bg-[#color-code] p-4">
			<div class="container mx-auto flex justify-between items-center">
			  <span class="text-white text-2xl">DendrETH</span>
			  <div class="hidden md:flex space-x-4">
				<a href="#" class="text-white hover:text-gray-300">DendrETH</a>
				<a href="#" class="text-white hover:text-gray-300">Documents</a>
				<a href="#" class="text-white hover:text-gray-300">Live Bridges</a>
				<a href="#" class="text-white hover:text-gray-300">Career</a>
				<a href="#" class="text-white hover:text-gray-300">Blog</a>
			  </div>
			  <a href="#" class="btn border rounded-md text-white px-6 py-2 hover:bg-white hover:text-[#color-code]">DendrETH Wallet Demo</a>
			</div>
		  </nav>
		 -->
	</div>
</header>

<style>
</style>
