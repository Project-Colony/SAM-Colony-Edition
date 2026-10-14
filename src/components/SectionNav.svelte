<script lang="ts">
  import { t } from "../i18n/index.svelte";
  import type { View } from "../lib/types";

  let { view, onSelect }: { view: View; onSelect: (view: View) => void } = $props();

  const sections: { view: View; label: () => string }[] = [
    { view: "achievements", label: () => t("view_achievements") },
    { view: "statistics", label: () => t("view_statistics") },
  ];
</script>

<div class="nav">
  {#each sections as section (section.view)}
    <button
      class="nav__item"
      class:nav__item--selected={view === section.view}
      aria-current={view === section.view ? "page" : undefined}
      onclick={() => onSelect(section.view)}
    >
      {section.label()}
    </button>
  {/each}
</div>

<style>
  .nav { display: flex; flex-direction: column; gap: 2px; }
  /* Selection is a background, never a coloured label (navigation.md). */
  .nav__item {
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    border-radius: 8px;
    padding: 8px 14px;
    font-size: var(--fs-13);
    color: var(--colony-text-muted);
    cursor: pointer;
  }
  .nav__item:hover { background: var(--colony-bg-card-hover); }
  .nav__item--selected,
  .nav__item--selected:hover {
    background: var(--colony-accent);
    color: var(--colony-on-accent);
  }
</style>
