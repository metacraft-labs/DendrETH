<script>
	import { page } from '$app/stores';
	import { onMount } from 'svelte';

	let nonce = $page.params.nonce;
    export let data;

	let message = null;

	let isMobileView = window.innerWidth < 768;

	function handleResize() {
		isMobileView = window.innerWidth < 768;
	}

	onMount(async () => {
		try {
			window.addEventListener('resize', handleResize);
			for (let obj of data.messageData) {
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

{#if message && !isMobileView}
	<div
		class="center-details text-white flex flex-col gap-8"
		style="border-bottom: 1px solid #6F6F6F;"
	>
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
{:else if message}
	<div
		class="center-details text-white flex flex-col gap-8"
		style="border-bottom: 1px solid #6F6F6F;"
	>
		<h2>Message Details for #{nonce}</h2>
		<div
			class="grid grid-cols-1 gap-4 pb-10"
			style="border-bottom: 1px solid #6F6F6F;"
		>
			<div>
				<h3 class="text-white">Destination Chain</h3>
				<p>{message.destinationChain}</p>
			</div>
			<div>
				<h3 class="text-white">Message Nonce</h3>
				<p>{message.nonce}</p>
			</div>
			<div>
				<h3 class="text-white">Message Hash</h3>
				<p>{message.hash}</p>
			</div>
			<div>
				<h3 class="text-white">Sender</h3>
				<p>*Need sender data*</p>
			</div>
			<div>
				<h3 class="text-white">Receiver</h3>
				<p>*Need receiver data*</p>
			</div>
			<div>
				<h3 class="text-white">Data</h3>
				<p>*Need Data*</p>
			</div>
		</div>
        <h2>Source Transaction details</h2>
        <div class="grid grid-cols-1 gap-4 pb-10" style="border-bottom: 1px solid #6F6F6F;">
            <div>
                <h3 class="text-white">Chain</h3>
                <p>{message.sourceChain}</p>
            </div>
            <div>
                <h3 class="text-white">Timestamp</h3>
                <p>{message.creationTime}</p>
            </div>
            <div>
                <h3 class="text-white">Transaction Hash</h3>
                <p>*Need source transaction hash*</p>
            </div>
            <div>
                <h3 class="text-white">Block Number</h3>
                <p>*Need source block number*</p>
            </div>
        </div>
        <h2>Destination Transaction details</h2>
        <div class="grid grid-cols-1 gap-4">
            <div>
                <h3 class="text-white">Chain</h3>
                <p>{message.destinationChain}</p>
            </div>
            <div>
                <h3 class="text-white">Timestamp</h3>
                <p>{message.creationTime}</p>
            </div>
            <div>
                <h3 class="text-white">Transaction Hash</h3>
                <p>*Need destination transaction hash*</p>
            </div>
            <div>
                <h3 class="text-white">Block Number</h3>
                <p>*Need destination block number*</p>
            </div>
        </div>
        
	</div>
{/if}

<style>
	.center-details {
		padding: 30px 5% 70px 5%;
	}
	h2 {
		font-family: 'ChakraPetch';
		font-size: 1.8rem;
	}
	h3 {
		font-family: 'Inter';
		color: #a6a6a6;
		font-size: 1rem;
	}
	p {
		font-family: 'Inter';
	}
</style>
