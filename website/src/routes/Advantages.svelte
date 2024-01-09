<script>
    import { GlobeOutline, ShieldCheckOutline, UsersOutline, DollarOutline } from 'flowbite-svelte-icons';
  </script>
  
  <section class="bg-black text-white px-10 center">
    <div class="">
      <div class="text-3xl font-bold mb-6">Advantages</div>
      <div class="grid md:grid-cols-2 gap-4">
        
        <!-- Card 1 -->
        <div class="bg-gray-800 p-8 rounded-lg shadow-lg flex flex-col items-center text-center">
          <!-- Replace 'YourChosenIcon' with the actual icon component you want to use -->
          <GlobeOutline class="text-blue-500 mb-4" size="lg" />
          <div class="text-lg font-semibold mb-2">One-shot Sync Capability</div>
          <p class="text-sm">
            * The fastest way to verify correctness of presented data based on state of the blockchain.
          </p>
          <p class="text-sm mt-2">
            • Better user experience for high latency systems like mobile wallets.
          </p>
        </div>
    
        <!-- Card 2 -->
        <div class="bg-gray-800 p-8 rounded-lg shadow-lg flex flex-col items-center text-center">
          <ShieldCheckOutline class="text-blue-500 mb-4" size="lg" />
          <div class="text-lg font-semibold mb-2">Permissionless</div>
          <p class="text-sm">
            The very core principle of blockchain technology. We bring true Censorship Resistance and the best User Experience possible.
          </p>
        </div>
    
        <!-- Card 3 -->
        <div class="bg-gray-800 p-8 rounded-lg shadow-lg flex flex-col items-center text-center">
          <UsersOutline class="text-blue-500 mb-4" size="lg" />
          <div class="text-lg font-semibold mb-2">Decentralized and Secure</div>
          <p class="text-sm">
            No trust assumptions on centralized actors, nor permissioned multising. Security is the same as the L1 consensus protocol.
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
  