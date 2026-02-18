<script>
  export let tab = 'monitoring';
  export let status = null;

  const readMessage = (value) => String(value || '').trim();

  $: tabStatus = status && typeof status === 'object' ? status : {};
  $: loading = tabStatus.loading === true;
  $: errorText = readMessage(tabStatus.error);
  $: empty = tabStatus.empty === true;
  $: statusMessage = readMessage(tabStatus.message);
  $: message = loading
    ? (statusMessage || 'Loading...')
    : (errorText || (empty ? (statusMessage || 'No data.') : ''));
  $: stateKind = loading ? 'loading' : (errorText ? 'error' : (empty ? 'empty' : ''));
  $: className = stateKind ? `tab-state tab-state--${stateKind}` : 'tab-state';
</script>

<div
  class={className}
  data-tab-state={tab}
  role="status"
  aria-live="polite"
  hidden={!stateKind}
>{message}</div>
