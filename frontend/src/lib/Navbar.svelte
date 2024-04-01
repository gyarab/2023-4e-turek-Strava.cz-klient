<script lang="ts">
	import {Dropdown, DropdownItem } from 'flowbite-svelte';
	import { logout, saveOrder} from '$lib/TauriComunicationLayer';
	import { goto } from '$app/navigation';
	import {username} from '$lib/store';
	import {account} from '$lib/store';

	async function saveOrders() {
		// await saveOrder();
	}
	async function logoutHandler() {
		await logout();
		goto('/');
	}
</script>

<nav
	class="dark:bg-slate-800 w-full flex flex-row align-middle justify-center  sticky
	top-0 z-40"
	style="height: 80px;"
>
	<div class="flex flex-col w-full md:w-3/4" style="height: 50px;">
		<div class="flex flex-row border-b w-100 border-white" style="height: 50px;">
			<h1 class="dark:text-white text-4xl text-center mt-auto mb-auto ms-2">Strava-klient</h1>
			<button
				class="dark:text-white flex flex-row me-2 ms-auto text-center border border-white rounded mt-1 mb-1 px-2"
				id="user_button"
				><svg
					style="height: 20px ; width: 20px;"
					xmlns="http://www.w3.org/2000/svg"
					fill="none"
					viewBox="0 0 24 24"
					stroke-width="1.5"
					stroke="currentColor"
					class="mt-auto mb-auto me-2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z"
					/>
				</svg><p class="mt-auto mb-auto">{$username}</p></button
			>
			{#key $username}
			{#if $username != 'Nepřihlášen'}
			<Dropdown class="bg-slate-800 rounded-md border border-white" triggeredBy="#user_button">
				<DropdownItem><button on:click={logoutHandler}><p class="mx-9 text-white">Odhlásit</p></button></DropdownItem>
			</Dropdown>
			{/if}
			{/key}
		</div>
		<div class="flex flex-row" style="height: 30px;">
			<a
				class="dark:text-white text-center mt-auto ms-2 mb-auto"
				style="display: block;"
				href="/objednavky">Objednávky</a
			>
			<a class="dark:text-white mt-auto text-center mb-auto ms-2 me-auto" href="/nastaveni">Nastavení</a>
			<p class="me-2 hidden sm:block  ms-auto text-white">Zůstatek na vašem účtu: {$account} Kč</p>
		</div>
	</div>
</nav>
