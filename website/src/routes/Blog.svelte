<script>
	import { Button } from 'flowbite-svelte';

	import { onMount } from 'svelte';
	import Glide from '@glidejs/glide';

	import posts from '$lib/database/blogPosts.json';

	let canSlidePrev = false;
	let canSlideNext = true;

	let glide;
	let totalSlides = 0;
	let perView = 3;

	const CustomLength = function (Glide, Components, Events) {
		return {
			mount() {
				totalSlides = Components.Sizes.length;
			}
		};
	};

	onMount(() => {
		glide = new Glide('.glide', {
			startAt: 0,
			type: 'slider',
			perView: perView,
			rewind: false,
			gap: 20
		}).mount({ CustomLength });

		glide.mount();

		const updatePerView = () => {
			switch (true) {
				case window.innerWidth < 769:
					perView = 1;
					break;
				case window.innerWidth < 1440:
					perView = 2;
					break;
				case window.innerWidth < 1920:
					perView = 3;
					break;
				case window.innerWidth < 2560:
					perView = 4;
					break;
				default:
					perView = 5;
					break;
				}
			if (totalSlides < glide.index + perView) {
				glide.index -= 1
			}
			glide.update({ perView: perView });
		};

		window.addEventListener('resize', updatePerView);

		glide.on('move', () => {
      		const currentIndex = glide.index;
      		const lastIndex = currentIndex + glide.settings.perView;

      		canSlideNext = lastIndex < totalSlides;
      		canSlidePrev = currentIndex > 0;
    	});
	});

	function goToPrevSlide() {
		glide.go('<');
	}

	function goToNextSlide() {
		glide.go('>');
	}
</script>

<div class="bg-[#F4F9FF] text-white md:p-24 p-8 flex flex-col content-center center md:mt-16">
	<div class="container flex md:flex-row flex-col justify-between w-full ma">
		<h1
			class="md:text-5xl text-2.2rem text-black mb-6 leading-tight text-center md:text-left md:mr-8 text-center"
		>
			<span style="color: #78CBFF;">Our</span> blog and news
		</h1>
		<div class="flex justify-between md:block mb-6">
			<button
				on:click={goToPrevSlide}
				class="left-button text-xl bg-transparent border border-black py-3 px-5 rounded-full text-black hover:bg-gray-200 mr-16"
				disabled={!canSlidePrev}
			>
				&lt;
			</button>
			<button
				on:click={goToNextSlide}
				class="right-button text-xl bg-transparent border border-black py-3 px-5 rounded-full text-black hover:bg-gray-200"
				disabled={!canSlideNext}
			>
				&gt;
			</button>
		</div>
	</div>
	<div class="glide">
		<div class="glide__track" data-glide-el="track">
			<ul class="flex flex-row">
				{#if posts}
					{#each posts as post}
						<li class="w-1/3 box-border">
							<div class="item flex flex-col bg-white rounded-xl justify-center">
								<img src={post.image} alt={post.title} class="max-w-full h-auto rounded-xl" />
								<h2 class="text-xl p-5 pb-2 text-black text-left">{post.title}</h2>
								<div class="justify-between pl-5 pr-5 flex flex-row">
									<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3">
										{post.category}
									</button>
									<p class="text-black text-xs">{post.date}</p>
								</div>
								<p class="p-5 pt-3 text-black text-xs leading-relaxed">
									{post.summary}
								</p>
							</div>
						</li>
					{/each}
				{/if}
			</ul>
		</div>
	</div>
	<Button
		href="#"
		class="flex bg-transparent text-black text-xl hover:text-grey-300 hover:bg-transparent md:ml-auto mt-6"
	>
		Read all our stories →
	</Button>
</div>

<style>
	.glide__track {
		display: flex;
		flex-direction: row;
		width: 100%;
		overflow: hidden;
	}

	.container {
		max-width: 100%;
	}

	h1 {
		font-family: 'ChakraPetch';
	}

	span {
		font-family: 'BonaNova-Bold';
	}

	:disabled {
		color: rgb(156 163 175);
		border-color: rgb(156 163 175);
		background: transparent;
	}
</style>
