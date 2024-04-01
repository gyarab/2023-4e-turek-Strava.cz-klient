<script lang="ts">
	import Error from '$lib/Error.svelte';
	import { login} from '$lib/TauriComunicationLayer';
	import { goto } from '$app/navigation';
	import { cantine, username, account} from '$lib/store';
	let loading: boolean = false;
	let userLogin: string;
	let cantineId: number;
	let stayLogged: boolean = false;
	let showPassword: boolean = false;
	let message: string = '';
	$: type = showPassword ? 'text' : 'password';
	let value: string = '';

	async function submit() {
		(document.getElementById('login_button') as HTMLElement).blur();
		loading = true;
		let res = await login(userLogin, value, cantineId);
		console.log(res);
		console.log(res._t);
		switch (res._t) {
			case 'success':
				console.log(res.data);
				$username = res.data.username;
				$account = res.data.account.toString(10);
				$cantine = cantineId.toString(10);
				goto('/objednavky');
				break;
			case 'failure':
				loading = false;
				message = res.error;
				break;
			case 'unauthorized':
				loading = false;
				message = 'Nesprávné přihlašovací údaje';
				break;
		}
	}

	function onPasswordInput(e: Event) {
		value = (e.target as HTMLInputElement).value;
	}

	function handleShowPassword() {
		showPassword = !showPassword;
	}
</script>

<div class="flex-col">
	<div
		id="menu"
		class="dark:bg-slate-800 h-1/3 px-5 pt-1 rounded-md"
		style="width: 300px; height: 335px;"
	>
		<h2 class="dark:text-white my-3 w-full md:text-4xl text-2xl text-center">Přihlášení</h2>
		<form on:submit|preventDefault={submit} class="bg-slate-800 flex flex-col h-fit">
			<label class="text-white" for="cantine">Číslo jídelny:</label>
			<input
				class="menu-item dark:dark-mode-autofill bg-slate-800 text-white border-2 px-1 border-white rounded-md focus:outline-2 focus:outline-offset-0 focus:outline-violet-700 focus:outline focus:border-none focus:ring-0 focus:resize-none"
				type="text"
				name="cantine"
				id="cantine"
				bind:value={cantineId}
				required
			/>
			<label class="text-white mt-2" for="username">Uživatelské jméno:</label>
			<input
				class="menu-item dark:dark-mode-autofill bg-slate-800 text-white border-2 px-1 border-white rounded-md focus:outline-2 focus:outline-offset-0 focus:outline-violet-700 focus:outline focus:border-none focus:ring-0"
				type="text"
				name="username"
				id="username"
				bind:value={userLogin}
				required
			/>
			<label class="text-white mt-2" for="password">Heslo:</label>
			<div
				class=" flex flex-row border-2 border-white rounded-md px-1 focus-within:outline-2 focus-within:outline-violet-700 focus-within:outline focus-within:border-none menu-item focus:outline-none"
			>
				<input
					{type}
					{value}
					class="dark:dark-mode-autofill bg-slate-800 text-white border-none flex-grow focus-within:border-none focus-within:ring-0 focus-within:outline-none"
					name="password"
					id="password"
					on:input={onPasswordInput}
					required
				/>
				<button
					class="text-white me-0 select-none active:shadow-none"
					type="button"
					on:click={handleShowPassword}
					tabindex="-1"
				>
					<svg
						class="w-6 h-6 text-gray-800 dark:text-white"
						aria-hidden="true"
						xmlns="http://www.w3.org/2000/svg"
						fill="none"
						viewBox="0 0 24 24"
					>
						<path
							stroke="currentColor"
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d={showPassword
								? 'M4 14c-.5-.6-.9-1.3-1-2 0-1 4-6 9-6m7.6 3.8A5 5 0 0 1 21 12c0 1-3 6-9 6h-1m-6 1L19 5m-4 7a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z'
								: 'M21 12c0 1.2-4 6-9 6s-9-4.8-9-6c0-1.2 4-6 9-6s9 4.8 9 6Z M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z'}
						/>
					</svg>
				</button>
			</div>
			<div class="flex-row mt-2">
				<input
					class="non-expand focus:border-none focus:ring-0 focus:outline-offset-0 focus:outline-violet-700 rounded-sm"
					type="checkbox"
					name="stayLogged"
					id="stayLogged"
					bind:value={stayLogged}
				/>
				<label class="text-white ms-2" for="stayLogged">Zůstat přihlášen</label>
			</div>
			<button
				class="bg-violet-700 mt-3 rounded-md menu-item focus:ring-0 focus:border-none focus:outline-white focus:outline-1 outline-none focus:outline-offset-0 dark:dark-mode-autofill dark:bg-violet-700 dark:text-white"
				type="submit" 
				id="login_button"
				>{#if loading}
					<svg
						aria-hidden="true"
						class="inline w-6 h-6 text-gray-200 animate-spin dark:text-gray-600 dark:fill-white"
						viewBox="0 0 100 101"
						fill="none"
						xmlns="http://www.w3.org/2000/svg"
					>
						<path
							d="M100 50.5908C100 78.2051 77.6142 100.591 50 100.591C22.3858 100.591 0 78.2051 0 50.5908C0 22.9766 22.3858 0.59082 50 0.59082C77.6142 0.59082 100 22.9766 100 50.5908ZM9.08144 50.5908C9.08144 73.1895 27.4013 91.5094 50 91.5094C72.5987 91.5094 90.9186 73.1895 90.9186 50.5908C90.9186 27.9921 72.5987 9.67226 50 9.67226C27.4013 9.67226 9.08144 27.9921 9.08144 50.5908Z"
							fill="currentColor"
						/>
						<path
							d="M93.9676 39.0409C96.393 38.4038 97.8624 35.9116 97.0079 33.5539C95.2932 28.8227 92.871 24.3692 89.8167 20.348C85.8452 15.1192 80.8826 10.7238 75.2124 7.41289C69.5422 4.10194 63.2754 1.94025 56.7698 1.05124C51.7666 0.367541 46.6976 0.446843 41.7345 1.27873C39.2613 1.69328 37.813 4.19778 38.4501 6.62326C39.0873 9.04874 41.5694 10.4717 44.0505 10.1071C47.8511 9.54855 51.7191 9.52689 55.5402 10.0491C60.8642 10.7766 65.9928 12.5457 70.6331 15.2552C75.2735 17.9648 79.3347 21.5619 82.5849 25.841C84.9175 28.9121 86.7997 32.2913 88.1811 35.8758C89.083 38.2158 91.5421 39.6781 93.9676 39.0409Z"
							fill="currentFill"
						/>
					</svg>
					<span class="sr-only">Loading...</span>
				{:else}
					Přihlásit
				{/if}</button
			>
		</form>
	</div>
	{#if message != ''}
		<Error {message} />
	{/if}
</div>
