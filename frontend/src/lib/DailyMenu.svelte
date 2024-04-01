<script lang="ts">
	import { invoke } from '@tauri-apps/api/tauri';
	import { orderDish, saveOrder } from '$lib/TauriComunicationLayer'; // change to TauriComunicationLayer for Tauri version
	import { account } from './store';
	import {onDestroy} from 'svelte';
	export let date: string;
	export let menu: DailyMenu;
	export let error: string;

	let selected: any[] = [];
	
	async function selectDish(e: Event) {
		let name = (e.target as HTMLInputElement).value;
		let keys = Object.keys(menu);
		for (let i = 0; i < keys.length; i++) {
			if (keys[i] !== name) {
				menu[keys[i]].orderState = false;
			}
		}
		let res = await orderDish(menu[name].id, menu[name].orderState); // Web version from changed import for Tauri version
		switch (res._t) {
			case 'success':
				console.log(res.data);
				$account = res.data.toString(10);
				let saveRes = await saveOrder(); // Web version from changed import for Tauri version
				switch(saveRes._t) {
					case 'success':
						break;
					case 'failure':
						error = saveRes.error.message;
						$account = saveRes.error.account.toString(10);
						break;
				}
				break;
			case 'failure':
				(e.target as HTMLInputElement).checked = false;
				error = res.error;
				break;
		}
	}
</script>

<div class="bg-slate-800 rounded-md my-5 border-white border-1 md:w-3/4 w-full p-5" id="daily_menu">
	<h2 class="text-white text-2xl">{date}</h2>
	{#each Object.entries(menu) as [name, dish]}
		<div class="flex-row flex mt-2 align-middle">
			<input
				class="accent-violet-700 me-5 non-expand rounded-sm my-auto"
				style="width: 15px; height: 15px;"
				type="checkbox"
				bind:checked={dish.orderState}
				bind:value={name}
				on:change|preventDefault={selectDish}
			/>
			<p class="bg-slate-800 text-white text-lg">{name} <span class="text-gray-400">{dish.allergens}</span></p>
		</div>
	{/each}
</div>
