<script>
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import messageData from '$lib/database/explorerData.json';

	let nonce = $page.params.nonce;

	let message = null;
	console.log(nonce);

	onMount(async () => {
		try {
			for (let obj of messageData) {
				if (obj.nonce === nonce) {
					message = obj;
				}
			}
		} catch (error) {
			console.error('Error fetching message details:', error);
		}
		console.log(message);
	});
</script>

{#if message}
	<div class="center-details pb-0 text-white flex flex-col gap-8" style="border-bottom: 1px solid #6F6F6F;">
		<h2>Message Details for #{nonce}</h2>
		<div class="flex flex-row space-x-24 pb-10" style="border-bottom: 1px solid #6F6F6F;">
			<div class="flex flex-col space-y-8">
				<h3>Destination Chain</h3>
				<h3>Message Nonce</h3>
				<h3>Message Hash</h3>
				<h3>Sender</h3>
				<h3>Reciever</h3>
				<h3>Data</h3>
			</div>
			<div class="space-y-8">
				<p>{message.destinationChain}</p>
				<p>{message.nonce}</p>
				<p>{message.hash}</p>
				<p>*Need sender data*</p>
				<p>*Need reciever data*</p>
				<p>*Need Data*</p>
			</div>
		</div>
		<h2>Source Transaction details</h2>
		<div class="flex flex-row space-x-24 pb-10" style="border-bottom: 1px solid #6F6F6F;">
			<div class="flex flex-col space-y-8">
				<h3>Chain</h3>
				<h3>Timestamp</h3>
				<h3>Transaction Hash</h3>
				<h3>Block Number</h3>
			</div>
			<div class="space-y-8">
				<p>{message.sourceChain}</p>
				<p>{message.creationTime}</p>
				<p>*Need source transaction hash*</p>
				<p>*Need source block number*</p>
			</div>
		</div>
		<h2>Destination Transaction details</h2>
		<div class="flex flex-row space-x-24">
			<div class="flex flex-col space-y-8">
				<h3>Chain</h3>
				<h3>Timestamp</h3>
				<h3>Transaction Hash</h3>
				<h3>Block Number</h3>
			</div>
			<div class="space-y-8">
				<p>{message.destinationChain}</p>
				<p>{message.creationTime}</p>
				<p>*Need destination transaction hash*</p>
				<p>*Need destination block number*</p>
			</div>
		</div>
	</div>
{:else}
	<p>Loading message details...</p>
{/if}

<style>
	.center-details {
		padding: 30px 5% 25px 5%;
	}
    h2 {
		font-family: 'ChakraPetch';
		font-size: 1.8rem;
	}
	h3 {
		font-family: 'Inter';
		color: #A6A6A6;
		font-size: 1rem;
	}
	p {
		font-family: 'Inter';
	}
</style>
