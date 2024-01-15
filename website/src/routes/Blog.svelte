<script>
	import { Button } from 'flowbite-svelte';

	import { onMount } from 'svelte';
	import Glide from '@glidejs/glide';

	import blogImg from '$lib/images/blog.png';

	let glide;
	let totalSlides = 0;

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
			perView: 3,
			rewind: false
		}).mount({ CustomLength });

		glide.mount();

		glide.on('move', () => {
			const lastIndex = glide.index + glide.settings.perView;

			if (lastIndex >= totalSlides) {
				document.querySelector('.right-button').disabled = true;
			} else {
				document.querySelector('.right-button').disabled = false;
			}
            if (glide.index == 0) {
                document.querySelector('.left-button').disabled = true;
            } else {
                document.querySelector('.left-button').disabled = false;
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
				class="left-button text-xl bg-transparent border border-black py-3 px-5 rounded-full text-black hover:bg-gray-200 mr-16"
			>
				&lt;
			</button>
			<button
				on:click={goToNextSlide}
				class="right-button text-xl bg-transparent border border-black py-3 px-5 rounded-full text-black hover:bg-gray-200"
			>
				&gt;
			</button>
		</div>
	</div>
	<div class="glide">
		<div class="glide__track" data-glide-el="track">
			<ul class="flex flex-row">
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
				<li class="w-1/3 max-w-lg box-border">
					<div class="item flex flex-col bg-white rounded-xl justify-center">
						<img src={blogImg} alt="Descriptive Text" class="max-w-full h-auto rounded-xl" />
						<h2 class="text-xl p-5 pb-2 text-black text-left">
							Enhancing Blockchain Privacy: How zkSNARKs are Revolutionizing Decentralized Systems
						</h2>
						<div class="justify-between pl-5 pr-5 flex flex-row">
							<button class="text-xs border text-white bg-[#9CB5E2] rounded-lg py-1 px-3"
								>Crypto</button>
							<p class="text-black text-xs">22 Aug 2023</p>
						</div>
						<p class="p-5 pt-3 text-black text-xs leading-relaxed">
							Blockchain technology has ushered in a new era of decentralized applications, but
							concerns about privacy and scalability remain. In this article, we delve into how
							zkSNARKs are reshaping the landscap...
						</p>
					</div>
				</li>
			</ul>
		</div>
	</div>
</div>

<style>
	.glide__track {
		display: flex;
		flex-direction: row;
		width: 100%;
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
        background: transparent
    }
</style>
