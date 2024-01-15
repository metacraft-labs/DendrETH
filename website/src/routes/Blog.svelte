<script>
	import { Button } from 'flowbite-svelte';

	import { onMount } from 'svelte';
	import Glide from '@glidejs/glide';

	let glide;
	let totalSlides = 0;

	const CustomLength = function (Glide, Components, Events) {
		return {
			mount() {
				totalSlides = Components.Sizes.length - 1;
			}
		};
	};

	onMount(() => {
		glide = new Glide('.glide', {
			startAt: 0,
			type: 'slider',
			perView: 3,
			rewind: false,
		}).mount({ CustomLength });

		glide.mount();

		glide.on('move', () => {
			const lastIndex = glide.index + glide.settings.perView;
			console.log(lastIndex, totalSlides);

			if (lastIndex >= totalSlides) {
				document.querySelector('.right-button').disabled = true;
			} else {
				document.querySelector('.right-button').disabled = false;
			}
		});
	});

	function goToPrevSlide() {
		glide.go('<');
	}

	function goToNextSlide() {
		glide.go('>');
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
				class="left-button text-xl bg-transparent border border-gray-400 py-3 px-5 rounded-full text-gray-400 hover:border-black hover:text-black mr-16"
			>
				&lt;
			</button>
			<button
				on:click={goToNextSlide}
				class="right-button text-xl bg-transparent border border-gray-400 py-3 px-5 rounded-full text-gray-400 hover:border-black hover:text-black"
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
		border: 1px solid #ccc;
		padding: 20px;
		background-color: gray;
		box-shadow: 0px 2px 4px rgba(0, 0, 0, 0.1);
		border-radius: 8px;
		text-align: center;
	}

	.slide-content h2 {
		font-size: 24px;
		color: black;
		margin-bottom: 10px;
	}

	.slide-content p {
		font-size: 16px;
		color: #333;
	}
	.glide__track {
		display: flex;
		flex-direction: row;
		overflow: hidden;
		width: 100%;
	}

	.glide__slides {
		display: flex;
		transition: transform 0.3s ease-in-out;
	}

	.glide__slide {
		flex: 0 0 33.33%;
		max-width: 300px;
		padding: 20px;
		box-sizing: border-box;
	}
</style>
