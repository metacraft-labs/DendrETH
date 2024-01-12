<script>
    import { Button } from 'flowbite-svelte';

    const handleClick = () => {
        console.log("Button clicked!");
  };
  </script>
  
  <div class="bg-[#F4F9FF] text-white p-24 flex flex-col content-center center mt-16">
    <div class="container flex justify-between">
    
        <h1 class="text-5xl text-black mb-6 leading-tight mr-8 max-w-lg text-left">
            <span style="color: #78CBFF;">Our</span> blog and news
        </h1>
        <div>
            <button on:click={handleClick} class="text-xl bg-transparent border border-gray-400 py-3 px-5 rounded-full text-gray-400 hover:border-black hover:text-black mr-16">
                &lt;
            </button>
            <button on:click={handleClick} class="text-xl bg-transparent border border-gray-400 py-3 px-5 rounded-full text-gray-400 hover:border-black hover:text-black">
                &gt;
            </button>
        </div>
    </div>
  </div>