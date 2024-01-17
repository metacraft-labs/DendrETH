<script>
	import {
		TableSearch,
		TableBody,
		TableBodyCell,
		TableBodyRow,
		TableHead,
		TableHeadCell,
		Input,
		Button,
		ButtonGroup
	} from 'flowbite-svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';

	let searchTerm = '';
	const filters = ['All', 'Avalanche', 'Binance', 'Polygon', 'Ethereum'];

	export let data;
	let messageData = [];

	const pageSize = 10;
	let currentPage = 1;
	let paginatedData = [];

	let expandedRow = null;

	function toggleDetail(nonce) {
		if (expandedRow === nonce) {
			expandedRow = null;
		} else {
			expandedRow = nonce;
		}
	}

	onMount(async () => {
		messageData = data.messageData;
		paginateData();
	});

	function paginateData() {
		const startIndex = (currentPage - 1) * pageSize;
		const endIndex = startIndex + pageSize;
		paginatedData = messageData.slice(startIndex, endIndex);
	}

	function changePage(pageNumber) {
		currentPage = pageNumber;
		paginateData();
	}

	function setFilter(filter) {
		searchTerm = filter;
	}

	function goToDetailPage(nonce) {
		// console.log(`Navigate to details for message with id: ${id}`);
		const detailUrl = `/explorer/${nonce}`;
        goto(detailUrl);
    }

	const maxButtons = 5;

	$: totalPages = Math.ceil(messageData.length / pageSize);
</script>

<div
	class="flex flex-col space-y-4 bg-[#121316] p-4 pt-20 center"
	style="border-bottom: 1px solid #6F6F6F;"
>
	<div class="flex flex-col items-center bg-[#121316]">
		<div class="flex items-center w-full max-w-xl py-4 space-x-4">
			<Input
				placeholder="&#128269; Search by notch or hash"
				type="search"
				class="bg-transparent text-white placeholder-gray-400 focus:outline-none"
				bind:value={searchTerm}
			/>
		</div>
		<div class="flex flex-row gap-2">
			{#each filters as filter}
				<Button
					pill
					class="bg-black text-white border border-white text-xs"
					on:click={() => setFilter(filter)}
				>
					{filter}
				</Button>
			{/each}
		</div>
	</div>

	<div class="text-white pt-10">
		<h1>Recent messages</h1>
	</div>
	<TableSearch
		placeholder="Search ..."
		hoverable={true}
		bind:inputValue={searchTerm}
		classInput="hidden"
		classSvgDiv="hidden"
	>
		<TableHead class="bg-[#121316] text-white text-md" style="border-bottom: 1px solid white;">
			<TableHeadCell>Nonce</TableHeadCell>
			<TableHeadCell>Time Stamp</TableHeadCell>
			<TableHeadCell>Creation Time</TableHeadCell>
			<TableHeadCell>Source Chain</TableHeadCell>
			<TableHeadCell>Destination Chain</TableHeadCell>
			<TableHeadCell>Hash</TableHeadCell>
			<TableHeadCell>Status</TableHeadCell>
			<TableHeadCell>Details</TableHeadCell>
		</TableHead>
		<TableBody>
			{#each paginatedData as message, index (message.nonce)}
				<TableBodyRow
					class="cursor-pointer bg-[#121316] hover:bg-[#393939] table-row"
				>
					<TableBodyCell class="text-white">{message.nonce}</TableBodyCell>
					<TableBodyCell class="text-white">{message.timestamp}</TableBodyCell>
					<TableBodyCell class="text-white">{message.creationTime}</TableBodyCell>
					<TableBodyCell class="text-white">{message.sourceChain}</TableBodyCell>
					<TableBodyCell class="text-white">{message.destinationChain}</TableBodyCell>
					<TableBodyCell class="text-white">{message.hash}</TableBodyCell>
					<TableBodyCell class="text-white">{message.status}</TableBodyCell>
					<TableBodyCell>
						<a href="#" class="text-blue-500 hover:text-gray-300" on:click|preventDefault={() => goToDetailPage(message.nonce)}>View Details</a>
					</TableBodyCell>
				</TableBodyRow>
			{/each}
		</TableBody>
	</TableSearch>
	<div class="flex flex-row justify-end items-center">
		<div class="text-white mr-2">
			Page {currentPage}
		</div>
		<button
			class="rounded-md bg-[#121316] text-white mx-1 p-1 px-4 border border-white pagination-button hover:bg-[#393939]"
			on:click={() => changePage(currentPage - 1)}
			disabled={currentPage === 1}>Previous</button
		>
		<button
			class="rounded-md bg-[#121316] text-white mx-1 p-1 px-4 border border-white pagination-button hover:bg-[#393939]"
			on:click={() => changePage(currentPage + 1)}
			disabled={currentPage === totalPages}>Next</button
		>
	</div>
</div>

<style>
	.center {
		padding: 30px 5% 100px 5%;
	}
	h1 {
		font-family: 'ChakraPetch';
		font-size: 3.5rem;
		line-height: 120%;
	}
	.pagination-button:disabled {
		color: #393939;
		border-color: #393939;
		background-color: #121316;
	}
	.table-row.active {
		background-color: #393939;
	}
</style>
