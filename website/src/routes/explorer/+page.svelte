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
  
	let searchTerm = '';
    const filters = ['All', 'Avalanche', 'Binance', 'Polygon', 'Ethereum'];

	const messageData = [
	  { nonce: '4351', destinationChain: 'Avalanche', msgSender: '0xa3b31028893', msgReceiver: '0x4fda...85Ad47', timeStamp: 'Waiting for proof of consensus' },
	  { nonce: '4352', destinationChain: 'Ethereum', msgSender: '0xa3b31028893', msgReceiver: '0x4fda...85Ad47', timeStamp: 'Waiting for proof of consensus' },
	  // ...other message data objects
	];
  
	function setFilter(filter) {
	  searchTerm = filter;
	}
  
	function goToDetailPage(id) {
	  console.log(`Navigate to details for message with id: ${id}`);
	  // Implement navigation logic here
	}
  </script>
  
  <div class="flex flex-col space-y-4 bg-black p-4">
	<div class="flex flex-col items-center bg-black">
		<div class="flex items-center w-full max-w-xl py-4 space-x-4">
			<Input
			placeholder="Search by notch or hash"
			type="search"
			class="bg-transparent text-white placeholder-gray-400 focus:outline-none"
			bind:value={searchTerm}
		  />
		  </div>
		
		  <ButtonGroup class="ml-4">
			{#each filters as filter}
			  <Button
			    pill
				color="dark"
				on:click={() => setFilter(filter)}
			  >
				{filter}
			  </Button>
			{/each}
		  </ButtonGroup>
	</div>
  
	<TableSearch placeholder="Search ..." hoverable={true} bind:inputValue={searchTerm} classInput="hidden" classSvgDiv="hidden">
	  <TableHead>
		  <TableHeadCell>Nonce</TableHeadCell>
		  <TableHeadCell>Destination Chain</TableHeadCell>
		  <TableHeadCell>Msg Sender</TableHeadCell>
		  <TableHeadCell>Msg Receiver</TableHeadCell>
		  <TableHeadCell>Time Stamp</TableHeadCell>
		  <TableHeadCell>Details</TableHeadCell>
	  </TableHead>
	  <TableBody>
		{#each messageData as message, index (message.nonce)}
		  <TableBodyRow class="cursor-pointer hover:bg-gray-100" on:click={() => goToDetailPage(message.nonce)}>
			<TableBodyCell>{message.nonce}</TableBodyCell>
			<TableBodyCell>{message.destinationChain}</TableBodyCell>
			<TableBodyCell>{message.msgSender}</TableBodyCell>
			<TableBodyCell>{message.msgReceiver}</TableBodyCell>
			<TableBodyCell>{message.timeStamp}</TableBodyCell>
			<TableBodyCell>
			  <a href="#" class="text-blue-600 hover:text-blue-800">View details</a>
			</TableBodyCell>
		  </TableBodyRow>
		{/each}
	  </TableBody>
	</TableSearch>
  </div>
  