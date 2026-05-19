<script lang="ts">
  import type { View } from "../lib/types";

  let { view = $bindable() }: { view: View } = $props();
</script>

<div class="tabs">
  <button
    class="tab"
    class:tab--active={view === "achievements"}
    onclick={() => (view = "achievements")}
  >
    Achievements
  </button>
  <button
    class="tab"
    class:tab--active={view === "statistics"}
    onclick={() => (view = "statistics")}
  >
    Statistics
  </button>
</div>

<style>
  .tabs { display: flex; gap: 6px; }
  .tab {
    flex: 1;
    background: var(--parchment);
    border: 1px solid var(--red-soft);
    border-radius: var(--radius-input);
    padding: 6px 8px;
    font-family: var(--font-body);
    font-size: var(--fs-sm);
    font-weight: var(--fw-medium);
    letter-spacing: var(--tracking-wide);
    text-transform: uppercase;
    color: var(--ink-dim);
    cursor: pointer;
  }
  .tab:hover { background: var(--parchment-hover); color: var(--ink); }
  .tab--active {
    background: var(--red);
    border-color: var(--red);
    color: var(--parchment);
    font-weight: var(--fw-semibold);
  }
</style>
