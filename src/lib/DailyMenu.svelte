<script>
  export let date;
  export let dishes;
</script>

<div id="daily_menu">
  <h2>{date}</h2>
  
  {#each Object.entries(dishes) as [dish, ordered]}
    <div>{dish} {ordered[0]}</div>
  {/each}
</div>
