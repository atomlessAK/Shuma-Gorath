<script>
  export let title = '';
  export let compact = false;
</script>

<div class={`card panel panel-border ${compact ? 'pad-md' : 'pad-lg'}`}>
  <h3 class="caps-label">{title}</h3>
  <slot />
</div>
