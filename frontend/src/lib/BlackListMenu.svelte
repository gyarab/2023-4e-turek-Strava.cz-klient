<script lang="ts">
	import BlackList from './BlackList.svelte';
	import { createEventDispatcher } from 'svelte';

	export let sourceList: Dish[] = [];
	export let targetList: Dish[] = [];
	export let draggedItem: Dish;

	const dispatch = createEventDispatcher();
	
	function onDropToSourceList() {
		if (draggedItem !== undefined) {
			if (draggedItem !== undefined) {
			if (constains(sourceList, draggedItem) === -1) {
				sourceList = [...sourceList, draggedItem];
			}
			if (constains(targetList, draggedItem) !== -1) {
				targetList.splice(constains(targetList, draggedItem), 1);
			}
			targetList = targetList;
		}
		}
	}
	const constains = (list: Dish[], item: Dish): number => {
		for (let i = 0; i < list.length; i++) {
			if (
				list[i].name === item.name &&
				JSON.stringify(list[i].allergens) === JSON.stringify(item.allergens)
			) {
				return i;
			}
		}
		return -1;
	};

	function onDropToTargetList() {
		if (draggedItem !== undefined) {
			if (constains(targetList, draggedItem) === -1) {
				targetList = [...targetList, draggedItem];
			}
			if (constains(sourceList, draggedItem) !== -1) {
				sourceList.splice(constains(sourceList, draggedItem), 1);
			}
			sourceList = sourceList;
		}
	}
</script>

<div class="flex flex-1 flex-row h-96 mb-2 justify-center">
	<BlackList
		bind:draggedItem
		bind:list={sourceList}
		on:drop={(e) => {
			onDropToSourceList();
			dispatch('dropedToSource');
		}}
		on:itemClicked={(e) => {
			onDropToTargetList();
			dispatch('targetItemClicked');
		}}
		on:query={(e) => {
			dispatch('querySource', e); ;
		}}
	/>
	<BlackList
		bind:draggedItem
		bind:list={targetList}
		on:drop={(e) => {
			onDropToTargetList();
			dispatch('dropedToTarget');
		}}
		on:itemClicked={(e) => {
			onDropToSourceList();
			dispatch('sourceItemClicked');
		}}
		on:query={(e) => {
			dispatch('queryTarget', e);
		}}
	/>
</div>
