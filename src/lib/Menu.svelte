<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/tauri";
  import { each } from "svelte/internal";
  import DailyMenu from "./DailyMenu.svelte";
  let menuData = {}; 
  let ww =[]
  onMount(async () => {
    menuData =  await invoke("get_menu_data", {});    
    ww =  await invoke("sort_menus_keys", {keys: Object.keys(menuData)});
    console.log(Object.entries(menuData));  
    console.log(ww);  
  });
 
 
</script>

<div id="menu">
 
  {#each ww as w}
    <DailyMenu date={w} dishes={menuData[w]} />
   
  {/each}
</div>
