<script>
	import { ShieldCheckOutline } from 'flowbite-svelte-icons';
</script>

<section class="bg-[#121316] text-white px-10 center">
	<div>
		<!-- <div class="text-3xl font-bold mb-6">Use Cases</div> -->
		<div class="grid md:grid-cols-2 gap-4 space-y-20 items-center">
			<!-- Card 1 -->
			<div class="flex md:mt-14">
				<div class="flex flex-row">
					<div class="bg-[#121316] p-8 rounded-lg flex flex-col items-left text-left">
						<h2 class="text-lg font-semibold mb-4">Permissionless Bridges</h2>
						<ul class="text-sm ml-4">
							<li class="list-disc mb-6">
								Trust based on Math and Code, not People and Organisations.
							</li>
							<li class="list-disc mb-6">
								Fully Decentralized, Trustless and Censorship Resistant.
							</li>
							<li class="list-disc mb-6">No need to give up control of your assets.</li>
							<li class="list-disc">No need to trust centralized actors, nor multising.</li>
						</ul>
					</div>
				</div>
			</div>

			<div
				class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left border border-gray-500"
			>
				<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2" />
				<div class="text-2xl font-semibold mb-2">Security</div>
				<p class="text-sm mt-2 leading-relaxed">
					Exchange value in a trustless way between EOS and Ethereum.
					<br />
					*Add 2 rows of copy here
					<br />
					*Add 2 rows of copy here
				</p>
			</div>
			<!-- Card 2 -->
			<div class="flex mt-14">
				<div class="flex flex-row">
					<div class="bg-[#121316] p-8 rounded-lg flex flex-col items-left text-left">
						<h2 class="text-lg font-semibold mb-4">Oracles</h2>
						<ul class="text-sm ml-4">
							<li class="list-disc mb-6">
								DendrETH helps you verify data settled on Ethereum from other chains.
							</li>
							<li class="list-disc">
								Build a custom oracle for Governance, Staking, Cross-Chain Messaging and more.
							</li>
						</ul>
					</div>
				</div>
			</div>

			<div
				class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left border border-gray-500"
			>
				<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2" />
				<div class="text-2xl font-semibold mb-2">Security</div>
				<p class="text-sm mt-2 leading-relaxed">
					Exchange value in a trustless way between EOS and Ethereum.
					<br />
					*Add 2 rows of copy here
					<br />
					*Add 2 rows of copy here
				</p>
			</div>
			<!-- Card 3 -->
			<div class="flex mt-14">
				<div class="flex flex-row">
					<div class="bg-[#121316] p-8 rounded-lg flex flex-col items-left text-left">
						<div class="text-lg font-semibold mb-4">One-Shot Sync</div>
						<ul class="text-sm ml-4">
							<li class="list-disc mb-6">
								Blazing fast Sync and verify state from Ethereum's consensus layer.
							</li>
							<li class="list-disc mb-6">Accurate data, no Intermediaries</li>
							<li class="list-disc">Maximum Security, Best UX</li>
						</ul>
					</div>
				</div>
			</div>

			<div
				class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left border border-gray-500"
			>
				<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2" />
				<div class="text-2xl font-semibold mb-2">Security</div>
				<p class="text-sm mt-2 leading-relaxed">
					Exchange value in a trustless way between EOS and Ethereum.
					<br />
					*Add 2 rows of copy here
					<br />
					*Add 2 rows of copy here
				</p>
			</div>
			<!-- Card 4 -->
			<div class="flex mt-14">
				<div class="flex flex-row">
					<div class="bg-[#121316] p-8 rounded-lg flex flex-col items-left text-left">
						<div class="text-lg font-semibold mb-4">Liquid Stacking</div>
						<ul class="text-sm ml-4">
							<li class="list-disc">TODO...</li>
						</ul>
					</div>
				</div>
			</div>

			<div
				class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left border border-gray-500"
			>
				<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2" />
				<div class="text-2xl font-semibold mb-2">Security</div>
				<p class="text-sm mt-2 leading-relaxed">
					Exchange value in a trustless way between EOS and Ethereum.
					<br />
					*Add 2 rows of copy here
					<br />
					*Add 2 rows of copy here
				</p>
			</div>
		</div>

		<!-- Benefits grid disabled for now -->
		<!-- <div class="grid lg:grid-cols-4 md:grid-cols-2 gap-4 mt-24">
			<div class="benefit-box">
				<div class="h-full w-1 bg-gray-400 bg-opacity-30 md:hidden sm:block mx-4" />
				<div class="benefit-text">
					<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-16 mt-8" />
					<h2 class="benefit-name">Benefit #1 Label</h2>
					<p class="mb-8">
						Smart Contract Implementation of the Veacon chain light client sync protocol
					</p>
				</div>
			</div>
			<div class="benefit-box">
				<div class="h-full w-1 bg-gray-400 bg-opacity-30 mx-4" />
				<div class="benefit-text">
					<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-16 mt-8" />
					<h2 class="benefit-name">Benefit #2 Label</h2>
					<p class="mb-8">Empowering Trustless Connection to the Ethereum Network</p>
				</div>
			</div>
			<div class="benefit-box">
				<div class="h-full w-1 bg-gray-400 bg-opacity-30 md:hidden lg:block sm:block mx-4" />
				<div class="benefit-text">
					<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-16 mt-8" />
					<h2 class="benefit-name">Benefit #3 Label</h2>
					<p class="mb-8">Enabling the Creation of Secure Cross-Blockchain Bridges</p>
				</div>
			</div>
			<div class="benefit-box">
				<div class="h-full w-1 bg-gray-400 bg-opacity-30 mx-4" />
				<div class="benefit-text">
					<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-16 mt-8" />
					<h2 class="benefit-name">Benefit #4 Label</h2>
					<p class="mb-8">
						Using Zero Knowledge circuit to verify BLS signatures of the ETH 2 validators
					</p>
				</div>
			</div>
		</div> -->
	</div>
</section>

<style>
	.center {
		padding: 30px 10% 100px 10%;
	}

	.benefit-box {
		display: flex;
		flex-direction: row;
	}

	.benefit-text {
		display: flex;
		flex-direction: column;
	}

	.benefit-name {
		font-size: 1rem;
		font-weight: 600;
		margin-bottom: 0.75rem;
	}

	.code1 {
		height: auto;
		width: 460px;
	}
</style>
