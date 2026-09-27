<script lang="ts">
    import BookOpen from "@lucide/svelte/icons/book-open";
    import FolderOpen from "@lucide/svelte/icons/folder-open";
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
    <div class="brand">
        <span class="brand-mark" aria-hidden="true"><BookOpen size={18} strokeWidth={2.1} /></span>
        <div class="title-block">
            <h1>{title}</h1>
            {#if subtitle}
                <p title={subtitle}>{subtitle}</p>
            {/if}
        </div>
    </div>

    {#if showZipAction}
        <Button variant="toolbar-pill" size="sm" class="zip-action" onclick={onPickZip}>
            <FolderOpen size={16} aria-hidden="true" />
            <span>사전 변경</span>
        </Button>
    {/if}
</header>

<style>
    .title-toolbar {
        min-height: 68px;
        border-bottom: 1px solid var(--color-border);
        background: var(--color-surface-soft);
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding: 8px 16px;
        min-width: 0;
        box-sizing: border-box;
    }

    .title-toolbar.compact {
        min-height: 58px;
        padding: 7px 14px;
    }

    .brand {
        display: flex;
        align-items: center;
        gap: 10px;
        min-width: 0;
    }

    .brand-mark {
        width: 36px;
        height: 36px;
        flex: none;
        display: grid;
        place-items: center;
        border: 1px solid var(--color-accent-border);
        border-radius: 11px;
        background: var(--color-accent-soft);
        color: var(--color-accent);
    }

    .compact .brand-mark {
        width: 32px;
        height: 32px;
        border-radius: 10px;
    }

    .title-block {
        min-width: 0;
        display: grid;
        gap: 3px;
    }

    .title-block h1 {
        margin: 0;
        font-size: 15px;
        line-height: 1.2;
        font-weight: 750;
        letter-spacing: -0.025em;
        color: var(--color-text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .title-block p {
        margin: 0;
        font-size: 11px;
        line-height: 1.25;
        color: var(--color-text-subtle);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    /* Matches the reader toolbar pills so header actions read as one family. */
    .title-toolbar :global(.zip-action) {
        min-height: 34px;
        font-size: 12px;
    }

    @media (max-width: 420px) {
        .brand-mark {
            display: none;
        }
    }
</style>
