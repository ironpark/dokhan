<script lang="ts">
    import Button from "$lib/components/ui/Button.svelte";

    let {
        title = "Dokhan",
        subtitle = "",
        compact = false,
        showZipAction = false,
        onPickZip = () => {}
    }: {
        title?: string;
        subtitle?: string;
        compact?: boolean;
        showZipAction?: boolean;
        onPickZip?: () => void;
    } = $props();
</script>

<header class="title-toolbar" class:compact>
    <div class="title-block">
        <h1>{title}</h1>
        {#if subtitle}
            <p>{subtitle}</p>
        {/if}
    </div>

    {#if showZipAction}
        <Button variant="outline" size="sm" onclick={onPickZip}>ZIP 변경</Button>
    {/if}
</header>

<style>
    .title-toolbar {
        height: 56px;
        border-bottom: 1px solid var(--color-border);
        background: color-mix(in oklab, var(--color-surface), #f8fbff 14%);
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 10px;
        padding: 0 14px;
        min-width: 0;
    }

    .title-toolbar.compact {
        height: 50px;
        padding: 0 12px;
    }

    .title-block {
        min-width: 0;
        display: grid;
        gap: 1px;
    }

    .title-block h1 {
        margin: 0;
        font-size: 14px;
        line-height: 1.15;
        font-weight: 700;
        color: var(--color-text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .title-block p {
        margin: 0;
        font-size: 10px;
        letter-spacing: 0.03em;
        text-transform: uppercase;
        color: var(--color-text-subtle);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }
</style>
