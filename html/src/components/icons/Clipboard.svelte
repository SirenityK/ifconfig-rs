<script lang="ts">
  import type { ClipboardProps } from "lucide-svelte/icons/clipboard";
  import Clipboard from "lucide-svelte/icons/clipboard";
  import ClipboardCheck from "lucide-svelte/icons/clipboard-check";

  let props: ClipboardProps = $props();

  let clipboard = $state({
    pressed: false,
    failed: false,
  });
</script>

<button
  class="btn btn-sm btn-soft"
  onclick={async (e) => {
    clipboard.pressed = true;

    const text =
      e.currentTarget.parentElement?.previousElementSibling?.textContent;

    if (text) await navigator.clipboard.writeText(text);
    else {
      clipboard.failed = true;
      clipboard.pressed = false;
    }
  }}
>
  Copy
  {#if clipboard.pressed}
    <ClipboardCheck {...props} />
  {:else}
    <Clipboard {...props} />
  {/if}
</button>

{#if clipboard.pressed}
  <p class="text-success animate-pulse">Copied to clipboard!</p>
{/if}

{#if clipboard.failed}
  <p class="text-error animate-pulse">Failed to copy to clipboard!</p>
{/if}
