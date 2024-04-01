<script lang="ts">
	import { Alert } from 'flowbite-svelte';
	export let message: string = '';
</script>

{#if message != ''}
	<div
		class="bg-red-300 text-white p-3 rounded-md align-middle justify-start flex flex-row sticky bottom-0 w-full"
	>
		<p class="align-middle text-red-700 ms-2">{message}</p>
		<button class="ms-auto me-2 text-red-700" on:click={() => (message = '')}
			><svg
				class="w-6 h-6 text-red-700"
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
					d="M6 18 18 6m0 12L6 6"
				/>
			</svg></button
		>
	</div>
{/if}
