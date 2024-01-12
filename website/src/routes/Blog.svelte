<script>
	import { Button } from 'flowbite-svelte';

	import { onMount } from 'svelte';
	import Glide from '@glidejs/glide';

	let glide;

	onMount(() => {
		glide = new Glide('.glide', {
            startAt: 1,
			type: 'slider',
            perView: 3,
		});

		glide.mount();
	});
	function goToPrevSlide() {
		if (glide) {
			glide.go('<');
		}
	}

	function goToNextSlide() {
		if (glide) {
			glide.go('>');
		}
	}
</script>

<div class="bg-[#F4F9FF] text-white p-24 flex flex-col content-center center mt-16">
	<div class="container flex justify-between">
		<h1 class="text-5xl text-black mb-6 leading-tight mr-8 max-w-lg text-left">
			<span style="color: #78CBFF;">Our</span> blog and news
		</h1>
		<div>
			<button
				on:click={goToPrevSlide}
				class="text-xl bg-transparent border border-gray-400 py-3 px-5 rounded-full text-gray-400 hover:border-black hover:text-black mr-16"
			>
				&lt;
			</button>
			<button
				on:click={goToNextSlide}
				class="text-xl bg-transparent border border-gray-400 py-3 px-5 rounded-full text-gray-400 hover:border-black hover:text-black"
			>
				&gt;
			</button>
		</div>
	</div>
	<div class="glide">
		<div class="glide__track" data-glide-el="track">
			<ul class="glide__slides">
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 1</h2>
						<p>Content for Slide 1</p>
					</div>
				</li>
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 2</h2>
						<p>Content for Slide 2</p>
					</div>
				</li>
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 3</h2>
						<p>Content for Slide 3</p>
					</div>
				</li>
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 1</h2>
						<p>Content for Slide 1</p>
					</div>
				</li>
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 2</h2>
						<p>Content for Slide 2</p>
					</div>
				</li>
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 3</h2>
						<p>Content for Slide 3</p>
					</div>
				</li>
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 1</h2>
						<p>Content for Slide 1</p>
					</div>
				</li>
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 2</h2>
						<p>Content for Slide 2</p>
					</div>
				</li>
				<li class="glide__slide">
					<div class="slide-content">
						<h2>Slide 3</h2>
						<p>Content for Slide 3</p>
					</div>
				</li>
			</ul>
		</div>
	</div>
</div>

<style>
	.slide-content {
		border: 1px solid #ccc; /* Add a border to create the box shape */
		padding: 20px; /* Add padding to create space inside the box */
		background-color: gray; /* Optional: Set a background color */
		box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.1); /* Optional: Add a box shadow */
		border-radius: 8px; /* Optional: Add rounded corners */
		text-align: center; /* Optional: Center the content horizontally */
	}

	/* Style the headings and paragraphs inside the slides */
	.slide-content h2 {
		font-size: 24px;
        color: black; /* Adjust the heading font size */
		margin-bottom: 10px; /* Optional: Add margin below the heading */
	}

	.slide-content p {
		font-size: 16px; /* Adjust the paragraph font size */
		color: #333; /* Optional: Customize the text color */
	}
	.glide__track {
		display: flex;
		flex-direction: row; /* Arrange slides horizontally */
		overflow: hidden; /* Hide overflow content */
		width: 100%; /* Ensure the track takes the full width */
	}

	/* Adjust the width and transition for smooth sliding */
	.glide__slides {
		display: flex;
		transition: transform 0.3s ease-in-out; /* Add smooth slide transition */
	}

	/* Define the style for each slide */
	.glide__slide {
		flex: 0 0 33.33%; /* Ensure slides don't grow/shrink */
		max-width: 300px; /* Each slide takes full width */
		padding: 20px; /* Add padding to slides (optional) */
		box-sizing: border-box; /* Ensure padding is included in width */
		/* Add other styles as needed */
	}
</style>
