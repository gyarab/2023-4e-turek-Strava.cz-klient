<script lang="ts">
	import Menu from '$lib/Menu.svelte';
	import Navbar from '$lib/Navbar.svelte';
	import Alert from '$lib/Alert.svelte';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import {getUserMenu} from '$lib/TauriComunicationLayer';
    
	let error: string = '';
	let menuData: MenuData = {}
	let days: string[] = [];

	onMount(async () => {
	   let data = await getUserMenu();
	   switch (data._t) {
		   case 'success':
			   menuData = data.data;
			   console.log(menuData);
			   days = Object.keys(menuData);
			   break;
		   case 'failure':
			   error = data.error;
			   break;
		   case 'unauthorized':
			   goto('/');
			   break;
	   }
	});
</script>

<Navbar/>

{#key menuData}
	<Menu  menuData={menuData } days={days} />
{/key}
{#key error}
	<Alert bind:message={error} />
{/key}



