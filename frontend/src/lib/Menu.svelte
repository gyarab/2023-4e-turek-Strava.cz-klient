<script lang="ts">
	import DailyMenu from '$lib/DailyMenu.svelte';
	import Alert from '$lib/Alert.svelte';

	export let menuData: Menu;
	export let days: string[];
	
	let error:string = '';
</script>

<div class="flex flex-col items-center w-full" id="menu">
	{#each days as date}
		<DailyMenu bind:error {date} menu={menuData[date] || new Map()} />
	{/each}
	{#key error}
		<Alert bind:message={error} />
	{/key}
</div>
