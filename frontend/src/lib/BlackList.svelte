<script lang="ts">
	import { createEventDispatcher } from 'svelte';

	export let list: Dish[] = [];
	export let draggedItem: Dish;
	const dispatch = createEventDispatcher();

	function onDragStart(e: DragEvent) {
		let source = (e.target as HTMLElement).getElementsByTagName('p');
		draggedItem = {
			name: source.length > 0 ? source[0].innerText : '',
			allergens: source.length > 1 ? source[1].innerText.split(',')[0] === ""? [] :source[1].innerText.split(',') : []
		};
	}
	function onClick(e: MouseEvent) {
		let source = (e.currentTarget as HTMLElement).getElementsByTagName('p');
		draggedItem = {
			name: source.length > 0 ? source[0].innerText : '',
			allergens: source.length > 1 ? source[1].innerText.split(',')[0] === ""? [] :source[1].innerText.split(',') : []
		};
		dispatch('itemClicked');
	}
	function handeleQuery(Event: Event) {
		dispatch('query', (Event.target as HTMLInputElement).value);
	
	}
</script>

<div
	class="w-1/2 rounded-md h-96 overflow-y-scroll bg-slate-700 scrollbar-none border-2 border-white mx-4 mb-2 px-3"
	on:drop={() => {
		dispatch('drop');
	}}
	on:dragover|preventDefault
	role="cell"
	tabindex="-1"
>
	<div class="sticky top-0 bg-slate-700 py-2">
		<div
			class="border-separate border-2 bg-slate-800 border-white flex flex-row w-full rounded-full mx-auto p-2 h-10 text-center sticky top-0 z-0"
		>
			<input
				class="dark:dark-mode-autofill p-1 bg-slate-800 text-white border-none w-full focus-within:border-none focus-within:ring-0 focus-within:outline-none"
				type="text" on:input={handeleQuery}
			/>
			<svg
				class="text-gray-800 dark:text-white block h-8 w-8 me-2 my-auto"
				aria-hidden="true"
				xmlns="http://www.w3.org/2000/svg"
				fill="none"
				viewBox="0 0 24 24"
				style="height: 20px; width: 20px;"
			>
				<path
					class="my-auto block"
					stroke="currentColor"
					stroke-linecap="round"
					stroke-width="2"
					d="m21 21-3.5-3.5M17 10a7 7 0 1 1-14 0 7 7 0 0 1 14 0Z"
				/>
			</svg>
		</div>
		
	</div>

	{#each list as item}
		<button
			class="flex flex-col justify-start border-2 bg-slate-800 border-white rounded-md p-3 mt-3 opacity-100 w-full"
			draggable="true"
			on:dragstart={onDragStart}
			on:click|preventDefault={onClick}
			role="cell"
			tabindex="-1"
		>
			<p class="text-white text-start">{item.name}</p>
			<p class="text-gray-400 text-start ">{item.allergens}</p>
		</button>
	{/each}
	<div class="sticky bottom-0 h-3 bg-slate-700 rounded-none w-full"></div>
</div>
