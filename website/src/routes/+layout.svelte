<script>
	import '../app.postcss';
	import Header from './Header.svelte';
	import './styles.css';
</script>

<div class="app">
	<Header />

	<main class="p-0">
		<slot />
	</main>

	<footer class="text-white my">
		<div class="flex flex-col md:flex-row justify-between items-center space-y-4 md:space-y-0 max-w-6xl mx-auto px-4">
		  <span class="text-sm">
			Copyright © 2019-2023 Metacraft Lab's Ltd
		  </span>
		  <div class="flex gap-4">
			<!-- <a href="/cookie-policy" class="text-sm hover:text-gray-400">Cookie manager</a> -->
			<!-- <a href="/terms" class="text-sm hover:text-gray-400">Terms of use</a> -->
			<!-- <a href="/privacy" class="text-sm hover:text-gray-400">Privacy policy</a> -->
			<!-- <a href="/security" class="text-sm hover:text-gray-400">Security</a> -->
			<!-- <a href="/trademark" class="text-sm hover:text-gray-400">Trademark policy</a> -->
		  </div>
		</div>
	  </footer>
</div>

<style>
	.app {
		display: flex;
		flex-direction: column;
		min-height: 100vh;
	}

	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		width: 100%;
		/* max-width: 64rem; */
		margin: 0 auto;
		box-sizing: border-box;
	}

	@media (min-width: 480px) {
		footer {
			padding: 12px 0;
		}
	}
</style>
