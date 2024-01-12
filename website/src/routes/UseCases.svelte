<script>
	import {
		GlobeOutline,
		ShieldCheckOutline,
		UsersOutline,
		DollarOutline
	} from 'flowbite-svelte-icons';
	import code1 from '$lib/images/code1.svg';
</script>

<section class="bg-[#121316] text-white px-10 center">
	<div class="">
		<!-- <div class="text-3xl font-bold mb-6">Use Cases</div> -->
		<div class="grid md:grid-cols-2 gap-4">
			<!-- Card 1 -->
			<div class="flex mt-14">
				<div class="flex flex-row">
					<div class="flex flex-col items-center pt-10 h-full pb-10">
						<button class="bg-[#393939] rounded-full text-white px-4 py-2 mb-1">01</button>
						<div class="h-24 w-0.5 bg-gray-400 bg-opacity-20" />
						<!-- Vertical line -->
						<button
							class="bg-[#121316] border border-[#393939] rounded-full text-white px-4 py-2 mt-1"
							>02</button
						>
					</div>
					<div class="bg-[#121316] p-8 rounded-lg flex flex-col items-left text-left">
						<div class="text-lg font-semibold mb-4">Trusted Bridges</div>
						<ul class="text-sm ml-4">
							<li class="list-disc mb-6">
								Trusted bridges depend upon a central entity or system for their operations.
							</li>
							<li class="list-disc mb-6">
								They have trust assumptions with respect to the custody of security of the bridge.
								Users mostly rely on the bridge operator's reputation.
							</li>
							<li class="list-disc">Users need to give up control of their crypto assets.</li>
						</ul>
					</div>
				</div>
			</div>
			<!-- Card 2 -->
			<div class="ounded-lg shadow-lg flex flex-col items-center text-center">
				<img src={code1} alt="Descriptive Text" class="code1" />
			</div>
		</div>

		<!-- Benefits grid -->
		<div class="grid lg:grid-cols-4 md:grid-cols-2 gap-4 mt-24">
			<div class="benefit-box">
				<div class="h-full w-1 bg-gray-400 bg-opacity-0 mx-4" />
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
				<div class="h-full w-1 bg-gray-400 lg:bg-opacity-30 md:bg-opacity-0 mx-4" />
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
		</div>
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
