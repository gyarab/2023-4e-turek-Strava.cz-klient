<script lang="ts">
	import Navbar from '$lib/Navbar.svelte';
	import { onMount } from 'svelte';
	import { cantine } from '$lib/store';
	import {
		queryCantineHistory,
		fetchSettings,
		querySettings,
		updateSettings
	} from '$lib/TauriComunicationLayer';
	import { goto } from '$app/navigation';
	import Alert from '$lib/Alert.svelte';
	import BlackListMenu from '$lib/BlackListMenu.svelte';

	let strategy: string = '';
	let draggedItem: Dish;
	let error: string = '';
	let blackListSource: Dish[] = [];
	let blackListTarget: Dish[] = [];
	let whiteListSource: Dish[] = [];
	let whiteListTarget: Dish[] = [];
	let allergens: string[] = [
		'Lepek',
		'Korýši',
		'Vejce',
		'Ryby',
		'Arašídy',
		'Sójové boby',
		'Mléko',
		'Skořápkové plody',
		'Celer',
		'Hořčice',
		'Sezamová semena',
		'Oxid siřičitý a siřičitany',
		'Vlčí bob',
		'Měkkýši'
	];
	let strategies: strategy[] = [
		{ name: 'Odhlásit', value: 'cancel', id: 'strategy-cancel' },
		{ name: 'Nahradit', value: 'replace', id: 'strategy-replace' },
		{ name: 'Odhlásit všechny', value: 'cancelAll', id: 'strategy-cancel-all' },
		{ name: 'Vypnuto', value: 'disabled', id: 'strategy-disabled' }
	];

	let allergensGroup: string[] = [];
	async function handleWhiteListSoureceQuery(e: CustomEvent) {
		let res = await queryCantineHistory($cantine, e.detail.detail, 'whitelist');
		switch (res._t) {
			case 'success':
				whiteListSource = res.data;
				break;
			case 'failure':
				console.log(res.error);
				break;
		}
	}
	async function handleBlackListSoureceQuery(e: CustomEvent) {
		let res = await queryCantineHistory($cantine, e.detail.detail, 'blacklist');
		switch (res._t) {
			case 'success':
				blackListSource = res.data;
				break;
			case 'failure':
				console.log(res.error);
				break;
		}
	}
	async function handleQueryBlackList(e: CustomEvent) {
		let res = await querySettings(e.detail.detail, 'blacklist');
		switch (res._t) {
			case 'success':
				blackListTarget = res.data;
				break;
			case 'failure':
				console.log(res.error);
				break;
		}
	}
	async function handleQueryWhiteList(e: CustomEvent) {
		let res = await querySettings(e.detail.detail, 'whitelist');
		switch (res._t) {
			case 'success':
				whiteListTarget = res.data;
				break;
			case 'failure':
				console.log(res.error);
				break;
		}
	}
	async function handleAllergenSelection(event: Event) {
		let target = event.target as HTMLInputElement;
		let action: string;
		if (target.checked) {
			action = 'add';
		} else {
			action = 'remove';
		}
		updateSettings(target.value, action, 'allergens');
	}
	async function handleSettingsChange(dish: Dish | string, action: string, list: string) {
		let res = await updateSettings(dish, action, list);
		switch (res._t) {
			case 'success':
				break;
			case 'failure':
				error = res.error;
		}
	}
	async function handleAddToBlackList() {
		handleSettingsChange(draggedItem, 'add', 'blacklist');
	}
	async function handleRemoveFromBlackList() {
		handleSettingsChange(draggedItem, 'remove', 'blacklist');
	}
	async function handleAddToWhiteList() {
		handleSettingsChange(draggedItem, 'add', 'whitelist');
	}
	async function handleRemoveFromWhiteList() {
		console.log(draggedItem);
		handleSettingsChange(draggedItem, 'remove', 'whitelist');
	}
	async function handleStrategyChange() {
		handleSettingsChange(strategy, 'change', 'strategy');
	}
	onMount(async () => {
		let settingsRes = await fetchSettings();
		switch (settingsRes._t) {
			case 'success':
				blackListTarget = settingsRes.data.blacklistedDishes;
				whiteListTarget = settingsRes.data.whitelistedDishes;
				allergensGroup = settingsRes.data.blacklistedAllergens;
				strategy = settingsRes.data.strategy;
				break;
			case 'failure':
				error = settingsRes.error;
				break;
			case 'unauthorized':
				goto('/');
				break;
		}
		let historyRes = await queryCantineHistory($cantine, '', 'whitelist');
		switch (historyRes._t) {
			case 'success':
				whiteListSource = historyRes.data;
				break;
			case 'failure':
				error = historyRes.error;
				break;
		}
		historyRes = await queryCantineHistory($cantine, '', 'blacklist');
		switch (historyRes._t) {
			case 'success':
				blackListSource = historyRes.data;
				break;
			case 'failure':
				error = historyRes.error;
				break;
		}
	});
