<script lang="ts">
    export let secondary = false;
    export let active = false;
    export let highlight = "";
    export let vertical = false;
</script>

<div class="root" class:secondary={secondary} class:active={active} class:highlight={highlight.length > 0} class:vertical={vertical} title={highlight}>
    <slot />
</div>

<style lang="scss">
    @import "../conf/text.scss";
    @import "../conf/anime.scss";

    .root {
        padding: 2px 14px;
        color: $text__primary_color;
        font-family: $text__font_family;
        font-size: $text__primary_font_size;
        user-select: none;
        white-space: nowrap;

        // Force center.
        display: inline-block;
        flex: 0;

        &.active {
            color: $text__brand_foreground;
        }

        &.highlight {
            text-decoration: underline dotted;
        }

        &.secondary {
            color: $text__secondary_color;
            font-size: $text__secondary_font_size;

            &.active {
                color: $text__brand_foreground;
            }
        }

        &.vertical {
            text-orientation: mixed;
            writing-mode: vertical-rl;
            padding: 14px 2px;
        }
    }
</style>