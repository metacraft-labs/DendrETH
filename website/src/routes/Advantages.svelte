<script>
    import { GlobeOutline, ShieldCheckOutline, UsersOutline, DollarOutline } from 'flowbite-svelte-icons';
  </script>
  
  <section class="bg-[#121316] text-white px-10 center">
    <div class="">
      <div class="container flex justify-between mb-4">
        <div class="text-4xl font-bold mb-6 max-w-md leading-relaxed"><span style="color: #8FD3FE;">Advantages</span> we deliver right out of the box</div>
        <div class="text-xl max-w-md">Some copy about the approach taken and whats the value proposition that stands behind DendrETH</div>
      </div>
      <div class="grid md:grid-cols-2 gap-6">
        
        <!-- Card 1 -->
        <div class="bg-[#121316] p-8 rounded-xl shadow-lg flex flex-col items-left text-left border border-gray-500">
          <!-- Replace 'YourChosenIcon' with the actual icon component you want to use -->
          <GlobeOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2"/>
          <div class="text-2xl font-semibold mb-2">Interoperability</div>
          <p class="text-sm mt-2 leading-relaxed">
            Connect to Ethereum's 26 billion TLV network.
            <br>
            *Add 2 rows of copy here
            <br>
            *Add 2 rows of copy here
          </p>
        </div>
    
        <!-- Card 2 -->
        <div class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left  border border-gray-500">
          <ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2"/>
          <div class="text-2xl font-semibold mb-2">Security</div>
          <p class="text-sm mt-2 leading-relaxed">
            Exchange value in a trustless way between EOS and Ethereum.
            <br>
            *Add 2 rows of copy here
            <br>
            *Add 2 rows of copy here
          </p>
        </div>
    
        <!-- Card 3 -->
        <div class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left  border border-gray-500">
          <UsersOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2"/>
          <div class="text-2xl font-semibold mb-2">Expanded User Base</div>
          <p class="text-sm mt-2 leading-relaxed">
            Grow user numbers and network activity
            <br>
            *Add 2 rows of copy here
            <br>
            *Add 2 rows of copy here
          </p>
        </div>

        <div class="bg-[#121316] p-8 rounded-lg shadow-lg flex flex-col items-left text-left  border border-gray-500">
          <ShieldCheckOutline class="text-blue-500 mb-4 w-20 h-20 mb-12 ml-2"/>
          <div class="text-2xl font-semibold mb-2">Liquidity</div>
          <p class="text-sm mt-2 leading-relaxed">
            Increase EOS liquidity by connecting to Ethereum
            <br>
            *Add 2 rows of copy here
            <br>
            *Add 2 rows of copy here
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
  