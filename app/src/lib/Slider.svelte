<script lang="ts">
  interface Props {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    digits?: number;
  }

  let { label, value = $bindable(), min, max, step = 1, digits = 0 }: Props = $props();
</script>

<label class="slider">
  <span class="row">
    <span>{label}</span>
    <output>{value.toFixed(digits)}</output>
  </span>
  <input type="range" {min} {max} {step} bind:value />
</label>

<style>
  .slider {
    display: grid;
    gap: 4px;
  }
  .row {
    display: flex;
    justify-content: space-between;
  }
  output {
    color: var(--text-dim);
    font-variant-numeric: tabular-nums;
  }
</style>
