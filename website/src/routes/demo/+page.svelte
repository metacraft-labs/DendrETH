<script>
	import { Input, Button } from 'flowbite-svelte';

	import ethImg from '$lib/images/crypto/eth-img.svg';
	import btcImg from '$lib/images/crypto/btc-img.svg';
	import bnbImg from '$lib/images/crypto/bnb-img.svg';
	import polkadotImg from '$lib/images/crypto/polkadot-img.svg';

	let ethBalance = '0.6 ETH';
	let btcBalance = '0.03 BTC';
	let bnbBalance = '3 BNB';
	let dotBalance = '43 DOT';
	let walletBalance = '1455.54';

	let walletValue;

	function demoSync() {
		console.log('Demo sync initiated');
	}

	let activeButton = 'tokens';

	function toggleButton() {
		activeButton = activeButton === 'tokens' ? 'nfts' : 'tokens';
	}

	function handleEnter(event) {
		if (event.key === 'Enter') {
			console.log('SEARCHING YOUR WALLET: ', walletValue);
		}
	}
</script>

<section
	class="flex flex-col lg:flex-row bg-[#121316] text-white p-24 center space-y-10 items-center justify-between"
	style="border-bottom: 1px solid #6F6F6F;"
>
	<div class="flex flex-col content-center mt-16">
		<div class="mb-4">
			<button class="text-xs border border-[#8FD3FE] rounded-2xl py-1 px-3 hover:cursor-default">Product Demo</button>
		</div>
		<div class="container flex flex-col justify-between">
			<h1 class="text-5xl mb-6 leading-tight mr-8 max-w-lg text-left text-[#8FD3FE]">
				Ethereum <span class="text-white">one-shot sync capabilities</span>
			</h1>
			<p class="max-w-xl">
				Explain what will happen and what the user should expect in few simple steps. Two rows of
				copy.
			</p>
			<div class="flex flex-row">
				<div class="flex flex-col items-start pt-10 h-full pb-2">
					<div class="flex gap-4">
						<button class="bg-[#393939] cursor-default text-xl rounded-full text-white px-6 py-2"
							>01</button
						>
						<div class="flex flex-col">
							<h2>Step #1</h2>
							<p>Explanation and result that the user can expect</p>
						</div>
					</div>
					<div class="h-16 w-0.5 ml-8 m-2 bg-gray-400 bg-opacity-20" />
					<div class="flex gap-4">
						<button
							class="bg-[#121316] cursor-default text-xl border border-[#393939] rounded-full text-white px-6 py-2"
							>02</button
						>
						<div class="flex flex-col">
							<h2 class="text-gray-500">Step #2</h2>
							<p class="text-gray-500">Explanation and result that the user can expect</p>
						</div>
					</div>
					<div class="h-16 w-0.5 ml-8 mt-2 bg-gray-400 bg-opacity-20" />
				</div>
			</div>
			<div class="flex lg:flex-row flex-col lg:space-x-4 space-y-4 lg:space-y-0">
				<div class="w-full lg:max-w-sm">
					<Input
						placeholder="&#128269; Your Etherium address"
						type="search"
						class="bg-transparent text-white py-4 placeholder-gray-400 border border-gray-600 focus:outline-none"
						bind:value={walletValue}
						on:keydown={handleEnter}
					/>
				</div>
				<Button
					href="#"
					class="p-2 px-8 relative bg-white text-black rounded-bl-none mb-10 md:mb-0 rounded-tr-none text-xl hover:bg-gray-200"
					on:click={demoSync}
				>
					Demo&nbsp;sync
					<span
						class="absolute rounded-br-lg rounded-tl-lg inset-0 ring-1 ring-white hover:ring-gray-200"
						style="transform: translate(2px, 2px);"
					/>
				</Button>
			</div>
		</div>
	</div>
	<div class="flex flex-col relative lg:w-1/3 w-full max-w-md lg:pl-10 xl:px-10 xl:mr-10 min-w-max">
		<div class="flex flex-col gradient-background items-center px-16 py-14 rounded-t-xl">
			<div class="p-6">
				<h2 class="text-center text-3xl font-semibold">{walletBalance} $</h2>
				<div class="text-center text-gray-300">Main wallet</div>
			</div>
		</div>
		<div class="relative border-gradient rounded-b-xl">
			<div class="flex buttons-container mx-0.5 relative bg-black pt-0.5 justify-between">
				<button
					class="rounded-tr-xl font-bold py-2 w-full"
					class:class-active={activeButton === 'tokens'}
					class:class-inactive={activeButton !== 'tokens'}
					disabled={activeButton === 'tokens'}
					on:click={toggleButton}
				>
					<p>Tokens</p>
				</button>
				<button
					class="rounded-tl-xl font-bold py-2 w-full"
					class:class-active={activeButton === 'nfts'}
					class:class-inactive={activeButton !== 'nfts'}
					disabled={activeButton === 'nfts'}
					on:click={toggleButton}
				>
					<p>NFTs</p>
				</button>
			</div>
			<div class="bg-[#1B1B1B] rounded-b-xl relative z-10 mx-0.5 mb-0.5">
				<ul class="p-4 pb-8">
					<li class="flex justify-between items-top py-4">
						<div class="flex items-center">
							<img src={ethImg} alt="ETH" class="w-auto mr-6" />
							<div class="flex flex-col">
								<span class="text-lg">Ethereum</span>
								<span class="value text-gray-400">1.020,53 $</span>
							</div>
						</div>
						<p class="text-md">{ethBalance}</p>
					</li>
					<li class="flex justify-between items-top py-4">
						<div class="flex items-center">
							<img src={btcImg} alt="BTC" class="w-auto mr-6" />
							<div class="flex flex-col">
								<span class="text-lg">BTC</span>
								<span class="value text-gray-400">26.654,53 $</span>
							</div>
						</div>
						<p class="text-md">{btcBalance}</p>
					</li>
					<li class="flex justify-between items-top py-4">
						<div class="flex items-center">
							<img src={bnbImg} alt="BNB" class="w-auto mr-6" />
							<div class="flex flex-col">
								<span class="text-lg">BNB</span>
								<span class="value text-gray-400">213,34 $</span>
							</div>
						</div>
						<p class="text-md">{bnbBalance}</p>
					</li>
					<li class="flex justify-between items-top py-4">
						<div class="flex items-center">
							<img src={polkadotImg} alt="DOT" class="w-auto mr-6" />
							<div class="flex flex-col">
								<span class="text-lg">Polkadot</span>
								<span class="value text-gray-400">4,00 $</span>
							</div>
						</div>
						<p class="text-md">{dotBalance}</p>
					</li>
				</ul>
			</div>
		</div>
	</div>
</section>

<style>
	.center {
		padding: 30px 10% 100px 10%;
	}

	.value {
		font-size: 0.8rem;
	}

	.gradient-background {
		background: linear-gradient(
			0deg,
			#a8c5eb -38.43%,
			#555aa9 14.87%,
			#363571 42.36%,
			#0b061c 100%
		);
	}

	.class-active {
		color: #8fd3fe;
		background-color: #1b1b1b;
		border-bottom: 1px solid #1b1b1b;
	}

	.class-inactive {
		color: white;
		background-color: black;
		border-bottom: 1px solid #614ea3;
	}

	.buttons-container {
		z-index: 10;
	}

	h1 {
		font-family: 'BonaNova-Bold';
	}

	h2 {
		font-family: 'ChakraPetch';
	}

	p {
		font-family: 'Inter';
	}

	button {
		font-family: 'Inter';
		z-index: 10;
	}

	span {
		font-family: 'ChakraPetch';
	}

	.border-gradient {
		position: relative;
		overflow: hidden;
	}

	.border-gradient:before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		border: 2px solid transparent;
		border-radius: inherit;
		background: linear-gradient(to left, #6c77bc, #614ea3);
		z-index: 1;
	}
</style>
