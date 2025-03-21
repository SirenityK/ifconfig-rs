<script lang="ts">
  interface Props {
    object: Record<string, string>;
  }
  let { object }: Props = $props();

  function capitalize(str: string) {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }

  function capitalize_array(arr: string[]) {
    return arr.map((str) => capitalize(str)).join(" ");
  }
</script>

{#each Object.entries(object) as [key, value]}
  <div class="flex *:whitespace-nowrap *:p-2 max-md:flex-col">
    <p class="text-description grow">{capitalize_array(key.split(/[-_]/))}</p>
    <p class="overflow-x-auto">{value}</p>
  </div>
{/each}
