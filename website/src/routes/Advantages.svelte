<script>
	import {
		GlobeOutline,
		ShieldCheckOutline,
		UsersOutline
	} from 'flowbite-svelte-icons';
</script>

<section class="bg-[#121316] text-white px-10 center">
	<div class="">
		<div class="container flex justify-between mb-4">
			<div class="text-4xl font-bold mb-6 max-w-md leading-relaxed">
				<span style="color: #8FD3FE;">Advantages</span>
			</div>
		</div>
		<div class="grid md:grid-cols-2 gap-6">
			<!-- Card 1 -->
			<div
				class="bg-[#121316] md:col-span-2 p-8 rounded-xl shadow-lg flex flex-col items-left text-left border border-gray-500"
			>
				<!-- Replace 'YourChosenIcon' with the actual icon component you want to use -->
				<GlobeOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2" />
				<div class="text-2xl font-semibold mb-2">One-shot Sync Capability</div>
				<p class="text-sm mt-2 leading-relaxed">
					The fastest way to verify correctness of presented data based on state of the blockchain
					<br />
					*Better user experience for high latency systems like mobile wallets.
				</p>
			</div>

			<!-- Card 2 -->
			<div
				class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left border border-gray-500"
			>
				<ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2" />
				<div class="text-2xl font-semibold mb-2">Permissionless</div>
				<p class="text-sm mt-2 leading-relaxed">
					The very core principle of blockchain technology. We bring true Censhorship Resistance and
					the best User Experience possible.
				</p>
			</div>

			<!-- Card 3 -->
			<div
				class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left border border-gray-500"
			>
				<UsersOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2" />
				<div class="text-2xl font-semibold mb-2">Decentralized and Secure</div>
				<p class="text-sm mt-2 leading-relaxed">
					No trust assumptions on centralized actors, nor permissioned multising. Security is the
					same as the L1 consensus protocol.
				</p>
			</div>
		</div>
	</div>
</section>

<style>
	.center {
		padding: 30px 10% 100px 10%;
	}
</style>