</script>

<Navbar />
<div class="w-full md:w-3/4 flex flex-col justify-center py-2 mx-auto">
	<div class="rounded-md h-full bg-slate-800" style="width: calc(100%);">
		<h2 id="strategy" class="ms-2 text-white text-lg">Automatické odhlšování</h2>
		<div class="flex flex-col md:flex-row md:flex-wrap p-2">
			{#each strategies as strat}
				<div class="2xl:w-1/5 xl:w-1/4 w-1/2">
					<input
						type="radio"
						id={strat.id}
						value={strat.value}
						name="strategy"
						bind:group={strategy}
						on:change={handleStrategyChange}
					/>
					<label class="text-white" for={strat.id}>{strat.name}</label>
				</div>
			{/each}
		</div>
		<h2 id="allergens" class="ms-2 text-white text-lg">Alergeny</h2>
		<div class="flex flex-col md:flex-row md:flex-wrap p-2">
			{#each allergens as allergen}
				<div class="2xl:w-1/5 xl:w-1/4 w-1/2">
					<input
						type="checkbox"
						id={'alergen_' + allergens.indexOf(allergen).toString(10)}
						value={'0' + (allergens.indexOf(allergen) + 1).toString(10)}
						bind:group={allergensGroup}
						on:click={handleAllergenSelection}
					/>
					<label class="text-white" for={'alergen_' + allergens.indexOf(allergen).toString(10)}
						>{allergen}</label
					>
				</div>
			{/each}
		</div>
		<h2 id="blacklist" class="ms-2 text-white text-lg mb-2">Automaticky odhlašované pokrmy</h2>
		<BlackListMenu
			bind:draggedItem
			bind:sourceList={blackListSource}
			bind:targetList={blackListTarget}
			on:dropedToTarget={handleAddToBlackList}
			on:dropedToSource={handleRemoveFromBlackList}
			on:querySource={handleBlackListSoureceQuery}
			on:queryTarget={handleQueryBlackList}
			on:targetItemClicked={handleAddToBlackList}
			on:sourceItemClicked={handleRemoveFromBlackList}
		/>
		<h2 id="whitelist" class="ms-2 text-white text-lg mb-2">Preferované pokrmy</h2>
		<BlackListMenu
			bind:draggedItem
			bind:sourceList={whiteListSource}
			bind:targetList={whiteListTarget}
			on:querySource={handleWhiteListSoureceQuery}
			on:queryTarget={handleQueryWhiteList}
			on:dropedToTarget={handleAddToWhiteList}
			on:dropedToSource={handleRemoveFromWhiteList}
			on:targetItemClicked={handleAddToWhiteList}
			on:sourceItemClicked={handleRemoveFromWhiteList}
		/>
	</div>
	{#key error}
		<Alert message={error} />
	{/key}
</div>
