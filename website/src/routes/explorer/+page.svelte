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

	import chevronDown from '$lib/images/chevron-down.svg';

	let searchTerm = '';
	const filters = ['All', 'Avalanche', 'Binance', 'Polygon', 'Ethereum'];

	export let data;
	let messageData = [];
	const pageSize = 10;
	let currentPage = 1;
	let paginatedData = [];

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

	function goToDetailPage(id) {
		console.log(`Navigate to details for message with id: ${id}`);
		// Implement navigation logic here
	}

	const maxButtons = 5;

	function getPaginationRange(current, total) {
		const sideButtons = 2;
		const from = Math.max(1, current - sideButtons);
		const to = Math.min(total, current + sideButtons);

		let pages = [];
		let isStartEllipsesAdded = false;
		let isEndEllipsesAdded = false;

		for (let page = 1; page <= total; page++) {
			if (page <= 2 || page > total - 2 || (page >= from && page <= to)) {
				pages.push(page);
			} else if (page < from && !isStartEllipsesAdded) {
				pages.push('...');
				isStartEllipsesAdded = true;
			} else if (page > to && !isEndEllipsesAdded) {
				pages.push('...');
				isEndEllipsesAdded = true;
			}
		}

		if (pages.length < maxButtons) {
			if (pages[0] > 1) {
				const extraPages = Array.from({ length: maxButtons - pages.length }, (_, i) => i + 1);
				pages = extraPages.concat(pages);
			} else if (pages[pages.length - 1] < total) {
				const extraPages = Array.from(
					{ length: maxButtons - pages.length },
					(_, i) => total - i
				).reverse();
				pages = pages.concat(extraPages);
			}
		}

		if (pages[0] > 1) {
			pages.unshift('...');
		}

		if (pages[pages.length - 1] < total) {
			pages.push('...');
		}

		return pages;
	}
	$: totalPages = Math.ceil(messageData.length / pageSize);
	$: paginatedPages = getPaginationRange(currentPage, Math.ceil(messageData.length / pageSize));
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
					class="cursor-pointer bg-[#121316] hover:bg-[#393939]"
					on:click={() => goToDetailPage(message.nonce)}
				>
					<TableBodyCell class="text-white">{message.nonce}</TableBodyCell>
					<TableBodyCell class="text-white">{message.timestamp}</TableBodyCell>
					<TableBodyCell class="text-white">{message.creationTime}</TableBodyCell>
					<TableBodyCell class="text-white">{message.sourceChain}</TableBodyCell>
					<TableBodyCell class="text-white">{message.destinationChain}</TableBodyCell>
					<TableBodyCell class="text-white">{message.hash}</TableBodyCell>
					<TableBodyCell class="text-white">{message.status}</TableBodyCell>
					<TableBodyCell>
						<img src={chevronDown} alt="chevron" />
					</TableBodyCell>
				</TableBodyRow>
			{/each}
		</TableBody>
	</TableSearch>
	<div class="flex flex-row justify-center">
		<button
			class="rounded-md bg-[#121316] text-white mx-1 p-2 px-4 border border-white pagination-button {currentPage ===
			1
				? 'active'
				: ''}"
			on:click={() => changePage(1)}
			disabled={currentPage === 1}>1</button
		>
		{#each paginatedPages as page}
			{#if page != '1' && page != totalPages}
				<button
					on:click={() => changePage(page)}
					disabled={page === '...'}
					class="rounded-md bg-[#121316] text-white mx-1 p-2 px-4 border border-white pagination-button {currentPage ===
					page
						? 'active'
						: ''}"
				>
					{page}
				</button>
			{/if}
		{/each}
		<button
			class="rounded-md bg-[#121316] text-white mx-1 p-2 px-4 border border-white pagination-button {currentPage ===
			totalPages
				? 'active'
				: ''}"
			on:click={() => changePage(totalPages)}
			disabled={currentPage === totalPages}>{totalPages}</button
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
	.pagination-button.active {
		background-color: #393939;
	}
</style>
